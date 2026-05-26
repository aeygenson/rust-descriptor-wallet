pub mod db;
pub mod error;
pub mod models;
pub mod repository;


use repository::{address_book, locked_utxos, receive_history, wallets};
use sqlx::SqlitePool;

pub use error::WalletStorageError;
pub use models::{
    AddressBookEntryRecord,
    ImportWalletFile,
    LockedUtxoRecord,
    ReceiveAddressHistoryRecord,
    WalletRecord,
};

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

    // ---------------------------------------------------------------------
    // Wallets
    // ---------------------------------------------------------------------
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

    // ---------------------------------------------------------------------
    // Receive history
    // ---------------------------------------------------------------------
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

    // ---------------------------------------------------------------------
    // Address book
    // ---------------------------------------------------------------------
    pub async fn create_address_book_entry(
        &self,
        wallet_name: &str,
        network: &str,
        label: &str,
        address: &str,
        notes: Option<&str>,
    ) -> WalletStorageResult<AddressBookEntryRecord> {
        address_book::create_entry(
            &self.pool,
            wallet_name,
            network,
            label,
            address,
            notes,
        )
        .await
    }

    pub async fn list_address_book_entries(
        &self,
        wallet_name: &str,
    ) -> WalletStorageResult<Vec<AddressBookEntryRecord>> {
        address_book::list_entries(&self.pool, wallet_name).await
    }

    pub async fn get_address_book_entry_by_address(
        &self,
        wallet_name: &str,
        address: &str,
    ) -> WalletStorageResult<Option<AddressBookEntryRecord>> {
        address_book::get_entry_by_address(
            &self.pool,
            wallet_name,
            address,
        )
        .await
    }

    pub async fn delete_address_book_entry(
        &self,
        wallet_name: &str,
        address: &str,
    ) -> WalletStorageResult<bool> {
        address_book::delete_entry(
            &self.pool,
            wallet_name,
            address,
        )
        .await
    }

    // ---------------------------------------------------------------------
    // Locked UTXOs
    // ---------------------------------------------------------------------
    pub async fn lock_utxo(
        &self,
        wallet_name: &str,
        outpoint: &str,
        reason: Option<&str>,
    ) -> WalletStorageResult<LockedUtxoRecord> {
        locked_utxos::lock_utxo(
            &self.pool,
            wallet_name,
            outpoint,
            reason,
        )
        .await
    }

    pub async fn list_locked_utxos(
        &self,
        wallet_name: &str,
    ) -> WalletStorageResult<Vec<LockedUtxoRecord>> {
        locked_utxos::list_locked_utxos(&self.pool, wallet_name).await
    }

    pub async fn get_locked_utxo(
        &self,
        wallet_name: &str,
        outpoint: &str,
    ) -> WalletStorageResult<Option<LockedUtxoRecord>> {
        locked_utxos::get_locked_utxo(
            &self.pool,
            wallet_name,
            outpoint,
        )
        .await
    }

    pub async fn is_utxo_locked(
        &self,
        wallet_name: &str,
        outpoint: &str,
    ) -> WalletStorageResult<bool> {
        locked_utxos::is_locked(
            &self.pool,
            wallet_name,
            outpoint,
        )
        .await
    }

    pub async fn unlock_utxo(
        &self,
        wallet_name: &str,
        outpoint: &str,
    ) -> WalletStorageResult<bool> {
        locked_utxos::unlock_utxo(
            &self.pool,
            wallet_name,
            outpoint,
        )
        .await
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

pub use db::{default_app_path, default_db_path, default_wallet_db_path};

// -------------------------------------------------------------------------
// Re-exported repository helpers
// -------------------------------------------------------------------------
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

pub use address_book::{
    create_entry as create_address_book_entry,
    delete_entry as delete_address_book_entry,
    get_entry_by_address as get_address_book_entry_by_address,
    list_entries as list_address_book_entries,
};

pub use locked_utxos::{
    get_locked_utxo,
    is_locked as is_utxo_locked,
    list_locked_utxos,
    lock_utxo,
    unlock_utxo,
};
