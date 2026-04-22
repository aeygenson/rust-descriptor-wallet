use std::cmp::Ordering;

use bdk_wallet::LocalOutput;

use crate::error::WalletCoreError;
use crate::model::{WalletConsolidationStrategy, WalletInputSelectionMode};
use crate::types::WalletOutPoint;
use crate::WalletCoreResult;

/// Returns true when a `WalletInputSelectionConfig` carries an explicit include set.
///
/// This helper stays field-oriented so it can be reused from both
/// `WalletCoinControlInfo.selection` and `WalletConsolidationInfo.selection`
/// without coupling common utilities to a higher-level wrapper type.
pub fn has_explicit_include_set(include_outpoints: &[WalletOutPoint]) -> bool {
    !include_outpoints.is_empty()
}

/// Resolve the effective input-selection mode for a shared
/// `WalletInputSelectionConfig`, defaulting to:
/// - `StrictManual` when an explicit include set is present
/// - `AutomaticOnly` otherwise.
///
/// Manual auto-completion is deliberately opt-in: callers that pass explicit
/// outpoints usually expect an exact input set, especially for sweep and strict
/// coin-control flows.
pub fn effective_selection_mode(
    include_outpoints: &[WalletOutPoint],
    selection_mode: Option<WalletInputSelectionMode>,
) -> WalletInputSelectionMode {
    selection_mode.unwrap_or_else(|| {
        if has_explicit_include_set(include_outpoints) {
            WalletInputSelectionMode::StrictManual
        } else {
            WalletInputSelectionMode::AutomaticOnly
        }
    })
}

/// Returns true when a shared selection config effectively resolves to
/// `StrictManual`.
pub fn is_strict_manual_selection(
    include_outpoints: &[WalletOutPoint],
    selection_mode: Option<WalletInputSelectionMode>,
) -> bool {
    effective_selection_mode(include_outpoints, selection_mode)
        == WalletInputSelectionMode::StrictManual
}

/// Returns true when a satoshi value matches the optional min/max range filters.
pub fn matches_value_filters(
    value_sat: u64,
    min_value: Option<u64>,
    max_value: Option<u64>,
) -> bool {
    min_value.map(|min| value_sat >= min).unwrap_or(true)
        && max_value.map(|max| value_sat <= max).unwrap_or(true)
}

/// Compare two wallet UTXOs according to the requested consolidation strategy.
pub fn compare_local_outputs_by_strategy(
    a: &LocalOutput,
    b: &LocalOutput,
    strategy: WalletConsolidationStrategy,
) -> Ordering {
    let by_outpoint = || WalletOutPoint::from(a.outpoint).cmp(&WalletOutPoint::from(b.outpoint));

    match strategy {
        WalletConsolidationStrategy::SmallestFirst => a
            .txout
            .value
            .to_sat()
            .cmp(&b.txout.value.to_sat())
            .then_with(by_outpoint),
        WalletConsolidationStrategy::LargestFirst => b
            .txout
            .value
            .to_sat()
            .cmp(&a.txout.value.to_sat())
            .then_with(by_outpoint),
        WalletConsolidationStrategy::OldestFirst => {
            let a_height = match a.chain_position {
                bdk_chain::ChainPosition::Confirmed { anchor, .. } => anchor.block_id.height,
                bdk_chain::ChainPosition::Unconfirmed { .. } => u32::MAX,
            };
            let b_height = match b.chain_position {
                bdk_chain::ChainPosition::Confirmed { anchor, .. } => anchor.block_id.height,
                bdk_chain::ChainPosition::Unconfirmed { .. } => u32::MAX,
            };
            a_height.cmp(&b_height).then_with(by_outpoint)
        }
    }
}

/// Sort wallet UTXO candidates in-place according to the requested strategy.
pub fn sort_local_outputs_by_strategy(
    candidates: &mut Vec<&LocalOutput>,
    strategy: WalletConsolidationStrategy,
) {
    candidates.sort_by(|a, b| compare_local_outputs_by_strategy(a, b, strategy));
}

/// Validate the number of selected inputs against the requested minimum.
pub fn validate_selected_input_count(
    selected_len: usize,
    min_input_count: Option<usize>,
) -> WalletCoreResult<()> {
    validate_selected_input_count_bounds(selected_len, min_input_count, None)
}

/// Validate the number of selected inputs against optional minimum and maximum
/// bounds.
pub fn validate_selected_input_count_bounds(
    selected_len: usize,
    min_input_count: Option<usize>,
    max_input_count: Option<usize>,
) -> WalletCoreResult<()> {
    if selected_len == 0 {
        return Err(WalletCoreError::SelectionFailed(
            "no inputs selected".to_string(),
        ));
    }

    if let Some(max) = max_input_count {
        if selected_len > max {
            return Err(WalletCoreError::SelectionFailed(format!(
                "selected {} inputs but max_input_count={} allows fewer",
                selected_len, max
            )));
        }
    }

    if let Some(min) = min_input_count {
        if selected_len < min {
            return Err(WalletCoreError::SelectionFailed(format!(
                "selected {} inputs but min_input_count={} requires more",
                selected_len, min
            )));
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WalletOutPoint;
    use bitcoin::{Amount, BlockHash, OutPoint, ScriptBuf, TxOut};

    fn sample_local_output(
        value_sat: u64,
        confirmed: bool,
        confirmation_height: u32,
        vout: u32,
    ) -> LocalOutput {
        LocalOutput {
            outpoint: OutPoint::new(
                "d8d4ffb424e4cfc699ac1173fcabacab5c7f1a061ace368da18cb7dc9b00e01d"
                    .parse()
                    .unwrap(),
                vout,
            ),
            txout: TxOut {
                value: Amount::from_sat(value_sat),
                script_pubkey: ScriptBuf::new(),
            },
            keychain: bdk_wallet::KeychainKind::External,
            is_spent: false,
            derivation_index: 0,
            chain_position: if confirmed {
                bdk_chain::ChainPosition::Confirmed {
                    anchor: bdk_chain::ConfirmationBlockTime {
                        block_id: bdk_chain::BlockId {
                            height: confirmation_height,
                            hash:
                                "0000000000000000000000000000000000000000000000000000000000000000"
                                    .parse::<BlockHash>()
                                    .unwrap(),
                        },
                        confirmation_time: 0,
                    },
                    transitively: None,
                }
            } else {
                bdk_chain::ChainPosition::Unconfirmed {
                    last_seen: None,
                    first_seen: None,
                }
            },
        }
    }

    #[test]
    fn explicit_include_set_detection_works() {
        assert!(!has_explicit_include_set(&[]));
        assert!(has_explicit_include_set(&[WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap(),]));
    }

    #[test]
    fn effective_selection_mode_defaults_to_automatic_only_without_includes() {
        let mode = effective_selection_mode(&[], None);
        assert_eq!(mode, WalletInputSelectionMode::AutomaticOnly);
    }

    #[test]
    fn effective_selection_mode_defaults_to_strict_manual_with_includes() {
        let mode = effective_selection_mode(
            &[WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000001:0",
            )
            .unwrap()],
            None,
        );
        assert_eq!(mode, WalletInputSelectionMode::StrictManual);
    }

    #[test]
    fn effective_selection_mode_preserves_explicit_mode() {
        let mode = effective_selection_mode(
            &[WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000001:0",
            )
            .unwrap()],
            Some(WalletInputSelectionMode::StrictManual),
        );
        assert_eq!(mode, WalletInputSelectionMode::StrictManual);
    }

    #[test]
    fn strict_manual_selection_detection_defaults_to_true_with_includes() {
        assert!(!is_strict_manual_selection(&[], None));
        assert!(is_strict_manual_selection(
            &[WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000001:0"
            )
            .unwrap()],
            None,
        ));
    }

    #[test]
    fn strict_manual_selection_detection_is_true_when_explicitly_requested() {
        assert!(is_strict_manual_selection(
            &[WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000001:0"
            )
            .unwrap()],
            Some(WalletInputSelectionMode::StrictManual),
        ));
    }

    #[test]
    fn matches_value_filters_accepts_unbounded_value() {
        assert!(matches_value_filters(1_000, None, None));
    }

    #[test]
    fn matches_value_filters_respects_min_only() {
        assert!(matches_value_filters(2_000, Some(1_500), None));
        assert!(!matches_value_filters(1_000, Some(1_500), None));
    }

    #[test]
    fn matches_value_filters_respects_max_only() {
        assert!(matches_value_filters(1_000, None, Some(1_500)));
        assert!(!matches_value_filters(2_000, None, Some(1_500)));
    }

    #[test]
    fn matches_value_filters_respects_both_bounds() {
        assert!(matches_value_filters(1_500, Some(1_000), Some(2_000)));
        assert!(!matches_value_filters(500, Some(1_000), Some(2_000)));
        assert!(!matches_value_filters(2_500, Some(1_000), Some(2_000)));
    }

    #[test]
    fn compare_local_outputs_by_strategy_orders_smallest_first() {
        let small = sample_local_output(1_000, true, 100, 0);
        let large = sample_local_output(5_000, true, 100, 1);

        assert_eq!(
            compare_local_outputs_by_strategy(
                &small,
                &large,
                WalletConsolidationStrategy::SmallestFirst,
            ),
            Ordering::Less
        );
    }

    #[test]
    fn compare_local_outputs_by_strategy_orders_largest_first() {
        let small = sample_local_output(1_000, true, 100, 0);
        let large = sample_local_output(5_000, true, 100, 1);

        assert_eq!(
            compare_local_outputs_by_strategy(
                &small,
                &large,
                WalletConsolidationStrategy::LargestFirst,
            ),
            Ordering::Greater
        );
    }

    #[test]
    fn compare_local_outputs_by_strategy_orders_oldest_first() {
        let older = sample_local_output(1_000, true, 100, 0);
        let newer = sample_local_output(1_000, true, 200, 1);

        assert_eq!(
            compare_local_outputs_by_strategy(
                &older,
                &newer,
                WalletConsolidationStrategy::OldestFirst,
            ),
            Ordering::Less
        );
    }

    #[test]
    fn sort_local_outputs_by_strategy_sorts_candidates_in_place() {
        let a = sample_local_output(5_000, true, 100, 0);
        let b = sample_local_output(1_000, true, 100, 1);
        let c = sample_local_output(3_000, true, 100, 2);
        let mut candidates = vec![&a, &b, &c];

        sort_local_outputs_by_strategy(&mut candidates, WalletConsolidationStrategy::SmallestFirst);

        assert_eq!(candidates[0].txout.value.to_sat(), 1_000);
        assert_eq!(candidates[1].txout.value.to_sat(), 3_000);
        assert_eq!(candidates[2].txout.value.to_sat(), 5_000);
    }

    #[test]
    fn validate_selected_input_count_rejects_empty_selection() {
        let result = validate_selected_input_count(0, None);
        assert!(matches!(result, Err(WalletCoreError::SelectionFailed(_))));
    }

    #[test]
    fn validate_selected_input_count_rejects_too_few_inputs() {
        let result = validate_selected_input_count(1, Some(2));
        assert!(matches!(result, Err(WalletCoreError::SelectionFailed(_))));
    }

    #[test]
    fn validate_selected_input_count_accepts_sufficient_selection() {
        let result = validate_selected_input_count(2, Some(2));
        assert!(result.is_ok());
    }
}
