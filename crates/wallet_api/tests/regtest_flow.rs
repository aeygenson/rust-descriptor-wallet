mod regtest_suite {
use serial_test::serial;
use test_support::{mempool_contains, RegtestEnv};
use wallet_api::factory::build_default_api;

fn parse_regtest_address(
    s: &str,
) -> anyhow::Result<bitcoin::Address<bitcoin::address::NetworkChecked>> {
    Ok(s
        .parse::<bitcoin::Address<bitcoin::address::NetworkUnchecked>>()?
        .require_network(bitcoin::Network::Regtest)?)
}

fn parse_txid(s: &str) -> anyhow::Result<bitcoin::Txid> {
    Ok(s.parse()?)
}

fn outpoint_txid(outpoint: &str) -> &str {
    outpoint.split(':').next().unwrap_or("")
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_receives_funds_after_sync() -> anyhow::Result<()> {
    // 1. Start regtest environment
    let env = RegtestEnv::new();
    env.start()?;

    // 2. Build API
    let api = build_default_api().await?;

    let wallet_name = "regtest-local";

    // 3. Initial sync
    api.sync_wallet(wallet_name).await?;

    // 4. Get a new address
    let addr = api.address(wallet_name).await?;

    // 5. Fund the address (50_000 sats)
    let btc_addr = parse_regtest_address(&addr)?;
    env.fund_sats(&btc_addr, 50_000)?;

    // 6. Mine a block to confirm
    env.mine(1)?;

    // 7. Sync again
    api.sync_wallet(wallet_name).await?;

    // 8. Check balance
    let balance = api.balance(wallet_name).await?;

    // 9. Assert balance increased
    assert!(balance >= 50_000, "expected at least 50_000 sats, got {}", balance);

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_self_send_creates_change() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Make sure wallet state is up to date before building the spend.
    api.sync_wallet(wallet_name).await?;

    let balance_before = api.balance(wallet_name).await?;

    // Generate a fresh wallet address and send funds to ourselves.
    let destination = api.address(wallet_name).await?;
    let published = api
        .send_psbt(wallet_name, &destination, 10_000, 1)
        .await?;

    assert!(!published.txid.is_empty(), "expected broadcast txid to be present");

    // Sync to observe the unconfirmed transaction and its outputs.
    api.sync_wallet(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected self-send transaction to appear in tx list");

    let fee = sent_tx.fee.expect("expected self-send transaction fee to be present");
    assert!(fee > 0, "expected positive fee, got {}", fee);
    assert_eq!(sent_tx.net_value, -(fee as i64));
    assert!(!sent_tx.confirmed, "expected self-send transaction to be unconfirmed before mining");

    let utxos = api.utxos(wallet_name).await?;
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
                && u.value > 0
                && u.keychain == "internal"
        }),
        "expected internal change output for self-send transaction"
    );

    // Confirm the transaction, then re-sync and verify final accounting.
    env.mine(1)?;
    api.sync_wallet(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
    let sent_tx = txs
        .iter()
        .find(|tx| tx.txid == published.txid)
        .expect("expected self-send transaction after mining");
    assert!(sent_tx.confirmed, "expected self-send transaction to be confirmed after mining");

    let balance = api.balance(wallet_name).await?;
    assert_eq!(
        balance,
        balance_before - fee,
        "expected balance to decrease only by fee"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_bump_fee_replaces_unconfirmed_transaction() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Make sure wallet state is up to date and the wallet has enough funds.
    api.sync_wallet(wallet_name).await?;
    let mut balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = api.address(wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;
        balance_before = api.balance(wallet_name).await?;
    }

    // Create a self-send we can replace.
    let destination = api.address(wallet_name).await?;
    let original = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
    assert!(
        !original.txid.is_empty(),
        "expected original broadcast txid to be present"
    );
    let original_txid = parse_txid(&original.txid)?;
    assert!(
        mempool_contains(&original_txid)?,
        "expected original transaction to be present in mempool before bump"
    );

    api.sync_wallet(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
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
    let replacement = api.bump_fee(wallet_name, &original.txid, 5).await?;
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

    api.sync_wallet(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
    let replacement_tx = txs
        .iter()
        .find(|tx| tx.txid == replacement.txid)
        .or_else(|| {
            txs.iter().find(|tx| {
                tx.txid != original.txid
                    && !tx.confirmed
                    && tx.direction == "sent"
            })
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
    api.sync_wallet(wallet_name).await?;
    assert!(
        !mempool_contains(&replacement_txid_rpc)?,
        "expected replacement transaction to leave mempool after confirmation"
    );

    let txs = api.txs(wallet_name).await?;
    let replacement_tx = txs
        .iter()
        .find(|tx| tx.txid == replacement_txid)
        .expect("expected replacement transaction after mining");

    assert!(
        replacement_tx.confirmed,
        "expected replacement transaction to be confirmed after mining"
    );

    let utxos = api.utxos(wallet_name).await?;
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == replacement_txid && u.value == 10_000 && u.keychain == "external"
        }),
        "expected replacement recipient output with value 10000 sats"
    );
    assert!(
        utxos.iter().any(|u| {
            outpoint_txid(&u.outpoint) == replacement_txid && u.value > 0 && u.keychain == "internal"
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

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_cpfp_psbt_builds_for_unconfirmed_parent() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Make sure wallet state is current and we have enough confirmed funds.
    api.sync_wallet(wallet_name).await?;
    let balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = api.address(wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;
        // No need to reassign balance_before
    }

    // Create an unconfirmed parent transaction by self-sending.
    let destination = api.address(wallet_name).await?;
    let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
    assert!(!parent.txid.is_empty(), "expected parent txid to be present");

    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to be present in mempool before CPFP"
    );

    api.sync_wallet(wallet_name).await?;
    let balance_before_cpfp = api.balance(wallet_name).await?;
    // Select one of the parent's unconfirmed wallet-owned outputs for CPFP.
    let utxos = api.utxos(wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one parent output to be available for CPFP");

    let cpfp = api
        .cpfp_psbt(wallet_name, &parent.txid, &selected.outpoint, 5)
        .await?;

    assert!(
        !cpfp.psbt_base64.is_empty(),
        "expected CPFP PSBT payload to be present"
    );
    assert!(!cpfp.txid.is_empty(), "expected CPFP child txid to be present");
    assert_eq!(cpfp.parent_txid, parent.txid);
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
    assert!(cpfp.replaceable, "expected CPFP child transaction to be replaceable");

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

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_cpfp_psbt_uses_requested_parent_outpoint() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync_wallet(wallet_name).await?;
    let balance_before = api.balance(wallet_name).await?;

    if balance_before < 50_000 {
        let refill_addr = api.address(wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;
    }

    // Create an unconfirmed self-send parent transaction that should produce at least
    // an external recipient output and an internal change output.
    let destination = api.address(wallet_name).await?;
    let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
    assert!(!parent.txid.is_empty(), "expected parent txid to be present");

    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to be present in mempool before CPFP"
    );

    api.sync_wallet(wallet_name).await?;
    let utxos = api.utxos(wallet_name).await?;
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

    let cpfp = api
        .cpfp_psbt(wallet_name, &parent.txid, &requested.outpoint, 5)
        .await?;

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

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_cpfp_child_broadcasts_and_confirms() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Ensure the wallet is synced and has enough confirmed funds.
    api.sync_wallet(wallet_name).await?;
    let initial_balance = api.balance(wallet_name).await?;

    if initial_balance < 50_000 {
        let refill_addr = api.address(wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;
    }

    // Create a low-fee unconfirmed parent transaction.
    let destination = api.address(wallet_name).await?;
    let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
    assert!(!parent.txid.is_empty(), "expected parent txid to be present");

    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        mempool_contains(&parent_txid)?,
        "expected parent transaction to be present in mempool before CPFP"
    );

    api.sync_wallet(wallet_name).await?;
    let utxos = api.utxos(wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one parent output to be available for CPFP");

    // Build, sign, and publish the CPFP child transaction.
    let cpfp = api
        .cpfp_psbt(wallet_name, &parent.txid, &selected.outpoint, 5)
        .await?;
    assert!(
        !cpfp.psbt_base64.is_empty(),
        "expected CPFP PSBT payload to be present"
    );
    assert!(!cpfp.txid.is_empty(), "expected CPFP child txid to be present");
    assert_eq!(
        cpfp.selected_outpoint, selected.outpoint,
        "expected CPFP to use the explicitly requested outpoint"
    );
    assert_eq!(cpfp.input_value_sat, selected.value);

    let signed = api.sign_psbt(wallet_name, &cpfp.psbt_base64).await?;
    let published = api.publish_psbt(wallet_name, &signed.psbt_base64).await?;

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

    api.sync_wallet(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
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
    api.sync_wallet(wallet_name).await?;

    assert!(
        !mempool_contains(&parent_txid)?,
        "expected parent transaction to leave mempool after confirmation"
    );
    assert!(
        !mempool_contains(&child_txid)?,
        "expected CPFP child transaction to leave mempool after confirmation"
    );

    let txs = api.txs(wallet_name).await?;
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

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_cpfp_psbt_fails_for_confirmed_parent() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    // Ensure we have enough confirmed funds.
    api.sync_wallet(wallet_name).await?;
    let balance = api.balance(wallet_name).await?;

    if balance < 50_000 {
        let refill_addr = api.address(wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;
    }

    // Create a parent transaction and then confirm it.
    let destination = api.address(wallet_name).await?;
    let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
    assert!(!parent.txid.is_empty(), "expected parent txid to be present");
    api.sync_wallet(wallet_name).await?;
    let utxos = api.utxos(wallet_name).await?;
    let selected = utxos
        .iter()
        .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
        .expect("expected at least one parent output to be available before parent confirmation");

    env.mine(1)?;
    api.sync_wallet(wallet_name).await?;

    // Confirm the transaction is no longer in mempool.
    let parent_txid = parse_txid(&parent.txid)?;
    assert!(
        !mempool_contains(&parent_txid)?,
        "expected confirmed parent transaction to be absent from mempool"
    );

    // CPFP should fail for a confirmed parent.
    let err = api
        .cpfp_psbt(wallet_name, &parent.txid, &selected.outpoint, 5)
        .await
        .expect_err("expected CPFP PSBT creation to fail for confirmed parent");

    let msg = err.to_string();
    assert!(
        !msg.is_empty(),
        "expected CPFP confirmed-parent failure to produce an error message"
    );

    Ok(())
}

#[tokio::test(flavor = "multi_thread")]
#[serial]
async fn wallet_cpfp_psbt_fails_when_parent_not_found() -> anyhow::Result<()> {
    let env = RegtestEnv::new();
    env.start()?;

    let api = build_default_api().await?;
    let wallet_name = "regtest-local";

    api.sync_wallet(wallet_name).await?;

    // Use a deterministic fake txid that should not exist on regtest.
    let missing_parent_txid =
        "0000000000000000000000000000000000000000000000000000000000000001";

    let err = api
        .cpfp_psbt(
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
}}