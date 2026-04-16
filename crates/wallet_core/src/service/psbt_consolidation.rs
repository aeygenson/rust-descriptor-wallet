use std::collections::HashSet;

use bdk_wallet::KeychainKind;
use bitcoin::{FeeRate, Sequence};
use tracing::{debug, info};

use crate::model::{WalletConsolidationInfo, WalletConsolidationStrategy, WalletPsbtInfo};
use crate::types::FeeRateSatPerVb;
use crate::{WalletCoreError, WalletCoreResult};

use super::{psbt_common::parse_unique_outpoints, WalletService};

impl WalletService {
    /// Create an unsigned PSBT for a wallet-internal consolidation flow.
    ///
    /// Consolidation spends multiple wallet UTXOs into a smaller number of
    /// wallet-owned outputs, usually one internal output, to reduce UTXO
    /// fragmentation and future spending cost.
    pub fn create_consolidation_psbt(
        &mut self,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
        consolidation: Option<WalletConsolidationInfo>,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        debug!(
            "wallet_service: create_consolidation_psbt start fee_rate_sat_per_vb={} enable_rbf={} has_consolidation={} ",
            fee_rate_sat_per_vb.as_u64(),
            enable_rbf,
            consolidation.is_some(),
        );

        if fee_rate_sat_per_vb.as_u64() == 0 {
            return Err(WalletCoreError::InvalidFeeRate);
        }

        let fee_rate = FeeRate::from_sat_per_vb(fee_rate_sat_per_vb.as_u64())
            .ok_or(WalletCoreError::InvalidFeeRate)?;

        let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        let effective_cfg = consolidation.unwrap_or(WalletConsolidationInfo {
            confirmed_only: true,
            strategy: Some(WalletConsolidationStrategy::SmallestFirst),
            ..WalletConsolidationInfo::default()
        });

        let selected_inputs = if effective_cfg.has_explicit_include_set() {
            self.resolve_consolidation_selected_inputs(&effective_cfg, &wallet_utxos)?
        } else {
            self.select_consolidation_candidates(&effective_cfg, &wallet_utxos)?
        };

        if selected_inputs.len() < 2 {
            return Err(WalletCoreError::ConsolidationTooFewInputs);
        }

        if let Some(min_input_count) = effective_cfg.min_input_count {
            if selected_inputs.len() < min_input_count {
                return Err(WalletCoreError::ConsolidationMinInputNotMet {
                    required: min_input_count,
                    actual: selected_inputs.len(),
                });
            }
        }

        let selected_set: HashSet<_> = selected_inputs.iter().copied().collect();
        let selected_total_sat: u64 = wallet_utxos
            .iter()
            .filter(|u| selected_set.contains(&u.outpoint))
            .map(|u| u.txout.value.to_sat())
            .sum();

        let input_count = selected_inputs.len() as u64;
        let output_count = 1u64;
        let estimated_vsize = 11u64 + input_count * 58u64 + output_count * 43u64;
        let fee_estimate_sat = estimated_vsize * fee_rate_sat_per_vb.as_u64();

        if selected_total_sat <= fee_estimate_sat {
            return Err(WalletCoreError::ConsolidationAmountTooSmall);
        }

        if let Some(max_pct) = effective_cfg.max_fee_pct_of_input_value {
            let fee_pct = (fee_estimate_sat as u128) * 100u128;
            let total_pct_base = selected_total_sat as u128;
            if total_pct_base > 0 && fee_pct > total_pct_base * (max_pct as u128) {
                return Err(WalletCoreError::ConsolidationFeeTooHigh {
                    fee_sat: fee_estimate_sat,
                    total_input_sat: selected_total_sat,
                    max_pct,
                });
            }
        }

        let change_info = self.wallet.next_unused_address(KeychainKind::Internal);
        let change_script = change_info.address.script_pubkey();

        let excluded_inputs = self.resolve_consolidation_exclusions(
            Some(&effective_cfg),
            &wallet_utxos,
            &selected_set,
        );

        debug!(
            "wallet_service: create_consolidation_psbt selected_inputs={} excluded_inputs={} selected_total_sat={} estimated_fee_sat={}",
            selected_inputs.len(),
            excluded_inputs.len(),
            selected_total_sat,
            fee_estimate_sat,
        );

        let mut builder = self.wallet.build_tx();
        builder.fee_rate(fee_rate);
        builder.drain_to(change_script.clone());

        if enable_rbf {
            builder.set_exact_sequence(Sequence(0xFFFFFFFD));
        }

        for outpoint in &selected_inputs {
            builder.add_utxo(*outpoint).map_err(|e| {
                WalletCoreError::CoinControlOutpointNotSpendable(format!("{} ({})", outpoint, e))
            })?;
        }

        if !excluded_inputs.is_empty() {
            builder.unspendable(excluded_inputs);
        }

        let psbt = builder
            .finish()
            .map_err(|e| WalletCoreError::PsbtBuildFailed(e.to_string()))?;

        let actual_replaceable = psbt.unsigned_tx.is_explicitly_rbf();

        let selected_utxo_count = psbt.inputs.len();
        let selected_input_strings: Vec<String> = psbt
            .unsigned_tx
            .input
            .iter()
            .map(|txin| txin.previous_output.to_string())
            .collect();

        let total_input_sat: u64 = psbt
            .inputs
            .iter()
            .filter_map(|input| {
                input
                    .witness_utxo
                    .as_ref()
                    .map(|txout| txout.value.to_sat())
            })
            .sum();

        let total_output_sat: u64 = psbt
            .unsigned_tx
            .output
            .iter()
            .map(|txout| txout.value.to_sat())
            .sum();

        let fee_sat = total_input_sat
            .checked_sub(total_output_sat)
            .ok_or(WalletCoreError::FeeCalculationFailed)?;

        if total_output_sat == 0 {
            return Err(WalletCoreError::ConsolidationAmountTooSmall);
        }

        let output_amount_sat = psbt
            .unsigned_tx
            .output
            .iter()
            .find(|txout| txout.script_pubkey == change_script)
            .map(|txout| txout.value.to_sat())
            .unwrap_or(total_output_sat);

        let psbt_base64 = psbt.to_string();

        info!(
            "wallet_service: create_consolidation_psbt success amount_sat={} fee_sat={} fee_rate_sat_per_vb={} actual_replaceable={} selected_utxos={} outputs={} min_input_count={:?} max_input_count={:?} strategy={:?}",
            output_amount_sat,
            fee_sat,
            fee_rate_sat_per_vb.as_u64(),
            actual_replaceable,
            selected_utxo_count,
            psbt.unsigned_tx.output.len(),
            effective_cfg.min_input_count,
            effective_cfg.max_input_count,
            effective_cfg.strategy,
        );

        Ok(WalletPsbtInfo {
            psbt_base64,
            txid: psbt.unsigned_tx.compute_txid().to_string(),
            original_txid: None,
            to_address: change_info.address.to_string(),
            amount_sat: crate::types::AmountSat::from(output_amount_sat),
            fee_sat: crate::types::AmountSat::from(fee_sat),
            fee_rate_sat_per_vb: fee_rate_sat_per_vb.as_u64(),
            replaceable: actual_replaceable,
            change_amount_sat: Some(crate::types::AmountSat::from(output_amount_sat)),
            selected_utxo_count,
            selected_inputs: selected_input_strings,
            input_count: psbt.unsigned_tx.input.len(),
            output_count: psbt.unsigned_tx.output.len(),
            recipient_count: 1,
            estimated_vsize: psbt.unsigned_tx.vsize() as u64,
        })
    }

    fn resolve_consolidation_selected_inputs(
        &self,
        cfg: &WalletConsolidationInfo,
        wallet_utxos: &[bdk_wallet::LocalOutput],
    ) -> WalletCoreResult<Vec<bitcoin::OutPoint>> {
        let parsed = parse_unique_outpoints(&cfg.include_outpoints)?;
        let wallet_map: HashSet<_> = wallet_utxos.iter().map(|u| u.outpoint).collect();

        for outpoint in &parsed {
            if !wallet_map.contains(outpoint) {
                return Err(WalletCoreError::CoinControlOutpointNotFound(
                    outpoint.to_string(),
                ));
            }

            let utxo = wallet_utxos
                .iter()
                .find(|u| u.outpoint == *outpoint)
                .expect("checked above");

            if let Some(min_value_sat) = cfg.min_utxo_value_sat {
                if utxo.txout.value.to_sat() < min_value_sat {
                    return Err(WalletCoreError::ConsolidationValueFilterMismatch);
                }
            }

            if let Some(max_value_sat) = cfg.max_utxo_value_sat {
                if utxo.txout.value.to_sat() > max_value_sat {
                    return Err(WalletCoreError::ConsolidationValueFilterMismatch);
                }
            }

            if cfg.confirmed_only && !utxo.chain_position.is_confirmed() {
                return Err(WalletCoreError::CoinControlOutpointNotConfirmed(
                    outpoint.to_string(),
                ));
            }

            if cfg
                .exclude_outpoints
                .iter()
                .any(|item| item == &outpoint.to_string())
            {
                return Err(WalletCoreError::CoinControlConflict(format!(
                    "outpoint {} appears in both include and exclude sets",
                    outpoint
                )));
            }
        }

        if let Some(max_input_count) = cfg.max_input_count {
            if parsed.len() > max_input_count {
                return Err(WalletCoreError::CoinControlConflict(format!(
                    "selected {} inputs exceeds consolidation max_input_count {}",
                    parsed.len(),
                    max_input_count
                )));
            }
        }

        Ok(parsed)
    }

    fn select_consolidation_candidates(
        &self,
        cfg: &WalletConsolidationInfo,
        wallet_utxos: &[bdk_wallet::LocalOutput],
    ) -> WalletCoreResult<Vec<bitcoin::OutPoint>> {
        let excluded: HashSet<_> = cfg.exclude_outpoints.iter().cloned().collect();

        let mut eligible: Vec<_> = wallet_utxos
            .iter()
            .filter(|u| !excluded.contains(&u.outpoint.to_string()))
            .filter(|u| !cfg.confirmed_only || u.chain_position.is_confirmed())
            .filter(|u| {
                cfg.min_utxo_value_sat
                    .map(|min| u.txout.value.to_sat() >= min)
                    .unwrap_or(true)
            })
            .filter(|u| {
                cfg.max_utxo_value_sat
                    .map(|max| u.txout.value.to_sat() <= max)
                    .unwrap_or(true)
            })
            .collect();

        if eligible.is_empty() {
            return Err(WalletCoreError::ConsolidationNoEligibleUtxos);
        }

        match cfg
            .strategy
            .unwrap_or(WalletConsolidationStrategy::SmallestFirst)
        {
            WalletConsolidationStrategy::SmallestFirst => {
                eligible.sort_by(|a, b| {
                    a.txout
                        .value
                        .to_sat()
                        .cmp(&b.txout.value.to_sat())
                        .then_with(|| a.outpoint.to_string().cmp(&b.outpoint.to_string()))
                });
            }
            WalletConsolidationStrategy::LargestFirst => {
                eligible.sort_by(|a, b| {
                    b.txout
                        .value
                        .to_sat()
                        .cmp(&a.txout.value.to_sat())
                        .then_with(|| a.outpoint.to_string().cmp(&b.outpoint.to_string()))
                });
            }
            WalletConsolidationStrategy::OldestFirst => {
                eligible.sort_by(|a, b| a.outpoint.to_string().cmp(&b.outpoint.to_string()));
            }
        }

        if let Some(max_input_count) = cfg.max_input_count {
            eligible.truncate(max_input_count);
        }

        Ok(eligible.into_iter().map(|u| u.outpoint).collect())
    }

    fn resolve_consolidation_exclusions(
        &self,
        cfg: Option<&WalletConsolidationInfo>,
        wallet_utxos: &[bdk_wallet::LocalOutput],
        selected_set: &HashSet<bitcoin::OutPoint>,
    ) -> Vec<bitcoin::OutPoint> {
        let mut exclusions = Vec::new();
        let explicit_excludes: HashSet<String> = cfg
            .map(|c| c.exclude_outpoints.iter().cloned().collect())
            .unwrap_or_default();
        let strict_mode = cfg.map(|c| c.has_explicit_include_set()).unwrap_or(false);

        for utxo in wallet_utxos {
            let outpoint = utxo.outpoint;
            let outpoint_str = outpoint.to_string();

            if explicit_excludes.contains(&outpoint_str) {
                exclusions.push(outpoint);
                continue;
            }

            if strict_mode && !selected_set.contains(&outpoint) {
                exclusions.push(outpoint);
            }
        }

        exclusions
    }
}
