use anyhow::Result;
use wallet_api::WalletApi;
use tracing::{debug, info};

pub async fn address(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: address start name={}", name);
    let addr = api.address(name).await?;
    info!("cli runtime: address generated for wallet {}", name);
    println!("{addr}");
    Ok(())
}

pub async fn sync(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: sync start name={}", name);
    api.sync_wallet(name).await?;
    info!("cli runtime: sync success for wallet {}", name);
    println!("Synced wallet {name}");
    Ok(())
}

pub async fn balance(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: balance start name={}", name);
    let bal = api.balance(name).await?;
    info!("cli runtime: balance fetched for wallet {}", name);
    println!("balance={} sats", bal);
    Ok(())
}