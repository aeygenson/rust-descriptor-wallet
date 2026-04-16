use test_support::wallet::parse_regtest_address;
use test_support::RegtestEnv;
use wallet_api::WalletApi;

/// Ensure the wallet has at least `min_count` confirmed UTXOs with value >= `min_value_sat`.
///
/// If the wallet does not yet have enough eligible UTXOs, this helper funds fresh
/// receive addresses from the regtest miner wallet, mines one block, re-syncs, and
/// then returns the refreshed eligible set.
pub async fn ensure_confirmed_wallet_utxos(
    api: &WalletApi,
    env: &RegtestEnv,
    wallet_name: &str,
    min_count: usize,
    min_value_sat: u64,
) -> anyhow::Result<Vec<(String, u64)>> {
    api.sync_wallet(wallet_name).await?;

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
        api.sync_wallet(wallet_name).await?;

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
