use thiserror::Error;
use wallet_core::WalletCoreError;
use bdk_esplora::esplora_client;
use bdk_chain;
#[derive(Debug, Error)]
pub enum WalletSyncError {
    #[error(transparent)]
    Core(#[from] WalletCoreError),
    #[error(transparent)]
    Esplora(#[from] Box<esplora_client::Error>),
    #[error(transparent)]
    CannotConnectError(#[from] bdk_chain::local_chain::CannotConnectError)
}