pub mod error;
pub mod config;
pub mod core;
pub mod service;
pub mod model;
pub mod types;

pub use config::WalletConfig;
pub use core::WalletCore;
pub use error::WalletCoreError;
pub use service::WalletService;

pub type WalletCoreResult<T> = Result<T, WalletCoreError>;
