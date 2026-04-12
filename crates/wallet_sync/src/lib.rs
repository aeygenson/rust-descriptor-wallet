pub mod error;
mod service;

pub mod broadcast;
pub mod backend;
pub mod model;

pub use error::WalletSyncError;
pub use service::WalletSyncService;

pub type WalletSyncResult<T> = std::result::Result<T, WalletSyncError>;
