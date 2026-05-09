

use anyhow::Result;

use wallet_api::model::{
    ConsolidationRequestDto, CreatePsbtRequestDto, PublishPsbtRequestDto, SendMaxRequestDto,
    SignPsbtRequestDto, SweepRequestDto, TxBroadcastResultDto, WalletCoinControlDto,
    WalletConsolidationDto, WalletInputSelectionModeDto, WalletSignedPsbtDto,
};
use wallet_api::{service, WalletApi};

fn print_optional_rbf(replaceable: Option<bool>) {
    if let Some(replaceable) = replaceable {
        println!("replaceable={}", replaceable);
    }
}

fn print_broadcast_success(title: &str, published: &TxBroadcastResultDto) {
    println!("{}", title);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

}

fn build_coin_control_dto(
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> WalletCoinControlDto {
    WalletCoinControlDto {
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
    }
}

fn build_consolidation_dto(
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    max_input_count: Option<usize>,
    min_input_count: Option<usize>,
    min_utxo_value_sat: Option<u64>,
    max_utxo_value_sat: Option<u64>,
    max_fee_pct_of_input_value: Option<u8>,
    strategy: Option<wallet_api::model::WalletConsolidationStrategyDto>,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> WalletConsolidationDto {
    WalletConsolidationDto {
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        max_input_count,
        min_input_count,
        min_utxo_value_sat,
        max_utxo_value_sat,
        max_fee_pct_of_input_value,
        strategy,
        selection_mode,
    }
}

async fn sign_request(
    api: &WalletApi,
    name: &str,
    psbt_base64: String,
) -> wallet_api::WalletApiResult<WalletSignedPsbtDto> {
    service::psbt::sign(
        &api.storage,
        SignPsbtRequestDto {
            name: name.to_string(),
            psbt_base64,
        },
    )
    .await
}

async fn publish_request(
    api: &WalletApi,
    name: &str,
    psbt_base64: String,
) -> wallet_api::WalletApiResult<TxBroadcastResultDto> {
    service::psbt::publish(
        &api.storage,
        PublishPsbtRequestDto {
            name: name.to_string(),
            psbt_base64,
        },
    )
    .await
}

async fn sign_and_publish_request(
    api: &WalletApi,
    name: &str,
    psbt_base64: String,
) -> wallet_api::WalletApiResult<TxBroadcastResultDto> {
    let signed = sign_request(api, name, psbt_base64).await?;
    publish_request(api, name, signed.psbt_base64).await
}

pub async fn sign_psbt(api: &WalletApi, name: &str, psbt_base64: &str) -> Result<()> {
    let signed = sign_request(api, name, psbt_base64.to_string()).await?;

    println!("PSBT signed:");
    println!("finalized={}", signed.finalized);
    println!("signing_status={}", signed.signing_status);
    println!("psbt_base64:\n{}", signed.psbt_base64);

    Ok(())
}

pub async fn publish_psbt(api: &WalletApi, name: &str, psbt_base64: &str) -> Result<()> {
    let published = publish_request(api, name, psbt_base64.to_string()).await?;
    print_broadcast_success("PSBT published:", &published);
    Ok(())
}

pub async fn send_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    send_psbt_with_options(api, name, to, amount_sat, fee_rate_sat_per_vb, false, false).await
}

pub async fn send_psbt_with_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    confirmed_only: bool,
) -> Result<()> {
    let coin_control = if confirmed_only {
        Some(WalletCoinControlDto {
            confirmed_only: true,
            ..Default::default()
        })
    } else {
        None
    };

    let created = service::psbt::create(
        &api.storage,
        CreatePsbtRequestDto {
            name: name.to_string(),
            to_address: to.to_string(),
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control,
        },
    )
    .await?;

    let published = sign_and_publish_request(api, name, created.psbt_base64).await?;
    print_broadcast_success("Transaction broadcast:", &published);

    Ok(())
}

pub async fn send_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
) -> Result<()> {
    send_psbt_with_coin_control_and_options(
        api,
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        false,
        include_outpoints,
        exclude_outpoints,
        true,
        None,
    )
    .await
}

pub async fn send_psbt_with_coin_control_and_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    let created = service::psbt::create(
        &api.storage,
        CreatePsbtRequestDto {
            name: name.to_string(),
            to_address: to.to_string(),
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: Some(build_coin_control_dto(
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            )),
        },
    )
    .await?;

    let published = sign_and_publish_request(api, name, created.psbt_base64).await?;
    print_broadcast_success("Transaction broadcast with coin control:", &published);

    Ok(())
}

pub async fn send_max_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    send_max_psbt_with_options(api, name, to, fee_rate_sat_per_vb, false).await
}

pub async fn send_max_psbt_with_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
) -> Result<()> {
    let created = service::psbt::create_send_max(
        &api.storage,
        SendMaxRequestDto {
            name: name.to_string(),
            to_address: to.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: None,
        },
    )
    .await?;

    let published = sign_and_publish_request(api, name, created.psbt_base64).await?;
    print_broadcast_success("Send-max transaction broadcast:", &published);

    Ok(())
}

pub async fn send_max_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
) -> Result<()> {
    send_max_psbt_with_coin_control_and_options(
        api,
        name,
        to,
        fee_rate_sat_per_vb,
        false,
        include_outpoints,
        exclude_outpoints,
        true,
        None,
    )
    .await
}

pub async fn send_max_psbt_with_coin_control_and_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    let created = service::psbt::create_send_max(
        &api.storage,
        SendMaxRequestDto {
            name: name.to_string(),
            to_address: to.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: Some(build_coin_control_dto(
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            )),
        },
    )
    .await?;

    let published = sign_and_publish_request(api, name, created.psbt_base64).await?;
    print_broadcast_success("Send-max transaction broadcast with coin control:", &published);

    Ok(())
}

pub async fn sweep_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
) -> Result<()> {
    sweep_psbt_with_options(
        api,
        name,
        to,
        fee_rate_sat_per_vb,
        false,
        include_outpoints,
        exclude_outpoints,
        true,
        None,
    )
    .await
}

pub async fn sweep_psbt_with_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    let published = service::psbt::sweep(
        &api.storage,
        SweepRequestDto {
            name: name.to_string(),
            to_address: to.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: build_coin_control_dto(
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            ),
        },
    )
    .await?;

    print_broadcast_success("Sweep transaction broadcast:", &published);

    Ok(())
}

pub async fn consolidate_psbt(
    api: &WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    consolidate_psbt_with_options(
        api,
        name,
        fee_rate_sat_per_vb,
        false,
        Vec::new(),
        Vec::new(),
        true,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await
}

pub async fn consolidate_psbt_with_options(
    api: &WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    max_input_count: Option<usize>,
    min_input_count: Option<usize>,
    min_utxo_value_sat: Option<u64>,
    max_utxo_value_sat: Option<u64>,
    max_fee_pct_of_input_value: Option<u8>,
    strategy: Option<wallet_api::model::WalletConsolidationStrategyDto>,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    let published = service::psbt::consolidate(
        &api.storage,
        ConsolidationRequestDto {
            name: name.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            consolidation: build_consolidation_dto(
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                max_input_count,
                min_input_count,
                min_utxo_value_sat,
                max_utxo_value_sat,
                max_fee_pct_of_input_value,
                strategy,
                selection_mode,
            ),
        },
    )
    .await?;

    print_broadcast_success("Consolidation transaction broadcast:", &published);

    Ok(())
}