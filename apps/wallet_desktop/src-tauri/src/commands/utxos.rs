use tauri::{command, State};
use wallet_api::{
    model::{WalletUtxoDto, WalletUtxosRequestDto},
    service, WalletApi,
};

#[command]
pub async fn list_utxos(
    api: State<'_, WalletApi>,
    wallet_name: String,
) -> Result<Vec<WalletUtxoDto>, String> {
    service::inspect::utxos(
        &api.storage,
        WalletUtxosRequestDto {
            name: wallet_name,
        },
    )
    .await
    .map_err(|err| err.to_string())
}
