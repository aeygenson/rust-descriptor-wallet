use anyhow::Result;
use tracing::{debug, info};
use wallet_api::WalletApi;

pub async fn create_psbt_with_coin_control(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    include_outpoints: Vec<String>,
    exclude_outpoints: Vec<String>,
    confirmed_only: bool,
) -> Result<()> {
    debug!(
        "cli runtime: create_psbt_with_coin_control start name={} to={} amount={} fee_rate={} include={} exclude={} confirmed_only={}",
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only
    );

    let psbt = api
        .create_psbt_with_coin_control(
            name,
            to,
            amount_sat,
            fee_rate_sat_per_vb,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
            },
        )
        .await?;

    println!("PSBT created with coin control:");
    println!("txid={}", psbt.txid);
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("fee_rate={} sat/vB", psbt.fee_rate_sat_per_vb);
    println!("selected_utxos={}", psbt.selected_utxo_count);
    if !psbt.selected_inputs.is_empty() {
        println!("selected_inputs:");
        for input in &psbt.selected_inputs {
            println!("- {}", input);
        }
    }
    println!("inputs={}", psbt.input_count);
    println!("outputs={}", psbt.output_count);
    println!("estimated_vsize={} vB", psbt.estimated_vsize);

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}

pub async fn create_send_max_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    debug!(
        "cli runtime: create_send_max_psbt start name={} to={} fee_rate={}",
        name, to, fee_rate_sat_per_vb
    );

    let psbt = api
        .create_send_max_psbt(name, to, fee_rate_sat_per_vb)
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
) -> Result<()> {
    debug!(
        "cli runtime: create_send_max_psbt_with_coin_control start name={} to={} fee_rate={} include={} exclude={} confirmed_only={}",
        name,
        to,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only
    );

    let psbt = api
        .create_send_max_psbt_with_coin_control(
            name,
            to,
            fee_rate_sat_per_vb,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
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
) -> Result<()> {
    debug!(
        "cli runtime: create_sweep_psbt start name={} to={} fee_rate={} include={} exclude={} confirmed_only={}",
        name,
        to,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only
    );

    let psbt = api
        .create_sweep_psbt(
            name,
            to,
            fee_rate_sat_per_vb,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
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
) -> Result<()> {
    debug!(
        "cli runtime: create_consolidation_psbt start name={} fee_rate={} include={} exclude={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?}",
        name,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        max_input_count,
        min_input_count,
        min_utxo_value_sat,
        max_utxo_value_sat,
        max_fee_pct_of_input_value,
        strategy,
    );

    let psbt = api
        .create_consolidation(
            name,
            fee_rate_sat_per_vb,
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
    }
}

pub async fn address(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: address start name={}", name);
    let addr = api.address(name).await?;
    info!("cli runtime: address generated for wallet {}", name);
    println!("{addr}");
    Ok(())
}

pub async fn sync(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: sync start name={}", name);
    api.sync_wallet(name).await?;
    info!("cli runtime: sync success for wallet {}", name);
    println!("Synced wallet {name}");
    Ok(())
}
pub async fn balance(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: balance start name={}", name);
    let bal = api.balance(name).await?;
    info!("cli runtime: balance fetched for wallet {}", name);
    println!("balance={} sats", bal);
    Ok(())
}

pub async fn status(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: status start name={}", name);

    let status = api.status(name).await?;

    let last_block = status
        .last_block_height
        .map(|h| h.to_string())
        .unwrap_or_else(|| "unknown".to_string());

    info!(
        "cli runtime: status success name={} balance={} utxos={} last_block={}",
        name, status.balance, status.utxo_count, last_block
    );

    println!("wallet={}", name);
    println!("balance={} sats", status.balance);
    println!("utxos={}", status.utxo_count);
    println!("last_block={}", last_block);

    Ok(())
}

pub async fn txs(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: txs start name={}", name);

    let mut txs = api.txs(name).await?;
    txs.sort_by(|a, b| b.confirmation_height.cmp(&a.confirmation_height));

    if txs.is_empty() {
        println!("No transactions found.");
    } else {
        info!(
            "cli runtime: txs fetched count={} for wallet {}",
            txs.len(),
            name
        );

        for tx in txs {
            let fee = tx
                .fee
                .map(|v| format!("{} sats", v))
                .unwrap_or_else(|| "n/a".to_string());

            let fee_rate = tx
                .fee_rate_sat_per_vb
                .map(|v| format!("{} sat/vB", v))
                .unwrap_or_else(|| "n/a".to_string());

            let replaceable = tx.replaceable.to_string();

            let height = tx
                .confirmation_height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unconfirmed".to_string());

            println!(
                "txid={} | dir={:<8} | net={:>8} sats | fee={:<10} | fee_rate={:<10} | rbf={} | confirmed={} | height={}",
                tx.txid,
                tx.direction,
                tx.net_value,
                fee,
                fee_rate,
                replaceable,
                tx.confirmed,
                height
            );
        }
    }

    Ok(())
}

pub async fn utxos(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli runtime: utxos start name={}", name);

    let mut utxos = api.utxos(name).await?;
    utxos.sort_by(|a, b| b.confirmation_height.cmp(&a.confirmation_height));

    if utxos.is_empty() {
        println!("No UTXOs found.");
    } else {
        info!(
            "cli runtime: utxos fetched count={} for wallet {}",
            utxos.len(),
            name
        );

        for utxo in utxos {
            let address = utxo.address.as_deref().unwrap_or("unknown");

            let height = utxo
                .confirmation_height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unconfirmed".to_string());

            println!(
                "outpoint={} | value={} sats | addr={} | keychain={} | confirmed={} | height={}",
                utxo.outpoint, utxo.value, address, utxo.keychain, utxo.confirmed, height
            );
        }
    }

    Ok(())
}

pub async fn create_psbt(
    api: &WalletApi,
    name: &str,
    to: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
) -> Result<()> {
    debug!(
        "cli runtime: create_psbt start name={} to={} amount={} fee_rate={}",
        name, to, amount_sat, fee_rate_sat_per_vb
    );

    let psbt = api
        .create_psbt(name, to, amount_sat, fee_rate_sat_per_vb)
        .await?;

    info!(
        "cli runtime: create_psbt success name={} txid={} to={} amount={} fee={} inputs={} outputs={} recipients={} vsize={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
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
    debug!(
        "cli runtime: send start name={} to={} amount={} fee_rate={}",
        name, to, amount_sat, fee_rate_sat_per_vb
    );

    let published = api
        .send_psbt(name, to, amount_sat, fee_rate_sat_per_vb)
        .await?;

    info!(
        "cli runtime: send success name={} to={} amount={} txid={}",
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
) -> Result<()> {
    debug!(
        "cli runtime: send_psbt_with_coin_control start name={} to={} amount={} fee_rate={} include={} exclude={} confirmed_only={}",
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only
    );

    let published = api
        .send_psbt_with_coin_control(
            name,
            to,
            amount_sat,
            fee_rate_sat_per_vb,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
            },
        )
        .await?;

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
    debug!(
        "cli runtime: send_max_psbt start name={} to={} fee_rate={}",
        name, to, fee_rate_sat_per_vb
    );

    let published = api.send_max_psbt(name, to, fee_rate_sat_per_vb).await?;

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
) -> Result<()> {
    debug!(
        "cli runtime: send_max_psbt_with_coin_control start name={} to={} fee_rate={} include={} exclude={} confirmed_only={}",
        name,
        to,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only
    );

    let published = api
        .send_max_psbt_with_coin_control(
            name,
            to,
            fee_rate_sat_per_vb,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
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
) -> Result<()> {
    debug!(
        "cli runtime: sweep_psbt start name={} to={} fee_rate={} include={} exclude={} confirmed_only={}",
        name,
        to,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only
    );

    let published = api
        .sweep_psbt(
            name,
            to,
            fee_rate_sat_per_vb,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints,
                exclude_outpoints,
                confirmed_only,
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
) -> Result<()> {
    debug!(
        "cli runtime: consolidate_psbt start name={} fee_rate={} include={} exclude={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?}",
        name,
        fee_rate_sat_per_vb,
        include_outpoints.len(),
        exclude_outpoints.len(),
        confirmed_only,
        max_input_count,
        min_input_count,
        min_utxo_value_sat,
        max_utxo_value_sat,
        max_fee_pct_of_input_value,
        strategy,
    );

    let published = api
        .consolidate(
            name,
            fee_rate_sat_per_vb,
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
            ),
        )
        .await?;

    println!("Consolidation transaction sent successfully:");
    println!("txid={}", published.txid);
    print_optional_rbf(published.replaceable);

    Ok(())
}
