use anyhow::Result;
use wallet_api::WalletApi;
use wallet_api::dto::{WalletDetailsDto, WalletSummaryDto};

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
    println!("esplora_url={}", wallet.esplora_url);
    println!("external_descriptor={}", wallet.external_descriptor);
    println!("internal_descriptor={}", wallet.internal_descriptor);

    Ok(())
}

pub async fn import_wallet(api: &WalletApi, file: &str) -> Result<()> {
    api.import_wallet(file).await?;
    println!("Imported wallet from {file}");
    Ok(())
}

pub async fn delete_wallet(api: &WalletApi, name: &str) -> Result<()> {
    api.delete_wallet(name).await?;
    println!("Deleted wallet {name}");
    Ok(())
}
