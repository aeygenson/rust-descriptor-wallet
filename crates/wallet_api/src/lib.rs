pub mod error;
pub  mod api;
pub  mod factory;
mod service;
pub mod model;
pub use api::WalletApi;
pub use error::WalletApiError;

pub type WalletApiResult<T> = Result<T, WalletApiError>;