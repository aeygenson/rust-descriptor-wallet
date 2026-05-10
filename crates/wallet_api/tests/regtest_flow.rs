mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    CreatePsbtRequestDto, PublishPsbtRequestDto, SignPsbtRequestDto, WalletAddressRequestDto,
    WalletTransactionsRequestDto, WalletUtxosRequestDto, WalletReceiveAddressHistoryDto,
};
use wallet_api::service;

async fn wallet_address(
    api: &wallet_api::api::WalletApi,
    name: &str,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletReceiveAddressHistoryDto> {
    service::addresses::address(
        &api.storage,
        WalletAddressRequestDto {
            name: name.to_string(),
        },
    )
    .await
}

async fn wallet_txs(
    api: &wallet_api::api::WalletApi,
    name: &str,
) -> wallet_api::WalletApiResult<Vec<wallet_api::model::WalletTxDto>> {
    service::inspect::txs(
        &api.storage,
        WalletTransactionsRequestDto {
            name: name.to_string(),
        },
    )
    .await
}

async fn wallet_utxos(
    api: &wallet_api::api::WalletApi,
    name: &str,
) -> wallet_api::WalletApiResult<Vec<wallet_api::model::WalletUtxoDto>> {
    service::inspect::utxos(
        &api.storage,
        WalletUtxosRequestDto {
            name: name.to_string(),
        },
    )
    .await
}

async fn send_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    to_address: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    confirmed_only: bool,
) -> wallet_api::WalletApiResult<wallet_api::model::TxBroadcastResultDto> {
    let coin_control = if confirmed_only {
        Some(wallet_api::model::WalletCoinControlDto {
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
            to_address: to_address.to_string(),
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control,
        },
    )
    .await?;

    let signed = service::psbt::sign(
        &api.storage,
        SignPsbtRequestDto {
            name: name.to_string(),
            psbt_base64: created.psbt_base64,
        },
    )
    .await?;

    service::psbt::publish(
        &api.storage,
        PublishPsbtRequestDto {
            name: name.to_string(),
            psbt_base64: signed.psbt_base64,
        },
    )
    .await
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_receives_funds_after_sync() -> anyhow::Result<()> {
    // 1. Start regtest environment
    let env = RegtestEnv::new();
    env.start()?;

    // 2. Build API
    let api = build_default_api().await?;

    let wallet_name = "regtest-local";

    // 3. Initial sync
    api.sync(wallet_name).await?;

    // 4. Get a new address
    let addr = wallet_address(&api, wallet_name).await?;
    assert_eq!(addr.keychain, "external");
    assert!(addr.index.is_some());

    // 5. Fund the address (50_000 sats)
    let btc_addr = parse_regtest_address(&addr.address)?;
    env.fund_sats(&btc_addr, 50_000)?;

    // 6. Mine a block to confirm
    env.mine(1)?;

    // 7. Sync again
    api.sync(wallet_name).await?;

    // 8. Check balance
    let balance = api.balance(wallet_name).await?;

    // 9. Assert balance increased
    assert!(
        balance >= 50_000,
        "expected at least 50_000 sats, got {}",
        balance
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_self_send_creates_change() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Make sure wallet state is up to date before building the spend.
    api.sync(wallet_name).await?;

    let balance_before = api.balance(wallet_name).await?;

    // Generate a fresh wallet address and send funds to ourselves.
    let destination = wallet_address(&api, wallet_name).await?;
    assert_eq!(destination.keychain, "external");
    assert!(destination.index.is_some());

    let published = send_psbt(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        false,
        false,
    )
    .await?;

    assert!(
        !published.txid.is_empty(),
        "expected broadcast txid to be present"
    );

    // Sync to observe the unconfirmed transaction and its outputs.
    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected self-send transaction to appear in tx list");

    let fee = sent_tx
        .fee
        .expect("expected self-send transaction fee to be present");
    assert!(fee > 0, "expected positive fee, got {}", fee);
    assert_eq!(sent_tx.net_value, -(fee as i64));
    assert!(
        !sent_tx.confirmed,
        "expected self-send transaction to be unconfirmed before mining"
    );

    let utxos = wallet_utxos(&api, wallet_name).await?;
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == published.txid
                && u.value == 10_000
                && u.keychain == "external"
        }),
        "expected recipient output with value 10000 sats"
    );
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == published.txid
                && u.value == 10_000
                && u.keychain == "external"
                && u.derivation_index.is_some()
        }),
        "expected recipient output to preserve derivation index"
    );
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == published.txid && u.value > 0 && u.keychain == "internal"
        }),
        "expected internal change output for self-send transaction"
    );

    // Confirm the transaction, then re-sync and verify final accounting.
    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected self-send transaction after mining");
    assert!(
        sent_tx.confirmed,
        "expected self-send transaction to be confirmed after mining"
    );

    let balance = api.balance(wallet_name).await?;
    assert_eq!(
        balance,
        balance_before - fee,
        "expected balance to decrease only by fee"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_address_returns_increasing_external_indexes() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;

    let first = wallet_address(&api, wallet_name).await?;
    let second = wallet_address(&api, wallet_name).await?;
    let third = wallet_address(&api, wallet_name).await?;

    assert_eq!(first.keychain, "external");
    assert_eq!(second.keychain, "external");
    assert_eq!(third.keychain, "external");

    let first_index = first.index.expect("first receive address should expose index");
    let second_index = second.index.expect("second receive address should expose index");
    let third_index = third.index.expect("third receive address should expose index");

    assert_eq!(second_index, first_index + 1);
    assert_eq!(third_index, second_index + 1);

    assert_ne!(first.address, second.address);
    assert_ne!(second.address, third.address);
    assert_ne!(first.address, third.address);

    parse_regtest_address(&first.address)?;
    parse_regtest_address(&second.address)?;
    parse_regtest_address(&third.address)?;

    Ok(())
}