pub mod db;
pub mod error;
pub mod models;
pub mod repository;

use sqlx::SqlitePool;

pub use error::WalletStorageError;
pub use models::{ImportWalletFile, WalletRecord};

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
        repository::get_wallet_by_name(&self.pool, name).await
    }

    pub async fn list_wallets(&self) -> WalletStorageResult<Vec<WalletRecord>> {
        repository::list_wallets(&self.pool).await
    }

    pub async fn create_wallet(
        &self,
        name: &str,
        network: &str,
        external_descriptor: &str,
        internal_descriptor: &str,
        esplora_url: &str,
        is_watch_only: bool,
    ) -> WalletStorageResult<()> {
        repository::create_wallet(
            &self.pool,
            name,
            network,
            external_descriptor,
            internal_descriptor,
            esplora_url,
            is_watch_only,
        )
        .await
    }

    pub async fn delete_wallet(&self, name: &str) -> WalletStorageResult<()> {
        repository::delete_wallet(&self.pool, name).await
    }

    pub async fn import_wallet_from_file(&self, file_path: &str) -> WalletStorageResult<()> {
        repository::import_wallet_from_file(&self.pool, file_path).await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

pub use db::{default_app_path, default_db_path, default_wallet_db_path};
pub use repository::{
    create_wallet,
    delete_wallet,
    get_wallet_by_name,
    import_wallet_from_file,
    list_wallets,
};