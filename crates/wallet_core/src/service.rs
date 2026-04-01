use bdk_file_store::Store;
use bdk_wallet::{ChangeSet, KeychainKind, PersistedWallet, Wallet};

use crate::{WalletConfig, WalletCoreResult};

use tracing::{debug, info, error};

/// Magic bytes used by `bdk_file_store` to identify our database format.
///
/// Changing this will make existing databases incompatible.
const MAGIC_BYTES: &[u8] = b"rust-descriptor-wallet-v1";

#[derive(Debug)]
pub struct WalletService {
    /// BDK persisted wallet wrapper.
    wallet: PersistedWallet<Store<ChangeSet>>,

    /// File-backed store used by the persisted wallet.
    db: Store<ChangeSet>,
}

impl WalletService {
    /// Load existing wallet or create a new one if it does not exist.
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
            None => Wallet::create(
                external_descriptor,
                internal_descriptor,
            )
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
        info!("wallet_service: next_receive_address generated address={}", address_info.address);
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
        let _ = self
            .wallet
            .persist(&mut self.db)
            .map_err(|e| {
                error!("wallet_service: persist error: {}", e);
                crate::WalletCoreError::Persist(e.to_string())
            })?;
        debug!("wallet_service: persist done");
        Ok(())
    }
}