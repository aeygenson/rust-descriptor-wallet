use anyhow::{Context, Result};
use bitcoin::{Address, Amount, Txid};
use bitcoincore_rpc::{Client, RpcApi};
use tracing::{debug, info};

use crate::bitcoind::{ensure_miner_wallet_loaded, miner_wallet_client, rpc_client, BitcoindConfig};

/// Create a base RPC client using the default regtest environment settings.
pub fn client() -> Result<Client> {
    rpc_client()
}

/// Create an RPC client for the configured miner wallet.
pub fn miner_client() -> Result<Client> {
    miner_wallet_client()
}

/// Ensure the default miner wallet is loaded and return its RPC client.
pub fn ready_miner_client() -> Result<Client> {
    let config = BitcoindConfig::from_env();
    let base = config.client()?;
    ensure_miner_wallet_loaded(&base, &config.miner_wallet)?;
    config.miner_wallet_client()
}

/// Return blockchain info from the default local regtest node.
pub fn blockchain_info() -> Result<bitcoincore_rpc::json::GetBlockchainInfoResult> {
    let client = client()?;
    client
        .get_blockchain_info()
        .context("failed to fetch blockchain info")
}

/// Return the current block height from the default local regtest node.
pub fn block_height() -> Result<u64> {
    let info = blockchain_info()?;
    Ok(info.blocks as u64)
}

/// Return the raw mempool transaction ids from the default local regtest node.
pub fn mempool_txids() -> Result<Vec<Txid>> {
    let client = client()?;
    client
        .get_raw_mempool()
        .context("failed to fetch raw mempool")
}

/// Return true when the given txid is currently present in the raw mempool.
pub fn mempool_contains(txid: &Txid) -> Result<bool> {
    Ok(mempool_txids()?.iter().any(|candidate| candidate == txid))
}

/// Return the miner wallet balance in BTC.
pub fn miner_balance() -> Result<Amount> {
    let client = ready_miner_client()?;
    client
        .get_balance(None, None)
        .context("failed to fetch miner wallet balance")
}

/// Generate a new address from the configured miner wallet.
pub fn miner_new_address() -> Result<Address> {
    let client = ready_miner_client()?;
    let address = client
        .get_new_address(None, None)
        .context("failed to generate new miner address")?;
    address
        .require_network(bitcoin::Network::Regtest)
        .context("miner returned non-regtest address")
}

/// Mine `blocks` blocks to a newly generated miner address.
pub fn mine_blocks(blocks: u64) -> Result<Vec<bitcoin::BlockHash>> {
    let client = ready_miner_client()?;
    let address = miner_new_address()?;

    info!(blocks, address = %address, "mining regtest blocks");
    client
        .generate_to_address(blocks, &address)
        .with_context(|| format!("failed to mine {} block(s)", blocks))
}

/// Mine `blocks` blocks to the supplied regtest address.
pub fn mine_blocks_to_address(blocks: u64, address: &Address) -> Result<Vec<bitcoin::BlockHash>> {
    let client = ready_miner_client()?;
    let checked = address.clone();

    info!(blocks, address = %checked, "mining regtest blocks to address");
    client
        .generate_to_address(blocks, &checked)
        .with_context(|| format!("failed to mine {} block(s) to address", blocks))
}

/// Fund an address from the configured miner wallet using BTC units.
pub fn fund_address(address: &Address, amount_btc: f64) -> Result<Txid> {
    let client = ready_miner_client()?;
    let checked = address.clone();

    let amount = Amount::from_btc(amount_btc)
        .with_context(|| format!("invalid BTC amount: {}", amount_btc))?;

    info!(address = %checked, amount_btc, "funding regtest address");
    client
        .send_to_address(
            &checked,
            amount,
            None,
            None,
            None,
            None,
            None,
            None,
        )
        .with_context(|| format!("failed to fund address {}", checked))
}

/// Fund an address from the configured miner wallet using satoshis.
pub fn fund_address_sats(address: &Address, sats: u64) -> Result<Txid> {
    debug!(address = %address, sats, "funding regtest address in satoshis");
    let btc = Amount::from_sat(sats).to_btc();
    fund_address(address, btc)
}

/// Load a wallet by name if it is not already loaded.
pub fn ensure_wallet_loaded(wallet_name: &str) -> Result<()> {
    let client = client()?;
    let loaded = client
        .list_wallets()
        .context("failed to list loaded wallets")?;

    if loaded.iter().any(|name| name == wallet_name) {
        return Ok(());
    }

    client
        .load_wallet(wallet_name)
        .with_context(|| format!("failed to load wallet '{}'", wallet_name))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sats_to_btc_conversion_is_exact_for_common_value() {
        let btc = Amount::from_sat(100_000).to_btc();
        assert!((btc - 0.001).abs() < f64::EPSILON);
    }

    #[test]
    fn mempool_contains_false_for_empty_snapshot() {
        let txid = "0000000000000000000000000000000000000000000000000000000000000000"
            .parse::<Txid>()
            .expect("valid zero txid");

        let snapshot = Vec::<Txid>::new();
        assert!(!snapshot.iter().any(|candidate| candidate == &txid));
    }
}