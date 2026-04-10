use tracing::{debug, info};

use crate::model::WalletSignedPsbtInfo;
use crate::service::psbt_common::parse_psbt;
use crate::{WalletCoreError, WalletCoreResult};

use super::*;

impl WalletService {
    /// Sign an existing PSBT using the wallet's configured signers.
    ///
    /// Watch-only wallets cannot sign and return `WalletCoreError::WatchOnlyCannotSign`.
    pub fn sign_psbt(&mut self, psbt_base64: &str) -> WalletCoreResult<WalletSignedPsbtInfo> {
        debug!("wallet_service: sign_psbt start");

        let mut psbt = parse_psbt(psbt_base64)?;
        let original_psbt_base64 = psbt.to_string();
        if self.is_watch_only {
            return Err(WalletCoreError::WatchOnlyCannotSign);
        }

        debug!(
            "wallet_service: sign_psbt wallet_context external_descriptor=<unavailable> internal_descriptor=<unavailable>"
        );

        for (idx, input) in psbt.inputs.iter().enumerate() {
            debug!(
                "wallet_service: sign_psbt before input={} partial_sigs={} tap_key_sig={} tap_script_sigs={} final_script_sig={} final_script_witness={} witness_utxo={} non_witness_utxo={} bip32_derivation={} tap_key_origins={} redeem_script={} witness_script={}",
                idx,
                input.partial_sigs.len(),
                input.tap_key_sig.is_some(),
                input.tap_script_sigs.len(),
                input.final_script_sig.is_some(),
                input.final_script_witness.is_some(),
                input.witness_utxo.is_some(),
                input.non_witness_utxo.is_some(),
                input.bip32_derivation.len(),
                input.tap_key_origins.len(),
                input.redeem_script.is_some(),
                input.witness_script.is_some()
            );
            for (xonly, (leaf_hashes, (fingerprint, derivation_path))) in &input.tap_key_origins {
                debug!(
                    "wallet_service: sign_psbt before input={} tap_key_origin xonly={} leaf_hashes={} fingerprint={} derivation_path={}",
                    idx,
                    xonly,
                    leaf_hashes.len(),
                    fingerprint,
                    derivation_path
                );
            }
        }

        // New signing API (no SignOptions). Returns whether the PSBT is finalized.
        let finalized = self
            .wallet
            .sign(&mut psbt, Default::default())?;

        for (idx, input) in psbt.inputs.iter().enumerate() {
            debug!(
                "wallet_service: sign_psbt after input={} partial_sigs={} tap_key_sig={} tap_script_sigs={} final_script_sig={} final_script_witness={} witness_utxo={} non_witness_utxo={} bip32_derivation={} tap_key_origins={} redeem_script={} witness_script={}",
                idx,
                input.partial_sigs.len(),
                input.tap_key_sig.is_some(),
                input.tap_script_sigs.len(),
                input.final_script_sig.is_some(),
                input.final_script_witness.is_some(),
                input.witness_utxo.is_some(),
                input.non_witness_utxo.is_some(),
                input.bip32_derivation.len(),
                input.tap_key_origins.len(),
                input.redeem_script.is_some(),
                input.witness_script.is_some()
            );
            for (xonly, (leaf_hashes, (fingerprint, derivation_path))) in &input.tap_key_origins {
                debug!(
                    "wallet_service: sign_psbt after input={} tap_key_origin xonly={} leaf_hashes={} fingerprint={} derivation_path={}",
                    idx,
                    xonly,
                    leaf_hashes.len(),
                    fingerprint,
                    derivation_path
                );
            }
        }

        let txid = psbt.unsigned_tx.compute_txid().to_string();
        let psbt_base64 = psbt.to_string();
        let modified = psbt_base64 != original_psbt_base64;

        info!(
            "wallet_service: sign_psbt success inputs={} modified={} finalized={} txid={}",
            psbt.inputs.len(),
            modified,
            finalized,
            txid
        );

        Ok(WalletSignedPsbtInfo {
            psbt_base64,
            modified,
            finalized,
            txid,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::config::WalletConfig;
    use crate::model::{PsbtSigningStatus, WalletSignedPsbtInfo};

    const SIGNING_EXTERNAL_DESCRIPTOR: &str = "tr([73c5da0a/86'/1'/0']tprv8gytrHbFLhE7zLJ6BvZWEDDGJe8aS8VrmFnvqpMv8CEZtUbn2NY5KoRKQNpkcL1yniyCBRi7dAPy4kUxHkcSvd9jzLmLMEG96TPwant2jbX/0/*)#ps8nx7gn";
    const SIGNING_INTERNAL_DESCRIPTOR: &str = "tr([73c5da0a/86'/1'/0']tprv8gytrHbFLhE7zLJ6BvZWEDDGJe8aS8VrmFnvqpMv8CEZtUbn2NY5KoRKQNpkcL1yniyCBRi7dAPy4kUxHkcSvd9jzLmLMEG96TPwant2jbX/1/*)#syzjmtct";

    const WATCH_ONLY_EXTERNAL_DESCRIPTOR: &str = "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m";
    const WATCH_ONLY_INTERNAL_DESCRIPTOR: &str = "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr";

    const UNSIGNED_TEST_PSBT: &str = "cHNidP8BAIkCAAAAAc9GHAJ+0qYu4xXAbjEeNofTV2iW7wrR9V5VGybv5cMaAgAAAAD9////AugDAAAAAAAAIlEgO4KysqkYUxXab4DaXwbQRA2KXhRX+pM4fC2RnIbsh4aNIgAAAAAAACJRINc6z2Znt4UObgDiG7RSWixeLYiVaj0sNbC8BvSw3wG8+sMtAAABASsQJwAAAAAAACJRIDuCsrKpGFMV2m+A2l8G0EQNil4UV/qTOHwtkZyG7IeGIRZVNVyoPJc/HZfODjhDyF14kFrxa03FMbxIjlchLSMBFhkAc8XaClYAAIABAACAAAAAgAAAAAAAAAAAARcgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYAAQUgVTVcqDyXPx2Xzg44Q8hdeJBa8WtNxTG8SI5XIS0jARYhB1U1XKg8lz8dl84OOEPIXXiQWvFrTcUxvEiOVyEtIwEWGQBzxdoKVgAAgAEAAIAAAACAAAAAAAAAAAAAAQUgsQrJf2ds8fPM2ssLeBcSgrvpSpTfFDIBcA3Fm8wV82ghB7EKyX9nbPHzzNrLC3gXEoK76UqU3xQyAXANxZvMFfNoGQBzxdoKVgAAgAEAAIAAAACAAQAAAAAAAAAA";

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn unique_test_db_path(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let counter = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);
        let pid = std::process::id();
        std::env::temp_dir().join(format!(
            "{}_{}_{}_{}.wallet.db",
            name, pid, nanos, counter
        ))
    }

    fn signing_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            external_descriptor: SIGNING_EXTERNAL_DESCRIPTOR.to_string(),
            internal_descriptor: SIGNING_INTERNAL_DESCRIPTOR.to_string(),
            db_path: unique_test_db_path("wallet_core_psbt_sign"),
            esplora_url: "https://mutinynet.com/api".to_string(),
            is_watch_only: false,
        }
    }

    fn watch_only_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            external_descriptor: WATCH_ONLY_EXTERNAL_DESCRIPTOR.to_string(),
            internal_descriptor: WATCH_ONLY_INTERNAL_DESCRIPTOR.to_string(),
            db_path: unique_test_db_path("wallet_core_psbt_watch_only"),
            esplora_url: "https://mempool.space/signet/api".to_string(),
            is_watch_only: true,
        }
    }

    fn load_wallet(config: &WalletConfig) -> WalletService {
        WalletService::load_or_create(config).unwrap()
    }

    #[test]
    fn test_sign_psbt_success() {
        let config = signing_config();
        let mut service = load_wallet(&config);

        let result = service.sign_psbt(UNSIGNED_TEST_PSBT).unwrap();

        assert!(result.modified, "PSBT not modified → signer missing");
        assert!(result.finalized, "PSBT not finalized → signing failed");
        assert_eq!(result.signing_status(), PsbtSigningStatus::Finalized);
    }

    #[test]
    fn test_sign_psbt_watch_only_returns_error() {
        let config = watch_only_config();
        let mut service = load_wallet(&config);

        let result = service.sign_psbt(UNSIGNED_TEST_PSBT);

        assert!(matches!(result, Err(WalletCoreError::WatchOnlyCannotSign)));
    }

    #[test]
    fn test_signing_status_helper() {
        let info = WalletSignedPsbtInfo {
            psbt_base64: "dummy".to_string(),
            modified: true,
            finalized: true,
            txid: "dummy".to_string(),
        };

        assert_eq!(info.signing_status(), PsbtSigningStatus::Finalized);
    }
}