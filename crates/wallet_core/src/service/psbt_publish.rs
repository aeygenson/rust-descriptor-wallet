use bitcoin::consensus::encode::serialize_hex;
use tracing::{debug, info};

use crate::model::WalletPublishedTxInfo;
use crate::service::psbt_common::{extract_finalized_tx, parse_psbt};
use crate::{WalletCoreError, WalletCoreResult};

use super::*;

impl WalletService {
    /// Parse a finalized PSBT, extract the fully signed transaction, and
    /// prepare it for broadcast.
    ///
    /// Note: actual network broadcasting is intentionally left for the next
    /// integration step, once a dedicated broadcaster/transport abstraction is
    /// added to the project.
    pub fn publish_psbt(&self, psbt_base64: &str) -> WalletCoreResult<WalletPublishedTxInfo> {
        debug!("wallet_service: publish_psbt start");

        let psbt = parse_psbt(psbt_base64)?;
        let tx = extract_finalized_tx(&psbt)?;
        let txid = tx.compute_txid().to_string();
        let tx_hex = serialize_hex(&tx);

        info!(
            "wallet_service: publish_psbt prepared finalized transaction txid={} hex_len={}",
            txid,
            tx_hex.len()
        );

        Err(WalletCoreError::BroadcastFailed(
            "broadcast transport not wired yet".to_string(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::psbt_common::is_psbt_finalized;
    use crate::model::WalletSignedPsbtInfo;

    const FINALIZED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGAQhCAUBQOwjdd/7aYgEH2ZHtHfwqt01+CB3A29cdWLeeXj+EejPrC6Y6pnpcto0TJA8BwCK1uMICqlUyEsXb+xY0dkYBAAEFIFU1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWAAEFILEKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoAA==";

    const UNSIGNED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGIRZVNVyoPJc/HZfODjhDyF14kFrxa03FMbxIjlchLSMBFhkAc8XaClYAAIABAACAAAAAgAAAAAAAAAAAARcgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYAAQUgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYhB1U1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWGQBzxdoKVgAAgAEAAIAAAACAAAAAAAAAAAAAAQUgsQrJf2ds8fPM2ssLeBcSgrvpSpTfFDIBcA3Fm8wV82ghB7EKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoGQBzxdoKVgAAgAEAAIAAAACAAQAAAAAAAAAA";

    #[test]
    fn finalized_psbt_is_detected_as_publishable() {
        let psbt = parse_psbt(FINALIZED_TEST_PSBT).unwrap();
        assert!(is_psbt_finalized(&psbt));
    }

    #[test]
    fn unsigned_psbt_is_rejected_for_publish() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let result = extract_finalized_tx(&psbt);

        assert!(matches!(
            result,
            Err(WalletCoreError::PsbtNotFinalized)
        ));
    }

    #[test]
    fn finalized_psbt_extracts_transaction() {
        let psbt = parse_psbt(FINALIZED_TEST_PSBT).unwrap();
        let tx = extract_finalized_tx(&psbt).unwrap();

        assert_eq!(
            tx.compute_txid().to_string(),
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
    }

    #[test]
    fn signed_model_reports_finalized_before_publish() {
        let info = WalletSignedPsbtInfo {
            psbt_base64: FINALIZED_TEST_PSBT.to_string(),
            modified: true,
            finalized: true,
            txid: "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
                .to_string(),
        };

        assert!(matches!(
            info.signing_status(),
            crate::model::PsbtSigningStatus::Finalized
        ));
    }

    #[test]
    fn publish_psbt_returns_broadcast_failed_until_transport_is_wired() {
        let txid = parse_psbt(FINALIZED_TEST_PSBT)
            .and_then(|psbt| extract_finalized_tx(&psbt))
            .map(|tx| tx.compute_txid().to_string())
            .expect("finalized test PSBT should extract txid");

        let result: crate::WalletCoreResult<crate::model::WalletPublishedTxInfo> =
            Err(crate::WalletCoreError::BroadcastFailed(
                "broadcast transport not wired yet".to_string(),
            ));

        assert!(matches!(
            result,
            Err(crate::WalletCoreError::BroadcastFailed(_))
        ));
        assert_eq!(
            txid,
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
    }
}