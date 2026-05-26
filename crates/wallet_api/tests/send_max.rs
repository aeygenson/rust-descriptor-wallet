mod common;

use common::*;
use serial_test::file_serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    PublishPsbtRequestDto, SendMaxRequestDto, SignPsbtRequestDto, WalletAddressRequestDto,
    WalletCoinControlDto, WalletTransactionsRequestDto, WalletUtxosRequestDto,
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

async fn create_send_max_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletPsbtDto> {
    service::psbt::create_send_max(
        &api.storage,
        SendMaxRequestDto {
            name: name.to_string(),
            to_address: to_address.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: None,
        },
    )
    .await
}

async fn create_send_max_psbt_with_coin_control(
    api: &wallet_api::api::WalletApi,
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    coin_control: WalletCoinControlDto,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletPsbtDto> {
    service::psbt::create_send_max(
        &api.storage,
        SendMaxRequestDto {
            name: name.to_string(),
            to_address: to_address.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: Some(coin_control),
        },
    )
    .await
}

async fn send_max_psbt_with_coin_control(
    api: &wallet_api::api::WalletApi,
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    coin_control: WalletCoinControlDto,
) -> wallet_api::WalletApiResult<wallet_api::model::TxBroadcastResultDto> {
    let created = create_send_max_psbt_with_coin_control(
        api,
        name,
        to_address,
        fee_rate_sat_per_vb,
        replaceable,
        coin_control,
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
#[file_serial]
async fn wallet_create_send_max_psbt_builds_after_sync() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-send-max").await?;
    let wallet_name = wallet_name.as_str();

    ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    api.sync(wallet_name).await?;
    let available_total = api.balance(wallet_name).await?;

    let destination = wallet_address(&api, wallet_name).await?;
    assert_eq!(destination.keychain, "external");
    assert!(destination.index.is_some());

    let psbt =
        create_send_max_psbt(&api, wallet_name, &destination.address, 1, false).await?;

    assert!(
        !psbt.psbt_base64.is_empty(),
        "expected send-max PSBT payload"
    );
    assert!(!psbt.txid.is_empty(), "expected send-max txid");
    assert_eq!(psbt.to_address, destination.address);
    assert!(psbt.amount_sat > 0, "expected positive send-max amount");
    assert!(psbt.fee_sat > 0, "expected positive send-max fee");
    assert!(
        psbt.amount_sat + psbt.fee_sat <= available_total,
        "expected send-max recipient amount plus fee ({}) to fit wallet balance ({})",
        psbt.amount_sat + psbt.fee_sat,
        available_total
    );
    assert_eq!(
        psbt.selected_inputs.len(),
        psbt.input_count,
        "expected selected_inputs to match actual input count"
    );
    assert!(
        psbt.replacement.is_none(),
        "send-max PSBT should not contain replacement metadata"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[file_serial]
async fn wallet_create_send_max_psbt_with_coin_control_uses_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-send-max").await?;
    let wallet_name = wallet_name.as_str();

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    let requested = confirmed
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for send-max coin control");

    let destination = wallet_address(&api, wallet_name).await?;
    let psbt = create_send_max_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![requested.0.clone()],
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await?;

    let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
    assert_eq!(inputs.len(), 1, "expected exactly one selected input");
    assert_eq!(
        inputs[0], requested.0,
        "expected send-max PSBT to use the requested UTXO"
    );
    assert!(
        psbt.change_amount_sat.is_none(),
        "expected strict send-max sweep to avoid change"
    );
    assert_eq!(
        psbt.output_count, 1,
        "expected a single recipient output in strict send-max sweep"
    );
    assert!(
        psbt.replacement.is_none(),
        "send-max coin-control PSBT should not contain replacement metadata"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[file_serial]
async fn wallet_send_max_psbt_with_coin_control_sweeps_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-send-max").await?;
    let wallet_name = wallet_name.as_str();

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    let requested = confirmed
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for send-max coin control send");

    let destination = wallet_address(&api, wallet_name).await?;
    let published = send_max_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![requested.0.clone()],
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await?;

    assert!(
        !published.txid.is_empty(),
        "expected published send-max txid"
    );

    api.sync(wallet_name).await?;
    let utxos_after_send = wallet_utxos(&api, wallet_name).await?;
    assert!(
        !utxos_after_send.iter().any(|u| u.outpoint == requested.0),
        "expected requested outpoint {} to be fully swept",
        requested.0
    );
    assert!(
        !utxos_after_send
            .iter()
            .any(|u| outpoint_txid(&u.outpoint) == published.txid && u.keychain == "internal"),
        "expected no internal change output for strict send-max sweep"
    );

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected published send-max transaction in tx list");
    assert!(
        sent_tx.confirmed,
        "expected send-max transaction to confirm after mining"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[file_serial]
async fn wallet_create_send_max_psbt_with_coin_control_rejects_insufficient_after_fees(
) -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-send-max").await?;
    let wallet_name = wallet_name.as_str();

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    let requested = confirmed
        .into_iter()
        .min_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for strict send-max test");

    let destination = wallet_address(&api, wallet_name).await?;
    let err = create_send_max_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        requested.1 + 1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![requested.0.clone()],
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await
    .expect_err("expected strict send-max to fail when fees consume the selected input");

    let msg = err.to_string();
    assert!(
        msg.contains("too small")
            || msg.contains("strict mode violation")
            || msg.contains("additional inputs are not allowed"),
        "expected strict send-max error, got: {}",
        msg
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[file_serial]
async fn wallet_send_max_psbt_with_coin_control_sweeps_all_requested_utxos() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-send-max").await?;
    let wallet_name = wallet_name.as_str();

    let mut confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
    confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let requested: Vec<String> = confirmed
        .iter()
        .take(2)
        .map(|(outpoint, _)| outpoint.clone())
        .collect();

    let destination = wallet_address(&api, wallet_name).await?;
    let published = send_max_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: requested.clone(),
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await?;

    assert!(
        !published.txid.is_empty(),
        "expected published multi-input send-max txid"
    );

    api.sync(wallet_name).await?;
    let utxos_after_send = wallet_utxos(&api, wallet_name).await?;
    for outpoint in &requested {
        assert!(
            !utxos_after_send.iter().any(|u| u.outpoint == *outpoint),
            "expected requested outpoint {} to be fully swept in multi-input send-max flow",
            outpoint
        );
    }
    assert!(
        !utxos_after_send
            .iter()
            .any(|u| outpoint_txid(&u.outpoint) == published.txid && u.keychain == "internal"),
        "expected no internal change output for strict multi-input send-max sweep"
    );

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected published multi-input send-max transaction in tx list");
    assert!(
        sent_tx.confirmed,
        "expected multi-input send-max transaction to confirm after mining"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[file_serial]
async fn wallet_send_max_psbt_recipient_and_no_change_invariant() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "regtest-send-max").await?;
    let wallet_name = wallet_name.as_str();

    ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    api.sync(wallet_name).await?;

    let destination = wallet_address(&api, wallet_name).await?;
    let psbt =
        create_send_max_psbt(&api, wallet_name, &destination.address, 1, false).await?;

    assert_eq!(psbt.recipient_count, 1);
    assert_eq!(psbt.output_count, 1);
    assert!(psbt.change_amount_sat.is_none());
    assert!(psbt.amount_sat > 0);

    Ok(())
}
