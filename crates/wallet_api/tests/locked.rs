mod common;

use common::*;
use serial_test::file_serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    WalletAddressRequestDto, WalletCoinControlDto, WalletInputSelectionModeDto,
    WalletLockedUtxosRequestDto, WalletLockUtxosRequestDto, WalletUnlockUtxosRequestDto,
    WalletUtxosRequestDto,
};
use wallet_api::service::{addresses, inspect, locked_utxos};

#[tokio::test]
#[file_serial]
async fn lock_utxo_can_be_listed() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "locked-list").await?;
    let wallet_name = wallet_name.as_str();

    ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;

    let utxos = inspect::utxos(
        api.storage(),
        WalletUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    let outpoint = utxos
        .first()
        .expect("wallet should have at least one utxo")
        .outpoint
        .clone();

    let locked = locked_utxos::lock_utxos(
        api.storage(),
        WalletLockUtxosRequestDto {
            name: wallet_name.to_string(),
            outpoints: vec![outpoint.clone()],
            reason: Some("manual coin freeze".to_string()),
        },
    )
    .await?;

    assert_eq!(locked.wallet_name, wallet_name);
    assert_eq!(locked.locked_utxos.len(), 1);
    assert_eq!(locked.locked_utxos[0].outpoint, outpoint);
    assert_eq!(
        locked.locked_utxos[0].reason.as_deref(),
        Some("manual coin freeze")
    );

    let listed = locked_utxos::list_locked_utxos(
        api.storage(),
        WalletLockedUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    assert_eq!(listed.locked_utxos.len(), 1);
    assert_eq!(listed.locked_utxos[0].outpoint, outpoint);

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn unlock_utxo_removes_lock() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "locked-unlock").await?;
    let wallet_name = wallet_name.as_str();

    ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;

    let utxos = inspect::utxos(
        api.storage(),
        WalletUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    let outpoint = utxos
        .first()
        .expect("wallet should have at least one utxo")
        .outpoint
        .clone();

    locked_utxos::lock_utxos(
        api.storage(),
        WalletLockUtxosRequestDto {
            name: wallet_name.to_string(),
            outpoints: vec![outpoint.clone()],
            reason: None,
        },
    )
    .await?;

    let unlocked = locked_utxos::unlock_utxos(
        api.storage(),
        WalletUnlockUtxosRequestDto {
            name: wallet_name.to_string(),
            outpoints: vec![outpoint.clone()],
        },
    )
    .await?;

    assert!(unlocked.locked_utxos.is_empty());

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn inspect_utxos_marks_locked_coin() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "locked-inspect").await?;
    let wallet_name = wallet_name.as_str();

    ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;

    let initial_utxos = inspect::utxos(
        api.storage(),
        WalletUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    let outpoint = initial_utxos
        .first()
        .expect("wallet should have utxo")
        .outpoint
        .clone();

    locked_utxos::lock_utxos(
        api.storage(),
        WalletLockUtxosRequestDto {
            name: wallet_name.to_string(),
            outpoints: vec![outpoint.clone()],
            reason: Some("reserved for cpfp".to_string()),
        },
    )
    .await?;

    let utxos = inspect::utxos(
        api.storage(),
        WalletUtxosRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    let locked = utxos
        .into_iter()
        .find(|utxo| utxo.outpoint == outpoint)
        .expect("locked utxo should exist");

    assert!(locked.is_locked);
    assert_eq!(locked.lock_reason.as_deref(), Some("reserved for cpfp"));
    assert!(locked.locked_at.is_some());

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn create_psbt_rejects_explicitly_selected_locked_utxo() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "locked-select").await?;
    let wallet_name = wallet_name.as_str();

    let (_, funded) =
        fund_exact_confirmed_wallet_utxos(&api, &env, wallet_name, &[120_000]).await?;
    let locked_outpoint = funded
        .first()
        .expect("expected one newly funded utxo")
        .0
        .clone();

    locked_utxos::lock_utxos(
        api.storage(),
        WalletLockUtxosRequestDto {
            name: wallet_name.to_string(),
            outpoints: vec![locked_outpoint.clone()],
            reason: Some("reserved for future spend".to_string()),
        },
    )
    .await?;

    let destination = addresses::address(
        api.storage(),
        WalletAddressRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    let result = api
        .create_psbt_with_coin_control(
            wallet_name,
            &destination.address,
            50_000,
            1,
            false,
            WalletCoinControlDto {
                include_outpoints: vec![locked_outpoint.clone()],
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: Some(WalletInputSelectionModeDto::StrictManual),
            },
        )
        .await;

    let err = result.expect_err("locked selected input should fail");
    let message = err.to_string();
    assert!(
        message.contains("locked") && message.contains(&locked_outpoint),
        "expected locked-utxo error mentioning {} but got: {}",
        locked_outpoint,
        message
    );

    Ok(())
}

#[tokio::test]
#[file_serial]
async fn send_max_skips_locked_utxos_in_automatic_selection() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = clone_wallet_for_test(&api, "regtest-local", "locked-send-max").await?;
    let wallet_name = wallet_name.as_str();

    let (_, funded) =
        fund_exact_confirmed_wallet_utxos(&api, &env, wallet_name, &[90_000, 180_000]).await?;
    let locked_outpoint = funded
        .iter()
        .max_by_key(|(_, value)| *value)
        .expect("expected funded utxos")
        .0
        .clone();

    locked_utxos::lock_utxos(
        api.storage(),
        WalletLockUtxosRequestDto {
            name: wallet_name.to_string(),
            outpoints: vec![locked_outpoint.clone()],
            reason: Some("do not spend automatically".to_string()),
        },
    )
    .await?;

    let destination = addresses::address(
        api.storage(),
        WalletAddressRequestDto {
            name: wallet_name.to_string(),
        },
    )
    .await?;

    let psbt = api
        .create_send_max_psbt_with_coin_control(
            wallet_name,
            &destination.address,
            1,
            false,
            WalletCoinControlDto {
                include_outpoints: Vec::new(),
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: Some(WalletInputSelectionModeDto::AutomaticOnly),
            },
        )
        .await?;

    let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
    assert!(
        !inputs.contains(&locked_outpoint),
        "automatic selection should skip locked outpoint {}; inputs={:?}",
        locked_outpoint,
        inputs
    );

    Ok(())
}
