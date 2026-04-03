use std::sync::Arc;

use bitcoin::Network;
use tracing::{debug, info};

use crate::dto::{WalletStatusDto, WalletTxDto, WalletUtxoDto};
use crate::{WalletApiError, WalletApiResult};
use wallet_core::{WalletConfig, WalletCore, WalletService};
use wallet_storage::WalletStorage;
use wallet_sync::WalletSync;

/// Load wallet configuration from storage and convert it into the core config.
async fn load_wallet_config(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<WalletConfig> {
    let record = storage.get_wallet_by_name(name).await?;
    let network = parse_network(&record.network)?;

    Ok(WalletConfig {
        network,
        external_descriptor: record.external_descriptor,
        internal_descriptor: record.internal_descriptor,
        db_path: wallet_storage::default_wallet_db_path(&record.name)?,
        esplora_url: record.esplora_url,
    })
}

/// Parse stored network string into the Bitcoin network enum.
#[allow(clippy::result_large_err)]
fn parse_network(s: &str) -> WalletApiResult<Network> {
    match s {
        "testnet" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "mainnet" => Ok(Network::Bitcoin),
        _ => Err(WalletApiError::InvalidInput(format!("unknown network: {s}"))),
    }
}

pub async fn address(storage: &WalletStorage, name: &str) -> WalletApiResult<String> {
    debug!("api runtime: address start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;
    let address = wallet.next_receive_address()?;
    info!("api runtime: address success name={}", name);
    Ok(address)
}

pub async fn sync(storage: &WalletStorage, name: &str) -> WalletApiResult<()> {
    debug!("api runtime: sync start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;

    let core = Arc::new(WalletCore::new());
    let sync_service = WalletSync::new(core);
    sync_service.sync(&mut wallet, &config).await?;

    info!("api runtime: sync success name={}", name);
    Ok(())
}

pub async fn balance(storage: &WalletStorage, name: &str) -> WalletApiResult<u64> {
    debug!("api runtime: balance start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;
    let balance = wallet.balance_sat()?;
    info!("api runtime: balance success name={} balance={}", name, balance);
    Ok(balance)
}

/// Return wallet transaction history using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn txs(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<Vec<WalletTxDto>> {
    debug!("api runtime: txs start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;

    let txs: Vec<WalletTxDto> = wallet
        .transactions()
        .into_iter()
        .map(Into::into)
        .collect();

    info!("api runtime: txs success name={} count={}", name, txs.len());

    Ok(txs)
}

/// Return wallet UTXOs using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn utxos(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<Vec<WalletUtxoDto>> {
    debug!("api runtime: utxos start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;

    let utxos: Vec<WalletUtxoDto> = wallet
        .utxos()
        .into_iter()
        .map(Into::into)
        .collect();

    info!("api runtime: utxos success name={} count={}", name, utxos.len());

    Ok(utxos)
}

/// Return high-level wallet status using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn status(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<WalletStatusDto> {
    debug!("api runtime: status start name={}", name);

    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;

    let balance = wallet.balance_sat()?;
    let utxos = wallet.utxos();
    let utxo_count = utxos.len();
    let last_block_height = utxos
        .iter()
        .filter_map(|u| u.confirmation_height)
        .max();

    info!(
        "api runtime: status success name={} balance={} utxos={} last_block_height={:?}",
        name,
        balance,
        utxo_count,
        last_block_height
    );

    Ok(WalletStatusDto {
        balance,
        utxo_count,
        last_block_height,
    })
}