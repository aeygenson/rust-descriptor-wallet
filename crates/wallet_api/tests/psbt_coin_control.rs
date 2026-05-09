mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    CreatePsbtRequestDto, PublishPsbtRequestDto, SignPsbtRequestDto, WalletAddressRequestDto,
    WalletCoinControlDto, WalletTransactionsRequestDto, WalletUtxosRequestDto,
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

async fn create_psbt_with_coin_control(
    api: &wallet_api::api::WalletApi,
    name: &str,
    to_address: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    coin_control: WalletCoinControlDto,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletPsbtDto> {
    service::psbt::create(
        &api.storage,
        CreatePsbtRequestDto {
            name: name.to_string(),
            to_address: to_address.to_string(),
            amount_sat,
            fee_rate_sat_per_vb,
            replaceable,
            coin_control: Some(coin_control),
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
        Some(WalletCoinControlDto {
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

async fn send_psbt_with_coin_control(
    api: &wallet_api::api::WalletApi,
    name: &str,
    to_address: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    coin_control: WalletCoinControlDto,
) -> wallet_api::WalletApiResult<wallet_api::model::TxBroadcastResultDto> {
    let created = create_psbt_with_coin_control(
        api,
        name,
        to_address,
        amount_sat,
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
#[serial]
async fn wallet_create_psbt_with_coin_control_uses_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let destination = wallet_address(&api, wallet_name).await?;
    assert_eq!(destination.keychain, "external");
    assert!(destination.index.is_some());

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
    let requested = confirmed
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for coin control");

    let psbt = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        10_000,
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
        "expected PSBT to use the requested UTXO"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_psbt_with_coin_control_uses_all_requested_utxos() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let mut confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
    confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let requested: Vec<String> = confirmed
        .iter()
        .take(2)
        .map(|(outpoint, _)| outpoint.clone())
        .collect();

    let destination = wallet_address(&api, wallet_name).await?;
    let psbt = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        150_000,
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

    let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
    assert_eq!(inputs.len(), 2, "expected exactly two selected inputs");
    for outpoint in &requested {
        assert!(
            inputs.contains(outpoint),
            "expected PSBT inputs {:?} to contain requested outpoint {}",
            inputs,
            outpoint
        );
    }

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_psbt_with_coin_control_excludes_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let mut confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;
    confirmed.sort_by(|a, b| a.0.cmp(&b.0));

    let excluded = confirmed[0].0.clone();
    let destination = wallet_address(&api, wallet_name).await?;

    let psbt = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: Vec::new(),
            exclude_outpoints: vec![excluded.clone()],
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await?;

    let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
    assert!(
        !inputs.is_empty(),
        "expected PSBT to contain at least one input"
    );
    assert!(
        !inputs.contains(&excluded),
        "expected excluded outpoint {} not to be used in PSBT inputs {:?}",
        excluded,
        inputs
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_psbt_with_coin_control_rejects_unconfirmed_selected_utxo_when_confirmed_only(
) -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;

    let destination = wallet_address(&api, wallet_name).await?;
    let parent = send_psbt(
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
        !parent.txid.is_empty(),
        "expected parent txid to be present"
    );

    api.sync(wallet_name).await?;
    let utxos = wallet_utxos(&api, wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one unconfirmed wallet-owned output");

    let next_destination = wallet_address(&api, wallet_name).await?;
    let err = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &next_destination.address,
        5_000,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![selected.outpoint.clone()],
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await
    .expect_err("expected confirmed-only coin control to reject unconfirmed selected UTXO");

    let msg = err.to_string();
    assert!(
        msg.contains("not confirmed"),
        "expected error to mention not confirmed, got: {}",
        msg
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_send_psbt_with_coin_control_spends_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
    let requested = confirmed
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for coin control send");

    let destination = wallet_address(&api, wallet_name).await?;
    let published = send_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        10_000,
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
        "expected published txid to be present"
    );

    api.sync(wallet_name).await?;
    let utxos_after_send = wallet_utxos(&api, wallet_name).await?;
    assert!(
        !utxos_after_send.iter().any(|u| u.outpoint == requested.0),
        "expected requested outpoint {} to be spent after coin-control send",
        requested.0
    );

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected published coin-control transaction in tx list");
    assert!(
        sent_tx.confirmed,
        "expected coin-control send to confirm after mining"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_psbt_with_coin_control_rejects_invalid_outpoint() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;

    let destination = wallet_address(&api, wallet_name).await?;

    let err = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec!["invalid_outpoint".to_string()],
            exclude_outpoints: Vec::new(),
            confirmed_only: false,
            selection_mode: None,
        },
    )
    .await
    .expect_err("expected invalid outpoint to fail");

    assert!(matches!(err, wallet_api::WalletApiError::InvalidInput(_)));
    assert!(!err.to_string().is_empty());

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_psbt_with_coin_control_rejects_conflicting_rules() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
    let outpoint = confirmed[0].0.clone();

    let destination = wallet_address(&api, wallet_name).await?;

    let err = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![outpoint.clone()],
            exclude_outpoints: vec![outpoint.clone()],
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await
    .expect_err("expected conflicting include/exclude to fail");

    assert!(!err.to_string().is_empty());

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_psbt_with_coin_control_rejects_insufficient_selected_inputs(
) -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
    let requested = confirmed[0].0.clone();

    let destination = wallet_address(&api, wallet_name).await?;
    let err = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        500_000,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![requested],
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await
    .expect_err("expected insufficient selected inputs to fail");

    let msg = err.to_string();
    assert!(
        msg.contains("insufficient") || msg.contains("funds") || msg.contains("build"),
        "expected error to mention insufficient funds/build failure, got: {}",
        msg
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_send_psbt_with_coin_control_uses_all_requested_utxos() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let mut confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
    confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

    let requested: Vec<String> = confirmed
        .iter()
        .take(2)
        .map(|(outpoint, _)| outpoint.clone())
        .collect();

    let destination = wallet_address(&api, wallet_name).await?;
    let published = send_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        150_000,
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
        "expected published txid to be present"
    );

    api.sync(wallet_name).await?;
    let utxos_after_send = wallet_utxos(&api, wallet_name).await?;
    for outpoint in &requested {
        assert!(
            !utxos_after_send.iter().any(|u| u.outpoint == *outpoint),
            "expected requested outpoint {} to be spent after multi-input coin-control send",
            outpoint
        );
    }

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected published multi-input coin-control transaction in tx list");
    assert!(
        sent_tx.confirmed,
        "expected multi-input coin-control send to confirm after mining"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_coin_control_psbt_input_output_consistency() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
    let requested = confirmed[0].0.clone();

    let destination = wallet_address(&api, wallet_name).await?;
    let psbt = create_psbt_with_coin_control(
        &api,
        wallet_name,
        &destination.address,
        10_000,
        1,
        false,
        WalletCoinControlDto {
            include_outpoints: vec![requested.clone()],
            exclude_outpoints: Vec::new(),
            confirmed_only: true,
            selection_mode: None,
        },
    )
    .await?;

    assert_eq!(psbt.input_count, 1);
    assert_eq!(psbt.selected_inputs.len(), 1);
    assert_eq!(psbt.recipient_count, 1);
    assert!(psbt.output_count >= 1);

    assert!(
        psbt.replacement.is_none(),
        "normal coin-control PSBT should not include replacement metadata"
    );

    Ok(())
}
