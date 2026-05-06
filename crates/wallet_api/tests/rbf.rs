mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;

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
        let refill_addr = api.address(wallet_name).await?;
        let refill_addr = parse_regtest_address(&refill_addr)?;
        env.fund_sats(&refill_addr, 100_000)?;
        env.mine(1)?;
        api.sync(wallet_name).await?;
        balance_before = api.balance(wallet_name).await?;
    }

    // Create a self-send we can replace.
    let destination = api.address(wallet_name).await?;
    let original = api
        .send_psbt(wallet_name, &destination, 10_000, 1, true, false)
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

    api.sync(wallet_name).await?;

    let txs = api.txs(wallet_name).await?;
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
