pub mod error;
mod service;
mod esplora;

pub use error::WalletSyncError;
pub use service::WalletSync;

pub type WalletSyncResult<T> = std::result::Result<T, WalletSyncError>;
