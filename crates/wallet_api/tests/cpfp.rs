mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    CpfpRequestDto, CreatePsbtRequestDto, PublishPsbtRequestDto, SignPsbtRequestDto,
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

    let signed = sign_psbt(api, name, &created.psbt_base64).await?;
    publish_psbt(api, name, &signed.psbt_base64).await
}

async fn cpfp_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletCpfpPsbtDto> {
    service::psbt::cpfp_psbt(
        &api.storage,
        CpfpRequestDto {
            name: name.to_string(),
            parent_txid: parent_txid.to_string(),
            selected_outpoint: selected_outpoint.to_string(),
            fee_rate_sat_per_vb,
        },
    )
    .await
}

async fn sign_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    psbt_base64: &str,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletSignedPsbtDto> {
    service::psbt::sign(
        &api.storage,
        SignPsbtRequestDto {
            name: name.to_string(),
            psbt_base64: psbt_base64.to_string(),
        },
    )
    .await
}

async fn publish_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    psbt_base64: &str,
) -> wallet_api::WalletApiResult<wallet_api::model::TxBroadcastResultDto> {
    service::psbt::publish(
        &api.storage,
        PublishPsbtRequestDto {
            name: name.to_string(),
            psbt_base64: psbt_base64.to_string(),
        },
    )
    .await
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_cpfp_psbt_builds_for_unconfirmed_parent() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Make sure wallet state is current and we have enough confirmed funds.
    api.sync(wallet_name).await?;
    let balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = wallet_address(&api, wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr.address)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
        // No need to reassign balance_before
    }

    // Create an unconfirmed parent transaction by self-sending.
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

    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to be present in mempool before CPFP"
    );

    api.sync(wallet_name).await?;
    let balance_before_cpfp = api.balance(wallet_name).await?;
    // Select one of the parent's unconfirmed wallet-owned outputs for CPFP.
    let utxos = wallet_utxos(&api, wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one parent output to be available for CPFP");

    let cpfp = cpfp_psbt(&api, wallet_name, &parent.txid, &selected.outpoint, 5).await?;

    assert!(
        !cpfp.psbt_base64.is_empty(),
        "expected CPFP PSBT payload to be present"
    );
    assert!(
        !cpfp.txid.is_empty(),
        "expected CPFP child txid to be present"
    );
    assert_eq!(cpfp.parent_txid, parent.txid);
    assert_ne!(cpfp.txid, cpfp.parent_txid);
    assert_eq!(
        cpfp.selected_outpoint, selected.outpoint,
        "expected CPFP to use the explicitly requested outpoint"
    );
    assert_eq!(cpfp.input_value_sat, selected.value);
    assert!(
        cpfp.fee_sat > 0,
        "expected CPFP fee to be positive, got {}",
        cpfp.fee_sat
    );
    assert!(
        cpfp.child_output_value_sat < cpfp.input_value_sat,
        "expected child output value {} to be less than input value {}",
        cpfp.child_output_value_sat,
        cpfp.input_value_sat
    );
    assert_eq!(
        cpfp.input_value_sat - cpfp.child_output_value_sat,
        cpfp.fee_sat,
        "expected CPFP fee to equal input minus child output value"
    );
    assert_eq!(cpfp.fee_rate_sat_per_vb, 5);
    assert!(cpfp.estimated_vsize > 0, "expected positive virtual size");
    assert!(
        cpfp.replaceable,
        "expected CPFP child transaction to be replaceable"
    );

    // Building the CPFP PSBT should not alter chain state or wallet balance.
    let balance_after = api.balance(wallet_name).await?;
    assert_eq!(
        balance_after, balance_before_cpfp,
        "building a CPFP PSBT should not change wallet balance"
    );
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to remain in mempool after CPFP PSBT creation"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_cpfp_psbt_uses_requested_parent_outpoint() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;
    let balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = wallet_address(&api, wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr.address)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
    }

    // Create an unconfirmed self-send parent transaction that should produce at least
    // an external recipient output and an internal change output.
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

    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to be present in mempool before CPFP"
    );

    api.sync(wallet_name).await?;
    let utxos = wallet_utxos(&api, wallet_name).await?;
    let parent_outputs: Vec<_> = utxos
        .iter()
        .filter(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .collect();

    assert!(
        parent_outputs.len() >= 2,
        "expected at least two wallet-owned parent outputs for explicit CPFP selection"
    );

    // Pick a specific output deterministically and verify it is the one used.
    let requested = parent_outputs
        .iter()
        .max_by_key(|u| &u.outpoint)
        .expect("expected a requested parent output")
        .to_owned();

    let cpfp = cpfp_psbt(&api, wallet_name, &parent.txid, &requested.outpoint, 5).await?;

    assert_eq!(cpfp.parent_txid, parent.txid);
    assert_eq!(
        cpfp.selected_outpoint, requested.outpoint,
        "expected CPFP to honor the explicitly requested outpoint"
    );
    assert_eq!(cpfp.input_value_sat, requested.value);
    assert!(
        cpfp.fee_sat > 0,
        "expected CPFP fee to be positive, got {}",
        cpfp.fee_sat
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_cpfp_child_broadcasts_and_confirms() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Ensure the wallet is synced and has enough confirmed funds.
    api.sync(wallet_name).await?;
    let initial_balance = api.balance(wallet_name).await?;

    if initial_balance < 50_000 {
        let refill_addr = wallet_address(&api, wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr.address)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
    }

    // Create a low-fee unconfirmed parent transaction.
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

    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to be present in mempool before CPFP"
    );

    api.sync(wallet_name).await?;
    let utxos = wallet_utxos(&api, wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one parent output to be available for CPFP");

    // Build, sign, and publish the CPFP child transaction.
    let cpfp = cpfp_psbt(&api, wallet_name, &parent.txid, &selected.outpoint, 5).await?;
    assert!(
        !cpfp.psbt_base64.is_empty(),
        "expected CPFP PSBT payload to be present"
    );
    assert!(
        !cpfp.txid.is_empty(),
        "expected CPFP child txid to be present"
    );
    assert_eq!(
        cpfp.selected_outpoint, selected.outpoint,
        "expected CPFP to use the explicitly requested outpoint"
    );
    assert_eq!(cpfp.input_value_sat, selected.value);

    let signed = sign_psbt(&api, wallet_name, &cpfp.psbt_base64).await?;
    assert_eq!(signed.signing_status, "finalized");
    let published = publish_psbt(&api, wallet_name, &signed.psbt_base64).await?;

    assert_eq!(
        published.txid, cpfp.txid,
        "expected published CPFP child txid to match planned child txid"
    );

    let child_txid = parse_txid(&published.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to remain in mempool after CPFP child broadcast"
    );
    assert!(
        mempool_contains(&child_txid)?,
        "expected CPFP child transaction to be present in mempool after broadcast"
    );

    api.sync(wallet_name).await?;

    let txs = wallet_txs(&api, wallet_name).await?;
    let parent_tx = txs
        .iter()
        .find(|tx| tx.txid == parent.txid)
        .expect("expected parent transaction to appear in tx list after CPFP broadcast");
    let child_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected CPFP child transaction to appear in tx list after broadcast");

    assert!(
        !parent_tx.confirmed,
        "expected parent transaction to remain unconfirmed before mining"
    );
    assert!(
        !child_tx.confirmed,
        "expected CPFP child transaction to remain unconfirmed before mining"
    );

    // Mine a block and verify both transactions confirm.
    env.mine(1)?;
    api.sync(wallet_name).await?;

    assert!(
        !mempool_contains(&parent_txid)?,
        "expected parent transaction to leave mempool after confirmation"
    );
    assert!(
        !mempool_contains(&child_txid)?,
        "expected CPFP child transaction to leave mempool after confirmation"
    );

    let txs = wallet_txs(&api, wallet_name).await?;
    let parent_tx = txs
        .iter()
        .find(|tx| tx.txid == parent.txid)
        .expect("expected parent transaction after mining");
    let child_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected CPFP child transaction after mining");

    assert!(
        parent_tx.confirmed,
        "expected parent transaction to be confirmed after mining"
    );
    assert!(
        child_tx.confirmed,
        "expected CPFP child transaction to be confirmed after mining"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_cpfp_psbt_fails_for_confirmed_parent() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Ensure we have enough confirmed funds.
    api.sync(wallet_name).await?;
    let balance = api.balance(wallet_name).await?;

    if balance < 50_000 {
        let refill_addr = wallet_address(&api, wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr.address)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
    }

    // Create a parent transaction and then confirm it.
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
        .expect("expected at least one parent output to be available before parent confirmation");

    env.mine(1)?;
    api.sync(wallet_name).await?;

    // Confirm the transaction is no longer in mempool.
    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        !mempool_contains(&parent_txid)?,
        "expected confirmed parent transaction to be absent from mempool"
    );

    // CPFP should fail for a confirmed parent.
    let err = cpfp_psbt(&api, wallet_name, &parent.txid, &selected.outpoint, 5)
        .await
        .expect_err("expected CPFP PSBT creation to fail for confirmed parent");

    let msg = err.to_string();
    assert!(
        !msg.is_empty(),
        "expected CPFP confirmed-parent failure to produce an error message"
    );

    Ok(())
}

#[tokio::test(flavor = "current_thread")]
#[serial]
async fn wallet_cpfp_psbt_fails_when_parent_not_found() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync(wallet_name).await?;

    // Use a deterministic fake txid that should not exist on regtest.
    let missing_parent_txid = "0000000000000000000000000000000000000000000000000000000000000001";

    let err = cpfp_psbt(
        &api,
        wallet_name,
        missing_parent_txid,
        "0000000000000000000000000000000000000000000000000000000000000001:0",
        5,
    )
    .await
    .expect_err("expected CPFP PSBT creation to fail for missing parent transaction");

    let msg = err.to_string();
    assert!(
        !msg.is_empty(),
        "expected CPFP missing-parent failure to produce an error message"
    );

    Ok(())
}
