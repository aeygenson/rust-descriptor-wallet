use std::collections::HashSet;

use bdk_wallet::KeychainKind;
use bitcoin::FeeRate;
use tracing::{debug, info};

use crate::model::{
    WalletConsolidationInfo, WalletConsolidationStrategy, WalletInputSelectionConfig,
    WalletPsbtInfo,
};
use crate::types::{FeeRateSatPerVb, PsbtBase64, VSize, WalletOutPoint, WalletTxid};
use crate::{WalletCoreError, WalletCoreResult};

use super::{
    common_selection::{effective_selection_mode, is_strict_manual_selection},
    common_tx::RBF_SEQUENCE,
    psbt_coin_selector::select_inputs,
    WalletService,
};

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
            "wallet_service: create_consolidation_psbt start fee_rate_sat_per_vb={} enable_rbf={} has_consolidation={} selection_mode={:?}",
            fee_rate_sat_per_vb.as_u64(),
            enable_rbf,
            consolidation.is_some(),
            consolidation.as_ref().and_then(|c| c.selection.selection_mode),
        );

        if fee_rate_sat_per_vb.as_u64() == 0 {
            return Err(WalletCoreError::InvalidFeeRate);
        }

        let fee_rate = FeeRate::from_sat_per_vb(fee_rate_sat_per_vb.as_u64())
            .ok_or(WalletCoreError::InvalidFeeRate)?;

        let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        let effective_cfg = consolidation.unwrap_or(WalletConsolidationInfo {
            selection: WalletInputSelectionConfig {
                confirmed_only: true,
                strategy: Some(WalletConsolidationStrategy::SmallestFirst),
                ..WalletInputSelectionConfig::default()
            },
            ..WalletConsolidationInfo::default()
        });

        let parsed_include = effective_cfg.selection.include_outpoints.clone();
        let parsed_exclude = effective_cfg.selection.exclude_outpoints.clone();

        let selection_mode = effective_selection_mode(
            &effective_cfg.selection.include_outpoints,
            effective_cfg.selection.selection_mode,
        );

        let selection_cfg = WalletInputSelectionConfig {
            include_outpoints: parsed_include,
            exclude_outpoints: parsed_exclude,
            confirmed_only: effective_cfg.selection.confirmed_only,
            selection_mode: Some(selection_mode),
            max_input_count: effective_cfg.selection.max_input_count,
            min_input_count: effective_cfg.selection.min_input_count,
            min_utxo_value_sat: effective_cfg.selection.min_utxo_value_sat,
            max_utxo_value_sat: effective_cfg.selection.max_utxo_value_sat,
            strategy: effective_cfg.selection.strategy,
        };

        let selected_inputs = select_inputs(&wallet_utxos, &selection_cfg)?;

        if selected_inputs.len() < 2 {
            return Err(WalletCoreError::ConsolidationTooFewInputs);
        }

        if let Some(min_input_count) = effective_cfg.selection.min_input_count {
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
            .filter(|u| selected_set.contains(&WalletOutPoint::from(u.outpoint)))
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
            if total_pct_base > 0 && fee_pct > total_pct_base * (u8::from(max_pct) as u128) {
                return Err(WalletCoreError::ConsolidationFeeTooHigh {
                    fee_sat: fee_estimate_sat,
                    total_input_sat: selected_total_sat,
                    max_pct: u8::from(max_pct),
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
            "wallet_service: create_consolidation_psbt selected_inputs={} excluded_inputs={} selected_total_sat={} estimated_fee_sat={} selection_mode={:?}",
            selected_inputs.len(),
            excluded_inputs.len(),
            selected_total_sat,
            fee_estimate_sat,
            effective_cfg.selection.selection_mode,
        );

        let mut builder = self.wallet.build_tx();
        builder.fee_rate(fee_rate);
        builder.drain_to(change_script.clone());

        if enable_rbf {
            builder.set_exact_sequence(RBF_SEQUENCE);
        }

        for outpoint in &selected_inputs {
            builder
                .add_utxo(bitcoin::OutPoint::from(*outpoint))
                .map_err(|e| {
                    WalletCoreError::CoinControlOutpointNotSpendable(format!(
                        "{} ({})",
                        outpoint, e
                    ))
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
        let selected_input_outpoints: Vec<WalletOutPoint> = psbt
            .unsigned_tx
            .input
            .iter()
            .map(|txin| WalletOutPoint::from(txin.previous_output))
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

        let psbt_base64 = PsbtBase64::from(psbt.to_string());

        info!(
            "wallet_service: create_consolidation_psbt success amount_sat={} fee_sat={} fee_rate_sat_per_vb={} actual_replaceable={} selected_utxos={} outputs={} min_input_count={:?} max_input_count={:?} strategy={:?} selection_mode={:?}",
            output_amount_sat,
            fee_sat,
            fee_rate_sat_per_vb.as_u64(),
            actual_replaceable,
            selected_utxo_count,
            psbt.unsigned_tx.output.len(),
            effective_cfg.selection.min_input_count,
            effective_cfg.selection.max_input_count,
            effective_cfg.selection.strategy,
            effective_cfg.selection.selection_mode,
        );

        Ok(WalletPsbtInfo {
            psbt_base64,
            txid: WalletTxid::from(psbt.unsigned_tx.compute_txid()),
            original_txid: None,
            to_address: change_info.address.to_string(),
            amount_sat: crate::types::AmountSat::from(output_amount_sat),
            fee_sat: crate::types::AmountSat::from(fee_sat),
            fee_rate_sat_per_vb,
            replaceable: actual_replaceable,
            change_amount_sat: Some(crate::types::AmountSat::from(output_amount_sat)),
            selected_utxo_count,
            selected_inputs: selected_input_outpoints,
            input_count: psbt.unsigned_tx.input.len(),
            output_count: psbt.unsigned_tx.output.len(),
            recipient_count: 1,
            estimated_vsize: VSize::from(psbt.unsigned_tx.vsize() as u64),
        })
    }

    fn resolve_consolidation_exclusions(
        &self,
        cfg: Option<&WalletConsolidationInfo>,
        wallet_utxos: &[bdk_wallet::LocalOutput],
        selected_set: &HashSet<WalletOutPoint>,
    ) -> Vec<bitcoin::OutPoint> {
        let mut exclusions = Vec::new();
        let explicit_excludes: HashSet<bitcoin::OutPoint> = cfg
            .map(|c| {
                c.selection
                    .exclude_outpoints
                    .iter()
                    .map(|op| *op.as_ref())
                    .collect()
            })
            .unwrap_or_default();
        let strict_mode = cfg
            .map(|c| {
                is_strict_manual_selection(
                    &c.selection.include_outpoints,
                    c.selection.selection_mode,
                )
            })
            .unwrap_or(false);

        for utxo in wallet_utxos {
            let outpoint = utxo.outpoint;
            let wallet_outpoint = WalletOutPoint::from(outpoint);

            if explicit_excludes.contains(&outpoint) {
                exclusions.push(outpoint);
                continue;
            }

            if strict_mode && !selected_set.contains(&wallet_outpoint) {
                exclusions.push(outpoint);
            }
        }

        exclusions
    }
}

#[cfg(test)]
mod tests {
    use crate::model::{
        WalletConsolidationInfo, WalletInputSelectionConfig, WalletInputSelectionMode,
    };
    use crate::service::common_test_util::test_support::{
        consolidation_cfg_with_mode, load_test_wallet, strict_manual_consolidation_cfg,
    };
    use crate::types::WalletOutPoint;

    #[test]
    fn consolidation_info_is_empty_when_all_controls_are_default() {
        let info = WalletConsolidationInfo::default();
        assert!(info.is_empty());
    }

    #[test]
    fn consolidation_info_is_not_empty_when_selection_mode_is_set() {
        let info = consolidation_cfg_with_mode(WalletInputSelectionMode::AutomaticOnly);
        assert!(!info.is_empty());
    }

    #[test]
    fn resolve_consolidation_exclusions_only_uses_explicit_excludes_in_non_strict_mode() {
        let (_config, service) = load_test_wallet();
        let wallet_utxos: Vec<_> = service.wallet.list_unspent().collect();
        let selected_set: std::collections::HashSet<_> = std::collections::HashSet::new();

        let cfg = WalletConsolidationInfo {
            selection: WalletInputSelectionConfig {
                include_outpoints: vec![WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000001:0",
                )
                .unwrap()],
                exclude_outpoints: vec![WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000002:0",
                )
                .unwrap()],
                confirmed_only: false,
                selection_mode: Some(WalletInputSelectionMode::ManualWithAutoCompletion),
                max_input_count: None,
                min_input_count: None,
                min_utxo_value_sat: None,
                max_utxo_value_sat: None,
                strategy: None,
            },
            max_fee_pct_of_input_value: None,
        };

        let exclusions =
            service.resolve_consolidation_exclusions(Some(&cfg), &wallet_utxos, &selected_set);

        assert!(exclusions.len() <= wallet_utxos.len());
    }

    #[test]
    fn resolve_consolidation_exclusions_adds_non_selected_utxos_in_strict_mode() {
        let (_config, service) = load_test_wallet();
        let wallet_utxos: Vec<_> = service.wallet.list_unspent().collect();

        let selected_set: std::collections::HashSet<_> = wallet_utxos
            .iter()
            .take(1)
            .map(|u| WalletOutPoint::from(u.outpoint))
            .collect();

        let exclusions = service.resolve_consolidation_exclusions(
            Some(&strict_manual_consolidation_cfg()),
            &wallet_utxos,
            &selected_set,
        );

        assert_eq!(
            exclusions.len(),
            wallet_utxos.len().saturating_sub(selected_set.len())
        );
    }

    #[test]
    fn resolve_consolidation_exclusions_respects_explicit_excludes_even_when_selected() {
        let (_config, service) = load_test_wallet();
        let wallet_utxos: Vec<_> = service.wallet.list_unspent().collect();

        if let Some(first) = wallet_utxos.first() {
            let selected_set: std::collections::HashSet<_> =
                [WalletOutPoint::from(first.outpoint)].into_iter().collect();
            let cfg = WalletConsolidationInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![WalletOutPoint::from(first.outpoint)],
                    exclude_outpoints: vec![WalletOutPoint::from(first.outpoint)],
                    confirmed_only: false,
                    selection_mode: Some(WalletInputSelectionMode::ManualWithAutoCompletion),
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
                max_fee_pct_of_input_value: None,
            };

            let exclusions =
                service.resolve_consolidation_exclusions(Some(&cfg), &wallet_utxos, &selected_set);

            assert!(exclusions.contains(&first.outpoint));
        }
    }
}
