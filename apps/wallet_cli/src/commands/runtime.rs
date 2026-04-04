use anyhow::Result;
use wallet_api::WalletApi;
use tracing::{debug, info};

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
        name,
        status.balance,
        status.utxo_count,
        last_block
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
        info!("cli runtime: txs fetched count={} for wallet {}", txs.len(), name);

        for tx in txs {
            let fee = tx
                .fee
                .map(|v| format!("{} sats", v))
                .unwrap_or_else(|| "n/a".to_string());

            let height = tx
                .confirmation_height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unconfirmed".to_string());

            println!(
                "txid={} | dir={:<8} | net={:>8} sats | fee={:<10} | confirmed={} | height={}",
                tx.txid,
                tx.direction,
                tx.net_value,
                fee,
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
        info!("cli runtime: utxos fetched count={} for wallet {}", utxos.len(), name);

        for utxo in utxos {
            let address = utxo.address.as_deref().unwrap_or("unknown");

            let height = utxo
                .confirmation_height
                .map(|h| h.to_string())
                .unwrap_or_else(|| "unconfirmed".to_string());

            println!(
                "outpoint={} | value={} sats | addr={} | keychain={} | confirmed={} | height={}",
                utxo.outpoint,
                utxo.value,
                address,
                utxo.keychain,
                utxo.confirmed,
                height
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
        name,
        to,
        amount_sat,
        fee_rate_sat_per_vb
    );

    let psbt = api
        .create_psbt(name, to, amount_sat, fee_rate_sat_per_vb)
        .await?;

    info!(
        "cli runtime: create_psbt success name={} to={} amount={} fee={} inputs={}",
        name,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.selected_utxo_count
    );

    println!("PSBT created:");
    println!("to={}", psbt.to_address);
    println!("amount={} sats", psbt.amount_sat);
    println!("fee={} sats", psbt.fee_sat);
    println!("inputs={}", psbt.selected_utxo_count);

    if let Some(change) = psbt.change_amount_sat {
        println!("change={} sats", change);
    }

    println!("\npsbt_base64:\n{}", psbt.psbt_base64);

    Ok(())
}