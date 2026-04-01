use std::sync::Arc;

use bitcoin::Network;

use crate::{WalletApiError, WalletApiResult};
use wallet_core::{WalletConfig, WalletCore, WalletService};
use wallet_storage::WalletStorage;
use wallet_sync::WalletSync;

async fn load_wallet_config(storage: &WalletStorage, name: &str) -> WalletApiResult<WalletConfig> {
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

fn parse_network(s: &str) -> WalletApiResult<Network> {
    match s {
        "testnet" => Ok(Network::Testnet),
        "signet" => Ok(Network::Signet),
        "mainnet" => Ok(Network::Bitcoin),
        _ => Err(WalletApiError::InvalidInput(format!("unknown network: {s}"))),
    }
}

pub async fn address(storage: &WalletStorage, name: &str) -> WalletApiResult<String> {
    let config = load_wallet_config(storage, name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;
    let address = wallet.next_receive_address()?;
    Ok(address)
}

pub async fn sync(storage: &WalletStorage, name: &str) -> WalletApiResult<()> {
    let config = load_wallet_config(storage, name).await?;
    let mut wallet = WalletService::load_or_create(&config)?;

    let core = Arc::new(WalletCore::new());
    let sync_service = WalletSync::new(core);
    sync_service.sync(&mut wallet,&config).await?;

    Ok(())
}

pub async fn balance(storage: &WalletStorage, name: &str) -> WalletApiResult<u64> {
    let config = load_wallet_config(storage, name).await?;
    let wallet = WalletService::load_or_create(&config)?;
    let balance = wallet.balance_sat()?;
    Ok(balance)
}