use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletStorageError {
    #[error(transparent)]
    Database(#[from] sqlx::Error),

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

    #[error("invalid backend config: {0}")]
    InvalidBackend(String),

    #[error("invalid wallet config: {0}")]
    InvalidConfig(String),

    #[error("invalid path: {0}")]
    InvalidPath(String),
}
