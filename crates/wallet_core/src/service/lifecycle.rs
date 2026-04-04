use bdk_wallet::KeychainKind;
use tracing::{debug, error, info};
use crate::{WalletConfig, WalletCoreResult};
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
        info!("wallet_service: load_or_create start path={}", config.db_path.display());
        let (mut db, _changeset) =
            Store::<ChangeSet>::load_or_create(MAGIC_BYTES, &config.db_path)?;

        let external_descriptor = config.external_descriptor.clone();
        let internal_descriptor = config.internal_descriptor.clone();

        let wallet = match Wallet::load()
            .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
            .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
            .check_network(config.network)
            .load_wallet(&mut db)
            .map_err(|e| {
                error!("wallet_service: load_wallet error: {}", e);
                crate::WalletCoreError::Load(e.to_string())
            })?
        {
            None => Wallet::create(external_descriptor, internal_descriptor)
                .network(config.network)
                .create_wallet(&mut db)
                .map_err(|e| {
                    error!("wallet_service: create_wallet error: {}", e);
                    crate::WalletCoreError::Create(e.to_string())
                })?,
            Some(wallet) => wallet,
        };

        info!("wallet_service: load_or_create success");
        Ok(Self { wallet, db })
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
    use bitcoin::Network;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            external_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
            internal_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            db_path: unique_test_db_path("wallet_core_lifecycle"),
            esplora_url: "https://mempool.space/signet/api".to_string(),
        }
    }

    fn unique_test_db_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir().join(format!("{}_{}_{}.db", prefix, std::process::id(), nanos))
    }

    #[test]
    fn load_or_create_creates_wallet_successfully() {
        let config = test_config();

        let wallet = WalletService::load_or_create(&config);

        assert!(wallet.is_ok(), "expected wallet to load or create successfully");
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
            let mut reloaded = WalletService::load_or_create(&config)
                .expect("wallet should reload successfully");
            reloaded
                .next_receive_address()
                .expect("next receive address after reload should be created")
        };

        assert_ne!(
            first_address,
            second_address_after_reload,
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
