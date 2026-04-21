use crate::model::{WalletTxDto, WalletUtxoDto};
use crate::WalletApiResult;
use tokio::task;

use wallet_core::WalletService;
use wallet_storage::WalletStorage;

use super::wallet::load_wallet_config;

use tracing::{debug, info};

async fn spawn_wallet_blocking<T, E>(
    f: impl FnOnce() -> Result<T, E> + Send + 'static,
) -> WalletApiResult<T>
where
    T: Send + 'static,
    E: Into<crate::WalletApiError> + Send + 'static,
{
    task::spawn_blocking(f)
        .await
        .map_err(|e| {
            crate::WalletApiError::InvalidInput(format!("blocking wallet task failed: {e}"))
        })?
        .map_err(Into::into)
}

/// Return wallet transaction history using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn txs(storage: &WalletStorage, name: &str) -> WalletApiResult<Vec<WalletTxDto>> {
    debug!("api inspect: txs start name={}", name);

    let config = load_wallet_config(storage, name).await?;

    let txs: Vec<WalletTxDto> = spawn_wallet_blocking(move || {
        let wallet = WalletService::load_or_create(&config)?;

        let txs: Vec<WalletTxDto> = wallet.transactions().into_iter().map(Into::into).collect();

        Ok::<_, crate::WalletApiError>(txs)
    })
    .await?;

    info!("api inspect: txs success name={} count={}", name, txs.len());

    Ok(txs)
}

/// Return wallet UTXOs using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn utxos(storage: &WalletStorage, name: &str) -> WalletApiResult<Vec<WalletUtxoDto>> {
    debug!("api inspect: utxos start name={}", name);

    let config = load_wallet_config(storage, name).await?;

    let utxos: Vec<WalletUtxoDto> = spawn_wallet_blocking(move || {
        let wallet = WalletService::load_or_create(&config)?;

        let utxos: Vec<WalletUtxoDto> = wallet.utxos().into_iter().map(Into::into).collect();

        Ok::<_, crate::WalletApiError>(utxos)
    })
    .await?;

    info!(
        "api inspect: utxos success name={} count={}",
        name,
        utxos.len()
    );

    Ok(utxos)
}
