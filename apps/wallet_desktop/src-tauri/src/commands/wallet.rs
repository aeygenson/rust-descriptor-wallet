use tauri::{command, State};
use wallet_api::{
    model::{
        WalletBackendHealthDto,
        WalletReceiveAddressHistoryDto,
        WalletStatusDto,
        WalletSummaryDto,
    },
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

#[allow(non_snake_case)]
#[command]
pub async fn get_wallet_status(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletStatusDto, String> {
    api.status(&walletName)
        .await
        .map_err(|err| err.to_string())
}
#[allow(non_snake_case)]
#[command]
pub async fn sync_wallet(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletStatusDto, String> {
    api.sync(&walletName)
        .await
        .map_err(|err| err.to_string())?;
    api.status(&walletName)
        .await
        .map_err(|err| err.to_string())
}

#[allow(non_snake_case)]
#[command]
pub async fn backend_health(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletBackendHealthDto, String> {
    api.backend_health(&walletName)
        .await
        .map_err(|err| err.to_string())
}

#[allow(non_snake_case)]
#[command]
pub async fn get_receive_address(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletReceiveAddressHistoryDto, String> {
    api.address(&walletName)
        .await
        .map_err(|err| err.to_string())
}

#[allow(non_snake_case)]
#[command]
pub async fn list_receive_addresses(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<Vec<WalletReceiveAddressHistoryDto>, String> {
    api.list_receive_addresses(&walletName)
        .await
        .map_err(|err| err.to_string())
}

#[allow(non_snake_case)]
#[command]
pub async fn label_receive_address(
    api: State<'_, WalletApi>,
    walletName: String,
    address: String,
    label: String,
) -> Result<WalletReceiveAddressHistoryDto, String> {
    api.label_receive_address(&walletName, &address, &label)
        .await
        .map_err(|err| err.to_string())
}

#[allow(non_snake_case)]
#[command]
pub async fn clear_receive_address_label(
    api: State<'_, WalletApi>,
    walletName: String,
    address: String,
) -> Result<WalletReceiveAddressHistoryDto, String> {
    api.clear_receive_address_label(&walletName, &address)
        .await
        .map_err(|err| err.to_string())
}
