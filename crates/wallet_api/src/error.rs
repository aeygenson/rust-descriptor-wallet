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

    #[error("transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("transaction already confirmed: {0}")]
    TransactionAlreadyConfirmed(String),

    #[error("transaction is not replaceable (RBF disabled): {0}")]
    TransactionNotReplaceable(String),

    #[error(
        "fee rate too low for bump (original: {original_sat_per_vb} sat/vB, requested: {requested_sat_per_vb} sat/vB)"
    )]
    FeeRateTooLowForBump {
        original_sat_per_vb: u64,
        requested_sat_per_vb: u64,
    },

    #[error("fee bump build failed: {0}")]
    FeeBumpBuildFailed(String),

    #[error("invalid destination address: {0}")]
    InvalidDestinationAddress(String),

    #[error("destination network mismatch: {0}")]
    DestinationNetworkMismatch(String),

    #[error("psbt build failed: {0}")]
    PsbtBuildFailed(String),

    #[error("fee calculation failed")]
    FeeCalculationFailed,

    #[error("invalid psbt: {0}")]
    InvalidPsbt(String),

    #[error("psbt signing failed: {0}")]
    SignPsbtFailed(String),

    #[error("wallet is watch-only and cannot sign")]
    WatchOnlyCannotSign,

    #[error("psbt is not finalized")]
    PsbtNotFinalized,

    #[error("signed PSBT must be finalized before publish")]
    SendNotFinalized,

    #[error("failed to extract transaction from psbt: {0}")]
    ExtractTxFailed(String),

    #[error("transaction broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("broadcast transport error: {0}")]
    BroadcastTransport(String),

    #[error("mempool conflict: {0}")]
    BroadcastMempoolConflict(String),

    #[error("transaction already confirmed: {0}")]
    BroadcastAlreadyConfirmed(String),

    #[error("missing inputs: {0}")]
    BroadcastMissingInputs(String),

    #[error("insufficient relay fee: {0}")]
    BroadcastInsufficientFee(String),

}

impl From<WalletCoreError> for WalletApiError {
    fn from(e: WalletCoreError) -> Self {
        match e {
            WalletCoreError::InvalidAmount => WalletApiError::InvalidAmount,
            WalletCoreError::InvalidFeeRate => WalletApiError::InvalidFeeRate,
            WalletCoreError::TransactionNotFound(txid) => {
                WalletApiError::TransactionNotFound(txid)
            }
            WalletCoreError::TransactionAlreadyConfirmed(txid) => {
                WalletApiError::TransactionAlreadyConfirmed(txid)
            }
            WalletCoreError::TransactionNotReplaceable(txid) => {
                WalletApiError::TransactionNotReplaceable(txid)
            }
            WalletCoreError::FeeRateTooLowForBump {
                original_sat_per_vb,
                requested_sat_per_vb,
                ..
            } => WalletApiError::FeeRateTooLowForBump {
                original_sat_per_vb: original_sat_per_vb.as_u64(),
                requested_sat_per_vb: requested_sat_per_vb.as_u64(),
            },
            WalletCoreError::FeeBumpBuildFailed { reason, .. } => {
                WalletApiError::FeeBumpBuildFailed(reason)
            }
            WalletCoreError::InvalidDestinationAddress(s) => {
                WalletApiError::InvalidDestinationAddress(s)
            }
            WalletCoreError::DestinationNetworkMismatch(s) => {
                WalletApiError::DestinationNetworkMismatch(s)
            }
            WalletCoreError::PsbtBuildFailed(s) => WalletApiError::PsbtBuildFailed(s),
            WalletCoreError::FeeCalculationFailed => WalletApiError::FeeCalculationFailed,
            WalletCoreError::InvalidPsbt(e) => {
                WalletApiError::InvalidPsbt(e.to_string())
            }
            WalletCoreError::PsbtConversionFailed { reason, .. } => {
                WalletApiError::InvalidPsbt(reason)
            }
            WalletCoreError::SignPsbtFailed(e) => {
                WalletApiError::SignPsbtFailed(e.to_string())
            }
            WalletCoreError::WatchOnlyCannotSign => {
                WalletApiError::WatchOnlyCannotSign
            }
            WalletCoreError::PsbtNotFinalized => {
                WalletApiError::PsbtNotFinalized
            }
            WalletCoreError::ExtractTxFailed(s) => {
                WalletApiError::ExtractTxFailed(s)
            }
            WalletCoreError::BroadcastFailed(s) => {
                let normalized = s.to_ascii_lowercase();

                if normalized.contains("non-final") {
                    WalletApiError::PsbtNotFinalized
                } else {
                    WalletApiError::BroadcastFailed(s)
                }
            }
            WalletCoreError::BroadcastTransport(s) => {
                WalletApiError::BroadcastTransport(s)
            }
            WalletCoreError::BroadcastMempoolConflict(s) => {
                WalletApiError::BroadcastMempoolConflict(s)
            }
            WalletCoreError::BroadcastAlreadyConfirmed(s) => {
                WalletApiError::BroadcastAlreadyConfirmed(s)
            }
            WalletCoreError::BroadcastMissingInputs(s) => {
                WalletApiError::BroadcastMissingInputs(s)
            }
            WalletCoreError::BroadcastInsufficientFee(s) => {
                WalletApiError::BroadcastInsufficientFee(s)
            }
            WalletCoreError::InvalidConfig(s) => {
                WalletApiError::InvalidInput(s)
            }
            other => WalletApiError::Core(other),
        }
    }
}