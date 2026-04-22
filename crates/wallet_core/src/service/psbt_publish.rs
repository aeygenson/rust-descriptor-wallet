use tracing::{debug, info};

use crate::model::WalletFinalizedTxInfo;
use crate::service::common_tx::{extract_finalized_tx, finalized_tx_broadcast_info};
use crate::types::PsbtBase64;
use crate::WalletCoreResult;

use super::*;

impl WalletService {
    /// Parse a finalized PSBT, extract the fully signed transaction, and
    /// return the finalized transaction data needed for broadcasting.
    pub fn finalize_psbt_for_broadcast(
        &self,
        psbt_base64: &PsbtBase64,
    ) -> WalletCoreResult<WalletFinalizedTxInfo> {
        debug!("wallet_service: finalize_psbt_for_broadcast start");

        let psbt = psbt_base64.to_psbt()?;
        let tx = extract_finalized_tx(&psbt)?;
        let (txid, tx_hex, replaceable) = finalized_tx_broadcast_info(&tx);

        info!(
            "wallet_service: finalize_psbt_for_broadcast prepared finalized transaction txid={} hex_len={}",
            txid,
            tx_hex.as_str().len()
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
    use crate::model::WalletSignedPsbtInfo;
    use crate::service::common_test_util::test_support::{
        test_config, unique_test_db_path, UNSIGNED_TEST_PSBT,
    };
    use crate::service::common_tx::RBF_SEQUENCE;
    use crate::service::common_tx::{is_psbt_finalized, parse_psbt};
    use crate::types::WalletTxid;
    use crate::WalletCoreError;
    use bitcoin::psbt::Psbt;
    use bitcoin::{absolute, transaction, Amount, OutPoint, ScriptBuf, TxIn, TxOut, Witness};

    /// Build a finalized PSBT fixture for publish/finalization tests.
    ///
    /// This file does not participate in wallet-core coin-control or typed
    /// outpoint selection flows. The only `OutPoint` usage here is the raw
    /// Bitcoin `OutPoint::null()` used to construct a standalone synthetic
    /// transaction fixture for PSBT extraction tests.
    fn finalized_test_psbt_base64() -> PsbtBase64 {
        let unsigned = bitcoin::Transaction {
            version: transaction::Version(2),
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence: RBF_SEQUENCE,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(1_000),
                script_pubkey: ScriptBuf::new(),
            }],
        };

        let mut psbt =
            Psbt::from_unsigned_tx(unsigned).expect("unsigned tx should build into psbt");

        // Provide required UTXO info so extract_tx() works
        psbt.inputs[0].witness_utxo = Some(TxOut {
            value: Amount::from_sat(1_000),
            script_pubkey: ScriptBuf::new(),
        });

        // Mark as finalized
        psbt.inputs[0].final_script_witness = Some(Witness::new());
        PsbtBase64::from(psbt.to_string())
    }

    #[test]
    fn finalized_psbt_is_detected_as_publishable() {
        let finalized_psbt = finalized_test_psbt_base64();
        let psbt = finalized_psbt.to_psbt().unwrap();
        assert!(is_psbt_finalized(&psbt));
    }

    #[test]
    fn unsigned_psbt_is_rejected_for_publish() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let result = extract_finalized_tx(&psbt);

        assert!(matches!(result, Err(WalletCoreError::PsbtNotFinalized)));
    }

    #[test]
    fn finalized_psbt_extracts_transaction() {
        let finalized_psbt = finalized_test_psbt_base64();
        let psbt = finalized_psbt.to_psbt().unwrap();
        let tx = extract_finalized_tx(&psbt).unwrap();

        assert_eq!(
            tx.compute_txid().to_string(),
            psbt.unsigned_tx.compute_txid().to_string()
        );
    }

    #[test]
    fn finalized_psbt_produces_broadcast_payload() {
        let mut config = test_config();
        config.db_path = unique_test_db_path("wallet_core_finalize_publish");
        let service = WalletService::load_or_create(&config)
            .expect("watch-only wallet should load for finalize test");

        let finalized_psbt = finalized_test_psbt_base64();
        let parsed = finalized_psbt.to_psbt().unwrap();
        let expected_txid = WalletTxid::from(parsed.unsigned_tx.compute_txid());

        let result = service.finalize_psbt_for_broadcast(&finalized_psbt);

        assert!(result.is_ok());
        let finalized = result.unwrap();
        assert_eq!(finalized.txid, expected_txid);
        assert!(finalized.replaceable);
        assert!(!finalized.tx_hex.as_str().is_empty());
    }

    #[test]
    fn signed_model_reports_finalized_before_publish() {
        let info = WalletSignedPsbtInfo {
            psbt_base64: finalized_test_psbt_base64(),
            modified: true,
            finalized: true,
            txid: WalletTxid::parse(
                "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d",
            )
            .unwrap(),
        };

        assert!(matches!(
            info.signing_status(),
            crate::model::PsbtSigningStatus::Finalized
        ));
    }
}
