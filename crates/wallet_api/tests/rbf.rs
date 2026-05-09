mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    BumpFeeRequestDto, CreatePsbtRequestDto, PublishPsbtRequestDto, SignPsbtRequestDto,
    WalletAddressRequestDto, WalletTransactionsRequestDto, WalletUtxosRequestDto,
};
use wallet_api::service;

async fn wallet_address(
    api: &wallet_api::api::WalletApi,
    name: &str,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletReceiveAddressDto> {
    service::wallet::address(
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

async fn bump_fee_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletPsbtDto> {
    service::psbt::bump_fee_psbt(
        &api.storage,
        BumpFeeRequestDto {
            name: name.to_string(),
            txid: txid.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await
}

async fn bump_fee(
    api: &wallet_api::api::WalletApi,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> wallet_api::WalletApiResult<wallet_api::model::TxBroadcastResultDto> {
    service::psbt::bump_fee(
        &api.storage,
        BumpFeeRequestDto {
            name: name.to_string(),
            txid: txid.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_bump_fee_replaces_unconfirmed_transaction() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Make sure wallet state is up to date and the wallet has enough funds.
    api.sync(wallet_name).await?;
    let mut balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = wallet_address(&api, wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr.address)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
        balance_before = api.balance(wallet_name).await?;
    }

    // Create a self-send we can replace.
    let destination = wallet_address(&api, wallet_name).await?;
    let original = send_psbt(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        true,
        false,
    )
    .await?;
    assert!(
        !original.txid.is_empty(),
        "expected original broadcast txid to be present"
    );
    let original_txid = parse_txid(&original.txid)?;
    assert!(
        mempool_contains(&original_txid)?,
        "expected original transaction to be present in mempool before bump"
    );

    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let original_tx = txs
        .iter()
        .find(|tx| tx.txid == original.txid)
        .expect("expected original unconfirmed transaction to appear in tx list");
    let original_fee = original_tx
        .fee
        .expect("expected original transaction fee to be present");
    assert!(
        !original_tx.confirmed,
        "expected original transaction to be unconfirmed before bump"
    );

    // Replace it with a higher fee transaction.
    let replacement = bump_fee(&api, wallet_name, &original.txid, 5).await?;
    assert!(
        !replacement.txid.is_empty(),
        "expected replacement broadcast txid to be present"
    );
    assert_ne!(
        replacement.txid, original.txid,
        "expected replacement txid to differ from original txid"
    );
    let replacement_txid_rpc = parse_txid(&replacement.txid)?;
    assert!(
        mempool_contains(&replacement_txid_rpc)?,
        "expected replacement transaction to be present in mempool after bump"
    );
    assert!(
        !mempool_contains(&original_txid)?,
        "expected original transaction to be removed from mempool after replacement"
    );

    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let replacement_tx = txs
        .iter()
        .find(|tx| tx.txid == replacement.txid)
        .or_else(|| {
            txs.iter()
                .find(|tx| tx.txid != original.txid && !tx.confirmed && tx.direction == "sent")
        })
        .expect("expected replacement transaction to appear in tx list");
    let replacement_txid = replacement_tx.txid.clone();
    let replacement_fee = replacement_tx
        .fee
        .expect("expected replacement transaction fee to be present");

    assert!(
        !replacement_tx.confirmed,
        "expected replacement transaction to be unconfirmed before mining"
    );
    assert!(
        replacement_fee >= original_fee,
        "expected replacement fee ({}) to be >= original fee ({})",
        replacement_fee,
        original_fee
    );
    assert_eq!(replacement_tx.net_value, -(replacement_fee as i64));

    // Confirm replacement and verify final accounting.
    env.mine(1)?;
    api.sync(wallet_name).await?;
    assert!(
        !mempool_contains(&replacement_txid_rpc)?,
        "expected replacement transaction to leave mempool after confirmation"
    );

    let txs = wallet_txs(&api, wallet_name).await?;
    let replacement_tx = txs
        .iter()
        .find(|tx| tx.txid == replacement_txid)
        .expect("expected replacement transaction after mining");

    assert!(
        replacement_tx.confirmed,
        "expected replacement transaction to be confirmed after mining"
    );

    let utxos = wallet_utxos(&api, wallet_name).await?;
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == replacement_txid
                && u.value == 10_000
                && u.keychain == "external"
        }),
        "expected replacement recipient output with value 10000 sats"
    );
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == replacement_txid
                && u.value > 0
                && u.keychain == "internal"
        }),
        "expected replacement internal change output"
    );

    let balance = api.balance(wallet_name).await?;
    assert_eq!(
        balance,
        balance_before - replacement_fee,
        "expected balance to decrease only by final replacement fee"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_bump_fee_psbt_returns_replacement_metadata() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;
    let mut balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = wallet_address(&api, wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr.address)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
        balance_before = api.balance(wallet_name).await?;
    }

    assert!(
        balance_before >= 50_000,
        "expected wallet to have enough funds for RBF PSBT test"
    );

    let destination = wallet_address(&api, wallet_name).await?;
    let original = send_psbt(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        true,
        false,
    )
    .await?;

    assert!(
        !original.txid.is_empty(),
        "expected original broadcast txid to be present"
    );

    let original_txid = parse_txid(&original.txid)?;
    assert!(
        mempool_contains(&original_txid)?,
        "expected original transaction to be present in mempool before bump PSBT"
    );

    api.sync(wallet_name).await?;

    let replacement_psbt = bump_fee_psbt(&api, wallet_name, &original.txid, 5).await?;

    assert_eq!(replacement_psbt.original_txid.as_deref(), Some(original.txid.as_str()));
    assert_ne!(replacement_psbt.txid, original.txid);
    assert!(replacement_psbt.replaceable);
    assert!(replacement_psbt.fee_sat > 0);
    assert_eq!(replacement_psbt.fee_rate_sat_per_vb, 5);
    assert!(replacement_psbt.estimated_vsize > 0);

    let replacement = replacement_psbt
        .replacement
        .as_ref()
        .expect("expected replacement metadata on bump-fee PSBT");

    assert_eq!(replacement.replaced_txid, original.txid);
    assert_eq!(replacement.replacement_txid, replacement_psbt.txid);
    assert_eq!(replacement.replacement_depth, 1);
    assert_eq!(
        replacement.replacement_chain,
        vec![original.txid.clone(), replacement_psbt.txid.clone()]
    );

    Ok(())
}
