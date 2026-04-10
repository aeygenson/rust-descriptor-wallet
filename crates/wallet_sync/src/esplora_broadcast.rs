use crate::broadcast::TxBroadcaster;
use crate::{WalletSyncError, WalletSyncResult};
use std::thread::sleep;
use std::time::Duration;

/// Abstraction for broadcasting a fully signed raw transaction to the network.
///
/// `wallet_sync` defines the behavior contract together with concrete backend
/// implementations (Esplora, Electrum, Bitcoin Core RPC, mocks, etc.).
///
/// Implementations should:
/// - return `Ok(())` only when the transaction is accepted for broadcast
/// - return structured `WalletSyncError::*` variants for failure cases
/// - avoid panicking on network or parsing errors
#[derive(Debug, Clone)]
pub struct EsploraBroadcaster {
    base_url: String,
    client: reqwest::blocking::Client,
    max_retries: usize,
    retry_delay: Duration,
}

impl EsploraBroadcaster {
    pub fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into().trim_end_matches('/').to_string(),
            client: reqwest::blocking::Client::new(),
            max_retries: 3,
            retry_delay: Duration::from_millis(300),
        }
    }

    fn tx_endpoint(&self) -> String {
        format!("{}/tx", self.base_url)
    }

    fn should_retry_status(status: reqwest::StatusCode) -> bool {
        status.is_server_error()
            || status == reqwest::StatusCode::TOO_MANY_REQUESTS
            || status == reqwest::StatusCode::REQUEST_TIMEOUT
    }

    fn classify_esplora_rejection(status: reqwest::StatusCode, body: &str) -> WalletSyncError {
        let normalized = body.to_ascii_lowercase();

        if normalized.contains("txn-mempool-conflict")
            || normalized.contains("mempool conflict")
        {
            return WalletSyncError::BroadcastMempoolConflict(body.to_string());
        }

        if normalized.contains("already in block chain")
            || normalized.contains("already confirmed")
            || normalized.contains("transaction already in block chain")
        {
            return WalletSyncError::BroadcastAlreadyConfirmed(body.to_string());
        }

        if normalized.contains("missing inputs") {
            return WalletSyncError::BroadcastMissingInputs(body.to_string());
        }

        if normalized.contains("non-bip68-final")
            || normalized.contains("non-final")
        {
            return WalletSyncError::PsbtNotFinalized;
        }

        if normalized.contains("min relay fee")
            || normalized.contains("insufficient fee")
            || normalized.contains("fee not met")
        {
            return WalletSyncError::BroadcastInsufficientFee(body.to_string());
        }

        WalletSyncError::BroadcastFailed(format!(
            "esplora rejected transaction: status={} body={}",
            status, body
        ))
    }

    /// Broadcast raw transaction hex without requiring callers to import the
    /// `TxBroadcaster` trait into scope.
    #[inline]
    pub fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()> {
        <Self as TxBroadcaster>::broadcast_tx_hex(self, tx_hex)
    }
}

impl TxBroadcaster for EsploraBroadcaster {
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()> {
        let endpoint = self.tx_endpoint();
        let mut last_error: Option<WalletSyncError> = None;

        for attempt in 0..=self.max_retries {
            let response = self
                .client
                .post(&endpoint)
                .header(reqwest::header::CONTENT_TYPE, "text/plain")
                .body(tx_hex.to_string())
                .send();

            match response {
                Ok(resp) if resp.status().is_success() => return Ok(()),
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp
                        .text()
                        .unwrap_or_else(|_| "<unable to read response body>".to_string());

                    let err = Self::classify_esplora_rejection(status, &body);

                    if attempt < self.max_retries && Self::should_retry_status(status) {
                        last_error = Some(err);
                        sleep(self.retry_delay);
                        continue;
                    }

                    return Err(err);
                }
                Err(e) => {
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

        Err(last_error.unwrap_or_else(|| {
            WalletSyncError::BroadcastFailed("unknown broadcast failure".to_string())
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

    #[test]
    fn trims_trailing_slash_from_base_url() {
        let broadcaster = EsploraBroadcaster::new("https://mempool.space/signet/api/");
        assert_eq!(
            broadcaster.tx_endpoint(),
            "https://mempool.space/signet/api/tx"
        );
    }

    #[test]
    fn broadcast_tx_hex_succeeds_on_200() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/tx")
                .body("deadbeef");
            then.status(200);
        });

        let broadcaster = EsploraBroadcaster::new(server.base_url());
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(result.is_ok());
        mock.assert();
    }

    #[test]
    fn broadcast_tx_hex_maps_mempool_conflict_from_http_response() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/tx")
                .body("deadbeef");
            then.status(400)
                .body("sendrawtransaction RPC error: {\"code\":-26,\"message\":\"txn-mempool-conflict\"}");
        });

        let broadcaster = EsploraBroadcaster::new(server.base_url());
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastMempoolConflict(msg)) if msg.contains("txn-mempool-conflict")
        ));
        mock.assert();
    }

    #[test]
    fn broadcast_tx_hex_maps_non_final_from_http_response() {
        let server = MockServer::start();

        let mock = server.mock(|when, then| {
            when.method(POST)
                .path("/tx")
                .body("deadbeef");
            then.status(400)
                .body("sendrawtransaction RPC error: {\"code\":-26,\"message\":\"non-BIP68-final\"}");
        });

        let broadcaster = EsploraBroadcaster::new(server.base_url());
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(result, Err(WalletSyncError::PsbtNotFinalized)));
        mock.assert();
    }

    #[test]
    fn classifies_mempool_conflict_error() {
        let err = EsploraBroadcaster::classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"txn-mempool-conflict\"}",
        );

        match err {
            WalletSyncError::BroadcastMempoolConflict(msg) => {
                assert!(msg.contains("txn-mempool-conflict"));
            }
            other => panic!("expected BroadcastMempoolConflict, got {:?}", other),
        }
    }

    #[test]
    fn classifies_already_confirmed_error() {
        let err = EsploraBroadcaster::classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-27,\"message\":\"transaction already in block chain\"}",
        );

        match err {
            WalletSyncError::BroadcastAlreadyConfirmed(msg) => {
                assert!(msg.contains("already in block chain"));
            }
            other => panic!("expected BroadcastAlreadyConfirmed, got {:?}", other),
        }
    }

    #[test]
    fn classifies_missing_inputs_error() {
        let err = EsploraBroadcaster::classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-25,\"message\":\"missing inputs\"}",
        );

        match err {
            WalletSyncError::BroadcastMissingInputs(msg) => {
                assert!(msg.contains("missing inputs"));
            }
            other => panic!("expected BroadcastMissingInputs, got {:?}", other),
        }
    }

    #[test]
    fn classifies_insufficient_fee_error() {
        let err = EsploraBroadcaster::classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"min relay fee not met\"}",
        );

        match err {
            WalletSyncError::BroadcastInsufficientFee(msg) => {
                assert!(msg.contains("min relay fee"));
            }
            other => panic!("expected BroadcastInsufficientFee, got {:?}", other),
        }
    }

    #[test]
    fn classifies_unknown_rejection_as_broadcast_failed() {
        let err = EsploraBroadcaster::classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "some unexpected rejection",
        );

        match err {
            WalletSyncError::BroadcastFailed(msg) => {
                assert!(msg.contains("status=400"));
                assert!(msg.contains("some unexpected rejection"));
            }
            other => panic!("expected BroadcastFailed, got {:?}", other),
        }
    }

    #[test]
    fn retries_and_succeeds_on_retryable_server_error() {
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
                    stream
                        .write_all(response.as_bytes())
                        .expect("write 503 response");
                } else {
                    let response =
                        "HTTP/1.1 200 OK\r\ncontent-length: 0\r\nconnection: close\r\n\r\n";
                    stream
                        .write_all(response.as_bytes())
                        .expect("write 200 response");
                }
                stream.flush().expect("flush response");
            }
        });

        let broadcaster = EsploraBroadcaster::new(format!("http://{}", addr));
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(result.is_ok(), "unexpected result: {:?}", result);
        handle.join().expect("server thread join");
    }
}