use thiserror::Error;
use wallet_storage::error::WalletStorageError;
use wallet_sync::error::WalletSyncError;
use wallet_core::error::WalletCoreError;
#[derive(Debug, Error)]
pub enum WalletApiError {
    #[error(transparent)]
    Sync(#[from] WalletSyncError),
    #[error(transparent)]
    Storage(#[from] WalletStorageError),
    #[error(transparent)]
    Core(#[from] WalletCoreError),
}