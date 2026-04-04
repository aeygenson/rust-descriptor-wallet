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
    Core(WalletCoreError),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("invalid amount")]
    InvalidAmount,

    #[error("invalid fee rate")]
    InvalidFeeRate,

    #[error("invalid destination address: {0}")]
    InvalidDestinationAddress(String),

    #[error("destination network mismatch: {0}")]
    DestinationNetworkMismatch(String),

    #[error("psbt build failed: {0}")]
    PsbtBuildFailed(String),

    #[error("fee calculation failed")]
    FeeCalculationFailed,
}

impl From<WalletCoreError> for WalletApiError {
    fn from(e: WalletCoreError) -> Self {
        match e {
            WalletCoreError::InvalidAmount => WalletApiError::InvalidAmount,
            WalletCoreError::InvalidFeeRate => WalletApiError::InvalidFeeRate,
            WalletCoreError::InvalidDestinationAddress(s) => {
                WalletApiError::InvalidDestinationAddress(s)
            }
            WalletCoreError::DestinationNetworkMismatch(s) => {
                WalletApiError::DestinationNetworkMismatch(s)
            }
            WalletCoreError::PsbtBuildFailed(s) => WalletApiError::PsbtBuildFailed(s),
            WalletCoreError::FeeCalculationFailed => WalletApiError::FeeCalculationFailed,
            other => WalletApiError::Core(other),
        }
    }
}