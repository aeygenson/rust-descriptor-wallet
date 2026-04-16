use crate::model::PsbtSigningStatus;
use crate::WalletCoreResult;

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

    /// Convenience helper delegating to the model-layer status enum.
    pub fn classify_psbt_signing(&self, modified: bool, finalized: bool) -> PsbtSigningStatus {
        match (modified, finalized) {
            (_, true) => PsbtSigningStatus::Finalized,
            (true, false) => PsbtSigningStatus::PartiallySigned,
            (false, false) => PsbtSigningStatus::Unchanged,
        }
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
    fn classify_psbt_signing_states() {
        let core = WalletCore::new();

        assert_eq!(
            core.classify_psbt_signing(false, false),
            PsbtSigningStatus::Unchanged
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
}
