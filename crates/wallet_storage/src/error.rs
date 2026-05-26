use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletStorageError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Migration(#[from] sqlx::migrate::MigrateError),

    #[error(transparent)]
    Serialization(#[from] serde_json::Error),

    #[error(transparent)]
    IO(#[from] std::io::Error),

    #[error("home directory not found")]
    HomeDirNotFound,

    #[error("not found: {0}")]
    NotFound(String),

    #[error("wallet already exists: {0}")]
    AlreadyExists(String),

    #[error("address book label already exists: {0}")]
    DuplicateAddressBookLabel(String),


    #[error("address book address already exists: {0}")]
    DuplicateAddressBookAddress(String),

    #[error("locked utxo already exists: {0}")]
    DuplicateLockedUtxo(String),

    #[error("locked utxo not found: {0}")]
    LockedUtxoNotFound(String),

    #[error("utxo is locked and cannot be spent: {0}")]
    LockedUtxo(String),

    #[error("invalid address book address: {0}")]
    InvalidAddressBookAddress(String),

    #[error("invalid backend config: {0}")]
    InvalidBackend(String),

    #[error("invalid wallet config: {0}")]
    InvalidConfig(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),
}
