use thiserror::Error;
use wallet_core::WalletCoreError;
#[derive(Debug, Error)]
pub enum WalletSyncError {
    #[error("core error: {0}")]
    Core(#[from] WalletCoreError),
}