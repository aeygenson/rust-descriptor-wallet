use crate::model::{
    BumpFeeRequestDto, ConsolidationRequestDto, CpfpRequestDto, CreatePsbtRequestDto,
    PublishPsbtRequestDto, SendMaxRequestDto, SignPsbtRequestDto, SweepRequestDto,
    TxBroadcastResultDto, WalletCpfpPsbtDto, WalletPsbtDto, WalletSignedPsbtDto,
};
use crate::WalletApiResult;

use wallet_core::types::{AmountSat, FeeRateSatPerVb, PsbtBase64, WalletOutPoint};
use wallet_core::{WalletCore, WalletService};
use wallet_storage::WalletStorage;
use wallet_sync::{WalletSyncError, WalletSyncService};

use super::wallet::load_wallet_config;

use tokio::runtime::Handle;
use tokio::task;
use tracing::{debug, info};

async fn spawn_wallet_blocking<T, E>(
    f: impl FnOnce() -> Result<T, E> + Send + 'static,
) -> WalletApiResult<T>
where
    T: Send + 'static,
    E: Into<crate::WalletApiError> + Send + 'static,
{
    task::spawn_blocking(f)
        .await
        .map_err(|e| {
            crate::WalletApiError::InvalidInput(format!("blocking wallet task failed: {e}"))
        })?
        .map_err(Into::into)
}

fn log_publish_error(name: &str, error: &WalletSyncError) {
    match error {
        WalletSyncError::BroadcastTransport(msg) => {
            tracing::error!(
                "api psbt: publish transport_failed name={} error={}",
                name,
                msg
            );
        }
        WalletSyncError::BroadcastMempoolConflict(msg) => {
            tracing::error!(
                "api psbt: publish mempool_conflict name={} error={}",
                name,
                msg
            );
        }
        WalletSyncError::BroadcastAlreadyConfirmed(msg) => {
            tracing::error!(
                "api psbt: publish already_confirmed name={} error={}",
                name,
                msg
            );
        }
        WalletSyncError::BroadcastMissingInputs(msg) => {
            tracing::error!(
                "api psbt: publish missing_inputs name={} error={}",
                name,
                msg
            );
        }
        WalletSyncError::BroadcastInsufficientFee(msg) => {
            tracing::error!(
                "api psbt: publish insufficient_fee name={} error={}",
                name,
                msg
            );
        }
        WalletSyncError::PsbtNotFinalized => {
            tracing::error!("api psbt: publish not_finalized name={}", name,);
        }
        _ => {
            tracing::error!("api psbt: publish failed name={} error={}", name, error);
        }
    }
}

async fn load_locked_outpoints(
    storage: &WalletStorage,
    name: &str,
) -> WalletApiResult<Vec<WalletOutPoint>> {
    storage
        .list_locked_utxos(name)
        .await?
        .into_iter()
        .map(|record| {
            WalletOutPoint::parse(&record.outpoint).map_err(|e| {
                crate::WalletApiError::InvalidInput(format!(
                    "invalid locked utxo outpoint '{}' for wallet '{}': {}",
                    record.outpoint, name, e
                ))
            })
        })
        .collect()
}

fn apply_locked_outpoints_to_coin_control(
    coin_control: Option<wallet_core::model::WalletCoinControlInfo>,
    locked_outpoints: &[WalletOutPoint],
) -> WalletApiResult<Option<wallet_core::model::WalletCoinControlInfo>> {
    let Some(mut coin_control) = coin_control else {
        if locked_outpoints.is_empty() {
            return Ok(None);
        }

        let core = WalletCore::new();
        return Ok(Some(wallet_core::model::WalletCoinControlInfo {
            selection: wallet_core::model::WalletInputSelectionConfig {
                exclude_outpoints: core.merge_locked_into_excluded(&[], locked_outpoints),
                ..Default::default()
            },
        }));
    };

    let core = WalletCore::new();
    core.ensure_outpoints_unlocked(&coin_control.selection.include_outpoints, locked_outpoints)?;
    coin_control.selection.exclude_outpoints = core.merge_locked_into_excluded(
        &coin_control.selection.exclude_outpoints,
        locked_outpoints,
    );

    Ok(Some(coin_control))
}

fn apply_locked_outpoints_to_consolidation(
    mut consolidation: wallet_core::model::WalletConsolidationInfo,
    locked_outpoints: &[WalletOutPoint],
) -> WalletApiResult<wallet_core::model::WalletConsolidationInfo> {
    let core = WalletCore::new();
    core.ensure_outpoints_unlocked(&consolidation.selection.include_outpoints, locked_outpoints)?;
    consolidation.selection.exclude_outpoints = core.merge_locked_into_excluded(
        &consolidation.selection.exclude_outpoints,
        locked_outpoints,
    );

    Ok(consolidation)
}

fn ensure_selected_outpoint_unlocked(
    selected_outpoint: &WalletOutPoint,
    locked_outpoints: &[WalletOutPoint],
) -> WalletApiResult<()> {
    WalletCore::new().ensure_outpoints_unlocked(
        std::slice::from_ref(selected_outpoint),
        locked_outpoints,
    )?;

    Ok(())
}

/// Create an unsigned PSBT for a send flow.
///
/// This is the first API orchestration step in the PSBT transaction pipeline.
pub async fn create(
    storage: &WalletStorage,
    request: CreatePsbtRequestDto,
) -> WalletApiResult<WalletPsbtDto> {
    let CreatePsbtRequestDto {
        name,
        to_address,
        amount_sat,
        fee_rate_sat_per_vb,
        replaceable,
        coin_control,
    } = request;
    debug!(
        "api psbt: create start name={} to={} amount_sat={} fee_rate_sat_per_vb={} replaceable={} has_coin_control={}",
        name,
        to_address,
        amount_sat,
        fee_rate_sat_per_vb,
        replaceable,
        coin_control.is_some()
    );

    let config = load_wallet_config(storage, &name).await?;
    let amount_sat = AmountSat::new(amount_sat)?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;
    let locked_outpoints = load_locked_outpoints(storage, &name).await?;
    let coin_control = coin_control.map(|dto| dto.try_into_core()).transpose()?;
    let coin_control = apply_locked_outpoints_to_coin_control(coin_control, &locked_outpoints)?;
    let locked_count = locked_outpoints.len();
    let name_for_error = name.clone();

    let to_address = to_address.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_psbt_with_coin_control(
                config.network,
                &to_address,
                amount_sat,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create failed name={} to={} amount_sat={} fee_rate_sat_per_vb={} replaceable={} error={}",
                    name_for_error,
                    to_address,
                    amount_sat.as_u64(),
                    fee_rate_sat_per_vb.as_u64(),
                    replaceable,
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} locked_exclusions={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.selected_inputs.len(),
        locked_count,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.as_str().len()
    );

    Ok(psbt.into())
}


/// Create an unsigned PSBT for a send-max flow.
///
/// This sends the maximum available amount (after fees) to the destination.
pub async fn create_send_max(
    storage: &WalletStorage,
    request: SendMaxRequestDto,
) -> WalletApiResult<WalletPsbtDto> {
    let SendMaxRequestDto {
        name,
        to_address,
        fee_rate_sat_per_vb,
        replaceable,
        coin_control,
    } = request;
    debug!(
        "api psbt: create_send_max start name={} to={} fee_rate_sat_per_vb={} replaceable={}",
        name, to_address, fee_rate_sat_per_vb, replaceable
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;
    let locked_outpoints = load_locked_outpoints(storage, &name).await?;
    let coin_control = coin_control.map(|dto| dto.try_into_core()).transpose()?;
    let coin_control = apply_locked_outpoints_to_coin_control(coin_control, &locked_outpoints)?;
    let locked_count = locked_outpoints.len();
    let name_for_error = name.clone();

    let to_address = to_address.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_send_max_psbt_with_coin_control(
                config.network,
                &to_address,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_send_max failed name={} to={} fee_rate_sat_per_vb={} replaceable={} error={}",
                    name_for_error,
                    to_address,
                    fee_rate_sat_per_vb.as_u64(),
                    replaceable,
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_send_max success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} locked_exclusions={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.selected_inputs.len(),
        locked_count,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.as_str().len()
    );

    Ok(psbt.into())
}


/// Create an unsigned PSBT for a sweep flow using explicit coin control.
///
/// Sweep is implemented as strict send-max with an explicit include set.
pub async fn create_sweep(
    storage: &WalletStorage,
    request: SweepRequestDto,
) -> WalletApiResult<WalletPsbtDto> {
    let SweepRequestDto {
        name,
        to_address,
        fee_rate_sat_per_vb,
        replaceable,
        coin_control,
    } = request;
    debug!(
        "api psbt: create_sweep start name={} to={} fee_rate_sat_per_vb={} replaceable={} include_outpoints={} exclude_outpoints={} confirmed_only={} selection_mode={:?}",
        name,
        to_address,
        fee_rate_sat_per_vb,
        replaceable,
        coin_control.include_outpoints.len(),
        coin_control.exclude_outpoints.len(),
        coin_control.confirmed_only,
        coin_control.selection_mode,
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;
    let locked_outpoints = load_locked_outpoints(storage, &name).await?;
    let to_address = to_address.to_string();
    let coin_control = coin_control.try_into_core()?;
    let coin_control =
        apply_locked_outpoints_to_coin_control(Some(coin_control), &locked_outpoints)?
            .expect("sweep coin control should remain present");
    let locked_count = locked_outpoints.len();
    let name_for_error = name.clone();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_sweep_psbt(
                config.network,
                &to_address,
                fee_rate_sat_per_vb,
                replaceable,
                coin_control,
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_sweep failed name={} to={} fee_rate_sat_per_vb={} replaceable={} error={}",
                    name_for_error,
                    to_address,
                    fee_rate_sat_per_vb.as_u64(),
                    replaceable,
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_sweep success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} locked_exclusions={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.selected_inputs.len(),
        locked_count,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.as_str().len()
    );

    Ok(psbt.into())
}

/// Create an unsigned PSBT for a wallet-internal consolidation flow.
///
/// Consolidation spends multiple wallet UTXOs into a smaller number of
/// wallet-owned outputs, usually one internal output, to reduce fragmentation.
pub async fn create_consolidation(
    storage: &WalletStorage,
    request: ConsolidationRequestDto,
) -> WalletApiResult<WalletPsbtDto> {
    let ConsolidationRequestDto {
        name,
        fee_rate_sat_per_vb,
        replaceable,
        consolidation,
    } = request;
    debug!(
        "api psbt: create_consolidation start name={} fee_rate_sat_per_vb={} replaceable={} include_outpoints={} exclude_outpoints={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?} selection_mode={:?}",
        name,
        fee_rate_sat_per_vb,
        replaceable,
        consolidation.include_outpoints.len(),
        consolidation.exclude_outpoints.len(),
        consolidation.confirmed_only,
        consolidation.max_input_count,
        consolidation.min_input_count,
        consolidation.min_utxo_value_sat,
        consolidation.max_utxo_value_sat,
        consolidation.max_fee_pct_of_input_value,
        consolidation.strategy,
        consolidation.selection_mode,
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;
    let locked_outpoints = load_locked_outpoints(storage, &name).await?;
    let consolidation = consolidation.try_into_core()?;
    let consolidation = apply_locked_outpoints_to_consolidation(consolidation, &locked_outpoints)?;
    let locked_count = locked_outpoints.len();
    let name_for_error = name.clone();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_consolidation_psbt(fee_rate_sat_per_vb, replaceable, Some(consolidation))
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_consolidation failed name={} fee_rate_sat_per_vb={} replaceable={} error={}",
                    name_for_error,
                    fee_rate_sat_per_vb.as_u64(),
                    replaceable,
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_consolidation success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} locked_exclusions={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.selected_inputs.len(),
        locked_count,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.as_str().len()
    );

    Ok(psbt.into())
}

pub async fn sign(
    storage: &WalletStorage,
    request: SignPsbtRequestDto,
) -> WalletApiResult<WalletSignedPsbtDto> {
    let SignPsbtRequestDto { name, psbt_base64 } = request;

    debug!("api psbt: sign start name={}", name);

    let config = load_wallet_config(storage, &name).await?;
    let psbt_base64 = PsbtBase64::from(psbt_base64);
    let name_for_error = name.clone();

    let signed = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet.sign_psbt(&psbt_base64).map_err(|e| {
            tracing::error!("api psbt: sign failed name={} error={}", name_for_error, e);
            e
        })
    })
    .await?;

    info!(
        "api psbt: sign status={} name={} modified={} finalized={} txid={} psbt_len={}",
        signed.signing_status(),
        name,
        signed.modified,
        signed.finalized,
        signed.txid,
        signed.psbt_base64.as_str().len()
    );

    Ok(signed.into())
}

pub async fn publish(
    storage: &WalletStorage,
    request: PublishPsbtRequestDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    let PublishPsbtRequestDto { name, psbt_base64 } = request;

    debug!("api psbt: publish start name={}", name);

    let config = load_wallet_config(storage, &name).await?;
    let psbt_base64 = PsbtBase64::from(psbt_base64);
    let name_for_error = name.clone();

    let published = spawn_wallet_blocking(move || -> WalletApiResult<TxBroadcastResultDto> {
        let wallet = WalletService::load_or_create(&config)?;
        let sync_service = WalletSyncService::new();

        let finalized = wallet.finalize_psbt_for_broadcast(&psbt_base64)?;

        sync_service
            .broadcast_tx_hex(&config, finalized.tx_hex.as_str())
            .map_err(|e| {
                log_publish_error(&name_for_error, &e);
                e
            })?;

        Ok(TxBroadcastResultDto {
            txid: finalized.txid.to_string(),
            replaceable: Some(finalized.replaceable),
        })
    })
    .await?;

    info!(
        "api psbt: publish success name={} txid={} replaceable={:?}",
        name, published.txid, published.replaceable,
    );

    Ok(published)
}

/// Create, sign, and publish a sweep transaction.
pub async fn sweep(
    storage: &WalletStorage,
    request: SweepRequestDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    let name = request.name.clone();
    let created = create_sweep(storage, request).await?;

    let signed = sign(
        storage,
        SignPsbtRequestDto {
            name: name.clone(),
            psbt_base64: created.psbt_base64,
        },
    )
    .await?;

    if !signed.finalized {
        return Err(crate::WalletApiError::SendNotFinalized);
    }

    publish(
        storage,
        PublishPsbtRequestDto {
            name,
            psbt_base64: signed.psbt_base64,
        },
    )
    .await
}

/// Create, sign, and publish a wallet-internal consolidation transaction.
pub async fn consolidate(
    storage: &WalletStorage,
    request: ConsolidationRequestDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    let name = request.name.clone();
    let created = create_consolidation(storage, request).await?;

    let signed = sign(
        storage,
        SignPsbtRequestDto {
            name: name.clone(),
            psbt_base64: created.psbt_base64,
        },
    )
    .await?;

    if !signed.finalized {
        return Err(crate::WalletApiError::SendNotFinalized);
    }

    publish(
        storage,
        PublishPsbtRequestDto {
            name,
            psbt_base64: signed.psbt_base64,
        },
    )
    .await
}

/// Build a replacement PSBT for an existing unconfirmed RBF transaction.
///
/// This mirrors `create(...)` but targets an existing replaceable transaction
/// identified by `txid` and requests a higher fee rate.
pub async fn bump_fee_psbt(
    storage: &WalletStorage,
    request: BumpFeeRequestDto,
) -> WalletApiResult<WalletPsbtDto> {
    let BumpFeeRequestDto {
        name,
        txid,
        fee_rate_sat_per_vb,
    } = request;
    debug!(
        "api psbt: bump_fee_psbt start name={} txid={} fee_rate_sat_per_vb={}",
        name, txid, fee_rate_sat_per_vb
    );
    info!(
        "api psbt: bump_fee_psbt request received name={} txid={} requested_fee_rate_sat_per_vb={}",
        name, txid, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;
    info!(
        "api psbt: bump_fee_psbt fee rate validated name={} txid={} requested_fee_rate_sat_per_vb={}",
        name,
        txid,
        fee_rate_sat_per_vb.as_u64()
    );

    let txid_for_log = txid.clone();
    let name_for_error = name.clone();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;
        tracing::info!(
            "api psbt: bump_fee_psbt calling wallet_core txid={} requested_fee_rate_sat_per_vb={}",
            txid,
            fee_rate_sat_per_vb.as_u64()
        );
        wallet
            .bump_fee_psbt(&txid, fee_rate_sat_per_vb)
            .map_err(|e| {
                tracing::error!(
                    "api psbt: bump_fee_psbt failed name={} txid={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    txid,
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: bump_fee_psbt success name={} original_txid={} replacement_txid={} replacement_depth={:?} replacement_chain_len={} requested_fee_rate_sat_per_vb={} result_fee_sat={} result_fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        txid_for_log,
        psbt.txid,
        psbt.replacement.as_ref().map(|r| r.replacement_depth),
        psbt.replacement
            .as_ref()
            .map(|r| r.replacement_chain.len())
            .unwrap_or_default(),
        fee_rate_sat_per_vb.as_u64(),
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.selected_inputs.len(),
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.as_str().len()
    );

    Ok(psbt.into())
}

/// Build a CPFP PSBT for an existing unconfirmed parent transaction.
///
/// This mirrors `bump_fee_psbt(...)`, but instead of replacing the parent,
/// it creates a child transaction that spends an unconfirmed wallet output
/// belonging to the parent transaction.
pub async fn cpfp_psbt(
    storage: &WalletStorage,
    request: CpfpRequestDto,
) -> WalletApiResult<WalletCpfpPsbtDto> {
    let CpfpRequestDto {
        name,
        parent_txid,
        selected_outpoint,
        fee_rate_sat_per_vb,
    } = request;
    debug!(
        "api psbt: cpfp_psbt start name={} parent_txid={} selected_outpoint={} fee_rate_sat_per_vb={}",
        name,
        parent_txid,
        selected_outpoint,
        fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let selected_outpoint_str = selected_outpoint.clone();
    let selected_outpoint = WalletOutPoint::parse(&selected_outpoint).map_err(|e| {
        crate::WalletApiError::InvalidInput(format!(
            "invalid selected_outpoint '{}': {}",
            selected_outpoint_str, e
        ))
    })?;
    let locked_outpoints = load_locked_outpoints(storage, &name).await?;
    ensure_selected_outpoint_unlocked(&selected_outpoint, &locked_outpoints)?;
    let name_for_error = name.clone();

    let handle = Handle::current();

    let cpfp = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        handle.block_on(async {
            wallet
                .create_cpfp_psbt(&parent_txid, &selected_outpoint, fee_rate_sat_per_vb.as_u64())
                .await
                .map_err(|e| {
                    tracing::error!(
                        "api psbt: cpfp_psbt failed name={} parent_txid={} selected_outpoint={} fee_rate_sat_per_vb={} error={}",
                        name_for_error,
                        parent_txid,
                        selected_outpoint_str,
                        fee_rate_sat_per_vb.as_u64(),
                        e
                    );
                    e
                })
        })
    })
    .await?;

    info!(
        "api psbt: cpfp_psbt success name={} parent_txid={} child_txid={} selected_outpoint={} input_value_sat={} child_output_value_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} estimated_vsize={} psbt_len={}",
        name,
        cpfp.parent_txid,
        cpfp.txid,
        cpfp.selected_outpoint,
        cpfp.input_value_sat.as_u64(),
        cpfp.child_output_value_sat.as_u64(),
        cpfp.fee_sat.as_u64(),
        cpfp.fee_rate_sat_per_vb,
        cpfp.replaceable,
        cpfp.estimated_vsize,
        cpfp.psbt_base64.as_str().len()
    );

    Ok(cpfp.into())
}

/// Build, sign, and publish a replacement transaction for an existing
/// unconfirmed RBF transaction.
pub async fn bump_fee(
    storage: &WalletStorage,
    request: BumpFeeRequestDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    let BumpFeeRequestDto {
        name,
        txid,
        fee_rate_sat_per_vb,
    } = request;
    debug!(
        "api psbt: bump_fee start name={} txid={} fee_rate_sat_per_vb={}",
        name, txid, fee_rate_sat_per_vb
    );
    info!(
        "api psbt: bump_fee request received name={} txid={} requested_fee_rate_sat_per_vb={}",
        name, txid, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;
    info!(
        "api psbt: bump_fee fee rate validated name={} txid={} requested_fee_rate_sat_per_vb={}",
        name,
        txid,
        fee_rate_sat_per_vb.as_u64()
    );

    let txid_for_log = txid.clone();
    let name_for_error = name.clone();

    let published = spawn_wallet_blocking(move || -> WalletApiResult<TxBroadcastResultDto> {
        let mut wallet = WalletService::load_or_create(&config)?;
        let sync_service = WalletSyncService::new();

        let bumped = wallet
            .bump_fee_psbt(&txid, fee_rate_sat_per_vb)
            .map_err(|e| {
                tracing::error!(
                "api psbt: bump_fee build failed name={} txid={} fee_rate_sat_per_vb={} error={}",
                name_for_error,
                txid,
                fee_rate_sat_per_vb.as_u64(),
                e
            );
                e
            })?;

        let signed = wallet.sign_psbt(&bumped.psbt_base64).map_err(|e| {
            tracing::error!(
                "api psbt: bump_fee sign failed name={} txid={} error={}",
                name_for_error,
                txid,
                e
            );
            e
        })?;

        let finalized = wallet.finalize_psbt_for_broadcast(&signed.psbt_base64)?;

        sync_service
            .broadcast_tx_hex(&config, finalized.tx_hex.as_str())
            .map_err(|e| {
                log_publish_error(&name_for_error, &e);
                e
            })?;

        Ok(TxBroadcastResultDto {
            txid: finalized.txid.to_string(),
            replaceable: Some(finalized.replaceable),
        })
    })
    .await?;

    info!(
        "api psbt: bump_fee success name={} original_txid={} replacement_txid={} replaceable={:?}",
        name, txid_for_log, published.txid, published.replaceable,
    );

    Ok(published)
}

/// Build, sign, and publish a CPFP transaction for an existing unconfirmed
/// parent transaction.
pub async fn cpfp(
    storage: &WalletStorage,
    request: CpfpRequestDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    let CpfpRequestDto {
        name,
        parent_txid,
        selected_outpoint,
        fee_rate_sat_per_vb,
    } = request;
    debug!(
        "api psbt: cpfp start name={} parent_txid={} selected_outpoint={} fee_rate_sat_per_vb={}",
        name, parent_txid, selected_outpoint, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, &name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let selected_outpoint_str = selected_outpoint.clone();
    let selected_outpoint = WalletOutPoint::parse(&selected_outpoint).map_err(|e| {
        crate::WalletApiError::InvalidInput(format!(
            "invalid selected_outpoint '{}': {}",
            selected_outpoint_str, e
        ))
    })?;
    let locked_outpoints = load_locked_outpoints(storage, &name).await?;
    ensure_selected_outpoint_unlocked(&selected_outpoint, &locked_outpoints)?;
    let parent_txid_for_log = parent_txid.clone();
    let selected_outpoint_for_log = selected_outpoint_str.clone();
    let fee_rate_sat_per_vb_for_log = fee_rate_sat_per_vb.as_u64();
    let name_for_error = name.clone();

    let handle = Handle::current();

    let published = spawn_wallet_blocking(move || -> WalletApiResult<TxBroadcastResultDto> {
        let mut wallet = WalletService::load_or_create(&config)?;
        let sync_service = WalletSyncService::new();

        let cpfp_psbt = handle
            .block_on(async {
                wallet
                    .create_cpfp_psbt(&parent_txid, &selected_outpoint, fee_rate_sat_per_vb.as_u64())
                    .await
            })
            .map_err(|e| {
                tracing::error!(
                    "api psbt: cpfp build failed name={} parent_txid={} selected_outpoint={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    parent_txid,
                    selected_outpoint_str,
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })?;

        let signed = wallet.sign_psbt(&cpfp_psbt.psbt_base64).map_err(|e| {
            tracing::error!(
                "api psbt: cpfp sign failed name={} parent_txid={} error={}",
                name_for_error,
                parent_txid,
                e
            );
            e
        })?;

        let finalized = wallet.finalize_psbt_for_broadcast(&signed.psbt_base64)?;

        sync_service
            .broadcast_tx_hex(&config, finalized.tx_hex.as_str())
            .map_err(|e| {
                log_publish_error(&name_for_error, &e);
                e
            })?;

        Ok(TxBroadcastResultDto {
            txid: finalized.txid.to_string(),
            replaceable: Some(finalized.replaceable),
        })
    })
    .await?;

    info!(
        "api psbt: cpfp success name={} parent_txid={} selected_outpoint={} child_txid={} fee_rate_sat_per_vb={} replaceable={:?}",
        name,
        parent_txid_for_log,
        selected_outpoint_for_log,
        published.txid,
        fee_rate_sat_per_vb_for_log,
        published.replaceable,
    );

    Ok(published)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::model::{
        BumpFeeRequestDto, ConsolidationRequestDto, CpfpRequestDto, CreatePsbtRequestDto,
        PublishPsbtRequestDto, SendMaxRequestDto, SignPsbtRequestDto, SweepRequestDto,
        WalletCoinControlDto, WalletConsolidationDto,
    };

    #[test]
    fn log_publish_error_handles_known_sync_variants_without_panicking() {
        let errors = vec![
            WalletSyncError::BroadcastTransport("transport down".to_string()),
            WalletSyncError::BroadcastMempoolConflict("conflict".to_string()),
            WalletSyncError::BroadcastAlreadyConfirmed("confirmed".to_string()),
            WalletSyncError::BroadcastMissingInputs("missing inputs".to_string()),
            WalletSyncError::BroadcastInsufficientFee("insufficient fee".to_string()),
            WalletSyncError::PsbtNotFinalized,
            WalletSyncError::SyncFailed("generic sync failure".to_string()),
        ];

        for error in &errors {
            log_publish_error("test-wallet", error);
        }
    }

    #[test]
    fn create_psbt_request_can_carry_optional_coin_control() {
        let dto = WalletCoinControlDto {
            include_outpoints: vec![
                "0000000000000000000000000000000000000000000000000000000000000001:0".to_string(),
            ],
            exclude_outpoints: vec![
                "0000000000000000000000000000000000000000000000000000000000000002:1".to_string(),
            ],
            confirmed_only: true,
            selection_mode: Some(crate::model::WalletInputSelectionModeDto::StrictManual),
        };

        assert_eq!(dto.include_outpoints.len(), 1);
        assert_eq!(dto.exclude_outpoints.len(), 1);
        assert!(dto.confirmed_only);
        assert!(matches!(
            dto.selection_mode,
            Some(crate::model::WalletInputSelectionModeDto::StrictManual)
        ));

        let request = CreatePsbtRequestDto {
            name: "regtest-local".to_string(),
            to_address: "bcrt1qexampledestination".to_string(),
            amount_sat: 10_000,
            fee_rate_sat_per_vb: 2,
            replaceable: true,
            coin_control: Some(dto),
        };

        assert_eq!(request.name, "regtest-local");
        assert_eq!(request.to_address, "bcrt1qexampledestination");
        assert_eq!(request.amount_sat, 10_000);
        assert_eq!(request.fee_rate_sat_per_vb, 2);
        assert!(request.replaceable);
        assert!(request.coin_control.is_some());
    }

    #[test]
    fn send_max_request_can_represent_simple_and_coin_control_flows() {
        let simple = SendMaxRequestDto {
            name: "regtest-local".to_string(),
            to_address: "bcrt1qsendmaxdestination".to_string(),
            fee_rate_sat_per_vb: 1,
            replaceable: false,
            coin_control: None,
        };

        assert_eq!(simple.name, "regtest-local");
        assert_eq!(simple.to_address, "bcrt1qsendmaxdestination");
        assert_eq!(simple.fee_rate_sat_per_vb, 1);
        assert!(!simple.replaceable);
        assert!(simple.coin_control.is_none());

        let with_coin_control = SendMaxRequestDto {
            name: "regtest-local".to_string(),
            to_address: "bcrt1qsendmaxdestination".to_string(),
            fee_rate_sat_per_vb: 3,
            replaceable: true,
            coin_control: Some(WalletCoinControlDto {
                include_outpoints: vec![
                    "0000000000000000000000000000000000000000000000000000000000000004:0"
                        .to_string(),
                ],
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: Some(crate::model::WalletInputSelectionModeDto::StrictManual),
            }),
        };

        assert_eq!(with_coin_control.fee_rate_sat_per_vb, 3);
        assert!(with_coin_control.replaceable);
        assert!(with_coin_control.coin_control.is_some());
    }

    #[test]
    fn sweep_request_requires_coin_control() {
        let request = SweepRequestDto {
            name: "regtest-local".to_string(),
            to_address: "bcrt1qsweepdestination".to_string(),
            fee_rate_sat_per_vb: 2,
            replaceable: true,
            coin_control: WalletCoinControlDto {
                include_outpoints: vec![
                    "0000000000000000000000000000000000000000000000000000000000000005:0"
                        .to_string(),
                ],
                exclude_outpoints: Vec::new(),
                confirmed_only: true,
                selection_mode: Some(crate::model::WalletInputSelectionModeDto::StrictManual),
            },
        };

        assert_eq!(request.name, "regtest-local");
        assert_eq!(request.to_address, "bcrt1qsweepdestination");
        assert_eq!(request.fee_rate_sat_per_vb, 2);
        assert!(request.replaceable);
        assert_eq!(request.coin_control.include_outpoints.len(), 1);
        assert!(request.coin_control.exclude_outpoints.is_empty());
        assert!(request.coin_control.confirmed_only);
    }

    #[test]
    fn wallet_psbt_dto_can_carry_selected_inputs() {
        let dto = WalletPsbtDto {
            psbt_base64: "dummy_psbt".to_string(),
            txid: "dummy_txid".to_string(),
            original_txid: None,
            replacement: None,
            to_address: "tb1qexampleaddress".to_string(),
            amount_sat: 10_000,
            fee_sat: 123,
            fee_rate_sat_per_vb: 1,
            replaceable: true,
            change_amount_sat: Some(9_000),
            selected_utxo_count: 2,
            selected_inputs: vec![
                "0000000000000000000000000000000000000000000000000000000000000001:0".to_string(),
                "0000000000000000000000000000000000000000000000000000000000000002:1".to_string(),
            ],
            input_count: 2,
            output_count: 2,
            recipient_count: 1,
            estimated_vsize: 140,
        };

        assert_eq!(dto.selected_utxo_count, 2);
        assert_eq!(dto.selected_inputs.len(), 2);
        assert_eq!(dto.input_count, 2);
        assert!(dto.replaceable);
    }

    #[test]
    fn wallet_psbt_dto_can_represent_non_replaceable_send_result() {
        let dto = WalletPsbtDto {
            psbt_base64: "dummy_non_rbf_psbt".to_string(),
            txid: "dummy_non_rbf_txid".to_string(),
            original_txid: None,
            replacement: None,
            to_address: "tb1qnonrbfexampleaddress".to_string(),
            amount_sat: 9_000,
            fee_sat: 155,
            fee_rate_sat_per_vb: 1,
            replaceable: false,
            change_amount_sat: Some(49_376),
            selected_utxo_count: 1,
            selected_inputs: vec![
                "0000000000000000000000000000000000000000000000000000000000000005:0".to_string(),
            ],
            input_count: 1,
            output_count: 2,
            recipient_count: 1,
            estimated_vsize: 137,
        };

        assert_eq!(dto.amount_sat, 9_000);
        assert_eq!(dto.fee_sat, 155);
        assert_eq!(dto.fee_rate_sat_per_vb, 1);
        assert_eq!(dto.selected_utxo_count, 1);
        assert_eq!(dto.selected_inputs.len(), 1);
        assert_eq!(dto.input_count, 1);
        assert_eq!(dto.output_count, 2);
        assert_eq!(dto.recipient_count, 1);
        assert_eq!(dto.estimated_vsize, 137);
        assert!(!dto.replaceable);
    }

    #[test]
    fn wallet_psbt_dto_can_represent_send_max_result() {
        let dto = WalletPsbtDto {
            psbt_base64: "dummy_send_max_psbt".to_string(),
            txid: "dummy_send_max_txid".to_string(),
            original_txid: None,
            replacement: None,
            to_address: "tb1qsendmaxexampleaddress".to_string(),
            amount_sat: 49_500,
            fee_sat: 500,
            fee_rate_sat_per_vb: 2,
            replaceable: true,
            change_amount_sat: None,
            selected_utxo_count: 1,
            selected_inputs: vec![
                "0000000000000000000000000000000000000000000000000000000000000003:0".to_string(),
            ],
            input_count: 1,
            output_count: 1,
            recipient_count: 1,
            estimated_vsize: 110,
        };

        assert_eq!(dto.amount_sat, 49_500);
        assert_eq!(dto.fee_sat, 500);
        assert_eq!(dto.selected_utxo_count, 1);
        assert_eq!(dto.selected_inputs.len(), 1);
        assert!(dto.change_amount_sat.is_none());
        assert!(dto.replaceable);
    }

    #[test]
    fn consolidation_request_can_carry_consolidation_controls() {
        let dto = WalletConsolidationDto {
            include_outpoints: vec![
                "0000000000000000000000000000000000000000000000000000000000000001:0".to_string(),
                "0000000000000000000000000000000000000000000000000000000000000002:1".to_string(),
            ],
            exclude_outpoints: vec![
                "0000000000000000000000000000000000000000000000000000000000000003:0".to_string(),
            ],
            confirmed_only: true,
            max_input_count: Some(8),
            min_input_count: Some(2),
            min_utxo_value_sat: Some(1_000),
            max_utxo_value_sat: Some(100_000),
            max_fee_pct_of_input_value: Some(5),
            strategy: Some(crate::model::WalletConsolidationStrategyDto::SmallestFirst),
            selection_mode: Some(crate::model::WalletInputSelectionModeDto::AutomaticOnly),
        };

        assert_eq!(dto.include_outpoints.len(), 2);
        assert_eq!(dto.exclude_outpoints.len(), 1);
        assert!(dto.confirmed_only);
        assert_eq!(dto.max_input_count, Some(8));
        assert_eq!(dto.min_input_count, Some(2));
        assert_eq!(dto.min_utxo_value_sat, Some(1_000));
        assert_eq!(dto.max_utxo_value_sat, Some(100_000));
        assert_eq!(dto.max_fee_pct_of_input_value, Some(5));
        assert!(matches!(
            dto.strategy,
            Some(crate::model::WalletConsolidationStrategyDto::SmallestFirst)
        ));
        assert!(matches!(
            dto.selection_mode,
            Some(crate::model::WalletInputSelectionModeDto::AutomaticOnly)
        ));

        let request = ConsolidationRequestDto {
            name: "regtest-local".to_string(),
            fee_rate_sat_per_vb: 4,
            replaceable: true,
            consolidation: dto,
        };

        assert_eq!(request.name, "regtest-local");
        assert_eq!(request.fee_rate_sat_per_vb, 4);
        assert!(request.replaceable);
        assert_eq!(request.consolidation.include_outpoints.len(), 2);
        assert_eq!(request.consolidation.exclude_outpoints.len(), 1);
        assert_eq!(request.consolidation.max_input_count, Some(8));
    }

    #[test]
    fn wallet_psbt_dto_can_represent_bump_fee_result() {
        let dto = WalletPsbtDto {
            psbt_base64: "dummy_bump_fee_psbt".to_string(),
            txid: "replacement_txid".to_string(),
            original_txid: Some("original_txid".to_string()),
            replacement: Some(crate::model::WalletReplacementDto {
                replaced_txid: "original_txid".to_string(),
                replacement_txid: "replacement_txid".to_string(),
                replacement_depth: 1,
                replacement_chain: vec![
                    "original_txid".to_string(),
                    "replacement_txid".to_string(),
                ],
            }),
            to_address: "".to_string(),
            amount_sat: 0,
            fee_sat: 2_466,
            fee_rate_sat_per_vb: 18,
            replaceable: true,
            change_amount_sat: Some(97_534),
            selected_utxo_count: 1,
            selected_inputs: vec![
                "0000000000000000000000000000000000000000000000000000000000000004:0".to_string(),
            ],
            input_count: 1,
            output_count: 2,
            recipient_count: 1,
            estimated_vsize: 137,
        };

        assert_eq!(dto.original_txid.as_deref(), Some("original_txid"));
        let replacement = dto.replacement.as_ref().expect("replacement metadata should exist");
        assert_eq!(replacement.replaced_txid, "original_txid");
        assert_eq!(replacement.replacement_txid, "replacement_txid");
        assert_eq!(replacement.replacement_depth, 1);
        assert_eq!(replacement.replacement_chain.len(), 2);
        assert_eq!(dto.txid, "replacement_txid");
        assert_eq!(dto.fee_sat, 2_466);
        assert_eq!(dto.fee_rate_sat_per_vb, 18);
        assert_eq!(dto.estimated_vsize, 137);
        assert_eq!(dto.selected_utxo_count, 1);
        assert_eq!(dto.selected_inputs.len(), 1);
        assert_eq!(dto.input_count, 1);
        assert_eq!(dto.output_count, 2);
        assert_eq!(dto.recipient_count, 1);
        assert!(dto.replaceable);
    }

    #[test]
    fn bump_fee_preview_fee_matches_requested_rate_times_vsize() {
        let requested_fee_rate_sat_per_vb = 18_u64;
        let estimated_vsize = 137_u64;

        let estimated_fee_sat = requested_fee_rate_sat_per_vb.saturating_mul(estimated_vsize);

        assert_eq!(estimated_fee_sat, 2_466);
    }

    #[test]
    fn sign_publish_bump_and_cpfp_requests_carry_required_fields() {
        let sign = SignPsbtRequestDto {
            name: "regtest-local".to_string(),
            psbt_base64: "cHNidP8BAHECAAAA".to_string(),
        };
        assert_eq!(sign.name, "regtest-local");
        assert!(!sign.psbt_base64.is_empty());

        let publish = PublishPsbtRequestDto {
            name: "regtest-local".to_string(),
            psbt_base64: sign.psbt_base64.clone(),
        };
        assert_eq!(publish.name, "regtest-local");
        assert_eq!(publish.psbt_base64, sign.psbt_base64);

        let bump = BumpFeeRequestDto {
            name: "regtest-local".to_string(),
            txid: "0000000000000000000000000000000000000000000000000000000000000006".to_string(),
            fee_rate_sat_per_vb: 7,
        };
        assert_eq!(bump.name, "regtest-local");
        assert_eq!(bump.txid.len(), 64);
        assert_eq!(bump.fee_rate_sat_per_vb, 7);

        let cpfp = CpfpRequestDto {
            name: "regtest-local".to_string(),
            parent_txid: bump.txid.clone(),
            selected_outpoint:
                "0000000000000000000000000000000000000000000000000000000000000006:0".to_string(),
            fee_rate_sat_per_vb: 8,
        };
        assert_eq!(cpfp.name, "regtest-local");
        assert_eq!(cpfp.parent_txid, bump.txid);
        assert!(cpfp.selected_outpoint.ends_with(":0"));
        assert_eq!(cpfp.fee_rate_sat_per_vb, 8);
    }
}