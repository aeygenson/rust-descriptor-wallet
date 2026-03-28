pub mod error;
mod api;
mod factory;
mod services;
pub mod dto;

pub use api::WalletApi;
pub use error::WalletApiError;

pub type WalletApiResult<T> = Result<T, WalletApiError>;