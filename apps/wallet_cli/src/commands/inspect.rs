use anyhow::Result;
use tracing::{debug, info};
use wallet_api::model::WalletTxInputDto;
use wallet_api::WalletApi;

pub async fn txs(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli inspect: txs start name={}", name);

    let mut txs = api.txs(name).await?;
    txs.sort_by(|a, b| b.confirmation_height.cmp(&a.confirmation_height));

    if txs.is_empty() {
        println!("No transactions found.");
        return Ok(());
    }

    info!(
        "cli inspect: txs fetched count={} for wallet {}",
        txs.len(),
        name
    );

    for tx in txs {
        let fee = tx
            .fee
            .map(|value| format!("{} sats", value))
            .unwrap_or_else(|| "n/a".to_string());

        let fee_rate = tx
            .fee_rate_sat_per_vb
            .map(|value| format!("{} sat/vB", value))
            .unwrap_or_else(|| "n/a".to_string());

        let height = tx
            .confirmation_height
            .map(|height| height.to_string())
            .unwrap_or_else(|| "unconfirmed".to_string());

        let parent_txids = parent_txids_from_inputs(&tx.inputs);

        println!(
            "txid={} | dir={:<8} | net={:>8} sats | fee={:<10} | fee_rate={:<10} | rbf={} | confirmed={} | height={} | inputs={} | parents={} | wallet_outputs={}",
            tx.txid,
            tx.direction,
            tx.net_value,
            fee,
            fee_rate,
            tx.replaceable,
            tx.confirmed,
            height,
            tx.inputs.len(),
            parent_txids.len(),
            tx.outputs.len()
        );

        if !tx.inputs.is_empty() {
            println!("  inputs:");
            for input in &tx.inputs {
                println!("  - previous_outpoint={}", input.previous_outpoint);
            }
        }

        if !parent_txids.is_empty() {
            println!("  parents:");
            for parent_txid in parent_txids {
                println!("  - {}", parent_txid);
            }
        }

        if !tx.outputs.is_empty() {
            println!("  wallet_outputs:");
            for output in &tx.outputs {
                let address = output.address.as_deref().unwrap_or("unknown");
                let keychain = output.keychain.as_deref().unwrap_or("unknown");
                println!(
                    "  - outpoint={} | value={} sats | mine={} | keychain={} | address={}",
                    output.outpoint,
                    output.value_sat,
                    output.is_mine,
                    keychain,
                    address
                );
            }
        }
    }

    Ok(())
}

pub async fn utxos(api: &WalletApi, name: &str) -> Result<()> {
    debug!("cli inspect: utxos start name={}", name);

    let utxos = api.utxos(name).await?;

    if utxos.is_empty() {
        println!("No UTXOs found.");
        return Ok(());
    }

    info!(
        "cli inspect: utxos fetched count={} locked_count={} with_derivation_index={} for wallet {}",
        utxos.len(),
        utxos.iter().filter(|utxo| utxo.is_locked).count(),
        utxos.iter().filter(|utxo| utxo.derivation_index.is_some()).count(),
        name
    );

    for utxo in utxos {
        let address = utxo.address.as_deref().unwrap_or("unknown");

        let derivation_index = utxo
            .derivation_index
            .map(|index| index.to_string())
            .unwrap_or_else(|| "n/a".to_string());

        let lock_state = if utxo.is_locked { "locked" } else { "spendable" };
        let lock_reason = utxo.lock_reason.as_deref().unwrap_or("n/a");
        let locked_at = utxo.locked_at.as_deref().unwrap_or("n/a");

        println!(
            "outpoint={} | value={} sats | confirmed={} | state={} | keychain={} | index={} | address={} | lock_reason={} | locked_at={}",
            utxo.outpoint,
            utxo.value,
            utxo.confirmed,
            lock_state,
            utxo.keychain,
            derivation_index,
            address,
            lock_reason,
            locked_at
        );
    }

    Ok(())
}

fn parent_txids_from_inputs(inputs: &[WalletTxInputDto]) -> Vec<String> {
    let mut parents = Vec::new();

    for input in inputs {
        if let Some((txid, _vout)) = input.previous_outpoint.split_once(':') {
            if !txid.is_empty() && !parents.iter().any(|existing| existing == txid) {
                parents.push(txid.to_string());
            }
        }
    }

    parents
}
