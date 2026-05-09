pub mod api;
pub mod error;
pub mod factory;
pub mod model;
pub  mod service;
pub use api::WalletApi;
pub use error::WalletApiError;

pub type WalletApiResult<T> = Result<T, WalletApiError>;
