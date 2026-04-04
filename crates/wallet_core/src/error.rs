use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletCoreError {
    #[error("invalid state")]
    InvalidState,

    #[error("not implemented")]
    NotImplemented,

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
}