use anyhow::Result;

use wallet_api::model::{
    ConsolidationRequestDto, CreatePsbtRequestDto, SendMaxRequestDto, SweepRequestDto,
    WalletCoinControlDto, WalletConsolidationDto,
    WalletConsolidationStrategyDto, WalletInputSelectionModeDto, WalletPsbtDto,
};
use wallet_api::{service, WalletApi};

fn print_optional_rbf(replaceable: Option<bool>) {
    if let Some(replaceable) = replaceable {
        println!("replaceable={}", replaceable);
    }
}

pub fn print_wallet_psbt(title: &str, psbt: &WalletPsbtDto) {
    println!("{}", title);
    println!("psbt_base64:\n{}", psbt.psbt_base64);
    println!("fee_sat={}", psbt.fee_sat);
    println!("fee_rate_sat_per_vb={}", psbt.fee_rate_sat_per_vb);
    println!("input_count={}", psbt.input_count);
    println!("output_count={}", psbt.output_count);
    println!("replaceable={}", psbt.replaceable);

    if let Some(change_amount_sat) = psbt.change_amount_sat {
        println!("change_amount_sat={}", change_amount_sat);
    }

    if psbt.replacement.is_some() {
        println!("replacement=true");
    }
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
    strategy: Option<WalletConsolidationStrategyDto>,
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

pub async fn create_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    create_psbt_with_options(
        api,
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        false,
        false,
    )
    .await
}

pub async fn create_psbt_with_options(
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

    let psbt = service::psbt::create(
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

    print_wallet_psbt("PSBT created:", &psbt);
    Ok(())
}

pub async fn create_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
) -> Result<()> {
    create_psbt_with_coin_control_and_options(
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

pub async fn create_psbt_with_coin_control_and_options(
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
    let psbt = service::psbt::create(
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

    print_wallet_psbt("PSBT created with coin control:", &psbt);
    Ok(())
}

pub async fn create_send_max_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    create_send_max_psbt_with_options(api, name, to, fee_rate_sat_per_vb, false).await
}

pub async fn create_send_max_psbt_with_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
) -> Result<()> {
    let psbt = service::psbt::create_send_max(
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

    print_wallet_psbt("Send-max PSBT created:", &psbt);
    Ok(())
}

pub async fn create_send_max_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
) -> Result<()> {
    create_send_max_psbt_with_coin_control_and_options(
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

pub async fn create_send_max_psbt_with_coin_control_and_options(
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
    let psbt = service::psbt::create_send_max(
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

    print_wallet_psbt("Send-max PSBT created with coin control:", &psbt);
    Ok(())
}

pub async fn create_sweep_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
) -> Result<()> {
    create_sweep_psbt_with_options(
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

pub async fn create_sweep_psbt_with_options(
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
    let psbt = service::psbt::create_sweep(
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

    print_wallet_psbt("Sweep PSBT created:", &psbt);
    Ok(())
}

pub async fn create_consolidation_psbt(
    api: &WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    create_consolidation_psbt_with_options(
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

pub async fn create_consolidation_psbt_with_options(
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
    strategy: Option<WalletConsolidationStrategyDto>,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    let psbt = service::psbt::create_consolidation(
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

    print_wallet_psbt("Consolidation PSBT created:", &psbt);
    Ok(())
}
