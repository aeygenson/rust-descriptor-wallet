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
}