pub mod db;
pub mod error;
pub mod models;
pub mod repository;


use repository::{receive_history, wallets};
use sqlx::SqlitePool;

pub use error::WalletStorageError;
pub use models::{ImportWalletFile, ReceiveAddressHistoryRecord, WalletRecord};

pub type WalletStorageResult<T> = Result<T, WalletStorageError>;

#[derive(Debug, Clone)]
pub struct WalletStorage {
    pool: SqlitePool,
}

impl WalletStorage {
    pub async fn connect() -> WalletStorageResult<Self> {
        let pool = db::connect().await?;
        Ok(Self { pool })
    }

    pub async fn migrate(&self) -> WalletStorageResult<()> {
        db::migrate(&self.pool).await
    }

    pub async fn get_wallet_by_name(&self, name: &str) -> WalletStorageResult<WalletRecord> {
        wallets::get_wallet_by_name(&self.pool, name).await
    }

    pub async fn list_wallets(&self) -> WalletStorageResult<Vec<WalletRecord>> {
        wallets::list_wallets(&self.pool).await
    }

    pub async fn create_wallet(
        &self,
        name: &str,
        network: &str,
        external_descriptor: &str,
        internal_descriptor: &str,
        sync_backend: &str,
        broadcast_backend: Option<&str>,
        is_watch_only: bool,
    ) -> WalletStorageResult<()> {
        wallets::create_wallet(
            &self.pool,
            name,
            network,
            external_descriptor,
            internal_descriptor,
            sync_backend,
            broadcast_backend,
            is_watch_only,
        )
        .await
    }

    pub async fn delete_wallet(&self, name: &str) -> WalletStorageResult<()> {
        wallets::delete_wallet(&self.pool, name).await
    }

    pub async fn import_wallet_from_file(&self, file_path: &str) -> WalletStorageResult<()> {
        wallets::import_wallet_from_file(&self.pool, file_path).await
    }

    pub async fn record_receive_address(
        &self,
        wallet_name: &str,
        address: &str,
        keychain: &str,
        address_index: Option<i64>,
        bitcoin_uri: &str,
    ) -> WalletStorageResult<ReceiveAddressHistoryRecord> {
        receive_history::record_receive_address(
            &self.pool,
            wallet_name,
            address,
            keychain,
            address_index,
            bitcoin_uri,
        )
        .await
    }

    pub async fn list_receive_addresses(
        &self,
        wallet_name: &str,
    ) -> WalletStorageResult<Vec<ReceiveAddressHistoryRecord>> {
        receive_history::list_receive_addresses(&self.pool, wallet_name).await
    }

    pub async fn get_receive_address_by_wallet_and_address(
        &self,
        wallet_name: &str,
        address: &str,
    ) -> WalletStorageResult<Option<ReceiveAddressHistoryRecord>> {
        receive_history::get_receive_address_by_wallet_and_address(
            &self.pool,
            wallet_name,
            address,
        )
        .await
    }

    pub async fn label_receive_address(
        &self,
        wallet_name: &str,
        address: &str,
        label: &str,
    ) -> WalletStorageResult<ReceiveAddressHistoryRecord> {
        receive_history::label_receive_address(
            &self.pool,
            wallet_name,
            address,
            label,
        )
        .await
    }

    pub async fn clear_receive_address_label(
        &self,
        wallet_name: &str,
        address: &str,
    ) -> WalletStorageResult<ReceiveAddressHistoryRecord> {
        receive_history::clear_receive_address_label(
            &self.pool,
            wallet_name,
            address,
        )
        .await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

pub use db::{default_app_path, default_db_path, default_wallet_db_path};
pub use wallets::{
    create_wallet, delete_wallet, get_wallet_by_name, import_wallet_from_file, list_wallets,
};

pub use receive_history::{
    clear_receive_address_label,
    get_receive_address_by_wallet_and_address,
    label_receive_address,
    list_receive_addresses,
    record_receive_address,
};
