use tracing::{debug, info};

use bdk_wallet::bitcoin::Amount;
use bdk_wallet::KeychainKind;

// no need for parse_wallet_outpoint, use WalletOutPoint::parse directly
use crate::error::WalletCoreError;
use crate::model::{WalletCpfpBuildPlanInfo, WalletCpfpPsbtInfo};
use crate::types::{FeeRateSatPerVb, PsbtBase64, VSize, WalletOutPoint, WalletTxid};
use crate::{WalletCoreResult, WalletService};

const CPFP_INPUT_VBYTES: u64 = 58;
const CPFP_OUTPUT_VBYTES: u64 = 43;
const CPFP_OVERHEAD_VBYTES: u64 = 11;

/// Create a CPFP PSBT for a given parent transaction.
///
/// This is a minimal scaffold implementation. It does NOT yet build
/// a real transaction — it only defines the flow and logs steps.
impl WalletService {
    fn estimate_cpfp_vsize() -> VSize {
        VSize::from(CPFP_INPUT_VBYTES + CPFP_OUTPUT_VBYTES + CPFP_OVERHEAD_VBYTES)
    }

    fn build_cpfp_plan(
        parent_txid: &str,
        selected_outpoint: &WalletOutPoint,
        input_value_sat: u64,
        fee_rate_sat_per_vb: u64,
    ) -> WalletCoreResult<WalletCpfpBuildPlanInfo> {
        let estimated_vsize = Self::estimate_cpfp_vsize();
        let fee_sat = fee_rate_sat_per_vb
            .checked_mul(estimated_vsize.as_u64())
            .ok_or_else(|| WalletCoreError::CpfpBuildFailed {
                parent_txid: parent_txid.to_string(),
                reason: "fee calculation overflow".to_string(),
            })?;

        if fee_sat >= input_value_sat {
            return Err(WalletCoreError::CpfpInsufficientValue(
                selected_outpoint.to_string(),
            ));
        }

        let child_output_value_sat = input_value_sat - fee_sat;

        Ok(WalletCpfpBuildPlanInfo {
            input_outpoint: *selected_outpoint,
            input_value_sat: crate::types::AmountSat(input_value_sat),
            child_output_value_sat: crate::types::AmountSat(child_output_value_sat),
            fee_sat: crate::types::AmountSat(fee_sat),
            estimated_vsize,
        })
    }

    fn build_cpfp_psbt_from_plan(
        &mut self,
        parent_txid: &str,
        build_plan: &WalletCpfpBuildPlanInfo,
    ) -> WalletCoreResult<(PsbtBase64, WalletTxid)> {
        // Convert the typed wallet-domain outpoint into the raw Bitcoin outpoint
        // only at the BDK transaction-builder boundary.
        let outpoint = bitcoin::OutPoint::from(build_plan.input_outpoint);

        let internal_addr = self.wallet.peek_address(KeychainKind::Internal, 0);

        let mut builder = self.wallet.build_tx();
        builder.manually_selected_only();
        builder
            .add_utxo(outpoint)
            .map_err(|e| WalletCoreError::CpfpBuildFailed {
                parent_txid: parent_txid.to_string(),
                reason: e.to_string(),
            })?;
        builder.add_recipient(
            internal_addr.address.script_pubkey(),
            Amount::from_sat(build_plan.child_output_value_sat.0),
        );
        builder.fee_absolute(Amount::from_sat(build_plan.fee_sat.0));

        let psbt = builder
            .finish()
            .map_err(|e| WalletCoreError::CpfpBuildFailed {
                parent_txid: parent_txid.to_string(),
                reason: e.to_string(),
            })?;

        let child_txid = WalletTxid::from(psbt.unsigned_tx.compute_txid());
        let psbt_base64 = PsbtBase64::from(psbt.to_string());

        Ok((psbt_base64, child_txid))
    }

    pub async fn create_cpfp_psbt(
        &mut self,
        parent_txid: &str,
        selected_outpoint: &WalletOutPoint,
        fee_rate: u64,
    ) -> WalletCoreResult<WalletCpfpPsbtInfo> {
        info!(
            parent_txid = %parent_txid,
            selected_outpoint = %selected_outpoint,
            fee_rate,
            "starting CPFP PSBT creation"
        );

        // --- Step 1: Validate inputs ---
        if parent_txid.is_empty() {
            return Err(WalletCoreError::CpfpEmptyParentTxid);
        }

        if fee_rate == 0 {
            return Err(WalletCoreError::InvalidFeeRate);
        }

        if self.is_watch_only {
            return Err(WalletCoreError::WatchOnlyCannotSign);
        }

        // --- Step 2: Locate candidate UTXO ---
        debug!("locating unconfirmed UTXO for parent transaction");

        let candidates = self.unconfirmed_utxos_for_txid(parent_txid);

        // Wallet UTXOs now expose strongly-typed WalletOutPoint values, so the
        // explicitly requested outpoint can be matched directly without any
        // string parsing or intermediate Bitcoin outpoint conversion.
        let selected = candidates
            .iter()
            .find(|u| u.outpoint == *selected_outpoint)
            .ok_or_else(|| WalletCoreError::CpfpNoCandidateUtxo(selected_outpoint.to_string()))?;

        let input_value_sat = selected.value.as_u64();

        // --- Step 3: Build child transaction plan ---
        let build_plan =
            Self::build_cpfp_plan(parent_txid, selected_outpoint, input_value_sat, fee_rate)?;

        debug!(
            outpoint = %build_plan.input_outpoint,
            input_value_sat = build_plan.input_value_sat.0,
            child_output_value_sat = build_plan.child_output_value_sat.0,
            fee_sat = build_plan.fee_sat.0,
            estimated_vsize = %build_plan.estimated_vsize,
            "built CPFP child transaction plan"
        );

        // --- Step 4: Create PSBT ---
        debug!("creating PSBT from child transaction plan");
        let (psbt_base64, child_txid) = self.build_cpfp_psbt_from_plan(parent_txid, &build_plan)?;

        // --- Step 5: Return result ---
        Ok(WalletCpfpPsbtInfo {
            psbt_base64,
            txid: child_txid,
            parent_txid: WalletTxid::parse(parent_txid)
                .map_err(|_| WalletCoreError::InvalidTxid(parent_txid.to_string()))?,
            selected_outpoint: *selected_outpoint,
            input_value_sat: build_plan.input_value_sat,
            child_output_value_sat: build_plan.child_output_value_sat,
            fee_sat: build_plan.fee_sat,
            fee_rate_sat_per_vb: FeeRateSatPerVb::from(fee_rate),
            replaceable: true,
            estimated_vsize: build_plan.estimated_vsize,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn build_cpfp_plan_calculates_fee_and_output() {
        let plan = WalletService::build_cpfp_plan(
            "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee",
            &WalletOutPoint::parse(
                "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee:1",
            )
            .unwrap(),
            100_000,
            2,
        )
        .expect("plan should build");

        let expected_vsize = CPFP_INPUT_VBYTES + CPFP_OUTPUT_VBYTES + CPFP_OVERHEAD_VBYTES;
        let expected_fee = expected_vsize * 2;

        assert_eq!(plan.estimated_vsize, VSize::from(expected_vsize));
        assert_eq!(plan.fee_sat.0, expected_fee);
        assert_eq!(plan.input_value_sat.0, 100_000);
        assert_eq!(plan.child_output_value_sat.0, 100_000 - expected_fee);
        assert_eq!(
            plan.input_outpoint,
            WalletOutPoint::parse(
                "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee:1"
            )
            .unwrap()
        );
    }

    #[test]
    fn build_cpfp_plan_fails_when_fee_consumes_entire_input() {
        let err = WalletService::build_cpfp_plan(
            "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee",
            &WalletOutPoint::parse(
                "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee:1",
            )
            .unwrap(),
            100,
            2,
        )
        .expect_err("fee >= input should fail");

        assert!(matches!(err, WalletCoreError::CpfpInsufficientValue(_)));
    }

    #[test]
    fn build_cpfp_plan_rejects_insufficient_value_edge_case() {
        let err = WalletService::build_cpfp_plan(
            "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee",
            &WalletOutPoint::parse(
                "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee:1",
            )
            .unwrap(),
            1,
            10,
        )
        .expect_err("should fail when fee exceeds input");

        assert!(matches!(err, WalletCoreError::CpfpInsufficientValue(_)));
    }
}
