use super::common_outpoint::ensure_no_outpoint_overlap;
use super::common_selection::{
    matches_value_filters, sort_local_outputs_by_strategy, validate_selected_input_count_bounds,
};
use crate::model::{
    WalletConsolidationStrategy, WalletInputSelectionConfig, WalletInputSelectionMode,
};
use crate::types::WalletOutPoint;
use crate::{WalletCoreError, WalletCoreResult};
use bdk_wallet::LocalOutput;
use bitcoin::OutPoint;
use std::collections::HashSet;

/// Select wallet inputs using typed outpoint configuration.
///
/// The selection result remains in the wallet-core domain model as
/// `WalletOutPoint` values. Raw Bitcoin outpoints are used only as a temporary
/// comparison form when scanning BDK wallet UTXOs.
pub fn select_inputs(
    utxos: &[LocalOutput],
    cfg: &WalletInputSelectionConfig,
) -> WalletCoreResult<Vec<WalletOutPoint>> {
    ensure_no_outpoint_overlap(&cfg.include_outpoints, &cfg.exclude_outpoints)
        .map_err(|e| WalletCoreError::SelectionFailed(e.to_string()))?;

    let include_bitcoin: HashSet<OutPoint> = cfg
        .include_outpoints
        .iter()
        .map(|op| OutPoint::from(*op))
        .collect();
    let exclude_bitcoin: HashSet<OutPoint> = cfg
        .exclude_outpoints
        .iter()
        .map(|op| OutPoint::from(*op))
        .collect();

    // 1. Build candidate pool
    let mut candidates: Vec<&LocalOutput> = utxos
        .iter()
        .filter(|u| {
            (!cfg.confirmed_only || u.chain_position.is_confirmed())
                && !exclude_bitcoin.contains(&u.outpoint)
                && matches_value_filters(
                    u.txout.value.to_sat(),
                    cfg.min_utxo_value_sat,
                    cfg.max_utxo_value_sat,
                )
        })
        .collect();

    // 2. Resolve manual inputs (pinned) against the full wallet set first,
    // then validate them against the configured filters.
    let mut selected: Vec<&LocalOutput> = Vec::new();
    let mut selected_outpoints: HashSet<OutPoint> = HashSet::new();

    for inc in &include_bitcoin {
        let utxo = utxos.iter().find(|u| u.outpoint == *inc).ok_or_else(|| {
            WalletCoreError::SelectionFailed(format!("missing included outpoint {}", inc))
        })?;

        if cfg.confirmed_only && !utxo.chain_position.is_confirmed() {
            return Err(WalletCoreError::SelectionFailed(format!(
                "included outpoint {} is not confirmed",
                inc
            )));
        }

        if exclude_bitcoin.contains(&utxo.outpoint) {
            return Err(WalletCoreError::SelectionFailed(format!(
                "included outpoint {} is also excluded",
                inc
            )));
        }

        if !matches_value_filters(
            utxo.txout.value.to_sat(),
            cfg.min_utxo_value_sat,
            cfg.max_utxo_value_sat,
        ) {
            return Err(WalletCoreError::SelectionFailed(format!(
                "included outpoint {} does not match value filters",
                inc
            )));
        }

        if selected_outpoints.insert(utxo.outpoint) {
            selected.push(utxo);
        }
    }

    // Remove already selected from candidates.
    candidates.retain(|u| !selected_outpoints.contains(&u.outpoint));

    // 3. If strict manual → return early
    if cfg
        .selection_mode
        .unwrap_or(WalletInputSelectionMode::AutomaticOnly)
        == WalletInputSelectionMode::StrictManual
    {
        validate_selected_input_count_bounds(
            selected.len(),
            cfg.min_input_count,
            cfg.max_input_count,
        )?;
        return Ok(selected
            .into_iter()
            .map(|u| WalletOutPoint::from(u.outpoint))
            .collect());
    }

    // 4. Sort candidates by strategy
    sort_local_outputs_by_strategy(
        &mut candidates,
        cfg.strategy
            .unwrap_or(WalletConsolidationStrategy::SmallestFirst),
    );

    // 5. Auto-complete selection
    for c in candidates {
        if let Some(max) = cfg.max_input_count {
            if selected.len() >= max {
                break;
            }
        }
        if selected_outpoints.insert(c.outpoint) {
            selected.push(c);
        }
    }

    // 6. Final validation
    validate_selected_input_count_bounds(selected.len(), cfg.min_input_count, cfg.max_input_count)?;

    Ok(selected
        .into_iter()
        .map(|u| WalletOutPoint::from(u.outpoint))
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::common_test_util::test_support::{
        default_selection_config, sample_local_output,
    };

    #[test]
    fn empty_utxos_fails_selection() {
        let cfg = default_selection_config();
        let result = select_inputs(&[], &cfg);
        assert!(result.is_err());
    }

    #[test]
    fn strict_manual_returns_only_included() {
        let utxo = sample_local_output(1000, 0, true);
        let mut cfg = default_selection_config();
        cfg.include_outpoints = vec![WalletOutPoint::from(utxo.outpoint)];
        cfg.selection_mode = Some(WalletInputSelectionMode::StrictManual);

        let result = select_inputs(&[utxo.clone()], &cfg).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], WalletOutPoint::from(utxo.outpoint));
    }

    #[test]
    fn exclude_removes_candidates() {
        let u1 = sample_local_output(1000, 0, true);
        let u2 = sample_local_output(2000, 1, true);

        let mut cfg = default_selection_config();
        cfg.exclude_outpoints = vec![WalletOutPoint::from(u1.outpoint)];

        let result = select_inputs(&[u1.clone(), u2.clone()], &cfg).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], WalletOutPoint::from(u2.outpoint));
    }

    #[test]
    fn respects_max_input_count() {
        let u1 = sample_local_output(1000, 0, true);
        let u2 = sample_local_output(2000, 1, true);
        let u3 = sample_local_output(3000, 2, true);

        let mut cfg = default_selection_config();
        cfg.max_input_count = Some(2);

        let result = select_inputs(&[u1, u2, u3], &cfg).unwrap();

        assert_eq!(result.len(), 2);
    }

    #[test]
    fn respects_min_input_count() {
        let u1 = sample_local_output(1000, 0, true);

        let mut cfg = default_selection_config();
        cfg.min_input_count = Some(2);

        let result = select_inputs(&[u1], &cfg);

        assert!(result.is_err());
    }

    #[test]
    fn confirmed_only_filters_unconfirmed() {
        let confirmed = sample_local_output(1000, 0, true);
        let unconfirmed = sample_local_output(2000, 1, false);

        let mut cfg = default_selection_config();
        cfg.confirmed_only = true;

        let result = select_inputs(&[confirmed.clone(), unconfirmed], &cfg).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], WalletOutPoint::from(confirmed.outpoint));
    }
}
