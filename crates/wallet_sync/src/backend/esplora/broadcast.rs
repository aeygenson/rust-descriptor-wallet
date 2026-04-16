use super::classify::classify_esplora_rejection;
use crate::broadcast::TxBroadcaster;
use crate::{WalletSyncError, WalletSyncResult};
use std::thread::sleep;
use std::time::Duration;
use tracing::{debug, error, info, warn};

/// Esplora-backed broadcaster for fully signed raw transactions.
///
/// This backend implementation is internal to `wallet_sync`. Higher layers
/// should use the sync facade from `service.rs` instead of depending on this
/// type directly unless they explicitly need backend-specific behavior.
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
        info!("broadcasting transaction via Esplora");
        debug!("endpoint = {}", endpoint);
        let mut last_error: Option<WalletSyncError> = None;

        for attempt in 0..=self.max_retries {
            debug!("broadcast attempt {}", attempt);
            let response = self
                .client
                .post(&endpoint)
                .header(reqwest::header::CONTENT_TYPE, "text/plain")
                .body(tx_hex.to_string())
                .send();

            match response {
                Ok(resp) if resp.status().is_success() => {
                    info!("transaction broadcast successful");
                    return Ok(());
                }
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp
                        .text()
                        .unwrap_or_else(|_| "<unable to read response body>".to_string());
                    warn!(
                        "http error during broadcast: status = {}, body = {}",
                        status, body
                    );
                    let err = classify_esplora_rejection(status, &body);
                    debug!("classified error = {:?}", err);

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
            when.method(POST).path("/tx").body("deadbeef");
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
            when.method(POST).path("/tx").body("deadbeef");
            then.status(400).body(
                "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"txn-mempool-conflict\"}",
            );
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
            when.method(POST).path("/tx").body("deadbeef");
            then.status(400).body(
                "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"non-BIP68-final\"}",
            );
        });

        let broadcaster = EsploraBroadcaster::new(server.base_url());
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(result, Err(WalletSyncError::PsbtNotFinalized)));
        mock.assert();
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
