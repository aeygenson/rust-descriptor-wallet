use crate::{ WalletApiResult};
use wallet_storage::WalletStorage;

use crate::dto::{WalletDetailsDto, WalletSummaryDto};

/// List all wallets
pub async fn list_wallets(storage: &WalletStorage) -> WalletApiResult<Vec<WalletSummaryDto>> {
    let wallets = storage.list_wallets().await?;

    Ok(wallets
        .into_iter()
        .map(|w| WalletSummaryDto {
            name: w.name,
            network: w.network,
            is_watch_only: w.is_watch_only,
        })
        .collect())
}

/// Import wallet from JSON file
pub async fn import_wallet(
    storage: &WalletStorage,
    file_path: &str,
) -> WalletApiResult<()> {
    storage.import_wallet_from_file(file_path).await?;
    Ok(())
}

/// Delete wallet by name
pub async fn delete_wallet(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<()> {
    storage.delete_wallet(name).await?;
    Ok(())
}

/// Get wallet details
pub async fn get_wallet(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<WalletDetailsDto> {
    let wallet = storage.get_wallet_by_name(name).await?;

    Ok(WalletDetailsDto {
        name: wallet.name,
        network: wallet.network,
        external_descriptor: wallet.external_descriptor,
        internal_descriptor: wallet.internal_descriptor,
        esplora_url: wallet.esplora_url,
        is_watch_only: wallet.is_watch_only,
    })
}
