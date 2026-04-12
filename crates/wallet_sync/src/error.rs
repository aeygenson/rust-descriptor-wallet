use thiserror::Error;
use wallet_core::WalletCoreError;

#[derive(Debug, Error)]
pub enum WalletSyncError {
    // ---- core passthrough ----
    #[error(transparent)]
    Core(#[from] WalletCoreError),

    // ---- backend selection / config ----
    #[error("invalid backend: {0}")]
    InvalidBackend(String),

    // ---- sync / connectivity ----
    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("sync failed: {0}")]
    SyncFailed(String),

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

impl From<bdk_chain::local_chain::CannotConnectError> for WalletSyncError {
    fn from(err: bdk_chain::local_chain::CannotConnectError) -> Self {
        WalletSyncError::SyncFailed(err.to_string())
    }
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