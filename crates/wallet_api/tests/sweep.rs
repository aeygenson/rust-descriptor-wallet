mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_sweep_psbt_uses_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    let requested = confirmed
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for sweep coin control");

    let destination = api.address(wallet_name).await?;
    let psbt = api
        .create_sweep_psbt(
            wallet_name,
            &destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
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
        "expected sweep PSBT to use the requested UTXO"
    );
    assert!(
        psbt.change_amount_sat.is_none(),
        "expected strict sweep to avoid change"
    );
    assert_eq!(
        psbt.output_count, 1,
        "expected a single recipient output in strict sweep"
    );
    assert_eq!(
        psbt.selected_inputs,
        vec![requested.0.clone()],
        "expected sweep selected_inputs to contain only the requested outpoint"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_sweep_psbt_rejects_missing_selected_outpoint() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;

    let destination = api.address(wallet_name).await?;
    let err = api
        .create_sweep_psbt(
            wallet_name,
            &destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints: vec![
                    "0000000000000000000000000000000000000000000000000000000000000001:0"
                        .to_string(),
                ],
                exclude_outpoints: Vec::new(),
                confirmed_only: false,
                selection_mode: None,
            },
        )
        .await
        .expect_err("expected sweep PSBT creation to fail for missing selected outpoint");

    let msg = err.to_string();
    assert!(
        msg.contains("not found") || msg.contains("outpoint"),
        "expected missing-outpoint error, got: {}",
        msg
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_sweep_psbt_rejects_conflicting_rules() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
    let outpoint = confirmed[0].0.clone();

    let destination = api.address(wallet_name).await?;
    let err = api
        .create_sweep_psbt(
            wallet_name,
            &destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints: vec![outpoint.clone()],
                exclude_outpoints: vec![outpoint.clone()],
                confirmed_only: true,
                selection_mode: None,
            },
        )
        .await
        .expect_err("expected sweep include/exclude conflict to fail");

    let msg = err.to_string();
    assert!(
        msg.contains("conflict") || msg.contains("include") || msg.contains("exclude"),
        "expected conflict error, got: {}",
        msg
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_sweep_psbt_rejects_unconfirmed_selected_utxo_when_confirmed_only(
) -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;

    let destination = api.address(wallet_name).await?;
    let parent = api
        .send_psbt(wallet_name, &destination, 10_000, 1, false, false)
        .await?;
    assert!(
        !parent.txid.is_empty(),
        "expected parent txid to be present"
    );

    api.sync(wallet_name).await?;
    let utxos = api.utxos(wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one unconfirmed wallet-owned output");

    let next_destination = api.address(wallet_name).await?;
    let err = api
        .create_sweep_psbt(
            wallet_name,
            &next_destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints: vec![selected.outpoint.clone()],
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: None,
            },
        )
        .await
        .expect_err("expected confirmed-only sweep to reject unconfirmed selected UTXO");

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
async fn wallet_create_sweep_psbt_rejects_insufficient_after_fees() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    let requested = confirmed
        .into_iter()
        .min_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for strict sweep test");

    let destination = api.address(wallet_name).await?;
    let err = api
        .create_sweep_psbt(
            wallet_name,
            &destination,
            requested.1 + 1,
            false,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints: vec![requested.0.clone()],
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: None,
            },
        )
        .await
        .expect_err("expected strict sweep to fail when fees consume the selected input");

    let msg = err.to_string();
    assert!(
        msg.contains("too small")
            || msg.contains("strict mode violation")
            || msg.contains("additional inputs are not allowed"),
        "expected strict sweep error, got: {}",
        msg
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_sweep_psbt_sweeps_requested_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
    let requested = confirmed
        .into_iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected a confirmed UTXO for sweep send");

    let destination = api.address(wallet_name).await?;
    let published = api
        .sweep_and_broadcast(
            wallet_name,
            &destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints: vec![requested.0.clone()],
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: None,
            },
        )
        .await?;

    assert!(!published.txid.is_empty(), "expected published sweep txid");

    api.sync(wallet_name).await?;
    let utxos_after_send = api.utxos(wallet_name).await?;
    assert!(
        !utxos_after_send.iter().any(|u| u.outpoint == requested.0),
        "expected requested outpoint {} to be fully swept",
        requested.0
    );
    assert!(
        !utxos_after_send
            .iter()
            .any(|u| outpoint_txid(&u.outpoint) == published.txid && u.keychain == "internal"),
        "expected no internal change output for strict sweep"
    );

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected published sweep transaction in tx list");
    assert!(
        sent_tx.confirmed,
        "expected sweep transaction to confirm after mining"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_create_sweep_psbt_uses_all_requested_utxos() -> anyhow::Result<()> {
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

    let destination = api.address(wallet_name).await?;
    let psbt = api
        .create_sweep_psbt(
            wallet_name,
            &destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
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
            "expected sweep PSBT inputs {:?} to contain requested outpoint {}",
            inputs,
            outpoint
        );
    }
    assert!(
        psbt.change_amount_sat.is_none(),
        "expected strict multi-input sweep to avoid change"
    );
    assert_eq!(
        psbt.output_count, 1,
        "expected a single recipient output in strict multi-input sweep"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_sweep_psbt_sweeps_all_requested_utxos() -> anyhow::Result<()> {
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

    let destination = api.address(wallet_name).await?;
    let published = api
        .sweep_and_broadcast(
            wallet_name,
            &destination,
            1,
            false,
            wallet_api::model::WalletCoinControlDto {
                include_outpoints: requested.clone(),
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: None,
            },
        )
        .await?;

    assert!(
        !published.txid.is_empty(),
        "expected published multi-input sweep txid"
    );

    api.sync(wallet_name).await?;
    let utxos_after_send = api.utxos(wallet_name).await?;
    for outpoint in &requested {
        assert!(
            !utxos_after_send.iter().any(|u| u.outpoint == *outpoint),
            "expected requested outpoint {} to be fully swept in multi-input sweep flow",
            outpoint
        );
    }
    assert!(
        !utxos_after_send
            .iter()
            .any(|u| outpoint_txid(&u.outpoint) == published.txid && u.keychain == "internal"),
        "expected no internal change output for strict multi-input sweep"
    );

    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected published multi-input sweep transaction in tx list");
    assert!(
        sent_tx.confirmed,
        "expected multi-input sweep transaction to confirm after mining"
    );

    Ok(())
}
