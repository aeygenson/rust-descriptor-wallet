use anyhow::Result;
use tracing::{debug, info};
use wallet_api::model::WalletInputSelectionModeDto;
use wallet_api::WalletApi;

/// Create a PSBT with coin control through the CLI/runtime boundary.
///
/// This layer intentionally keeps outpoints as `Vec<String>` because it sits
/// above `wallet_api`. Conversion into typed `WalletOutPoint` values happens in
/// the API/model layer before entering `wallet_core`.
pub async fn create_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    create_psbt_with_coin_control_and_options(
        api,
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        true,
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
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
    debug!(
        "cli runtime: create_psbt_with_coin_control_and_options start name={} to={} amount={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} selection_mode={:?}",
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        selection_mode,
    );

    let psbt = api
        .create_psbt_with_coin_control(
            name,
            to,
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            },
        )
        .await?;

    println!("PSBT created with coin control:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn create_send_max_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    create_send_max_psbt_with_options(api, name, to, fee_rate_sat_per_vb, true).await
}

pub async fn create_send_max_psbt_with_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
) -> Result<()> {
    debug!(
        "cli runtime: create_send_max_psbt_with_options start name={} to={} fee_rate={} replaceable={}",
        name, to, fee_rate_sat_per_vb, replaceable
    );

    let psbt = api
        .create_send_max_psbt(name, to, fee_rate_sat_per_vb, replaceable)
        .await?;

    println!("Send-max PSBT created:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn create_send_max_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    create_send_max_psbt_with_coin_control_and_options(
        api,
        name,
        to,
        fee_rate_sat_per_vb,
        true,
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
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
    debug!(
        "cli runtime: create_send_max_psbt_with_coin_control_and_options start name={} to={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} selection_mode={:?}",
        name,
        to,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        selection_mode,
    );

    let psbt = api
        .create_send_max_psbt_with_coin_control(
            name,
            to,
            fee_rate_sat_per_vb,
            replaceable,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            },
        )
        .await?;

    println!("Send-max PSBT created with coin control:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn create_sweep_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    create_sweep_psbt_with_options(
        api,
        name,
        to,
        fee_rate_sat_per_vb,
        true,
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
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
    debug!(
        "cli runtime: create_sweep_psbt_with_options start name={} to={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} selection_mode={:?}",
        name,
        to,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        selection_mode,
    );

    let psbt = api
        .create_sweep_psbt(
            name,
            to,
            fee_rate_sat_per_vb,
            replaceable,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            },
        )
        .await?;

    println!("Sweep PSBT created:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn create_consolidation_psbt(
    api: &WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
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
    create_consolidation_psbt_with_options(
        api,
        name,
        fee_rate_sat_per_vb,
        true,
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
    strategy: Option<wallet_api::model::WalletConsolidationStrategyDto>,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    debug!(
        "cli runtime: create_consolidation_psbt_with_options start name={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?} selection_mode={:?}",
        name,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        max_input_count,
        min_input_count,
        min_utxo_value_sat,
        max_utxo_value_sat,
        max_fee_pct_of_input_value,
        strategy,
        selection_mode,
    );

    let psbt = api
        .create_consolidation(
            name,
            fee_rate_sat_per_vb,
            replaceable,
            build_consolidation_dto(
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
        )
        .await?;

    println!("Consolidation PSBT created:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

fn print_optional_rbf(replaceable: Option<bool>) {
    if let Some(rbf) = replaceable {
        println!("rbf={}", rbf);
    }
}

fn print_broadcast_success(title: &str, txid: &str, replaceable: Option<bool>) {
    println!("{}", title);
    println!("txid={}", txid);
    print_optional_rbf(replaceable);
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
) -> wallet_api::model::WalletConsolidationDto {
    wallet_api::model::WalletConsolidationDto {
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
    create_psbt_with_options(api, name, to, amount_sat, fee_rate_sat_per_vb, true, false).await
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
    debug!(
        "cli runtime: create_psbt_with_options start name={} to={} amount={} fee_rate={} replaceable={} confirmed_only={}",
        name, to, amount_sat, fee_rate_sat_per_vb, replaceable, confirmed_only
    );

    let psbt = api
        .create_psbt(
            name,
            to,
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            confirmed_only,
        )
        .await?;

    info!(
        "cli runtime: create_psbt_with_options success name={} txid={} to={} amount={} fee={} replaceable={} inputs={} outputs={} recipients={} vsize={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.replaceable,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
    );

    println!("PSBT created:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn bump_fee_psbt(
    api: &WalletApi,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    debug!(
        "cli runtime: bump_fee_psbt start name={} txid={} fee_rate={}",
        name, txid, fee_rate_sat_per_vb
    );

    let psbt = api.bump_fee_psbt(name, txid, fee_rate_sat_per_vb).await?;

    info!(
        "cli runtime: bump_fee_psbt success name={} original_txid={} replacement_txid={} fee={} inputs={} outputs={} recipients={} vsize={}",
        name,
        txid,
        psbt.txid,
        psbt.fee_sat,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
    );

    println!("Replacement PSBT created:");
    println!("original_txid={}", txid);
    println!("replacement_txid={}", psbt.txid);
    if let Some(original_txid) = &psbt.original_txid {
        println!("tracked_original_txid={}", original_txid);
    }
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("recipients={}", psbt.recipient_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

/// Create a CPFP PSBT through the CLI/runtime boundary.
///
/// The selected outpoint remains a string at this layer and is parsed into a
/// typed `WalletOutPoint` inside `wallet_api` before calling `wallet_core`.
pub async fn cpfp_psbt(
    api: &WalletApi,
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    debug!(
        "cli runtime: cpfp_psbt start name={} parent_txid={} selected_outpoint={} fee_rate={}",
        name, parent_txid, selected_outpoint, fee_rate_sat_per_vb
    );

    let psbt = api
        .cpfp_psbt(name, parent_txid, selected_outpoint, fee_rate_sat_per_vb)
        .await?;

    info!(
        "cli runtime: cpfp_psbt success name={} parent_txid={} child_txid={} selected_outpoint={} input_value_sat={} child_output_value_sat={} fee_sat={} vsize={}",
        name,
        psbt.parent_txid,
        psbt.txid,
        psbt.selected_outpoint,
        psbt.input_value_sat,
        psbt.child_output_value_sat,
        psbt.fee_sat,
        psbt.estimated_vsize,
    );

    println!("CPFP PSBT created:");
    println!("parent_txid={}", psbt.parent_txid);
    println!("child_txid={}", psbt.txid);
    println!("selected_outpoint={}", psbt.selected_outpoint);
    println!("input_value={} sats", psbt.input_value_sat);
    println!("child_output_value={} sats", psbt.child_output_value_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("replaceable={}", psbt.replaceable);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);
    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn bump_fee(
    api: &WalletApi,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    debug!(
        "cli runtime: bump_fee start name={} txid={} fee_rate={}",
        name, txid, fee_rate_sat_per_vb
    );

    let published = api.bump_fee(name, txid, fee_rate_sat_per_vb).await?;

    info!(
        "cli runtime: bump_fee success name={} original_txid={} replacement_txid={}",
        name, txid, published.txid
    );

    println!("Replacement transaction broadcasted successfully:");
    println!("original_txid={}", txid);
    println!("replacement_txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}

/// Build and broadcast a CPFP transaction through the CLI/runtime boundary.
///
/// The selected outpoint remains a string at this layer and is parsed into a
/// typed `WalletOutPoint` inside `wallet_api` before calling `wallet_core`.
pub async fn cpfp(
    api: &WalletApi,
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    debug!(
        "cli runtime: cpfp start name={} parent_txid={} selected_outpoint={} fee_rate={}",
        name, parent_txid, selected_outpoint, fee_rate_sat_per_vb
    );

    let published = api
        .cpfp(name, parent_txid, selected_outpoint, fee_rate_sat_per_vb)
        .await?;

    info!(
        "cli runtime: cpfp success name={} parent_txid={} child_txid={}",
        name, parent_txid, published.txid
    );

    println!("CPFP transaction broadcasted successfully:");
    println!("parent_txid={}", parent_txid);
    println!("selected_outpoint={}", selected_outpoint);
    println!("child_txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}

pub async fn sign_psbt(api: &WalletApi, name: &str, psbt_base64: &str) -> Result<()> {
    debug!("cli runtime: sign_psbt start name={}", name);

    let signed = api.sign_psbt(name, psbt_base64).await?;

    info!(
        "cli runtime: sign_psbt success name={} modified={} finalized={} txid={}",
        name, signed.modified, signed.finalized, signed.txid
    );

    match signed.signing_status.as_str() {
        "finalized" => println!("PSBT finalized successfully:"),
        "partially_signed" => println!("PSBT partially signed:"),
        _ => println!("No signatures were added to the PSBT:"),
    }

    println!("txid={}", signed.txid);
    println!("modified={}", signed.modified);
    println!("finalized={}", signed.finalized);
    println!("\npsbt_base64:\n{}", signed.psbt_base64);

    Ok(())
}

pub async fn publish_psbt(api: &WalletApi, name: &str, psbt_base64: &str) -> Result<()> {
    debug!("cli runtime: publish_psbt start name={}", name);

    let published = api.publish_psbt(name, psbt_base64).await?;

    info!(
        "cli runtime: publish_psbt success name={} txid={}",
        name, published.txid
    );

    print_broadcast_success(
        "Transaction broadcasted successfully:",
        &published.txid,
        published.replaceable,
    );

    Ok(())
}

pub async fn send_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    send_psbt_with_options(api, name, to, amount_sat, fee_rate_sat_per_vb, true, false).await
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
    debug!(
        "cli runtime: send_psbt_with_options start name={} to={} amount={} fee_rate={} replaceable={} confirmed_only={}",
        name, to, amount_sat, fee_rate_sat_per_vb, replaceable, confirmed_only
    );

    let created = api
        .create_psbt(
            name,
            to,
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            confirmed_only,
        )
        .await?;
    let published = api.sign_and_publish(name, &created.psbt_base64).await?;

    info!(
        "cli runtime: send_psbt_with_options success name={} to={} amount={} txid={}",
        name, to, amount_sat, published.txid
    );

    println!("Transaction sent successfully:");
    println!("to={}", to);
    println!("amount={} sats", amount_sat);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

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
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    send_psbt_with_coin_control_and_options(
        api,
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        true,
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
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
    debug!(
        "cli runtime: send_psbt_with_coin_control_and_options start name={} to={} amount={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} selection_mode={:?}",
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        selection_mode,
    );

    let created = api
        .create_psbt_with_coin_control(
            name,
            to,
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            },
        )
        .await?;
    let published = api.sign_and_publish(name, &created.psbt_base64).await?;

    println!("Transaction sent with coin control:");
    println!("to={}", to);
    println!("amount={} sats", amount_sat);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}

pub async fn send_max_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    send_max_psbt_with_options(api, name, to, fee_rate_sat_per_vb, true).await
}

pub async fn send_max_psbt_with_options(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
) -> Result<()> {
    debug!(
        "cli runtime: send_max_psbt_with_options start name={} to={} fee_rate={} replaceable={}",
        name, to, fee_rate_sat_per_vb, replaceable
    );

    let published = api
        .send_max_psbt(name, to, fee_rate_sat_per_vb, replaceable)
        .await?;

    println!("Send-max transaction sent successfully:");
    println!("to={}", to);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}

pub async fn send_max_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    send_max_psbt_with_coin_control_and_options(
        api,
        name,
        to,
        fee_rate_sat_per_vb,
        true,
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
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
    debug!(
        "cli runtime: send_max_psbt_with_coin_control_and_options start name={} to={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} selection_mode={:?}",
        name,
        to,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        selection_mode,
    );

    let published = api
        .send_max_psbt_with_coin_control(
            name,
            to,
            fee_rate_sat_per_vb,
            replaceable,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            },
        )
        .await?;

    println!("Send-max transaction sent with coin control:");
    println!("to={}", to);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}

pub async fn sweep_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
    selection_mode: Option<WalletInputSelectionModeDto>,
) -> Result<()> {
    sweep_psbt_with_options(
        api,
        name,
        to,
        fee_rate_sat_per_vb,
        true,
        include_outpoints,
        exclude_outpoints,
        confirmed_only,
        selection_mode,
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
    debug!(
        "cli runtime: sweep_psbt_with_options start name={} to={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} selection_mode={:?}",
        name,
        to,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        selection_mode,
    );

    let published = api
        .sweep_and_broadcast(
            name,
            to,
            fee_rate_sat_per_vb,
            replaceable,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
                selection_mode,
            },
        )
        .await?;

    println!("Sweep transaction sent successfully:");
    println!("to={}", to);
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}

pub async fn consolidate_psbt(
    api: &WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
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
    consolidate_psbt_with_options(
        api,
        name,
        fee_rate_sat_per_vb,
        true,
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
    debug!(
        "cli runtime: consolidate_psbt_with_options start name={} fee_rate={} replaceable={} include={} exclude={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?} selection_mode={:?}",
        name,
        fee_rate_sat_per_vb,
        replaceable,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        max_input_count,
        min_input_count,
        min_utxo_value_sat,
        max_utxo_value_sat,
        max_fee_pct_of_input_value,
        strategy,
        selection_mode,
    );

    let published = api
        .consolidate_and_broadcast(
            name,
            fee_rate_sat_per_vb,
            replaceable,
            build_consolidation_dto(
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
        )
        .await?;

    println!("Consolidation transaction sent successfully:");
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}
