use thiserror::Error;
#[derive(Debug, Error)]
pub enum WalletCoreError {
    #[error("invalid state")]
    InvalidState,
}