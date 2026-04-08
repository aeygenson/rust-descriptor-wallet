

use std::sync::Arc;

use bitcoin::Network;
use tracing::{debug, info};

use crate::dto::WalletStatusDto;
use crate::{WalletApiError, WalletApiResult};

use wallet_core::{WalletConfig, WalletCore, WalletService};
use wallet_storage::WalletStorage;
use wallet_sync::WalletSync;

/// Load wallet configuration from storage and convert it into the core config.
pub(crate) async fn load_wallet_config(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<WalletConfig> {
    let record = storage.get_wallet_by_name(name).await?;
    let network = parse_network(&record.network)?;

    Ok(WalletConfig {
        network,
        external_descriptor: record.external_descriptor,
        internal_descriptor: record.internal_descriptor,
        db_path: record.db_path.into(),
        esplora_url: record.esplora_url,
        is_watch_only: record.is_watch_only,
    })
}

/// Parse stored network string into the Bitcoin network enum.
#[allow(clippy::result_large_err)]
fn parse_network(s: &str) -> WalletApiResult<Network> {
    match s {
        "bitcoin" | "mainnet" => Ok(Network::Bitcoin),
        "testnet" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "regtest" => Ok(Network::Regtest),
        other => Err(WalletApiError::InvalidInput(format!(
            "unsupported network: {}",
            other
        ))),
    }
}

pub async fn address(storage: &WalletStorage, name: &str) -> WalletApiResult<String> {
    debug!("api wallet: address start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;
    let address = wallet.next_receive_address()?;
    info!("api wallet: address success name={}", name);
    Ok(address)
}

pub async fn sync(storage: &WalletStorage, name: &str) -> WalletApiResult<()> {
    debug!("api wallet: sync start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;

    let core = Arc::new(WalletCore::new());
    let sync_service = WalletSync::new(core);
    sync_service.sync(&mut wallet, &config).await?;

    info!("api wallet: sync success name={}", name);
    Ok(())
}

pub async fn balance(storage: &WalletStorage, name: &str) -> WalletApiResult<u64> {
    debug!("api wallet: balance start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;
    let balance = wallet.balance_sat()?;
    info!("api wallet: balance success name={} balance={}", name, balance);
    Ok(balance)
}

/// Return high-level wallet status using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn status(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<WalletStatusDto> {
    debug!("api wallet: status start name={}", name);

    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;

    let balance = wallet.balance_sat()?;
    let utxos = wallet.utxos();
    let utxo_count = utxos.len();
    let last_block_height = utxos.iter().filter_map(|u| u.confirmation_height).max();

    info!(
        "api wallet: status success name={} balance={} utxos={} last_block_height={:?}",
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