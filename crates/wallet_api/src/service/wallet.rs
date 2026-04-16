use bitcoin::Network;
use tracing::{debug, info};

use crate::model::WalletStatusDto;
use crate::{WalletApiError, WalletApiResult};

use wallet_core::{
    config::{BroadcastBackendConfig, SyncBackendConfig, WalletBackendConfig, WalletDescriptors},
    WalletConfig, WalletService,
};
use wallet_storage::WalletStorage;
use wallet_sync::WalletSyncService;

/// Load wallet configuration from storage and convert it into the core config.
pub(crate) async fn load_wallet_config(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<WalletConfig> {
    let record = storage.get_wallet_by_name(name).await?;
    let network = parse_network(&record.network)?;

    let sync_backend = record.parse_sync_backend().map_err(|e| {
        WalletApiError::InvalidInput(format!("invalid sync backend for wallet '{}': {}", name, e))
    })?;

    let broadcast_backend = record.parse_broadcast_backend().map_err(|e| {
        WalletApiError::InvalidInput(format!(
            "invalid broadcast backend for wallet '{}': {}",
            name, e
        ))
    })?;

    let sync_backend = match sync_backend {
        wallet_storage::models::SyncBackendFile::Esplora { url } => {
            SyncBackendConfig::Esplora { url }
        }
        wallet_storage::models::SyncBackendFile::Electrum { url } => {
            SyncBackendConfig::Electrum { url }
        }
    };

    let broadcast_backend = match broadcast_backend {
        Some(wallet_storage::models::BroadcastBackendFile::Esplora { url }) => {
            Some(BroadcastBackendConfig::Esplora { url })
        }
        Some(wallet_storage::models::BroadcastBackendFile::Rpc {
            url,
            rpc_user,
            rpc_pass,
        }) => Some(BroadcastBackendConfig::Rpc {
            url,
            rpc_user,
            rpc_pass,
        }),
        None => None,
    };

    Ok(WalletConfig {
        network,
        descriptors: WalletDescriptors {
            external: record.external_descriptor,
            internal: record.internal_descriptor,
        },
        backend: WalletBackendConfig {
            sync: sync_backend,
            broadcast: broadcast_backend,
        },
        db_path: record.db_path.into(),
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

    let sync_service = WalletSyncService::new();
    sync_service.sync(&mut wallet, &config).await?;

    info!("api wallet: sync success name={}", name);
    Ok(())
}

pub async fn balance(storage: &WalletStorage, name: &str) -> WalletApiResult<u64> {
    debug!("api wallet: balance start name={}", name);
    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;
    let balance = wallet.balance_sat()?;
    info!(
        "api wallet: balance success name={} balance={}",
        name, balance
    );
    Ok(balance)
}

/// Return high-level wallet status using the current synced wallet state.
///
/// This performs no network calls. Call `sync(...)` first if fresh chain data is needed.
pub async fn status(storage: &WalletStorage, name: &str) -> WalletApiResult<WalletStatusDto> {
    debug!("api wallet: status start name={}", name);

    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;

    let balance = wallet.balance_sat()?;
    let utxos = wallet.utxos();
    let utxo_count = utxos.len();
    let last_block_height = utxos.iter().filter_map(|u| u.confirmation_height).max();

    info!(
        "api wallet: status success name={} balance={} utxos={} last_block_height={:?}",
        name, balance, utxo_count, last_block_height
    );

    Ok(WalletStatusDto {
        balance,
        utxo_count,
        last_block_height,
    })
}
