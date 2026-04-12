use std::path::Path;
use anyhow::Result;
use wallet_api::WalletApi;
use wallet_api::model::{WalletDetailsDto, WalletSummaryDto};

pub async fn list_wallets(api: &WalletApi) -> Result<()> {
    let wallets: Vec<WalletSummaryDto> = api.list_wallets().await?;

    if wallets.is_empty() {
        println!("No wallets found.");
    } else {
        for w in wallets {
            println!(
                "name={} network={} watch_only={}",
                w.name, w.network, w.is_watch_only
            );
        }
    }

    Ok(())
}

pub async fn get_wallet(api: &WalletApi, name: &str) -> Result<()> {
    let wallet: WalletDetailsDto = api.get_wallet(name).await?;

    println!("name={}", wallet.name);
    println!("network={}", wallet.network);
    println!("watch_only={}", wallet.is_watch_only);
    // descriptors
    println!("external_descriptor={}", wallet.descriptors.external);
    println!("internal_descriptor={}", wallet.descriptors.internal);

    // backend
    match &wallet.backend.sync {
        wallet_api::model::SyncBackendDto::Esplora { url } => {
            println!("sync_backend=esplora url={}", url);
        }
        wallet_api::model::SyncBackendDto::Electrum { url } => {
            println!("sync_backend=electrum url={}", url);
        }
    }

    match &wallet.backend.broadcast {
        Some(wallet_api::model::BroadcastBackendDto::Esplora { url }) => {
            println!("broadcast_backend=esplora url={}", url);
        }
        Some(wallet_api::model::BroadcastBackendDto::Rpc { url, .. }) => {
            println!("broadcast_backend=core_rpc url={}", url);
        }
        None => {
            println!("broadcast_backend=none");
        }
    }

    Ok(())
}

pub async fn import_wallet(api: &WalletApi, file: &Path) -> Result<()> {
    api.import_wallet(file.to_string_lossy().as_ref()).await?;
    println!("Imported wallet from {}", file.display());
    Ok(())
}

pub async fn delete_wallet(api: &WalletApi, name: &str) -> Result<()> {
    api.delete_wallet(name).await?;
    println!("Deleted wallet {name}");
    Ok(())
}
