use bitcoin::OutPoint;
use tracing::{debug, info};

use super::psbt_common::parse_outpoint;
use crate::error::WalletCoreError;
use crate::model::WalletCoinControlInfo;
use crate::{WalletCoreResult, WalletService};

impl WalletService {
    /// Resolve and validate explicitly included outpoints.
    ///
    /// This helper does not yet build a PSBT by itself. Instead it provides the
    /// same style of focused, testable service logic as `psbt_cpfp.rs`, so the
    /// next step can wire it into `psbt_create.rs` with minimal churn.
    pub(crate) fn resolve_coin_control_inputs(
        &self,
        coin_control: &WalletCoinControlInfo,
    ) -> WalletCoreResult<Vec<OutPoint>> {
        info!(
            include_count = coin_control.include_outpoints.len(),
            exclude_count = coin_control.exclude_outpoints.len(),
            confirmed_only = coin_control.confirmed_only,
            "resolving coin control inputs"
        );

        if coin_control.is_empty() {
            debug!("coin control request is empty; nothing to resolve");
            return Ok(Vec::new());
        }

        let included = self.parse_unique_outpoints(&coin_control.include_outpoints)?;
        let excluded = self.parse_unique_outpoints(&coin_control.exclude_outpoints)?;

        for outpoint in &included {
            if excluded.contains(outpoint) {
                return Err(WalletCoreError::CoinControlConflict(
                    outpoint.to_string(),
                ));
            }
        }

        let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        for requested in &included {
            let utxo = wallet_utxos
                .iter()
                .find(|u| u.outpoint == *requested)
                .ok_or_else(|| {
                    WalletCoreError::CoinControlOutpointNotFound(requested.to_string())
                })?;

            if coin_control.confirmed_only && !utxo.chain_position.is_confirmed() {
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

        Ok(included)
    }

    /// Resolve and validate explicitly excluded outpoints.
    pub(crate) fn resolve_coin_control_exclusions(
        &self,
        coin_control: &WalletCoinControlInfo,
    ) -> WalletCoreResult<Vec<OutPoint>> {
        if coin_control.exclude_outpoints.is_empty() {
            return Ok(Vec::new());
        }

        self.parse_unique_outpoints(&coin_control.exclude_outpoints)
    }

    fn parse_unique_outpoints(&self, raw: &[String]) -> WalletCoreResult<Vec<OutPoint>> {
        let mut parsed = Vec::with_capacity(raw.len());

        for item in raw {
            let (txid, vout) = parse_outpoint(item).map_err(|e| {
                WalletCoreError::CoinControlInvalidOutpoint(format!(
                    "{} ({})",
                    item, e
                ))
            })?;

            let txid = txid.parse().map_err(|e| {
                WalletCoreError::CoinControlInvalidOutpoint(format!(
                    "{} ({})",
                    item, e
                ))
            })?;

            let outpoint = OutPoint { txid, vout };

            if !parsed.contains(&outpoint) {
                parsed.push(outpoint);
            }
        }

        Ok(parsed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_coin_control_is_reported_correctly() {
        let cc = WalletCoinControlInfo::default();
        assert!(cc.include_outpoints.is_empty());
        assert!(cc.exclude_outpoints.is_empty());
        assert!(!cc.confirmed_only);
    }

    #[test]
    fn non_empty_coin_control_is_reported_correctly() {
        let cc = WalletCoinControlInfo {
            include_outpoints: vec![
                "0000000000000000000000000000000000000000000000000000000000000001:0"
                    .to_string(),
            ],
            exclude_outpoints: Vec::new(),
            confirmed_only: false,
        };

        assert!(!cc.include_outpoints.is_empty());
    }
}