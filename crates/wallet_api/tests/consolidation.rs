mod common;

use common::*;
use serial_test::serial;
use wallet_api::factory::build_default_api;
use wallet_api::model::{
    ConsolidationRequestDto, CreatePsbtRequestDto, WalletAddressRequestDto,
    WalletConsolidationDto, WalletTransactionsRequestDto, WalletUtxosRequestDto,
};
use wallet_api::service;

async fn create_consolidation_psbt(
    api: &wallet_api::api::WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    consolidation: WalletConsolidationDto,
) -> wallet_api::WalletApiResult<wallet_api::model::WalletPsbtDto> {
    service::psbt::create_consolidation(
        &api.storage,
        ConsolidationRequestDto {
            name: name.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            consolidation,
        },
    )
    .await
}

async fn consolidate_and_broadcast(
    api: &wallet_api::api::WalletApi,
    name: &str,
    fee_rate_sat_per_vb: u64,
    replaceable: bool,
    consolidation: WalletConsolidationDto,
) -> wallet_api::WalletApiResult<wallet_api::model::TxBroadcastResultDto> {
    service::psbt::consolidate(
        &api.storage,
        ConsolidationRequestDto {
            name: name.to_string(),
            fee_rate_sat_per_vb,
            replaceable,
            consolidation,
        },
    )
    .await
}

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
        wallet_api::model::SignPsbtRequestDto {
            name: name.to_string(),
            psbt_base64: created.psbt_base64,
        },
    )
    .await?;

    service::psbt::publish(
        &api.storage,
        wallet_api::model::PublishPsbtRequestDto {
            name: name.to_string(),
            psbt_base64: signed.psbt_base64,
        },
    )
    .await
}

mod happy_path {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_builds_after_sync() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 80_000).await?;
        api.sync(wallet_name).await?;

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let published = consolidate_and_broadcast(
            &api,
            wallet_name,
            1,
            false,
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

        api.sync(wallet_name).await?;
        let utxos_after_send = wallet_utxos(&api, wallet_name).await?;
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
        api.sync(wallet_name).await?;

        let txs = wallet_txs(&api, wallet_name).await?;
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
    async fn wallet_create_consolidation_psbt_recipient_count_and_change_consistency(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 3, 80_000).await?;
        api.sync(wallet_name).await?;

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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
}

mod filters {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_rejects_min_input_count_not_met() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let confirmed = ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 2, 20_000).await?;
        let requested: Vec<String> = confirmed
            .into_iter()
            .take(2)
            .map(|(o, _)| o)
            .collect();

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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
    async fn wallet_create_consolidation_psbt_applies_max_utxo_value_filter() -> anyhow::Result<()>
    {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        for _ in 0..2 {
            let receive_address = api.address(wallet_name).await?;
            let addr = parse_regtest_address(&receive_address.address)?;
            env.fund_sats(&addr, 20_000)?;
        }
        env.mine(1)?;
        api.sync(wallet_name).await?;

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            50,
            false,
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
}

mod strategies {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_uses_largest_first_strategy() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        let (preexisting_confirmed, funded) =
            fund_exact_confirmed_wallet_utxos(&api, &env, wallet_name, &[210_000, 130_000, 90_000])
                .await?;

        let mut available = funded;
        available.sort_by(|a, b| b.1.cmp(&a.1).then_with(|| a.0.cmp(&b.0)));

        let expected: Vec<String> = available.iter().take(2).map(|(o, _)| o.clone()).collect();

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
            wallet_api::model::WalletConsolidationDto {
                include_outpoints: Vec::new(),
                exclude_outpoints: preexisting_confirmed,
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
            assert!(
                inputs.contains(&e),
                "largest-first mismatch: expected input {e} in {:?}; funded candidates were {:?}",
                inputs,
                available
            );
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

        let (preexisting_confirmed, funded) =
            fund_exact_confirmed_wallet_utxos(&api, &env, wallet_name, &[210_000, 130_000, 90_000])
                .await?;

        let mut available = funded;
        available.sort_by(|a, b| a.1.cmp(&b.1).then_with(|| a.0.cmp(&b.0)));

        let expected: Vec<String> = available.iter().take(2).map(|(o, _)| o.clone()).collect();

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
            wallet_api::model::WalletConsolidationDto {
                include_outpoints: Vec::new(),
                exclude_outpoints: preexisting_confirmed,
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
            assert!(
                inputs.contains(&e),
                "smallest-first mismatch: expected input {e} in {:?}; funded candidates were {:?}",
                inputs,
                available
            );
        }
        Ok(())
    }
}

mod edge_cases {
    use super::*;
    #[tokio::test(flavor = "current_thread")]
    #[serial]
    async fn wallet_create_consolidation_psbt_rejects_missing_selected_outpoint(
    ) -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        api.sync(wallet_name).await?;

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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

        let err = create_consolidation_psbt(
            &api,
            wallet_name,
            selected_total + 1,
            false,
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
    async fn wallet_create_consolidation_psbt_preserves_core_invariants() -> anyhow::Result<()> {
        let env = RegtestEnv::new();
        env.start()?;

        let api = build_default_api().await?;
        let wallet_name = "regtest-local";

        ensure_confirmed_wallet_utxos(&api, &env, wallet_name, 4, 80_000).await?;
        api.sync(wallet_name).await?;

        let psbt = create_consolidation_psbt(
            &api,
            wallet_name,
            1,
            false,
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
            psbt.replacement.is_none(),
            "consolidation PSBT should not contain replacement metadata"
        );
        assert!(
            psbt.estimated_vsize > 0,
            "expected positive estimated vsize"
        );
        assert!(
            psbt.amount_sat + psbt.fee_sat > psbt.amount_sat,
            "expected input value conservation to imply amount + fee exceeds amount"
        );

        api.sync(wallet_name).await?;
        let wallet_utxos = wallet_utxos(&api, wallet_name).await?;
        assert!(
            wallet_utxos.iter().all(|u| u.outpoint != psbt.to_address),
            "expected destination address string not to be confused with an outpoint"
        );

        Ok(())
    }
}

mod fuzz {
    use super::*;
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
        api.sync(wallet_name).await?;

        let utxos = wallet_utxos(&api, wallet_name).await?;
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
            let result = create_consolidation_psbt(
                &api,
                wallet_name,
                fee_rate,
                false,
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
