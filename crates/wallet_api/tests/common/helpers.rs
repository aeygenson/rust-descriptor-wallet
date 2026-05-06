use std::collections::HashSet;

pub use test_support::mempool_contains;
pub use test_support::wallet::{
    decode_psbt_inputs, outpoint_txid, parse_regtest_address, parse_txid,
};
use wallet_api::WalletApi;

use super::RegtestEnv;

pub async fn ensure_confirmed_wallet_utxos(
    api: &WalletApi,
    env: &RegtestEnv,
    wallet_name: &str,
    min_count: usize,
    min_value_sat: u64,
) -> anyhow::Result<Vec<(String, u64)>> {
    api.sync(wallet_name).await?;

    let mut confirmed: Vec<(String, u64)> = api
        .utxos(wallet_name)
        .await?
        .into_iter()
        .map(|u| (u.outpoint, u.value, u.confirmed))
        .filter(|(_, value, confirmed)| *confirmed && *value >= min_value_sat)
        .map(|(outpoint, value, _)| (outpoint, value))
        .collect();

    if confirmed.len() < min_count {
        let missing = min_count - confirmed.len();

        for _ in 0..missing {
            let addr = api.address(wallet_name).await?;
            let addr = parse_regtest_address(&addr)?;
            env.fund_sats(&addr, 100_000)?;
        }

        env.mine(1)?;
        api.sync(wallet_name).await?;

        confirmed = api
            .utxos(wallet_name)
            .await?
            .into_iter()
            .map(|u| (u.outpoint, u.value, u.confirmed))
            .filter(|(_, value, confirmed)| *confirmed && *value >= min_value_sat)
            .map(|(outpoint, value, _)| (outpoint, value))
            .collect();
    }

    assert!(
        confirmed.len() >= min_count,
        "expected at least {} confirmed UTXOs with value >= {}, got {}",
        min_count,
        min_value_sat,
        confirmed.len()
    );

    Ok(confirmed)
}

pub async fn fund_exact_confirmed_wallet_utxos(
    api: &WalletApi,
    env: &RegtestEnv,
    wallet_name: &str,
    amounts_sat: &[u64],
) -> anyhow::Result<(Vec<String>, Vec<(String, u64)>)> {
    api.sync(wallet_name).await?;

    let preexisting_confirmed: Vec<String> = api
        .utxos(wallet_name)
        .await?
        .into_iter()
        .filter(|u| u.confirmed)
        .map(|u| u.outpoint)
        .collect();

    let mut funding_txids = HashSet::new();
    for amount_sat in amounts_sat {
        let addr = api.address(wallet_name).await?;
        let addr = parse_regtest_address(&addr)?;
        let txid = env.fund_sats(&addr, *amount_sat)?;
        funding_txids.insert(txid.to_string());
    }

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let funded: Vec<(String, u64)> = api
        .utxos(wallet_name)
        .await?
        .into_iter()
        .filter(|u| u.confirmed && funding_txids.contains(outpoint_txid(&u.outpoint)))
        .map(|u| (u.outpoint, u.value))
        .collect();

    assert_eq!(
        funded.len(),
        amounts_sat.len(),
        "expected exactly {} newly funded confirmed UTXOs, got {}",
        amounts_sat.len(),
        funded.len()
    );

    Ok((preexisting_confirmed, funded))
}
