use crate::{WalletConfig, WalletCoreResult};
use bdk_wallet::descriptor::IntoWalletDescriptor;
use bdk_wallet::keys::KeyMap;
use bdk_wallet::KeychainKind;
use tracing::{debug, error, info};

use super::*;

impl WalletService {
    /// Load existing wallet or create a new one if it does not exist.
    ///
    /// Flow:
    /// 1. Open or create file store
    /// 2. Try to load wallet from store
    /// 3. If not found → create new wallet
    /// 4. Persist initial state
    pub fn load_or_create(config: &WalletConfig) -> WalletCoreResult<Self> {
        info!(
            "wallet_service: load_or_create start path={}",
            config.db_path.display()
        );
        let (mut db, _changeset) =
            Store::<ChangeSet>::load_or_create(MAGIC_BYTES, &config.db_path)?;

        let external_descriptor = config.external_descriptor().to_string();
        let internal_descriptor = config.internal_descriptor().to_string();
        let core = crate::core::WalletCore::new();
        core.validate_signing_descriptors(
            config.external_descriptor(),
            config.internal_descriptor(),
            config.is_watch_only,
        )?;

        let mut wallet = match Wallet::load()
            .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
            .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
            .check_network(config.network)
            .load_wallet(&mut db)
            .map_err(|e| {
                error!("wallet_service: load_wallet error: {}", e);
                crate::WalletCoreError::Load(format!(
                    "failed to load wallet from {}: {}",
                    config.db_path.display(),
                    e
                ))
            })? {
            Some(wallet) => wallet,
            None => Wallet::create(external_descriptor, internal_descriptor)
                .network(config.network)
                .create_wallet(&mut db)
                .map_err(|e| {
                    error!("wallet_service: create_wallet error: {}", e);
                    crate::WalletCoreError::Create(format!(
                        "failed to create wallet at {}: {}",
                        config.db_path.display(),
                        e
                    ))
                })?,
        };

        Self::attach_signers_if_present(&mut wallet, config)?;
        info!(
            "wallet_service: load_or_create success path={} network={:?} watch_only= {}",
            config.db_path.display(),
            config.network,
            config.is_watch_only
        );
        Ok(Self {
            wallet,
            db,
            is_watch_only: config.is_watch_only,
        })
    }

    /// Attach explicit keymaps for private descriptors during wallet initialization.
    fn attach_signers_if_present(
        wallet: &mut Wallet,
        config: &WalletConfig,
    ) -> WalletCoreResult<()> {
        let core = crate::core::WalletCore::new();
        let external_private = core.descriptor_looks_private(config.external_descriptor());
        let internal_private = core.descriptor_looks_private(config.internal_descriptor());

        debug!(
            "wallet_service: attach_signers_if_present external_private={} internal_private={}",
            external_private, internal_private
        );

        if config.is_watch_only {
            debug!(
                "wallet_service: attach_signers_if_present skipped because wallet is watch-only"
            );
            return Ok(());
        }

        if !external_private && !internal_private {
            debug!(
                "wallet_service: attach_signers_if_present no private descriptor material detected; wallet is effectively watch-only"
            );
            return Ok(());
        }

        let secp = bdk_wallet::bitcoin::secp256k1::Secp256k1::new();

        if external_private {
            let (_descriptor, external_keymap): (_, KeyMap) = config
                .external_descriptor()
                .to_string()
                .into_wallet_descriptor(&secp, config.network)
                .map_err(|e| {
                    crate::WalletCoreError::Create(format!(
                        "external signer descriptor parse error: {}",
                        e
                    ))
                })?;

            debug!(
                "wallet_service: attach_signers_if_present setting external keymap entries={}",
                external_keymap.len()
            );
            wallet.set_keymap(KeychainKind::External, external_keymap);
        }

        if internal_private {
            let (_descriptor, internal_keymap): (_, KeyMap) = config
                .internal_descriptor()
                .to_string()
                .into_wallet_descriptor(&secp, config.network)
                .map_err(|e| {
                    crate::WalletCoreError::Create(format!(
                        "internal signer descriptor parse error: {}",
                        e
                    ))
                })?;

            debug!(
                "wallet_service: attach_signers_if_present setting internal keymap entries={}",
                internal_keymap.len()
            );
            wallet.set_keymap(KeychainKind::Internal, internal_keymap);
        }

        debug!("wallet_service: attach_signers_if_present explicit keymap attachment complete");
        Ok(())
    }

    /// Reveal next receive address.
    ///
    /// Important:
    /// - Advances internal derivation index
    /// - MUST be persisted to avoid address reuse
    pub fn next_receive_address(&mut self) -> WalletCoreResult<String> {
        debug!("wallet_service: next_receive_address start");
        let address_info = self.wallet.reveal_next_address(KeychainKind::External);
        self.persist()?;
        info!(
            "wallet_service: next_receive_address generated address={}",
            address_info.address
        );
        Ok(address_info.address.to_string())
    }

    /// Return total wallet balance in satoshis.
    pub fn balance_sat(&self) -> WalletCoreResult<u64> {
        debug!("wallet_service: balance_sat queried");
        Ok(self.wallet.balance().total().to_sat())
    }

    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    /// Whether this wallet is watch-only and cannot sign transactions.
    pub fn is_watch_only(&self) -> bool {
        self.is_watch_only
    }

    /// Mutable access to underlying BDK wallet.
    ///
    /// Used by sync layer to apply blockchain updates.
    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    /// Persist staged wallet changes to disk.
    pub fn persist(&mut self) -> WalletCoreResult<()> {
        debug!("wallet_service: persist start");
        let _ = self.wallet.persist(&mut self.db).map_err(|e| {
            error!("wallet_service: persist error: {}", e);
            crate::WalletCoreError::Persist(e.to_string())
        })?;
        debug!("wallet_service: persist done");
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::test_support::test_support::{signing_test_config, test_config};

    #[test]
    fn load_or_create_creates_wallet_successfully() {
        let config = test_config();

        let wallet = WalletService::load_or_create(&config);

        assert!(
            wallet.is_ok(),
            "expected wallet to load or create successfully"
        );
    }

    #[test]
    fn load_or_create_creates_signing_wallet_successfully() {
        let config = signing_test_config();

        let wallet = WalletService::load_or_create(&config);

        assert!(
            wallet.is_ok(),
            "expected signing wallet to load or create successfully"
        );
    }

    #[test]
    fn load_or_create_rejects_watch_only_wallet_with_private_descriptors() {
        let mut config = signing_test_config();
        config.is_watch_only = true;

        let err = WalletService::load_or_create(&config)
            .expect_err("watch-only wallet with private descriptors must be rejected");

        match err {
            crate::WalletCoreError::InvalidConfig(message) => {
                assert!(
                    message.contains("watch-only wallet must not contain private descriptors"),
                    "unexpected error message: {}",
                    message
                );
            }
            other => panic!("expected InvalidConfig, got {:?}", other),
        }
    }

    #[test]
    fn load_or_create_rejects_signing_wallet_with_public_descriptors_only() {
        let mut config = test_config();
        config.is_watch_only = false;

        let err = WalletService::load_or_create(&config)
            .expect_err("software-signing wallet with only public descriptors must be rejected");

        match err {
            crate::WalletCoreError::InvalidConfig(message) => {
                assert!(
                    message.contains("software-signing wallet requires private descriptors"),
                    "unexpected error message: {}",
                    message
                );
            }
            other => panic!("expected InvalidConfig, got {:?}", other),
        }
    }

    #[test]
    fn next_receive_address_returns_different_addresses() {
        let config = test_config();
        let mut wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let addr1 = wallet
            .next_receive_address()
            .expect("first receive address should be created");
        let addr2 = wallet
            .next_receive_address()
            .expect("second receive address should be created");

        assert_ne!(addr1, addr2, "receive addresses should advance and differ");
    }

    #[test]
    fn next_receive_address_persists_index_across_reload() {
        let config = test_config();

        let first_address = {
            let mut wallet = WalletService::load_or_create(&config)
                .expect("wallet should load or create successfully");
            wallet
                .next_receive_address()
                .expect("first receive address should be created")
        };

        let second_address_after_reload = {
            let mut reloaded =
                WalletService::load_or_create(&config).expect("wallet should reload successfully");
            reloaded
                .next_receive_address()
                .expect("next receive address after reload should be created")
        };

        assert_ne!(
            first_address, second_address_after_reload,
            "reloaded wallet should continue from persisted derivation state"
        );
    }

    #[test]
    fn balance_is_zero_for_fresh_wallet() {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let balance = wallet.balance_sat().expect("balance should be readable");

        assert_eq!(balance, 0, "fresh wallet should have zero balance");
    }
}
