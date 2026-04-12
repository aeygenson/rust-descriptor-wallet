use crate::broadcast::TxBroadcaster;
use crate::{WalletSyncError, WalletSyncResult};
use serde_json::json;
use std::thread::sleep;
use std::time::Duration;

use super::classify::classify_rpc_rejection;

use tracing::{debug, error, info, warn};

/// Bitcoin Core RPC broadcaster for fully signed raw transactions.
///
/// This backend implementation is internal to `wallet_sync`. Higher layers
/// should use the sync facade from `service.rs` instead of depending directly
/// on this type unless they explicitly need backend-specific behavior.
#[derive(Debug, Clone)]
pub struct CoreRpcBroadcaster {
    rpc_url: String,
    rpc_user: String,
    rpc_pass: String,
    client: reqwest::blocking::Client,
    max_retries: usize,
    retry_delay: Duration,
}

impl CoreRpcBroadcaster {
    pub fn new(
        rpc_url: impl Into<String>,
        rpc_user: impl Into<String>,
        rpc_pass: impl Into<String>,
    ) -> Self {
        Self {
            rpc_url: rpc_url.into(),
            rpc_user: rpc_user.into(),
            rpc_pass: rpc_pass.into(),
            client: reqwest::blocking::Client::new(),
            max_retries: 3,
            retry_delay: Duration::from_millis(300),
        }
    }

    fn should_retry_status(status: reqwest::StatusCode) -> bool {
        status.is_server_error()
            || status == reqwest::StatusCode::TOO_MANY_REQUESTS
            || status == reqwest::StatusCode::REQUEST_TIMEOUT
    }

    fn should_retry_rpc_error(code: i64) -> bool {
        matches!(code, -28)
    }

    /// Broadcast raw transaction hex without requiring callers to import the
    /// `TxBroadcaster` trait into scope.
    #[inline]
    pub fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()> {
        <Self as TxBroadcaster>::broadcast_tx_hex(self, tx_hex)
    }
}

impl TxBroadcaster for CoreRpcBroadcaster {
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()> {
        let payload = json!({
            "jsonrpc": "1.0",
            "id": "wallet_sync",
            "method": "sendrawtransaction",
            "params": [tx_hex],
        });

        info!("broadcasting transaction via Core RPC");
        debug!("rpc_url = {}", self.rpc_url);

        let mut last_error: Option<WalletSyncError> = None;

        for attempt in 0..=self.max_retries {
            debug!("broadcast attempt {}", attempt);

            let response = self
                .client
                .post(&self.rpc_url)
                .basic_auth(&self.rpc_user, Some(&self.rpc_pass))
                .json(&payload)
                .send();

            match response {
                Ok(resp) if resp.status().is_success() => {
                    let value: serde_json::Value = resp
                        .json()
                        .map_err(|e| WalletSyncError::BroadcastTransport(e.to_string()))?;

                    if value.get("error").is_none() || value["error"].is_null() {
                        info!("transaction broadcast successful");
                        return Ok(());
                    }

                    let code = value["error"]["code"].as_i64().unwrap_or(0);
                    let message = value["error"]["message"]
                        .as_str()
                        .unwrap_or("unknown RPC error")
                        .to_string();

                    let err = classify_rpc_rejection(code, &message);

                    warn!("rpc returned error: code = {}, message = {}", code, message);

                    if attempt < self.max_retries && Self::should_retry_rpc_error(code) {
                        last_error = Some(err);
                        sleep(self.retry_delay);
                        continue;
                    }

                    return Err(err);
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp
                        .text()
                        .unwrap_or_else(|_| "<unable to read response body>".to_string());

                    let err = WalletSyncError::BroadcastTransport(format!(
                        "rpc transport status={} body={}",
                        status, body
                    ));

                    warn!("http error during broadcast: status = {}, body = {}", status, body);

                    if attempt < self.max_retries && Self::should_retry_status(status) {
                        last_error = Some(err);
                        sleep(self.retry_delay);
                        continue;
                    }

                    return Err(err);
                }
                Err(e) => {
                    warn!("transport error during broadcast: {}", e);
                    let err = WalletSyncError::BroadcastTransport(e.to_string());

                    if attempt < self.max_retries {
                        last_error = Some(err);
                        sleep(self.retry_delay);
                        continue;
                    }

                    return Err(last_error.unwrap_or(err));
                }
            }
        }

        error!("broadcast failed after retries");
        Err(last_error.unwrap_or_else(|| {
            WalletSyncError::BroadcastFailed("unknown bitcoin core broadcast failure".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::thread;

    fn broadcaster(server: &MockServer) -> CoreRpcBroadcaster {
        CoreRpcBroadcaster::new(server.base_url(), "bitcoin", "bitcoin")
    }

    #[test]
    fn broadcast_tx_hex_succeeds_on_rpc_result() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({
                "result": "deadbeef-txid",
                "error": null,
                "id": "wallet_sync"
            }));
        });

        let result = broadcaster(&server).broadcast_tx_hex("deadbeef");
        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn maps_mempool_conflict_from_rpc_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({
                "result": null,
                "error": { "code": -26, "message": "txn-mempool-conflict" },
                "id": "wallet_sync"
            }));
        });

        let result = broadcaster(&server).broadcast_tx_hex("deadbeef");
        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastMempoolConflict(msg)) if msg.contains("txn-mempool-conflict")
        ));
        mock.assert();
    }

    #[test]
    fn maps_already_confirmed_from_rpc_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({
                "result": null,
                "error": { "code": -27, "message": "transaction already in block chain" },
                "id": "wallet_sync"
            }));
        });

        let result = broadcaster(&server).broadcast_tx_hex("deadbeef");
        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastAlreadyConfirmed(msg)) if msg.contains("already in block chain")
        ));
        mock.assert();
    }

    #[test]
    fn maps_missing_inputs_from_rpc_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({
                "result": null,
                "error": { "code": -25, "message": "missing inputs" },
                "id": "wallet_sync"
            }));
        });

        let result = broadcaster(&server).broadcast_tx_hex("deadbeef");
        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastMissingInputs(msg)) if msg.contains("missing inputs")
        ));
        mock.assert();
    }

    #[test]
    fn maps_insufficient_fee_from_rpc_error() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({
                "result": null,
                "error": { "code": -26, "message": "min relay fee not met" },
                "id": "wallet_sync"
            }));
        });

        let result = broadcaster(&server).broadcast_tx_hex("deadbeef");
        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastInsufficientFee(msg)) if msg.contains("min relay fee")
        ));
        mock.assert();
    }

    #[test]
    fn maps_non_final_to_psbt_not_finalized() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST).path("/");
            then.status(200).json_body(json!({
                "result": null,
                "error": { "code": -26, "message": "non-BIP68-final" },
                "id": "wallet_sync"
            }));
        });

        let result = broadcaster(&server).broadcast_tx_hex("deadbeef");
        assert!(matches!(result, Err(WalletSyncError::PsbtNotFinalized)));
        mock.assert();
    }

    #[test]
    fn retries_and_succeeds_on_retryable_http_status() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind test listener");
        let addr = listener.local_addr().expect("listener addr");

        let handle = thread::spawn(move || {
            for attempt in 0..2 {
                let (mut stream, _) = listener.accept().expect("accept connection");
                let mut buf = [0u8; 4096];
                let _ = stream.read(&mut buf).expect("read request");

                if attempt == 0 {
                    let body = "temporary server error";
                    let response = format!(
                        "HTTP/1.1 503 Service Unavailable\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(response.as_bytes()).expect("write 503 response");
                } else {
                    let body = json!({
                        "result": "deadbeef-txid",
                        "error": null,
                        "id": "wallet_sync"
                    })
                    .to_string();
                    let response = format!(
                        "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
                        body.len(),
                        body
                    );
                    stream.write_all(response.as_bytes()).expect("write 200 response");
                }
                stream.flush().expect("flush response");
            }
        });

        let broadcaster =
            CoreRpcBroadcaster::new(format!("http://{}", addr), "bitcoin", "bitcoin");
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(result.is_ok(), "unexpected result: {:?}", result);
        handle.join().expect("server thread join");
    }
}