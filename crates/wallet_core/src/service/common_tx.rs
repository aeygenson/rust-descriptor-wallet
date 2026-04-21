use std::str::FromStr;

use bitcoin::consensus::encode::serialize_hex;
use bitcoin::psbt::Psbt;
use bitcoin::Transaction;

use bdk_wallet::{bitcoin::FeeRate, Wallet};
use bitcoin::Sequence;
use bitcoin::Txid;

use crate::model::{TxDirection, WalletSignedPsbtInfo};
use crate::{WalletCoreError, WalletCoreResult};

/// Parse a PSBT from its encoded string representation.
pub fn parse_psbt(psbt_base64: &str) -> WalletCoreResult<Psbt> {
    Psbt::from_str(psbt_base64).map_err(|e| WalletCoreError::InvalidPsbt(e.to_string()))
}

/// Check if PSBT is finalized (ready for broadcast)
pub fn is_psbt_finalized(psbt: &Psbt) -> bool {
    psbt.inputs
        .iter()
        .all(|input| input.final_script_sig.is_some() || input.final_script_witness.is_some())
}

/// Extract finalized transaction from PSBT
pub fn extract_finalized_tx(psbt: &Psbt) -> WalletCoreResult<Transaction> {
    if !is_psbt_finalized(psbt) {
        return Err(WalletCoreError::PsbtNotFinalized);
    }

    psbt.clone().extract_tx().map_err(|e| {
        WalletCoreError::ExtractTxFailed(format!("failed to extract tx from PSBT: {}", e))
    })
}

/// Build broadcast-oriented metadata from a finalized transaction.
/// Returns `(txid, tx_hex, replaceable)`.
pub fn finalized_tx_broadcast_info(tx: &Transaction) -> (String, String, bool) {
    let txid = tx.compute_txid().to_string();
    let tx_hex = serialize_hex(tx);
    let replaceable = is_rbf_enabled(tx);
    (txid, tx_hex, replaceable)
}

/// Build signed-PSBT metadata from a PSBT plus its original encoded form.
pub fn signed_psbt_info(
    psbt: &Psbt,
    original_psbt_base64: &str,
    finalized: bool,
) -> WalletSignedPsbtInfo {
    let txid = psbt.unsigned_tx.compute_txid().to_string();
    let psbt_base64 = psbt.to_string();
    let modified = psbt_base64 != original_psbt_base64;

    WalletSignedPsbtInfo {
        txid,
        psbt_base64,
        finalized,
        modified,
    }
}

/// Check if a transaction is explicitly RBF-enabled.
pub fn is_rbf_enabled(tx: &Transaction) -> bool {
    tx.input
        .iter()
        .any(|txin| txin.sequence.0 < Sequence::ENABLE_LOCKTIME_NO_RBF.0)
}

/// Standard sequence used for opt-in RBF transactions.
pub const RBF_SEQUENCE: Sequence = Sequence(0xFFFFFFFD);

/// Returns true when the requested fee rate is a strict bump over the original.
pub fn is_strict_fee_bump(original_sat_per_vb: u64, requested_sat_per_vb: u64) -> bool {
    requested_sat_per_vb > original_sat_per_vb
}

/// Estimate the original fee rate for a wallet transaction using the wallet graph.
pub fn estimate_original_fee_rate_sat_per_vb(
    wallet: &Wallet,
    txid: &Txid,
) -> WalletCoreResult<FeeRate> {
    let tx_node = wallet
        .get_tx(*txid)
        .ok_or_else(|| WalletCoreError::TransactionNotFound(txid.to_string()))?;

    let fee = wallet
        .calculate_fee(&tx_node.tx_node.tx)
        .map_err(|_| WalletCoreError::FeeCalculationFailed)?;

    let weight = tx_node.tx_node.tx.weight();
    let vbytes = weight.to_vbytes_ceil();

    if vbytes == 0 {
        return Err(WalletCoreError::FeeCalculationFailed);
    }

    FeeRate::from_sat_per_vb(fee.to_sat() as u64 / vbytes)
        .ok_or(WalletCoreError::FeeCalculationFailed)
}

/// Classify transaction direction from wallet-relative sent/received values.
pub fn classify_tx_direction(sent_sat: u64, received_sat: u64, net_value: i64) -> TxDirection {
    let has_wallet_inputs = sent_sat > 0;
    let has_wallet_outputs = received_sat > 0;

    if !has_wallet_inputs && has_wallet_outputs {
        TxDirection::Received
    } else if has_wallet_inputs && net_value < 0 {
        TxDirection::Sent
    } else if has_wallet_inputs && has_wallet_outputs {
        TxDirection::SelfTransfer
    } else {
        TxDirection::Sent
    }
}

/// Derive fee rate in sat/vB from fee and virtual size.
pub fn fee_rate_sat_per_vb_from_fee_and_vsize(fee_sat: u64, vsize: u64) -> u64 {
    if vsize == 0 {
        0
    } else {
        fee_sat.div_ceil(vsize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::test_support::load_test_wallet_with_db_prefix;
    use crate::service::test_support::test_support::UNSIGNED_TEST_PSBT;
    use bitcoin::{absolute, transaction, Amount, OutPoint, ScriptBuf, TxIn, TxOut, Witness};

    fn build_tx_with_sequence(sequence: Sequence) -> Transaction {
        Transaction {
            version: transaction::Version(2),
            lock_time: absolute::LockTime::ZERO,
            input: vec![TxIn {
                previous_output: OutPoint::null(),
                script_sig: ScriptBuf::new(),
                sequence,
                witness: Witness::new(),
            }],
            output: vec![TxOut {
                value: Amount::from_sat(1_000),
                script_pubkey: ScriptBuf::new(),
            }],
        }
    }

    #[test]
    fn parse_psbt_works() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        assert_eq!(psbt.inputs.len(), 1);
    }

    #[test]
    fn parse_psbt_fails_for_invalid_string() {
        let result = parse_psbt("not-a-psbt");

        assert!(matches!(result, Err(WalletCoreError::InvalidPsbt(_))));
    }

    #[test]
    fn finalized_detection() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        assert!(!is_psbt_finalized(&psbt));
    }

    #[test]
    fn extract_finalized_tx_fails_for_unfinalized() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let result = extract_finalized_tx(&psbt);
        assert!(matches!(result, Err(WalletCoreError::PsbtNotFinalized)));
    }

    #[test]
    fn finalized_tx_broadcast_info_works_with_manual_tx() {
        let tx = build_tx_with_sequence(Sequence::MAX);
        let (txid, tx_hex, replaceable) = finalized_tx_broadcast_info(&tx);

        assert_eq!(txid, tx.compute_txid().to_string());
        assert!(!tx_hex.is_empty());
        assert_eq!(replaceable, is_rbf_enabled(&tx));
    }

    #[test]
    fn signed_psbt_info_reports_txid_finalized_and_modified_state() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let info = signed_psbt_info(&psbt, "different-original", false);

        assert_eq!(info.txid, psbt.unsigned_tx.compute_txid().to_string());
        assert_eq!(info.psbt_base64, psbt.to_string());
        assert!(!info.finalized);
        assert!(info.modified);
    }

    #[test]
    fn signed_psbt_info_reports_unmodified_when_psbt_is_unchanged() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let original = psbt.to_string();
        let info = signed_psbt_info(&psbt, &original, true);

        assert!(info.finalized);
        assert!(!info.modified);
    }

    #[test]
    fn extract_fails_for_unsigned() {
        let psbt = parse_psbt(UNSIGNED_TEST_PSBT).unwrap();
        let result = extract_finalized_tx(&psbt);

        assert!(matches!(result, Err(WalletCoreError::PsbtNotFinalized)));
    }

    #[test]
    fn detects_rbf_enabled_transaction() {
        let tx = build_tx_with_sequence(RBF_SEQUENCE);
        assert!(is_rbf_enabled(&tx));
    }

    #[test]
    fn detects_non_rbf_transaction() {
        let tx = build_tx_with_sequence(Sequence::MAX);
        assert!(!is_rbf_enabled(&tx));
    }

    #[test]
    fn strict_fee_bump_requires_requested_rate_to_be_greater() {
        assert!(is_strict_fee_bump(5, 6));
        assert!(!is_strict_fee_bump(5, 5));
        assert!(!is_strict_fee_bump(5, 4));
    }

    #[test]
    fn rbf_sequence_constant_is_replaceable() {
        let tx = build_tx_with_sequence(RBF_SEQUENCE);
        assert!(is_rbf_enabled(&tx));
    }

    #[test]
    fn estimate_original_fee_rate_fails_for_missing_transaction() {
        let (_cfg, wallet) = load_test_wallet_with_db_prefix("wallet_core_tx_common_fee");
        let txid: Txid = "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
            .parse()
            .unwrap();

        let result = estimate_original_fee_rate_sat_per_vb(wallet.wallet(), &txid);

        assert!(matches!(
            result,
            Err(WalletCoreError::TransactionNotFound(_))
        ));
    }

    #[test]
    fn classify_tx_direction_reports_received() {
        assert_eq!(
            classify_tx_direction(0, 1_000, 1_000),
            TxDirection::Received
        );
    }

    #[test]
    fn classify_tx_direction_reports_sent() {
        assert_eq!(classify_tx_direction(2_000, 500, -1_500), TxDirection::Sent);
    }

    #[test]
    fn classify_tx_direction_reports_self_transfer() {
        assert_eq!(
            classify_tx_direction(2_000, 1_900, 0),
            TxDirection::SelfTransfer
        );
    }

    #[test]
    fn fee_rate_sat_per_vb_from_fee_and_vsize_handles_zero_vsize() {
        assert_eq!(fee_rate_sat_per_vb_from_fee_and_vsize(100, 0), 0);
    }

    #[test]
    fn fee_rate_sat_per_vb_from_fee_and_vsize_rounds_up() {
        assert_eq!(fee_rate_sat_per_vb_from_fee_and_vsize(101, 100), 2);
    }
}
