use super::{
    psbt_common::{is_rbf_enabled, parse_txid},
    *,
};

use bdk_wallet::{
    bitcoin::{FeeRate, Sequence, Txid},
    Wallet,
};

use crate::{
    error::WalletCoreError, model::WalletPsbtInfo, types::FeeRateSatPerVb, WalletCoreResult,
};

/// Canonical opt-in RBF sequence used by the wallet when creating replaceable
/// transactions. Keeping the same value here ensures bump transactions remain
/// explicitly replaceable as well.
const RBF_SEQUENCE: Sequence = Sequence(0xFFFFFFFD);

impl WalletService {
    /// Build a replacement PSBT for an existing unconfirmed, RBF-enabled
    /// transaction using a higher fee rate.
    ///
    /// Notes:
    /// - This function is intentionally synchronous and pure wallet-core logic.
    /// - It validates existence, confirmation status, and replaceability before
    ///   delegating to BDK's fee-bump builder.
    /// - `WalletPsbtInfo::from_psbt_minimal(...)` is expected to be the same conversion path
    ///   used by your existing create/send PSBT flow.
    pub fn bump_fee_psbt(
        &mut self,
        txid: &str,
        new_fee_rate_sat_per_vb: FeeRateSatPerVb,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        let txid = parse_txid(txid)?;

        let tx_node = self
            .wallet
            .transactions()
            .find(|canonical_tx| canonical_tx.tx_node.txid == txid)
            .ok_or(WalletCoreError::TransactionNotFound(txid.to_string()))?;

        if tx_node.chain_position.is_confirmed() {
            return Err(WalletCoreError::TransactionAlreadyConfirmed(
                txid.to_string(),
            ));
        }

        let original_tx = &tx_node.tx_node.tx;

        if !is_rbf_enabled(original_tx) {
            return Err(WalletCoreError::TransactionNotReplaceable(txid.to_string()));
        }

        let original_fee_rate = estimate_original_fee_rate_sat_per_vb(&self.wallet, &txid)?;
        let new_fee_rate = FeeRate::try_from(new_fee_rate_sat_per_vb)?;

        let original_sat_per_vb = FeeRateSatPerVb::from(original_fee_rate);
        let requested_sat_per_vb = new_fee_rate_sat_per_vb;

        if !is_strict_fee_bump(original_sat_per_vb, requested_sat_per_vb) {
            return Err(WalletCoreError::FeeRateTooLowForBump {
                txid: txid.to_string(),
                original_sat_per_vb,
                requested_sat_per_vb,
            });
        }

        let mut builder = self.wallet.build_fee_bump(txid).map_err(|source| {
            WalletCoreError::FeeBumpBuildFailed {
                txid: txid.to_string(),
                reason: source.to_string(),
            }
        })?;

        // Preserve explicit opt-in RBF semantics on the replacement transaction.
        builder.set_exact_sequence(RBF_SEQUENCE);
        builder.fee_rate(new_fee_rate);

        let psbt = builder
            .finish()
            .map_err(|source| WalletCoreError::FeeBumpBuildFailed {
                txid: txid.to_string(),
                reason: source.to_string(),
            })?;

        let original_txid = txid.to_string();
        let mut info = WalletPsbtInfo::from_psbt_minimal(psbt).map_err(|source| {
            WalletCoreError::PsbtConversionFailed {
                txid: original_txid.clone(),
                reason: source.to_string(),
            }
        })?;
        info.original_txid = Some(original_txid);
        Ok(info)
    }
}

fn is_strict_fee_bump(
    original_sat_per_vb: FeeRateSatPerVb,
    requested_sat_per_vb: FeeRateSatPerVb,
) -> bool {
    requested_sat_per_vb.as_u64() > original_sat_per_vb.as_u64()
}

/// Estimate the original fee rate from the wallet graph.
///
/// This implementation uses wallet-derived fee/vsize data when available.
/// If your existing code already has a canonical "tx summary/details" path,
/// prefer routing through that instead so all fee calculations stay consistent.
fn estimate_original_fee_rate_sat_per_vb(
    wallet: &Wallet,
    txid: &Txid,
) -> WalletCoreResult<FeeRate> {
    let tx_node = wallet
        .transactions()
        .find(|canonical_tx| canonical_tx.tx_node.txid == *txid)
        .ok_or(WalletCoreError::TransactionNotFound(txid.to_string()))?;

    let fee = wallet
        .calculate_fee(&tx_node.tx_node.tx)
        .map_err(|source| WalletCoreError::TransactionFeeUnavailable {
            txid: txid.to_string(),
            reason: source.to_string(),
        })?;

    let vbytes = tx_node.tx_node.tx.vsize() as f32;
    if vbytes <= 0.0 {
        return Err(WalletCoreError::TransactionVsizeUnavailable(
            txid.to_string(),
        ));
    }

    let sat_per_vb = (fee.to_sat() as f32 / vbytes).ceil() as u64;
    FeeRate::from_sat_per_vb(sat_per_vb).ok_or(WalletCoreError::TransactionFeeUnavailable {
        txid: txid.to_string(),
        reason: format!("invalid derived fee rate: {sat_per_vb} sat/vB"),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::FeeRateSatPerVb;
    use bdk_wallet::bitcoin::{
        absolute, transaction, Amount, OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Witness,
    };

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
    fn detects_rbf_enabled_transaction() {
        let tx = build_tx_with_sequence(RBF_SEQUENCE);
        assert!(super::is_rbf_enabled(&tx));
    }

    #[test]
    fn detects_non_rbf_transaction() {
        let tx = build_tx_with_sequence(Sequence::MAX);
        assert!(!super::is_rbf_enabled(&tx));
    }

    #[test]
    fn strict_fee_bump_accepts_higher_fee_rate() {
        let original = FeeRateSatPerVb::from(2);
        let requested = FeeRateSatPerVb::from(5);

        assert!(is_strict_fee_bump(original, requested));
    }

    #[test]
    fn strict_fee_bump_rejects_equal_fee_rate() {
        let original = FeeRateSatPerVb::from(5);
        let requested = FeeRateSatPerVb::from(5);

        assert!(!is_strict_fee_bump(original, requested));
    }

    #[test]
    fn strict_fee_bump_rejects_lower_fee_rate() {
        let original = FeeRateSatPerVb::from(5);
        let requested = FeeRateSatPerVb::from(2);

        assert!(!is_strict_fee_bump(original, requested));
    }
}

// INTEGRATION NOTES
// -----------------
// Add or confirm the following WalletCoreError variants in your central error enum:
// - InvalidTxid(String)
// - TransactionNotFound(String)
// - TransactionAlreadyConfirmed(String)
// - TransactionNotReplaceable(String)
// - InvalidFeeRate
// - FeeRateTooLowForBump {
//     txid: String,
//     original_sat_per_vb: FeeRateSatPerVb,
//     requested_sat_per_vb: FeeRateSatPerVb,
//   }
// - FeeBumpBuildFailed { txid: String, reason: String }
// - TransactionFeeUnavailable { txid: String, reason: String }
// - TransactionVsizeUnavailable { txid: String }
// - PsbtConversionFailed { txid: String, reason: String }
//
// Expected model integration points:
// - FeeRateSatPerVb should expose an accessor returning sat/vB, such as `as_u64()`.
// - WalletPsbtInfo should expose `from_psbt_minimal(psbt) -> WalletCoreResult<Self>` or equivalent.
// - If your repository uses a wallet wrapper type instead of `bdk_wallet::Wallet`, adapt the
//   function receiver accordingly and keep this internal logic unchanged.
