use std::collections::HashSet;
use std::time::Duration;
use std::time::{SystemTime, UNIX_EPOCH};

use tokio::time::sleep;
pub use test_support::mempool_contains;
pub use test_support::wallet::{
    decode_psbt_inputs, outpoint_txid, parse_regtest_address, parse_txid,
};
use wallet_storage::WalletStorage;
use wallet_api::model::{WalletAddressRequestDto, WalletUtxosRequestDto};
use wallet_api::{service, WalletApi};

use super::RegtestEnv;

fn is_temp_test_wallet(name: &str) -> bool {
    [
        "regtest-address-",
        "regtest-coin-control-",
        "regtest-consolidation-",
        "regtest-consolidate-",
        "regtest-cpfp-",
        "regtest-flow-",
        "regtest-rbf-",
        "regtest-send-max-",
        "regtest-sweep-",
    ]
    .iter()
    .any(|prefix| name.starts_with(prefix))
}

#[derive(Debug)]
pub struct TestWallet {
    name: String,
    storage: WalletStorage,
}

impl TestWallet {
    pub fn as_str(&self) -> &str {
        &self.name
    }
}

impl Drop for TestWallet {
    fn drop(&mut self) {
        let name = self.name.clone();
        let storage = self.storage.clone();

        let _ = std::thread::Builder::new()
            .name(format!("cleanup-{name}"))
            .spawn(move || {
                let Ok(runtime) = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                else {
                    return;
                };

                let _ = runtime.block_on(storage.delete_wallet(&name));
            })
            .and_then(|handle| handle.join().map_err(|_| std::io::Error::other("cleanup panicked")));
    }
}

pub async fn clone_wallet_for_test(
    api: &WalletApi,
    source_wallet_name: &str,
    prefix: &str,
) -> anyhow::Result<TestWallet> {
    for wallet in api.storage.list_wallets().await? {
        if is_temp_test_wallet(&wallet.name) {
            let _ = api.storage.delete_wallet(&wallet.name).await;
        }
    }

    let source = api.storage.get_wallet_by_name(source_wallet_name).await?;
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)?
        .as_nanos();
    let wallet_name = format!("{prefix}-{}-{nonce}", std::process::id());

    api.storage
        .create_wallet(
            &wallet_name,
            &source.network,
            &source.external_descriptor,
            &source.internal_descriptor,
            &source.sync_backend,
            source.broadcast_backend.as_deref(),
            source.is_watch_only,
        )
        .await?;

    Ok(TestWallet {
        name: wallet_name,
        storage: api.storage.clone(),
    })
}

pub async fn ensure_confirmed_wallet_utxos(
    api: &WalletApi,
    env: &RegtestEnv,
    wallet_name: &str,
    min_count: usize,
    min_value_sat: u64,
) -> anyhow::Result<Vec<(String, u64)>> {
    api.sync(wallet_name).await?;

    let mut confirmed: Vec<(String, u64)> = service::inspect::utxos(
        &api.storage,
        WalletUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?
    .into_iter()
    .map(|u| (u.outpoint, u.value, u.confirmed))
    .filter(|(_, value, confirmed)| *confirmed && *value >= min_value_sat)
    .map(|(outpoint, value, _)| (outpoint, value))
    .collect();

    if confirmed.len() < min_count {
        let missing = min_count - confirmed.len();

        for _ in 0..missing {
            let receive_address = service::addresses::address(
                &api.storage,
                WalletAddressRequestDto {
                    name: wallet_name.to_string(),
                },
            )
            .await?;
            let addr = parse_regtest_address(&receive_address.address)?;
            env.fund_sats(&addr, 100_000)?;
        }

        env.mine(1)?;

        for _ in 0..10 {
            api.sync(wallet_name).await?;

            confirmed = service::inspect::utxos(
                &api.storage,
                WalletUtxosRequestDto {
                    name: wallet_name.to_string(),
                },
            )
            .await?
            .into_iter()
            .map(|u| (u.outpoint, u.value, u.confirmed))
            .filter(|(_, value, confirmed)| *confirmed && *value >= min_value_sat)
            .map(|(outpoint, value, _)| (outpoint, value))
            .collect();

            if confirmed.len() >= min_count {
                break;
            }

            sleep(Duration::from_millis(500)).await;
        }
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

    let preexisting_wallet_outpoints: Vec<String> = service::inspect::utxos(
        &api.storage,
        WalletUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?
    .into_iter()
    .map(|u| u.outpoint)
    .collect();

    for amount_sat in amounts_sat {
        let receive_address = service::addresses::address(
            &api.storage,
            WalletAddressRequestDto {
                name: wallet_name.to_string(),
            },
        )
        .await?;
        let addr = parse_regtest_address(&receive_address.address)?;
        env.fund_sats(&addr, *amount_sat)?;
    }

    let preexisting_wallet_outpoint_set: HashSet<&str> = preexisting_wallet_outpoints
        .iter()
        .map(String::as_str)
        .collect();

    env.mine(1)?;

    let mut funded = Vec::new();
    for _ in 0..10 {
        api.sync(wallet_name).await?;

        funded = service::inspect::utxos(
            &api.storage,
            WalletUtxosRequestDto {
                name: wallet_name.to_string(),
            },
        )
        .await?
        .into_iter()
        .filter(|u| u.confirmed && !preexisting_wallet_outpoint_set.contains(u.outpoint.as_str()))
        .map(|u| (u.outpoint, u.value))
        .collect();

        if funded.len() == amounts_sat.len() {
            break;
        }

        sleep(Duration::from_millis(500)).await;
    }

    assert_eq!(
        funded.len(),
        amounts_sat.len(),
        "expected exactly {} newly funded confirmed UTXOs, got {}",
        amounts_sat.len(),
        funded.len()
    );

    Ok((preexisting_wallet_outpoints, funded))
}
