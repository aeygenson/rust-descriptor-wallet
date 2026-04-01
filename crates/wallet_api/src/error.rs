use thiserror::Error;
use wallet_core::error::WalletCoreError;
use wallet_storage::error::WalletStorageError;
use wallet_sync::error::WalletSyncError;

#[derive(Debug, Error)]
pub enum WalletApiError {
    #[error(transparent)]
    Sync(#[from] WalletSyncError),

    #[error(transparent)]
    Storage(#[from] WalletStorageError),

    #[error(transparent)]
    Core(#[from] WalletCoreError),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),
}