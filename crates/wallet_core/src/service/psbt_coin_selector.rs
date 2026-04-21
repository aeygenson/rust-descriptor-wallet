use super::common_outpoint::ensure_no_outpoint_overlap;
use super::common_selection::{
    matches_value_filters, sort_local_outputs_by_strategy, validate_selected_input_count_bounds,
};
use crate::error::WalletCoreError;
use crate::model::{WalletConsolidationStrategy, WalletInputSelectionMode};
use crate::WalletCoreResult;
use bdk_wallet::LocalOutput;
use bitcoin::OutPoint;
use std::collections::HashSet;

#[derive(Debug, Clone)]
pub struct SelectionConfig {
    pub include: Vec<OutPoint>,
    pub exclude: Vec<OutPoint>,
    pub confirmed_only: bool,
    pub max_input_count: Option<usize>,
    pub min_input_count: Option<usize>,
    pub min_value: Option<u64>,
    pub max_value: Option<u64>,
    pub strategy: Option<WalletConsolidationStrategy>,
    pub mode: WalletInputSelectionMode,
}

pub fn select_inputs(
    utxos: &[LocalOutput],
    cfg: &SelectionConfig,
) -> WalletCoreResult<Vec<OutPoint>> {
    ensure_no_outpoint_overlap(&cfg.include, &cfg.exclude)
        .map_err(|e| WalletCoreError::SelectionFailed(e.to_string()))?;

    // 1. Build candidate pool
    let mut candidates: Vec<&LocalOutput> = utxos
        .iter()
        .filter(|u| {
            (!cfg.confirmed_only || u.chain_position.is_confirmed())
                && !cfg.exclude.contains(&u.outpoint)
                && matches_value_filters(u.txout.value.to_sat(), cfg.min_value, cfg.max_value)
        })
        .collect();

    // 2. Resolve manual inputs (pinned) against the full wallet set first,
    // then validate them against the configured filters.
    let mut selected: Vec<&LocalOutput> = Vec::new();
    let mut selected_outpoints: HashSet<OutPoint> = HashSet::new();

    for inc in &cfg.include {
        let utxo = utxos.iter().find(|u| u.outpoint == *inc).ok_or_else(|| {
            WalletCoreError::SelectionFailed(format!("missing included outpoint {}", inc))
        })?;

        if cfg.confirmed_only && !utxo.chain_position.is_confirmed() {
            return Err(WalletCoreError::SelectionFailed(format!(
                "included outpoint {} is not confirmed",
                inc
            )));
        }

        if cfg.exclude.contains(&utxo.outpoint) {
            return Err(WalletCoreError::SelectionFailed(format!(
                "included outpoint {} is also excluded",
                inc
            )));
        }

        if !matches_value_filters(utxo.txout.value.to_sat(), cfg.min_value, cfg.max_value) {
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
    if cfg.mode == WalletInputSelectionMode::StrictManual {
        validate_selected_input_count_bounds(
            selected.len(),
            cfg.min_input_count,
            cfg.max_input_count,
        )?;
        return Ok(selected.into_iter().map(|u| u.outpoint).collect());
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

    Ok(selected.into_iter().map(|u| u.outpoint).collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::test_support::{
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
        cfg.include = vec![utxo.outpoint];
        cfg.mode = WalletInputSelectionMode::StrictManual;

        let result = select_inputs(&[utxo.clone()], &cfg).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], utxo.outpoint);
    }

    #[test]
    fn exclude_removes_candidates() {
        let u1 = sample_local_output(1000, 0, true);
        let u2 = sample_local_output(2000, 1, true);

        let mut cfg = default_selection_config();
        cfg.exclude = vec![u1.outpoint];

        let result = select_inputs(&[u1.clone(), u2.clone()], &cfg).unwrap();

        assert_eq!(result.len(), 1);
        assert_eq!(result[0], u2.outpoint);
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
        assert_eq!(result[0], confirmed.outpoint);
    }
}
