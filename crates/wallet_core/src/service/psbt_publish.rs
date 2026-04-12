use bitcoin::consensus::encode::serialize_hex;
use tracing::{debug, info};

use crate::model::{WalletFinalizedTxInfo};
use crate::service::psbt_common::{extract_finalized_tx, parse_psbt};
use crate::WalletCoreResult;

use super::*;

impl WalletService {
    /// Parse a finalized PSBT, extract the fully signed transaction, and
    /// return the finalized transaction data needed for broadcasting.
    pub fn finalize_psbt_for_broadcast(
        &self,
        psbt_base64: &str,
    ) -> WalletCoreResult<WalletFinalizedTxInfo> {
        debug!("wallet_service: finalize_psbt_for_broadcast start");

        let psbt = parse_psbt(psbt_base64)?;
        let tx = extract_finalized_tx(&psbt)?;
        let txid = tx.compute_txid().to_string();
        let tx_hex = serialize_hex(&tx);
        let replaceable = tx.is_explicitly_rbf();

        info!(
            "wallet_service: finalize_psbt_for_broadcast prepared finalized transaction txid={} hex_len={}",
            txid,
            tx_hex.len()
        );

        Ok(WalletFinalizedTxInfo {
            txid,
            tx_hex,
            replaceable,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BroadcastBackendConfig, SyncBackendConfig, WalletBackendConfig, WalletDescriptors,
    };
    use crate::model::WalletSignedPsbtInfo;
    use crate::service::psbt_common::is_psbt_finalized;
    use crate::WalletCoreError;
    const FINALIZED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGAQhCAUBQOwjdd/7aYgEH2ZHtHfwqt01+CB3A29cdWLeeXj+EejPrC6Y6pnpcto0TJA8BwCK1uMICqlUyEsXb+xY0dkYBAAEFIFU1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWAAEFILEKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoAA==";

    const UNSIGNED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGIRZVNVyoPJc/HZfODjhDyF14kFrxa03FMbxIjlchLSMBFhkAc8XaClYAAIABAACAAAAAgAAAAAAAAAAAARcgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYAAQUgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYhB1U1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWGQBzxdoKVgAAgAEAAIAAAACAAAAAAAAAAAAAAQUgsQrJf2ds8fPM2ssLeBcSgrvpSpTfFDIBcA3Fm8wV82ghB7EKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoGQBzxdoKVgAAgAEAAIAAAACAAQAAAAAAAAAA";

    fn unique_db_path(prefix: &str) -> std::path::PathBuf {
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .expect("system time before unix epoch")
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}_{}.db", prefix, std::process::id(), nanos))
    }

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
    fn finalized_psbt_produces_broadcast_payload() {
        let service = WalletService::load_or_create(&crate::config::WalletConfig {
            network: bitcoin::Network::Signet,
            descriptors: WalletDescriptors {
                external: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
                internal: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            },
            backend: WalletBackendConfig {
                sync: SyncBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                },
                broadcast: Some(BroadcastBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                }),
            },
            db_path: unique_db_path("wallet_core_finalize_publish"),
            is_watch_only: true,
        })
        .expect("watch-only wallet should load for finalize test");

        let result = service.finalize_psbt_for_broadcast(FINALIZED_TEST_PSBT);

        assert!(result.is_ok());
        let finalized = result.unwrap();
        assert_eq!(
            finalized.txid,
            "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
        );
        assert!(finalized.replaceable);
        assert!(!finalized.tx_hex.is_empty());
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
}