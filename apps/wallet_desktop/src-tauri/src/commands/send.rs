use super::send_model::{
    BumpFeePsbtRequest, BumpFeeRequest, ConsolidatePsbtRequest, CpfpPsbtRequest, CpfpRequest,
    CreateConsolidationPsbtRequest, CreatePsbtRequest, CreatePsbtWithCoinControlRequest,
    CreateSendMaxPsbtRequest, CreateSendMaxPsbtWithCoinControlRequest, CreateSweepPsbtRequest,
    PublishPsbtRequest, SendMaxPsbtRequest, SendMaxPsbtWithCoinControlRequest, SendPsbtRequest,
    SendPsbtWithCoinControlRequest, SignPsbtRequest, SweepPsbtRequest,
};
use tauri::State;
use wallet_api::model::{
    BumpFeeRequestDto, ConsolidationRequestDto, CpfpRequestDto, CreatePsbtRequestDto,
    PublishPsbtRequestDto, SendMaxRequestDto, SignPsbtRequestDto, SweepRequestDto,
    TxBroadcastResultDto, WalletCpfpPsbtDto, WalletPsbtDto, WalletSignedPsbtDto,
};
use wallet_api::{service, WalletApi};

type CommandResult<T> = Result<T, String>;

#[tauri::command]
pub async fn create_psbt(
    api: State<'_, WalletApi>,
    request: CreatePsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: CreatePsbtRequestDto = request.into();

        tracing::info!(
            wallet_name = %dto.name,
            address = %dto.to_address,
            amount_sat = dto.amount_sat,
            fee_rate_sat_vb = dto.fee_rate_sat_per_vb,
            replaceable = dto.replaceable,
            confirmed_only = dto.coin_control.as_ref().is_some_and(|cc| cc.confirmed_only),
            "tauri command: create_psbt decoded request"
        );

        service::psbt::create(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn create_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: CreatePsbtWithCoinControlRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: CreatePsbtRequestDto = request.into();

        tracing::info!(
            wallet_name = %dto.name,
            address = %dto.to_address,
            amount_sat = dto.amount_sat,
            fee_rate_sat_vb = dto.fee_rate_sat_per_vb,
            replaceable = dto.replaceable,
            confirmed_only = dto.coin_control.as_ref().is_some_and(|cc| cc.confirmed_only),
            "tauri command: create_psbt_with_coin_control decoded request"
        );

        service::psbt::create(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn create_send_max_psbt(
    api: State<'_, WalletApi>,
    request: CreateSendMaxPsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: SendMaxRequestDto = request.into();

        service::psbt::create_send_max(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn create_send_max_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: CreateSendMaxPsbtWithCoinControlRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: SendMaxRequestDto = request.into();

        service::psbt::create_send_max(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn create_sweep_psbt(
    api: State<'_, WalletApi>,
    request: CreateSweepPsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: SweepRequestDto = request.into();

        service::psbt::create_sweep(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn create_consolidation_psbt(
    api: State<'_, WalletApi>,
    request: CreateConsolidationPsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: ConsolidationRequestDto = request.into();

        service::psbt::create_consolidation(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn sign_psbt(
    api: State<'_, WalletApi>,
    request: SignPsbtRequest,
) -> CommandResult<WalletSignedPsbtDto> {
    {
        let dto: SignPsbtRequestDto = request.into();

        service::psbt::sign(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn publish_psbt(
    api: State<'_, WalletApi>,
    request: PublishPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let dto: PublishPsbtRequestDto = request.into();

        service::psbt::publish(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

async fn sign_and_publish(
    api: &WalletApi,
    wallet_name: String,
    psbt_base64: String,
) -> CommandResult<TxBroadcastResultDto> {
    let signed = service::psbt::sign(
        &api.storage,
        SignPsbtRequestDto {
            name: wallet_name.clone(),
            psbt_base64,
        },
    )
    .await
    .map_err(api_error_to_string)?;

    service::psbt::publish(
        &api.storage,
        PublishPsbtRequestDto {
            name: wallet_name,
            psbt_base64: signed.psbt_base64,
        },
    )
    .await
    .map_err(api_error_to_string)
}

#[tauri::command]
pub async fn send_psbt(
    api: State<'_, WalletApi>,
    request: SendPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let create_dto: CreatePsbtRequestDto = request.into();
        let wallet_name = create_dto.name.clone();

        let created = service::psbt::create(&api.storage, create_dto)
            .await
            .map_err(api_error_to_string)?;

        sign_and_publish(&api, wallet_name, created.psbt_base64).await
    }
}

#[tauri::command]
pub async fn send_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: SendPsbtWithCoinControlRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let create_dto: CreatePsbtRequestDto = request.into();
        let wallet_name = create_dto.name.clone();

        let created = service::psbt::create(&api.storage, create_dto)
            .await
            .map_err(api_error_to_string)?;

        sign_and_publish(&api, wallet_name, created.psbt_base64).await
    }
}

#[tauri::command]
pub async fn send_max_psbt(
    api: State<'_, WalletApi>,
    request: SendMaxPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let create_dto: SendMaxRequestDto = request.into();
        let wallet_name = create_dto.name.clone();

        let created = service::psbt::create_send_max(&api.storage, create_dto)
            .await
            .map_err(api_error_to_string)?;

        sign_and_publish(&api, wallet_name, created.psbt_base64).await
    }
}

#[tauri::command]
pub async fn send_max_psbt_with_coin_control(
    api: State<'_, WalletApi>,
    request: SendMaxPsbtWithCoinControlRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let create_dto: SendMaxRequestDto = request.into();
        let wallet_name = create_dto.name.clone();

        let created = service::psbt::create_send_max(&api.storage, create_dto)
            .await
            .map_err(api_error_to_string)?;

        sign_and_publish(&api, wallet_name, created.psbt_base64).await
    }
}

#[tauri::command]
pub async fn send_sweep_psbt(
    api: State<'_, WalletApi>,
    request: SweepPsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let dto: SweepRequestDto = request.into();

        service::psbt::sweep(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn consolidate_psbt(
    api: State<'_, WalletApi>,
    request: ConsolidatePsbtRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let dto: ConsolidationRequestDto = request.into();

        service::psbt::consolidate(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn bump_fee_psbt(
    api: State<'_, WalletApi>,
    request: BumpFeePsbtRequest,
) -> CommandResult<WalletPsbtDto> {
    {
        let dto: BumpFeeRequestDto = request.into();

        tracing::info!(
            wallet_name = %dto.name,
            txid = %dto.txid,
            fee_rate_sat_vb = dto.fee_rate_sat_per_vb,
            "tauri command: bump_fee_psbt decoded request"
        );

        service::psbt::bump_fee_psbt(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn bump_fee(
    api: State<'_, WalletApi>,
    request: BumpFeeRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let dto: BumpFeeRequestDto = request.into();

        tracing::info!(
            wallet_name = %dto.name,
            txid = %dto.txid,
            fee_rate_sat_vb = dto.fee_rate_sat_per_vb,
            "tauri command: bump_fee decoded request"
        );

        service::psbt::bump_fee(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn cpfp_psbt(
    api: State<'_, WalletApi>,
    request: CpfpPsbtRequest,
) -> CommandResult<WalletCpfpPsbtDto> {
    {
        let dto: CpfpRequestDto = request.into();

        tracing::info!(
            wallet_name = %dto.name,
            parent_txid = %dto.parent_txid,
            selected_outpoint = %dto.selected_outpoint,
            fee_rate_sat_vb = dto.fee_rate_sat_per_vb,
            "tauri command: cpfp_psbt decoded request"
        );

        service::psbt::cpfp_psbt(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

#[tauri::command]
pub async fn cpfp(
    api: State<'_, WalletApi>,
    request: CpfpRequest,
) -> CommandResult<TxBroadcastResultDto> {
    {
        let dto: CpfpRequestDto = request.into();

        tracing::info!(
            wallet_name = %dto.name,
            parent_txid = %dto.parent_txid,
            selected_outpoint = %dto.selected_outpoint,
            fee_rate_sat_vb = dto.fee_rate_sat_per_vb,
            "tauri command: cpfp decoded request"
        );

        service::psbt::cpfp(&api.storage, dto)
            .await
            .map_err(api_error_to_string)
    }
}

fn api_error_to_string(err: impl std::fmt::Display) -> String {
    err.to_string()
}
