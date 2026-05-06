use tauri::{command, State};
use wallet_api::{
    model::{WalletBackendHealthDto, WalletStatusDto, WalletSummaryDto},
    WalletApi,
};
/// Returns a simple string to verify that the Rust backend is connected.
#[command]
pub fn get_app_info() -> String {
    "Rust Descriptor Wallet backend connected".to_string()
}

#[command]
pub async fn list_wallets(api: State<'_, WalletApi>) -> Result<Vec<WalletSummaryDto>, String> {
    api.list_wallets().await.map_err(|err| err.to_string())
}

#[command]
pub async fn get_wallet_status(
    api: State<'_, WalletApi>,
    wallet_name: String,
) -> Result<WalletStatusDto, String> {
    api.status(&wallet_name)
        .await
        .map_err(|err| err.to_string())
}
#[command]
pub async fn sync_wallet(
    api: State<'_, WalletApi>,
    wallet_name: String,
) -> Result<WalletStatusDto, String> {
    api.sync(&wallet_name)
        .await
        .map_err(|err| err.to_string())?;
    api.status(&wallet_name)
        .await
        .map_err(|err| err.to_string())
}

#[command]
pub async fn backend_health(
    api: State<'_, WalletApi>,
    wallet_name: String,
) -> Result<WalletBackendHealthDto, String> {
    api.backend_health(&wallet_name)
        .await
        .map_err(|err| err.to_string())
}
