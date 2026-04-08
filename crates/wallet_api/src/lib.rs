pub mod error;
mod api;
mod factory;
mod service;
pub mod dto;
pub mod broadcaster;

pub use api::WalletApi;
pub use error::WalletApiError;

pub type WalletApiResult<T> = Result<T, WalletApiError>;