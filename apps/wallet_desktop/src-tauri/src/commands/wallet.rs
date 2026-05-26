use tauri::{command, State};
use wallet_api::{
    model::{
        AddressBookEntryDto,
        WalletBackendHealthDto,
        WalletLockedUtxoDto,
        WalletReceiveAddressHistoryDto,
        WalletStatusDto,
        WalletSummaryDto,
    },
    WalletApi,
};

fn tauri_error<E>(err: E) -> String
where
    E: std::fmt::Display,
{
    err.to_string()
}

/// Returns a simple string to verify that the Rust backend is connected.
#[command]
pub fn get_app_info() -> String {
    "Rust Descriptor Wallet backend connected".to_string()
}

#[command]
pub async fn list_wallets(api: State<'_, WalletApi>) -> Result<Vec<WalletSummaryDto>, String> {
    api.list_wallets().await.map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn get_wallet_status(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletStatusDto, String> {
    api.status(&walletName)
        .await
        .map_err(tauri_error)
}
#[allow(non_snake_case)]
#[command]
pub async fn sync_wallet(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletStatusDto, String> {
    api.sync(&walletName)
        .await
        .map_err(tauri_error)?;
    api.status(&walletName)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn backend_health(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletBackendHealthDto, String> {
    api.backend_health(&walletName)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn get_receive_address(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<WalletReceiveAddressHistoryDto, String> {
    api.address(&walletName)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn list_receive_addresses(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<Vec<WalletReceiveAddressHistoryDto>, String> {
    api.list_receive_addresses(&walletName)
        .await
        .map_err(tauri_error)
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
        .map_err(tauri_error)
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
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn create_address_book_entry(
    api: State<'_, WalletApi>,
    walletName: String,
    label: String,
    address: String,
    notes: Option<String>,
) -> Result<AddressBookEntryDto, String> {
    api.create_address_book_entry(&walletName, &label, &address, notes)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn list_address_book_entries(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<Vec<AddressBookEntryDto>, String> {
    api.list_address_book_entries(&walletName)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn get_address_book_entry(
    api: State<'_, WalletApi>,
    walletName: String,
    address: String,
) -> Result<Option<AddressBookEntryDto>, String> {
    api.get_address_book_entry(&walletName, &address)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn delete_address_book_entry(
    api: State<'_, WalletApi>,
    walletName: String,
    address: String,
) -> Result<bool, String> {
    api.delete_address_book_entry(&walletName, &address)
        .await
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn lock_utxo(
    api: State<'_, WalletApi>,
    walletName: String,
    outpoint: String,
    reason: Option<String>,
) -> Result<Vec<WalletLockedUtxoDto>, String> {
    api.lock_utxo(&walletName, &outpoint, reason)
        .await
        .map(|result| result.locked_utxos)
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn unlock_utxo(
    api: State<'_, WalletApi>,
    walletName: String,
    outpoint: String,
) -> Result<Vec<WalletLockedUtxoDto>, String> {
    api.unlock_utxo(&walletName, &outpoint)
        .await
        .map(|result| result.locked_utxos)
        .map_err(tauri_error)
}

#[allow(non_snake_case)]
#[command]
pub async fn list_locked_utxos(
    api: State<'_, WalletApi>,
    walletName: String,
) -> Result<Vec<WalletLockedUtxoDto>, String> {
    api.locked_utxos(&walletName)
        .await
        .map_err(tauri_error)
}
