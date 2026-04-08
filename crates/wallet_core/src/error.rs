use bitcoin::psbt::PsbtParseError;
use thiserror::Error;

#[derive(Debug, Error )]
pub enum WalletCoreError {
    #[error("invalid state: {0}")]
    InvalidState(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error(transparent)]
    Store(#[from] bdk_file_store::StoreError),

    #[error(transparent)]
    StoreWithDump(#[from] bdk_file_store::StoreErrorWithDump<bdk_wallet::ChangeSet>),

    #[error("wallet load error: {0}")]
    Load(String),

    #[error("wallet create error: {0}")]
    Create(String),

    #[error("wallet persist error: {0}")]
    Persist(String),
    #[error("invalid fee rate")]
    InvalidFeeRate,

    #[error("invalid amount")]
    InvalidAmount,

    #[error("invalid destination address: {0}")]
    InvalidDestinationAddress(String),

    #[error("destination address network mismatch: {0}")]
    DestinationNetworkMismatch(String),

    #[error("psbt build failed: {0}")]
    PsbtBuildFailed(String),

    #[error("fee calculation failed")]
    FeeCalculationFailed,

    #[error("invalid psbt: {0}")]
    InvalidPsbt(String),
    
    #[allow(deprecated)]
    #[error(transparent)]
    SignPsbtFailed(#[from] bdk_wallet::signer::SignerError),

    #[error("wallet is watch-only and cannot sign")]
    WatchOnlyCannotSign,

    #[error("psbt is not finalized")]
    PsbtNotFinalized,

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

    #[error("insufficient fee: {0}")]
    BroadcastInsufficientFee(String),
}

impl From<PsbtParseError> for WalletCoreError {
    fn from(e: PsbtParseError) -> Self {
        WalletCoreError::InvalidPsbt(e.to_string())
    }
}