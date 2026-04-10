pub mod error;
mod service;
mod esplora_sync;
pub mod esplora_broadcast;
pub mod broadcast;

pub use error::WalletSyncError;
pub use service::WalletSyncService;

pub type WalletSyncResult<T> = std::result::Result<T, WalletSyncError>;
