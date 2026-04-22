use tracing::{debug, info};

use super::common_outpoint::ensure_no_outpoint_overlap;
use crate::model::{WalletCoinControlInfo, WalletCoinControlResolutionInfo};
use crate::types::WalletOutPoint;
use crate::{WalletCoreError, WalletCoreResult, WalletService};

impl WalletService {
    /// Resolve and validate explicitly included outpoints.
    ///
    /// This helper keeps included/excluded inputs in the typed wallet-core
    /// domain model (`WalletOutPoint`) and only relies on raw Bitcoin outpoints
    /// when matching against BDK wallet UTXOs.
    ///
    /// It does not yet build a PSBT by itself. Instead it provides the same
    /// style of focused, testable service logic as `psbt_cpfp.rs`, so the next
    /// step can wire it into `psbt_create.rs` with minimal churn.
    pub(crate) fn resolve_coin_control_inputs(
        &self,
        coin_control: &WalletCoinControlInfo,
    ) -> WalletCoreResult<WalletCoinControlResolutionInfo> {
        info!(
            include_count = coin_control.selection.include_outpoints.len(),
            exclude_count = coin_control.selection.exclude_outpoints.len(),
            confirmed_only = coin_control.selection.confirmed_only,
            selection_mode = ?coin_control.selection.selection_mode,
            "resolving coin control inputs"
        );

        if coin_control.is_empty() {
            debug!("coin control request is empty; nothing to resolve");
            return Ok(WalletCoinControlResolutionInfo {
                included_outpoints: Vec::new(),
                excluded_outpoints: Vec::new(),
                confirmed_only: coin_control.selection.confirmed_only,
                selection_mode: coin_control.selection.selection_mode,
                has_explicit_include_set: false,
            });
        }

        let included = coin_control.selection.include_outpoints.clone();
        let excluded = coin_control.selection.exclude_outpoints.clone();

        ensure_no_outpoint_overlap(&included, &excluded)?;

        let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        for requested in &included {
            let requested_outpoint = bitcoin::OutPoint::from(*requested);
            let utxo = wallet_utxos
                .iter()
                .find(|u| u.outpoint == requested_outpoint)
                .ok_or_else(|| {
                    WalletCoreError::CoinControlOutpointNotFound(requested.to_string())
                })?;

            if coin_control.selection.confirmed_only && !utxo.chain_position.is_confirmed() {
                return Err(WalletCoreError::CoinControlOutpointNotConfirmed(
                    requested.to_string(),
                ));
            }
        }

        debug!(
            resolved_includes = included.len(),
            resolved_excludes = excluded.len(),
            "coin control inputs resolved successfully"
        );

        Ok(WalletCoinControlResolutionInfo {
            included_outpoints: included,
            excluded_outpoints: excluded,
            confirmed_only: coin_control.selection.confirmed_only,
            selection_mode: coin_control.selection.selection_mode,
            has_explicit_include_set: coin_control.has_explicit_include_set(),
        })
    }

    /// Resolve and validate explicitly excluded outpoints.
    pub(crate) fn resolve_coin_control_exclusions(
        &self,
        coin_control: &WalletCoinControlInfo,
    ) -> WalletCoreResult<Vec<WalletOutPoint>> {
        if coin_control.selection.exclude_outpoints.is_empty() {
            return Ok(Vec::new());
        }
        Ok(coin_control.selection.exclude_outpoints.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::WalletOutPoint;

    #[test]
    fn empty_coin_control_is_reported_correctly() {
        let cc = WalletCoinControlInfo::default();
        assert!(cc.selection.include_outpoints.is_empty());
        assert!(cc.selection.exclude_outpoints.is_empty());
        assert!(!cc.selection.confirmed_only);
        assert!(cc.selection.selection_mode.is_none());
    }

    #[test]
    fn non_empty_coin_control_is_reported_correctly() {
        let cc = WalletCoinControlInfo {
            selection: crate::model::WalletInputSelectionConfig {
                include_outpoints: vec![WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000001:0",
                )
                .unwrap()],
                exclude_outpoints: Vec::new(),
                confirmed_only: false,
                selection_mode: None,
                max_input_count: None,
                min_input_count: None,
                min_utxo_value_sat: None,
                max_utxo_value_sat: None,
                strategy: None,
            },
        };

        assert!(!cc.selection.include_outpoints.is_empty());
        assert!(cc.selection.selection_mode.is_none());
    }

    #[test]
    fn empty_resolution_is_reported_correctly() {
        let resolution = WalletCoinControlResolutionInfo {
            included_outpoints: Vec::new(),
            excluded_outpoints: Vec::new(),
            confirmed_only: false,
            selection_mode: None,
            has_explicit_include_set: false,
        };

        assert!(resolution.is_noop());
        assert!(!resolution.has_constraints());
    }
}
