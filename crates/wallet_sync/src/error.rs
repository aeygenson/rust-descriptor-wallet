use thiserror::Error;
use wallet_core::WalletCoreError;
use bdk_esplora::esplora_client;
use bdk_chain;

#[derive(Debug, Error)]
pub enum WalletSyncError {
    // ---- passthroughs ----
    #[error(transparent)]
    Core(#[from] WalletCoreError),

    #[error(transparent)]
    Esplora(#[from] Box<esplora_client::Error>),

    #[error(transparent)]
    CannotConnectError(#[from] bdk_chain::local_chain::CannotConnectError),

    // ---- broadcast / transport ----
    #[error("broadcast transport error: {0}")]
    BroadcastTransport(String),

    #[error("broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("mempool conflict: {0}")]
    BroadcastMempoolConflict(String),

    #[error("transaction already confirmed: {0}")]
    BroadcastAlreadyConfirmed(String),

    #[error("missing inputs: {0}")]
    BroadcastMissingInputs(String),

    #[error("insufficient fee: {0}")]
    BroadcastInsufficientFee(String),

    #[error("psbt not finalized")]
    PsbtNotFinalized,
}

impl WalletSyncError {
    /// Whether the error is retryable at the transport/backend level.
    pub fn is_retryable(&self) -> bool {
        matches!(
            self,
            WalletSyncError::BroadcastTransport(_)
        )
    }

    /// Map sync-layer errors back into core-layer errors when required by callers.
    pub fn into_core(self) -> WalletCoreError {
        match self {
            WalletSyncError::Core(e) => e,
            WalletSyncError::PsbtNotFinalized => WalletCoreError::PsbtNotFinalized,
            other => WalletCoreError::InvalidState(other.to_string()),
        }
    }
}