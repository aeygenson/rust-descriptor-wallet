use super::send_model::{
    BumpFeePsbtRequest, BumpFeeRequest, ConsolidatePsbtRequest, CpfpPsbtRequest, CpfpRequest,
    CreateConsolidationPsbtRequest, CreatePsbtRequest, CreatePsbtWithCoinControlRequest,
    CreateSendMaxPsbtRequest, CreateSendMaxPsbtWithCoinControlRequest, CreateSweepPsbtRequest,
    PublishPsbtRequest, SendMaxPsbtRequest, SendMaxPsbtWithCoinControlRequest, SendPsbtRequest, SendPsbtWithCoinControlRequest,
    SignPsbtRequest, SweepPsbtRequest,
};
use tauri::State;
use wallet_api::model::{
    TxBroadcastResultDto, WalletCpfpPsbtDto, WalletPsbtDto, WalletSignedPsbtDto,
};
use wallet_api::WalletApi;

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn create_psbt(
    api: State<'_, WalletApi>,
    request: CreatePsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    let (wallet_name, address, amount_sat, fee_rate_sat_vb, replaceable, confirmed_only) =
        request.into_parts();

    tracing::info!(
        wallet_name = %wallet_name,
        address = %address,
        amount_sat = amount_sat,
        fee_rate_sat_vb = fee_rate_sat_vb,
        replaceable = replaceable,
        confirmed_only = confirmed_only,
        "tauri command: create_psbt decoded request"
    );

    api.create_psbt(
        &wallet_name,
        &address,
        amount_sat,
        fee_rate_sat_vb,
        replaceable,
        confirmed_only,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn create_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: CreatePsbtWithCoinControlRequest,
) -> CommandResult<WalletPsbtDto> {
    let (
        wallet_name,
        address,
        amount_sat,
        fee_rate_sat_vb,
        replaceable,
        confirmed_only,
        mut coin_control,
    ) = request.into_parts();

    tracing::info!(
        wallet_name = %wallet_name,
        address = %address,
        amount_sat = amount_sat,
        fee_rate_sat_vb = fee_rate_sat_vb,
        replaceable = replaceable,
        confirmed_only = confirmed_only,
        "tauri command: create_psbt_with_coin_control decoded request"
    );

    coin_control.confirmed_only = confirmed_only;

    api.create_psbt_with_coin_control(
        &wallet_name,
        &address,
        amount_sat,
        fee_rate_sat_vb,
        replaceable,
        coin_control,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn create_send_max_psbt(
    api: State<'_, WalletApi>,
    request: CreateSendMaxPsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    let (wallet_name, address, fee_rate_sat_vb, replaceable) = request.into_parts();

    api.create_send_max_psbt(&wallet_name, &address, fee_rate_sat_vb, replaceable)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn create_send_max_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: CreateSendMaxPsbtWithCoinControlRequest,
) -> CommandResult<WalletPsbtDto> {
    let (wallet_name, address, fee_rate_sat_vb, replaceable, coin_control) = request.into_parts();

    api.create_send_max_psbt_with_coin_control(
        &wallet_name,
        &address,
        fee_rate_sat_vb,
        replaceable,
        coin_control,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn create_sweep_psbt(
    api: State<'_, WalletApi>,
    request: CreateSweepPsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    let (wallet_name, address, fee_rate_sat_vb, replaceable, coin_control) = request.into_parts();

    api.create_sweep_psbt(
        &wallet_name,
        &address,
        fee_rate_sat_vb,
        replaceable,
        coin_control,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn create_consolidation_psbt(
    api: State<'_, WalletApi>,
    request: CreateConsolidationPsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    let (wallet_name, fee_rate_sat_vb, replaceable, consolidation) = request.into_parts();

    api.create_consolidation_psbt(&wallet_name, fee_rate_sat_vb, replaceable, consolidation)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn sign_psbt(
    api: State<'_, WalletApi>,
    request: SignPsbtRequest,
) -> CommandResult<WalletSignedPsbtDto> {
    let (wallet_name, psbt_base64) = request.into_parts();

    api.sign_psbt(&wallet_name, &psbt_base64)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn publish_psbt(
    api: State<'_, WalletApi>,
    request: PublishPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, psbt_base64) = request.into_parts();

    api.publish_psbt(&wallet_name, &psbt_base64)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn send_psbt(
    api: State<'_, WalletApi>,
    request: SendPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, address, amount_sat, fee_rate_sat_vb, replaceable, confirmed_only) =
        request.into_parts();

    api.send_psbt(
        &wallet_name,
        &address,
        amount_sat,
        fee_rate_sat_vb,
        replaceable,
        confirmed_only,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn send_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: SendPsbtWithCoinControlRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (
        wallet_name,
        address,
        amount_sat,
        fee_rate_sat_vb,
        replaceable,
        confirmed_only,
        mut coin_control,
    ) = request.into_parts();

    coin_control.confirmed_only = confirmed_only;

    api.send_psbt_with_coin_control(
        &wallet_name,
        &address,
        amount_sat,
        fee_rate_sat_vb,
        replaceable,
        coin_control,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn send_max_psbt(
    api: State<'_, WalletApi>,
    request: SendMaxPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, address, fee_rate_sat_vb, replaceable) = request.into_parts();

    api.send_max_psbt(&wallet_name, &address, fee_rate_sat_vb, replaceable)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn send_max_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: SendMaxPsbtWithCoinControlRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, address, fee_rate_sat_vb, replaceable, coin_control) = request.into_parts();

    api.send_max_psbt_with_coin_control(
        &wallet_name,
        &address,
        fee_rate_sat_vb,
        replaceable,
        coin_control,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn send_sweep_psbt(
    api: State<'_, WalletApi>,
    request: SweepPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, address, fee_rate_sat_vb, replaceable, coin_control) = request.into_parts();

    api.send_sweep_psbt(
        &wallet_name,
        &address,
        fee_rate_sat_vb,
        replaceable,
        coin_control,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn consolidate_psbt(
    api: State<'_, WalletApi>,
    request: ConsolidatePsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, fee_rate_sat_vb, replaceable, consolidation) = request.into_parts();

    api.send_consolidation_psbt(&wallet_name, fee_rate_sat_vb, replaceable, consolidation)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn bump_fee_psbt(
    api: State<'_, WalletApi>,
    request: BumpFeePsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    let (wallet_name, txid, fee_rate_sat_vb) = request.into_parts();

    tracing::info!(
        wallet_name = %wallet_name,
        txid = %txid,
        fee_rate_sat_vb = fee_rate_sat_vb,
        "tauri command: bump_fee_psbt decoded request"
    );

    api.bump_fee_psbt(&wallet_name, &txid, fee_rate_sat_vb)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn bump_fee(
    api: State<'_, WalletApi>,
    request: BumpFeeRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, txid, fee_rate_sat_vb) = request.into_parts();

    tracing::info!(
        wallet_name = %wallet_name,
        txid = %txid,
        fee_rate_sat_vb = fee_rate_sat_vb,
        "tauri command: bump_fee decoded request"
    );

    api.bump_fee(&wallet_name, &txid, fee_rate_sat_vb)
        .await
        .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn cpfp_psbt(
    api: State<'_, WalletApi>,
    request: CpfpPsbtRequest,
) -> CommandResult<WalletCpfpPsbtDto> {
    let (wallet_name, parent_txid, selected_outpoint, fee_rate_sat_vb) = request.into_parts();

    tracing::info!(
        wallet_name = %wallet_name,
        parent_txid = %parent_txid,
        selected_outpoint = %selected_outpoint,
        fee_rate_sat_vb = fee_rate_sat_vb,
        "tauri command: cpfp_psbt decoded request"
    );

    api.cpfp_psbt(
        &wallet_name,
        &parent_txid,
        &selected_outpoint,
        fee_rate_sat_vb,
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn cpfp(
    api: State<'_, WalletApi>,
    request: CpfpRequest,
) -> CommandResult<TxBroadcastResultDto> {
    let (wallet_name, parent_txid, selected_outpoint, fee_rate_sat_vb) = request.into_parts();

    tracing::info!(
        wallet_name = %wallet_name,
        parent_txid = %parent_txid,
        selected_outpoint = %selected_outpoint,
        fee_rate_sat_vb = fee_rate_sat_vb,
        "tauri command: cpfp decoded request"
    );

    api.cpfp(
        &wallet_name,
        &parent_txid,
        &selected_outpoint,
        fee_rate_sat_vb,
    )
    .await
    .map_err(api_error_to_string)
}

fn api_error_to_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}
