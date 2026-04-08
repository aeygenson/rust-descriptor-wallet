use wallet_core::service::broadcast::TxBroadcaster;
use wallet_core::{WalletCoreError, WalletCoreResult};
use std::thread::sleep;
use std::time::Duration;

/// Broadcast raw transaction hex through an Esplora-compatible API.
///
/// Expected endpoint:
/// POST {base_url}/tx
/// body = raw transaction hex
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

    fn classify_esplora_rejection(status: reqwest::StatusCode, body: &str) -> WalletCoreError {
        let normalized = body.to_ascii_lowercase();

        if normalized.contains("txn-mempool-conflict")
            || normalized.contains("mempool conflict")
        {
            return WalletCoreError::BroadcastMempoolConflict(body.to_string());
        }

        if normalized.contains("already in block chain")
            || normalized.contains("already confirmed")
            || normalized.contains("transaction already in block chain")
        {
            return WalletCoreError::BroadcastAlreadyConfirmed(body.to_string());
        }

        if normalized.contains("missing inputs") {
            return WalletCoreError::BroadcastMissingInputs(body.to_string());
        }

        if normalized.contains("non-bip68-final")
            || normalized.contains("non-final")
        {
            return WalletCoreError::PsbtNotFinalized;
        }

        if normalized.contains("min relay fee")
            || normalized.contains("insufficient fee")
            || normalized.contains("fee not met")
        {
            return WalletCoreError::BroadcastInsufficientFee(body.to_string());
        }

        WalletCoreError::BroadcastFailed(format!(
            "esplora rejected transaction: status={} body={}",
            status, body
        ))
    }
}

impl TxBroadcaster for EsploraBroadcaster {
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletCoreResult<()> {
        let endpoint = self.tx_endpoint();
        let mut last_error: Option<WalletCoreError> = None;

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
                    let err = WalletCoreError::BroadcastTransport(e.to_string());

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
            WalletCoreError::BroadcastFailed("unknown broadcast failure".to_string())
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use httpmock::Method::POST;
    use httpmock::MockServer;

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
            Err(WalletCoreError::BroadcastMempoolConflict(msg)) if msg.contains("txn-mempool-conflict")
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

        assert!(matches!(result, Err(WalletCoreError::PsbtNotFinalized)));
        mock.assert();
    }
}
    #[test]
    fn classifies_mempool_conflict_error() {
        let err = EsploraBroadcaster::classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"txn-mempool-conflict\"}",
        );

        match err {
            WalletCoreError::BroadcastMempoolConflict(msg) => {
                assert!(msg.contains("txn-mempool-conflict"));
            }
            other => panic!("expected BroadcastMempoolConflict, got {:?}", other),
        }
    }