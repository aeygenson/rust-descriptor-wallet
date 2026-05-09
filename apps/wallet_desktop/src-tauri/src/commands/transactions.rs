use tauri::{command, State};
use wallet_api::{
    model::{WalletTransactionsRequestDto, WalletTxDto},
    service, WalletApi,
};

#[command]
pub async fn list_transactions(
    api: State<'_, WalletApi>,
    wallet_name: String,
) -> Result<Vec<WalletTxDto>, String> {
    let txs = service::inspect::txs(
        &api.storage,
        WalletTransactionsRequestDto {
            name: wallet_name.clone(),
        },
    )
    .await
    .map_err(|err| err.to_string())?;

    // Debug: ensure inputs/outputs are present for parent/child graph
    log::debug!(
        "tauri list_transactions: wallet={} txs={} sample_inputs={} sample_outputs={}",
        wallet_name,
        txs.len(),
        txs.get(0).map(|t| t.inputs.len()).unwrap_or(0),
        txs.get(0).map(|t| t.outputs.len()).unwrap_or(0)
    );

    Ok(txs)
}
