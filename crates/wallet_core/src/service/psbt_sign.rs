use tracing::{debug, info};

use crate::model::WalletSignedPsbtInfo;
use crate::service::common_tx::{parse_psbt, signed_psbt_info};
use crate::{WalletCoreError, WalletCoreResult};

use super::*;

fn log_psbt_inputs(stage: &str, psbt: &bitcoin::psbt::Psbt) {
    for (idx, input) in psbt.inputs.iter().enumerate() {
        debug!(
            "wallet_service: sign_psbt {} input={} partial_sigs={} tap_key_sig={} tap_script_sigs={} final_script_sig={} final_script_witness={} witness_utxo={} non_witness_utxo={} bip32_derivation={} tap_key_origins={} redeem_script={} witness_script={}",
            stage,
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
                "wallet_service: sign_psbt {} input={} tap_key_origin xonly={} leaf_hashes={} fingerprint={} derivation_path={}",
                stage,
                idx,
                xonly,
                leaf_hashes.len(),
                fingerprint,
                derivation_path
            );
        }
    }
}

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

        log_psbt_inputs("before", &psbt);

        // New signing API (no SignOptions). Returns whether the PSBT is finalized.
        let finalized = self.wallet.sign(&mut psbt, Default::default())?;

        log_psbt_inputs("after", &psbt);

        let info = signed_psbt_info(&psbt, &original_psbt_base64, finalized);
        let txid = info.txid.clone();
        let modified = info.modified;

        info!(
            "wallet_service: sign_psbt success inputs={} modified={} finalized={} txid={}",
            psbt.inputs.len(),
            modified,
            finalized,
            txid
        );

        Ok(info)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::PsbtSigningStatus;
    use crate::service::test_support::test_support::{
        load_wallet, signing_test_config_with_db_prefix, test_config_with_db_prefix,
        UNSIGNED_TEST_PSBT,
    };

    #[test]
    fn test_sign_psbt_success() {
        let config = signing_test_config_with_db_prefix("wallet_core_psbt_sign");
        let mut service = load_wallet(&config);

        let result = service.sign_psbt(UNSIGNED_TEST_PSBT).unwrap();

        assert!(result.modified, "PSBT not modified → signer missing");
        assert!(result.finalized, "PSBT not finalized → signing failed");
        assert_eq!(result.signing_status(), PsbtSigningStatus::Finalized);
    }

    #[test]
    fn test_sign_psbt_watch_only_returns_error() {
        let config = test_config_with_db_prefix("wallet_core_psbt_watch_only");
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
