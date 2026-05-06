mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;

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
    let addr = api.address(wallet_name).await?;

    // 5. Fund the address (50_000 sats)
    let btc_addr = parse_regtest_address(&addr)?;
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
    let destination = api.address(wallet_name).await?;
    let published = api
        .send_psbt(wallet_name, &destination, 10_000, 1, false, false)
        .await?;

    assert!(
        !published.txid.is_empty(),
        "expected broadcast txid to be present"
    );

    // Sync to observe the unconfirmed transaction and its outputs.
    api.sync(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
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
            outpoint_txid(&u.outpoint) == published.txid && u.value > 0 && u.keychain == "internal"
        }),
        "expected internal change output for self-send transaction"
    );

    // Confirm the transaction, then re-sync and verify final accounting.
    env.mine(1)?;
    api.sync(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
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
