use std::collections::HashSet;

use crate::model::PsbtSigningStatus;
use crate::types::WalletOutPoint;
use crate::{WalletCoreError, WalletCoreResult};

/// Core domain layer.
///
/// This type hosts pure business logic only: validation and policy helpers
/// that do not require IO, networking, persistence, or wallet database access.
#[derive(Debug, Default)]
pub struct WalletCore;

impl WalletCore {
    pub fn new() -> Self {
        Self
    }

    /// Returns true when a descriptor string appears to contain private key
    /// material and therefore should be able to produce a signing keymap.
    ///
    /// This is intentionally a lightweight heuristic, not full descriptor
    /// parsing or semantic validation.
    pub fn descriptor_looks_private(&self, descriptor: &str) -> bool {
        descriptor.contains("xprv")
            || descriptor.contains("tprv")
            || descriptor.contains("yprv")
            || descriptor.contains("zprv")
    }

    /// Validate a software-signing wallet configuration at the pure domain level.
    pub fn validate_signing_descriptors(
        &self,
        external_descriptor: &str,
        internal_descriptor: &str,
        is_watch_only: bool,
    ) -> WalletCoreResult<()> {
        let external_private = self.descriptor_looks_private(external_descriptor);
        let internal_private = self.descriptor_looks_private(internal_descriptor);

        if is_watch_only && (external_private || internal_private) {
            return Err(crate::WalletCoreError::InvalidConfig(
                "watch-only wallet must not contain private descriptors".to_string(),
            ));
        }

        if !is_watch_only && (!external_private || !internal_private) {
            return Err(crate::WalletCoreError::InvalidConfig(
                "software-signing wallet requires private descriptors for both keychains"
                    .to_string(),
            ));
        }

        Ok(())
    }

    /// Returns true when the wallet mode allows local software signing.
    ///
    /// This is a pure domain helper for API/UI/workflow layers that need to
    /// reason about signing capability without duplicating watch-only logic.
    pub fn can_sign_locally(&self, is_watch_only: bool) -> bool {
        !is_watch_only
    }

    /// Convenience helper delegating to the model-layer status enum.
    pub fn classify_psbt_signing(&self, modified: bool, finalized: bool) -> PsbtSigningStatus {
        match (modified, finalized) {
            (_, true) => PsbtSigningStatus::Finalized,
            (true, false) => PsbtSigningStatus::PartiallySigned,
            (false, false) => PsbtSigningStatus::Unsigned,
        }
    }

    /// Returns the overlap between selected and locked outpoints.
    ///
    /// This helper is intentionally pure and deterministic. Storage/repository
    /// layers are responsible for loading lock state before calling into core.
    pub fn locked_outpoint_overlap(
        &self,
        selected: &[WalletOutPoint],
        locked: &[WalletOutPoint],
    ) -> Vec<WalletOutPoint> {
        let locked_set: HashSet<_> = locked.iter().cloned().collect();

        selected
            .iter()
            .filter(|outpoint| locked_set.contains(*outpoint))
            .cloned()
            .collect()
    }

    /// Validates that selected outpoints do not overlap locked outpoints.
    ///
    /// Intended for explicit/manual coin-control flows where spending a locked
    /// coin should fail with a user-correctable selection error.
    pub fn ensure_outpoints_unlocked(
        &self,
        selected: &[WalletOutPoint],
        locked: &[WalletOutPoint],
    ) -> WalletCoreResult<()> {
        let overlap = self.locked_outpoint_overlap(selected, locked);

        if let Some(first) = overlap.first() {
            return Err(WalletCoreError::LockedUtxo(first.to_string()));
        }

        Ok(())
    }

    /// Returns a merged exclusion set containing explicit exclusions plus
    /// implicitly excluded locked outpoints.
    ///
    /// Duplicates are automatically removed.
    pub fn merge_locked_into_excluded(
        &self,
        excluded: &[WalletOutPoint],
        locked: &[WalletOutPoint],
    ) -> Vec<WalletOutPoint> {
        let mut merged: HashSet<WalletOutPoint> = excluded.iter().cloned().collect();

        merged.extend(locked.iter().cloned());

        let mut result: Vec<_> = merged.into_iter().collect();
        result.sort();
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn descriptor_looks_private_detects_private_keys() {
        let core = WalletCore::new();

        assert!(core.descriptor_looks_private("wpkh(xprv...)"));
        assert!(core.descriptor_looks_private("tr(tprv...)"));
        assert!(core.descriptor_looks_private("yprv..."));
        assert!(core.descriptor_looks_private("zprv..."));
    }

    #[test]
    fn descriptor_looks_private_rejects_public_only() {
        let core = WalletCore::new();

        assert!(!core.descriptor_looks_private("wpkh(xpub...)"));
        assert!(!core.descriptor_looks_private("tr(tpub...)"));
    }

    #[test]
    fn validate_signing_descriptors_rejects_watch_only_with_private() {
        let core = WalletCore::new();

        let result = core.validate_signing_descriptors("wpkh(xprv...)", "wpkh(xprv...)", true);

        assert!(matches!(
            result,
            Err(crate::WalletCoreError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_signing_descriptors_rejects_signing_without_private() {
        let core = WalletCore::new();

        let result = core.validate_signing_descriptors("wpkh(xpub...)", "wpkh(xpub...)", false);

        assert!(matches!(
            result,
            Err(crate::WalletCoreError::InvalidConfig(_))
        ));
    }

    #[test]
    fn validate_signing_descriptors_accepts_valid_signing_wallet() {
        let core = WalletCore::new();

        let result = core.validate_signing_descriptors("wpkh(xprv...)", "wpkh(xprv...)", false);

        assert!(result.is_ok());
    }

    #[test]
    fn can_sign_locally_reflects_watch_only_mode() {
        let core = WalletCore::new();

        assert!(core.can_sign_locally(false));
        assert!(!core.can_sign_locally(true));
    }

    #[test]
    fn classify_psbt_signing_states() {
        let core = WalletCore::new();

        assert_eq!(
            core.classify_psbt_signing(false, false),
            PsbtSigningStatus::Unsigned
        );

        assert_eq!(
            core.classify_psbt_signing(true, false),
            PsbtSigningStatus::PartiallySigned
        );

        assert_eq!(
            core.classify_psbt_signing(true, true),
            PsbtSigningStatus::Finalized
        );

        assert_eq!(
            core.classify_psbt_signing(false, true),
            PsbtSigningStatus::Finalized
        );
    }

    #[test]
    fn locked_outpoint_overlap_detects_intersection() {
        let core = WalletCore::new();

        let selected = vec![
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000001:0",
            )
            .expect("valid selected outpoint"),
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000002:1",
            )
            .expect("valid selected outpoint"),
        ];

        let locked = vec![
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000002:1",
            )
            .expect("valid locked outpoint"),
        ];

        let overlap = core.locked_outpoint_overlap(&selected, &locked);

        assert_eq!(overlap.len(), 1);
        assert_eq!(overlap[0], locked[0]);
    }

    #[test]
    fn ensure_outpoints_unlocked_accepts_non_locked_selection() {
        let core = WalletCore::new();

        let selected = vec![
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000003:0",
            )
            .expect("valid selected outpoint"),
        ];

        let locked = vec![
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000004:1",
            )
            .expect("valid locked outpoint"),
        ];

        let result = core.ensure_outpoints_unlocked(&selected, &locked);

        assert!(result.is_ok());
    }

    #[test]
    fn ensure_outpoints_unlocked_rejects_locked_selection() {
        let core = WalletCore::new();

        let locked_outpoint = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000005:2",
        )
        .expect("valid locked outpoint");

        let selected = vec![locked_outpoint.clone()];
        let locked = vec![locked_outpoint.clone()];

        let result = core.ensure_outpoints_unlocked(&selected, &locked);

        assert!(matches!(
            result,
            Err(WalletCoreError::LockedUtxo(message))
                if message == locked_outpoint.to_string()
        ));
    }

    #[test]
    fn merge_locked_into_excluded_deduplicates_and_sorts() {
        let core = WalletCore::new();

        let excluded = vec![
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000007:1",
            )
            .expect("valid excluded outpoint"),
        ];

        let locked = vec![
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000006:0",
            )
            .expect("valid locked outpoint"),
            WalletOutPoint::parse(
                "0000000000000000000000000000000000000000000000000000000000000007:1",
            )
            .expect("duplicate locked outpoint"),
        ];

        let merged = core.merge_locked_into_excluded(&excluded, &locked);

        assert_eq!(merged.len(), 2);
        assert!(merged.contains(&excluded[0]));
        assert!(merged.contains(&locked[0]));
    }
}
