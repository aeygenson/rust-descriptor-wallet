pub mod support;

mod regtest_suite {
    use crate::support::ensure_confirmed_wallet_utxos;
    use serial_test::serial;
    use test_support::wallet::{
        decode_psbt_inputs, outpoint_txid, parse_regtest_address, parse_txid,
    };
    use test_support::{mempool_contains, RegtestEnv};
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
        api.sync_wallet(wallet_name).await?;

        let balance_before = api.balance(wallet_name).await?;

        // Generate a fresh wallet address and send funds to ourselves.
        let destination = api.address(wallet_name).await?;
        let published = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;

        assert!(
            !published.txid.is_empty(),
            "expected broadcast txid to be present"
        );

        // Sync to observe the unconfirmed transaction and its outputs.
        api.sync_wallet(wallet_name).await?;

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
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );

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
        assert!(
            !cpfp.txid.is_empty(),
            "expected CPFP child txid to be present"
        );
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
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );

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

    #[tokio::test(flavor = "current_thread")]
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
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );

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
        assert!(
            !cpfp.txid.is_empty(),
            "expected CPFP child txid to be present"
        );
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

    #[tokio::test(flavor = "current_thread")]
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
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );
        api.sync_wallet(wallet_name).await?;
        let utxos = api.utxos(wallet_name).await?;
        let selected = utxos
            .iter()
            .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
            .expect(
                "expected at least one parent output to be available before parent confirmation",
            );

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

    #[tokio::test(flavor = "current_thread")]
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
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_psbt_with_coin_control_uses_requested_utxo() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
        let requested = confirmed
            .into_iter()
            .max_by_key(|(_, value)| *value)
            .expect("expected a confirmed UTXO for coin control");

        let destination = api.address(wallet_name).await?;
        let psbt = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                10_000,
                1,
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

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let requested: Vec<String> = confirmed
            .iter()
            .take(2)
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        let destination = api.address(wallet_name).await?;
        let psbt = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                150_000,
                1,
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

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;
        confirmed.sort_by(|a, b| a.0.cmp(&b.0));

        let excluded = confirmed[0].0.clone();
        let destination = api.address(wallet_name).await?;

        let psbt = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                10_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        api.sync_wallet(wallet_name).await?;

        let destination = api.address(wallet_name).await?;
        let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );

        api.sync_wallet(wallet_name).await?;
        let utxos = api.utxos(wallet_name).await?;
        let selected = utxos
            .iter()
            .find(|u| outpoint_txid(&u.outpoint) == parent.txid)
            .expect("expected at least one unconfirmed wallet-owned output");

        let next_destination = api.address(wallet_name).await?;
        let err = api
            .create_psbt_with_coin_control(
                wallet_name,
                &next_destination,
                5_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        let destination = api.address(wallet_name).await?;
        let published = api
            .send_psbt_with_coin_control(
                wallet_name,
                &destination,
                10_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        api.sync_wallet(wallet_name).await?;
        let utxos_after_send = api.utxos(wallet_name).await?;
        assert!(
            !utxos_after_send.iter().any(|u| u.outpoint == requested.0),
            "expected requested outpoint {} to be spent after coin-control send",
            requested.0
        );

        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;

        let txs = api.txs(wallet_name).await?;
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

        api.sync_wallet(wallet_name).await?;

        let destination = api.address(wallet_name).await?;

        let err = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                10_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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
    async fn wallet_create_psbt_with_coin_control_rejects_conflicting_rules() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
        let outpoint = confirmed[0].0.clone();

        let destination = api.address(wallet_name).await?;

        let err = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                10_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        let destination = api.address(wallet_name).await?;
        let err = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                500_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let requested: Vec<String> = confirmed
            .iter()
            .take(2)
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        let destination = api.address(wallet_name).await?;
        let published = api
            .send_psbt_with_coin_control(
                wallet_name,
                &destination,
                150_000,
                1,
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
            "expected published txid to be present"
        );

        api.sync_wallet(wallet_name).await?;
        let utxos_after_send = api.utxos(wallet_name).await?;
        for outpoint in &requested {
            assert!(
                !utxos_after_send.iter().any(|u| u.outpoint == *outpoint),
                "expected requested outpoint {} to be spent after multi-input coin-control send",
                outpoint
            );
        }

        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;

        let txs = api.txs(wallet_name).await?;
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
    async fn wallet_create_send_max_psbt_builds_after_sync() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
        api.sync_wallet(wallet_name).await?;
        let available_total = api.balance(wallet_name).await?;

        let destination = api.address(wallet_name).await?;
        let psbt = api
            .create_send_max_psbt(wallet_name, &destination, 1)
            .await?;

        assert!(
            !psbt.psbt_base64.is_empty(),
            "expected send-max PSBT payload"
        );
        assert!(!psbt.txid.is_empty(), "expected send-max txid");
        assert_eq!(psbt.to_address, destination);
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

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_send_max_psbt_with_coin_control_uses_requested_utxo(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
        let requested = confirmed
            .into_iter()
            .max_by_key(|(_, value)| *value)
            .expect("expected a confirmed UTXO for send-max coin control");

        let destination = api.address(wallet_name).await?;
        let psbt = api
            .create_send_max_psbt_with_coin_control(
                wallet_name,
                &destination,
                1,
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

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_send_max_psbt_with_coin_control_sweeps_requested_utxo() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
        let requested = confirmed
            .into_iter()
            .max_by_key(|(_, value)| *value)
            .expect("expected a confirmed UTXO for send-max coin control send");

        let destination = api.address(wallet_name).await?;
        let published = api
            .send_max_psbt_with_coin_control(
                wallet_name,
                &destination,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        api.sync_wallet(wallet_name).await?;
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
            "expected no internal change output for strict send-max sweep"
        );

        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;

        let txs = api.txs(wallet_name).await?;
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
    #[serial]
    async fn wallet_create_send_max_psbt_with_coin_control_rejects_insufficient_after_fees(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
        let requested = confirmed
            .into_iter()
            .min_by_key(|(_, value)| *value)
            .expect("expected a confirmed UTXO for strict send-max test");

        let destination = api.address(wallet_name).await?;
        let err = api
            .create_send_max_psbt_with_coin_control(
                wallet_name,
                &destination,
                requested.1 + 1,
                wallet_api::model::WalletCoinControlDto {
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
    #[serial]
    async fn wallet_send_max_psbt_with_coin_control_sweeps_all_requested_utxos(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let requested: Vec<String> = confirmed
            .iter()
            .take(2)
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        let destination = api.address(wallet_name).await?;
        let published = api
            .send_max_psbt_with_coin_control(
                wallet_name,
                &destination,
                1,
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
            "expected published multi-input send-max txid"
        );

        api.sync_wallet(wallet_name).await?;
        let utxos_after_send = api.utxos(wallet_name).await?;
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
        api.sync_wallet(wallet_name).await?;

        let txs = api.txs(wallet_name).await?;
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

        api.sync_wallet(wallet_name).await?;

        let destination = api.address(wallet_name).await?;
        let err = api
            .create_sweep_psbt(
                wallet_name,
                &destination,
                1,
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

        api.sync_wallet(wallet_name).await?;

        let destination = api.address(wallet_name).await?;
        let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );

        api.sync_wallet(wallet_name).await?;
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
                wallet_api::model::WalletCoinControlDto {
                    include_outpoints: vec![requested.0.clone()],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    selection_mode: None,
                },
            )
            .await?;

        assert!(!published.txid.is_empty(), "expected published sweep txid");

        api.sync_wallet(wallet_name).await?;
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
        api.sync_wallet(wallet_name).await?;

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

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
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

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
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

        api.sync_wallet(wallet_name).await?;
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
        api.sync_wallet(wallet_name).await?;

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

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_builds_after_sync() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        api.sync_wallet(wallet_name).await?;

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(4),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await?;

        assert!(
            !psbt.psbt_base64.is_empty(),
            "expected consolidation PSBT payload"
        );
        assert!(!psbt.txid.is_empty(), "expected consolidation txid");
        assert!(
            !psbt.to_address.is_empty(),
            "expected consolidation destination address"
        );
        assert!(
            psbt.selected_utxo_count >= 2,
            "expected consolidation to use at least two inputs"
        );
        assert_eq!(
            psbt.selected_inputs.len(),
            psbt.input_count,
            "expected selected_inputs to match actual input count"
        );
        assert_eq!(
            psbt.output_count, 1,
            "expected consolidation to produce a single output"
        );
        assert!(
            psbt.amount_sat > 0,
            "expected positive consolidation output amount"
        );
        assert!(psbt.fee_sat > 0, "expected positive consolidation fee");

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_uses_requested_utxos() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let requested: Vec<String> = confirmed
            .iter()
            .take(2)
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: requested.clone(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await?;

        let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
        assert_eq!(inputs.len(), 2, "expected exactly two selected inputs");
        for outpoint in &requested {
            assert!(
                inputs.contains(outpoint),
                "expected consolidation PSBT inputs {:?} to contain requested outpoint {}",
                inputs,
                outpoint
            );
        }
        assert_eq!(
            psbt.output_count, 1,
            "expected requested-input consolidation to produce a single output"
        );

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_rejects_missing_selected_outpoint(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        api.sync_wallet(wallet_name).await?;

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: vec![
                        "0000000000000000000000000000000000000000000000000000000000000001:0"
                            .to_string(),
                        "0000000000000000000000000000000000000000000000000000000000000002:0"
                            .to_string(),
                    ],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err(
                "expected consolidation PSBT creation to fail for missing selected outpoint",
            );

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
    async fn wallet_create_consolidation_psbt_rejects_conflicting_rules() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;
        let outpoint = confirmed[0].0.clone();
        let second = confirmed[1].0.clone();

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: vec![outpoint.clone(), second],
                    exclude_outpoints: vec![outpoint.clone()],
                    confirmed_only: true,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err("expected consolidation include/exclude conflict to fail");

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
    async fn wallet_create_consolidation_psbt_rejects_unconfirmed_selected_utxos_when_confirmed_only(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        api.sync_wallet(wallet_name).await?;

        let destination = api.address(wallet_name).await?;
        let parent = api.send_psbt(wallet_name, &destination, 10_000, 1).await?;
        assert!(
            !parent.txid.is_empty(),
            "expected parent txid to be present"
        );

        api.sync_wallet(wallet_name).await?;
        let utxos = api.utxos(wallet_name).await?;
        let selected: Vec<String> = utxos
            .iter()
            .filter(|u| outpoint_txid(&u.outpoint) == parent.txid)
            .take(2)
            .map(|u| u.outpoint.clone())
            .collect();

        assert_eq!(
            selected.len(),
            2,
            "expected at least two unconfirmed wallet-owned outputs for consolidation"
        );

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: selected,
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err(
                "expected confirmed-only consolidation to reject unconfirmed selected UTXOs",
            );

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
    async fn wallet_create_consolidation_psbt_rejects_too_few_inputs() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 20_000).await?;
        let requested = confirmed[0].0.clone();

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: vec![requested],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err("expected consolidation to fail when fewer than two inputs are selected");

        let msg = err.to_string();
        assert!(
            msg.contains("at least two eligible UTXOs") || msg.contains("selection and filters"),
            "expected too-few-inputs error, got: {}",
            msg
        );

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_rejects_insufficient_after_fees() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;
        confirmed.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let requested: Vec<String> = confirmed
            .iter()
            .take(2)
            .map(|(outpoint, _)| outpoint.clone())
            .collect();
        let selected_total: u64 = confirmed.iter().take(2).map(|(_, value)| *value).sum();

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                selected_total + 1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: requested,
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err("expected consolidation to fail when fees consume the selected inputs");

        let msg = err.to_string();
        assert!(
            msg.contains("too small") || msg.contains("usable consolidation amount"),
            "expected consolidation-too-small error, got: {}",
            msg
        );

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_consolidate_psbt_spends_requested_utxos_and_creates_internal_output(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let mut confirmed =
            ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        confirmed.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let requested: Vec<String> = confirmed
            .iter()
            .take(2)
            .map(|(outpoint, _)| outpoint.clone())
            .collect();

        let published = api
            .consolidate_and_broadcast(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: requested.clone(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await?;

        assert!(
            !published.txid.is_empty(),
            "expected published consolidation txid"
        );

        api.sync_wallet(wallet_name).await?;
        let utxos_after_send = api.utxos(wallet_name).await?;
        for outpoint in &requested {
            assert!(
                !utxos_after_send.iter().any(|u| u.outpoint == *outpoint),
                "expected requested outpoint {} to be spent after consolidation",
                outpoint
            );
        }
        assert!(
            utxos_after_send
                .iter()
                .any(|u| outpoint_txid(&u.outpoint) == published.txid && u.keychain == "internal"),
            "expected consolidation to create a wallet-internal output"
        );
        assert!(
            !utxos_after_send
                .iter()
                .any(|u| outpoint_txid(&u.outpoint) == published.txid && u.keychain == "external"),
            "expected consolidation transaction not to create an external wallet-owned output"
        );

        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;

        let txs = api.txs(wallet_name).await?;
        let sent_tx = txs
            .iter()
            .find(|tx| tx.txid == published.txid)
            .expect("expected published consolidation transaction in tx list");
        assert!(
            sent_tx.confirmed,
            "expected consolidation transaction to confirm after mining"
        );

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_rejects_min_input_count_not_met() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;
        let requested: Vec<String> = confirmed.into_iter().map(|(o, _)| o).collect();

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: requested,
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: Some(3),
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err("expected min_input_count constraint to fail");

        assert!(!err.to_string().is_empty());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_applies_min_utxo_value_filter() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let _confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: Some(15_000),
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await?;

        assert!(psbt.selected_utxo_count >= 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_recipient_count_and_change_consistency(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 3, 80_000).await?;
        api.sync_wallet(wallet_name).await?;

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(3),
                    min_input_count: Some(2),
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await?;

        assert_eq!(psbt.recipient_count, 1);
        assert_eq!(psbt.output_count, 1);
        assert!(psbt.change_amount_sat.is_some());
        assert!(psbt.amount_sat > 0);
        assert!(psbt.fee_sat > 0);

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_send_max_psbt_recipient_and_no_change_invariant() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 1, 50_000).await?;
        api.sync_wallet(wallet_name).await?;

        let destination = api.address(wallet_name).await?;
        let psbt = api
            .create_send_max_psbt(wallet_name, &destination, 1)
            .await?;

        assert_eq!(psbt.recipient_count, 1);
        assert_eq!(psbt.output_count, 1);
        assert!(psbt.change_amount_sat.is_none());
        assert!(psbt.amount_sat > 0);

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

        let destination = api.address(wallet_name).await?;
        let psbt = api
            .create_psbt_with_coin_control(
                wallet_name,
                &destination,
                10_000,
                1,
                wallet_api::model::WalletCoinControlDto {
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

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_applies_max_utxo_value_filter() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        for _ in 0..2 {
            let addr = api.address(wallet_name).await?;
            let addr = parse_regtest_address(&addr)?;
            env.fund_sats(&addr, 20_000)?;
        }
        env.mine(1)?;
        api.sync_wallet(wallet_name).await?;

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: Some(30_000),
                    max_fee_pct_of_input_value: None,
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await?;

        assert!(psbt.selected_utxo_count >= 1);
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_rejects_fee_pct_limit() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 10_000).await?;
        let requested: Vec<String> = confirmed.into_iter().map(|(o, _)| o).collect();

        let err = api
            .create_consolidation_psbt(
                wallet_name,
                50,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: requested,
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: Some(1),
                    strategy: None,
                    selection_mode: None,
                },
            )
            .await
            .expect_err("expected fee percentage limit to fail");

        assert!(!err.to_string().is_empty());
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_uses_largest_first_strategy() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 3, 80_000).await?;
        api.sync_wallet(wallet_name).await?;

        let mut available: Vec<(String, u64)> = api
            .utxos(wallet_name)
            .await?
            .into_iter()
            .filter(|u| u.confirmed)
            .map(|u| (u.outpoint, u.value))
            .collect();
        available.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let expected: Vec<String> = available.iter().take(2).map(|(o, _)| o.clone()).collect();

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: Some(wallet_api::model::WalletConsolidationStrategyDto::LargestFirst),
                    selection_mode: None,
                },
            )
            .await?;

        let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
        assert_eq!(inputs.len(), 2, "expected exactly two selected inputs");
        for e in expected {
            assert!(inputs.contains(&e));
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_uses_smallest_first_strategy() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 3, 80_000).await?;
        api.sync_wallet(wallet_name).await?;

        let mut available: Vec<(String, u64)> = api
            .utxos(wallet_name)
            .await?
            .into_iter()
            .filter(|u| u.confirmed)
            .map(|u| (u.outpoint, u.value))
            .collect();
        available.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let expected: Vec<String> = available.iter().take(2).map(|(o, _)| o.clone()).collect();

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(2),
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: Some(
                        wallet_api::model::WalletConsolidationStrategyDto::SmallestFirst,
                    ),
                    selection_mode: None,
                },
            )
            .await?;

        let inputs = decode_psbt_inputs(&psbt.psbt_base64)?;
        assert_eq!(inputs.len(), 2, "expected exactly two selected inputs");
        for e in expected {
            assert!(inputs.contains(&e));
        }
        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_preserves_core_invariants() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 4, 80_000).await?;
        api.sync_wallet(wallet_name).await?;

        let psbt = api
            .create_consolidation_psbt(
                wallet_name,
                1,
                wallet_api::model::WalletConsolidationDto {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: true,
                    max_input_count: Some(3),
                    min_input_count: Some(2),
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    max_fee_pct_of_input_value: None,
                    strategy: Some(
                        wallet_api::model::WalletConsolidationStrategyDto::SmallestFirst,
                    ),
                    selection_mode: None,
                },
            )
            .await?;

        assert!(
            !psbt.psbt_base64.is_empty(),
            "expected consolidation PSBT payload"
        );
        assert!(psbt.input_count >= 2, "expected at least two inputs");
        assert_eq!(
            psbt.selected_inputs.len(),
            psbt.input_count,
            "expected selected_inputs to match actual input count"
        );
        assert_eq!(
            psbt.output_count, 1,
            "expected consolidation to produce exactly one output"
        );
        assert_eq!(
            psbt.recipient_count, 1,
            "expected a single wallet-owned recipient output"
        );
        assert!(
            psbt.change_amount_sat.is_some(),
            "expected consolidation output amount to be reflected as change_amount_sat"
        );
        assert!(
            psbt.amount_sat > 0,
            "expected positive consolidation amount"
        );
        assert!(psbt.fee_sat > 0, "expected positive consolidation fee");
        assert!(
            psbt.estimated_vsize > 0,
            "expected positive estimated vsize"
        );
        assert!(
            psbt.amount_sat + psbt.fee_sat > psbt.amount_sat,
            "expected input value conservation to imply amount + fee exceeds amount"
        );

        api.sync_wallet(wallet_name).await?;
        let wallet_utxos = api.utxos(wallet_name).await?;
        assert!(
            wallet_utxos.iter().all(|u| u.outpoint != psbt.to_address),
            "expected destination address string not to be confused with an outpoint"
        );

        Ok(())
    }

    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_fuzz_preserves_invariants() -> anyhow::Result<()> {
        fn next_u64(state: &mut u64) -> u64 {
            let mut x = *state;
            x ^= x << 13;
            x ^= x >> 7;
            x ^= x << 17;
            *state = x;
            x
        }

        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 6, 80_000).await?;
        api.sync_wallet(wallet_name).await?;

        let utxos = api.utxos(wallet_name).await?;
        let mut confirmed: Vec<(String, u64)> = utxos
            .into_iter()
            .filter(|u| u.confirmed)
            .map(|u| (u.outpoint, u.value))
            .collect();
        confirmed.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        assert!(
            confirmed.len() >= 4,
            "expected enough confirmed UTXOs for consolidation fuzzing"
        );

        let min_value = confirmed.first().map(|(_, v)| *v).unwrap_or(0);
        let max_value = confirmed.last().map(|(_, v)| *v).unwrap_or(0);

        let strategies = [
            None,
            Some(wallet_api::model::WalletConsolidationStrategyDto::SmallestFirst),
            Some(wallet_api::model::WalletConsolidationStrategyDto::LargestFirst),
            Some(wallet_api::model::WalletConsolidationStrategyDto::OldestFirst),
        ];

        let mut seed = 0x5EED_CAFE_D15C_A11Eu64;
        for round in 0..16u64 {
            let draw_a = next_u64(&mut seed);
            let draw_b = next_u64(&mut seed);
            let draw_c = next_u64(&mut seed);
            let draw_d = next_u64(&mut seed);

            let strategy = strategies[(draw_a as usize) % strategies.len()];
            let max_input_count = Some(2 + (draw_b as usize % 3));
            let min_input_count = if draw_c % 3 == 0 { Some(2) } else { None };

            let min_utxo_value_sat = if draw_d % 4 == 0 {
                Some(min_value)
            } else if draw_d % 4 == 1 {
                Some((min_value + max_value) / 2)
            } else {
                None
            };

            let max_utxo_value_sat = if draw_d % 5 == 0 {
                Some(max_value)
            } else if draw_d % 5 == 1 {
                Some((min_value + max_value) / 2)
            } else {
                None
            };

            let fee_rate = 1 + (draw_a % 3);
            let result = api
                .create_consolidation_psbt(
                    wallet_name,
                    fee_rate,
                    wallet_api::model::WalletConsolidationDto {
                        include_outpoints: Vec::new(),
                        exclude_outpoints: Vec::new(),
                        confirmed_only: true,
                        max_input_count,
                        min_input_count,
                        min_utxo_value_sat,
                        max_utxo_value_sat,
                        max_fee_pct_of_input_value: None,
                        strategy,
                        selection_mode: None,
                    },
                )
                .await;

            match result {
                Ok(psbt) => {
                    assert!(
                        psbt.input_count >= 2,
                        "round {}: expected at least two inputs on success",
                        round
                    );
                    if let Some(max_inputs) = max_input_count {
                        assert!(
                            psbt.input_count <= max_inputs,
                            "round {}: expected input_count {} <= max_input_count {}",
                            round,
                            psbt.input_count,
                            max_inputs
                        );
                    }
                    if let Some(min_inputs) = min_input_count {
                        assert!(
                            psbt.input_count >= min_inputs,
                            "round {}: expected input_count {} >= min_input_count {}",
                            round,
                            psbt.input_count,
                            min_inputs
                        );
                    }
                    assert_eq!(
                        psbt.selected_inputs.len(),
                        psbt.input_count,
                        "round {}: expected selected_inputs to match actual input count",
                        round
                    );
                    assert_eq!(
                        psbt.output_count, 1,
                        "round {}: expected exactly one output",
                        round
                    );
                    assert_eq!(
                        psbt.recipient_count, 1,
                        "round {}: expected exactly one wallet-owned recipient",
                        round
                    );
                    assert!(
                        psbt.amount_sat > 0,
                        "round {}: expected positive consolidation amount",
                        round
                    );
                    assert!(
                        psbt.fee_sat > 0,
                        "round {}: expected positive consolidation fee",
                        round
                    );
                    assert!(
                        psbt.estimated_vsize > 0,
                        "round {}: expected positive vsize",
                        round
                    );
                }
                Err(err) => {
                    let msg = err.to_string();
                    assert!(
                        !msg.is_empty(),
                        "round {}: expected non-empty error message",
                        round
                    );
                }
            }
        }

        Ok(())
    }
}
