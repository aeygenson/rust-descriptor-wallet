use crate::model::{
    TxBroadcastResultDto, WalletCoinControlDto, WalletConsolidationDto, WalletCpfpPsbtDto,
    WalletPsbtDto, WalletSignedPsbtDto,
};
use crate::WalletApiResult;

use wallet_core::types::{AmountSat, FeeRateSatPerVb, PsbtBase64, WalletOutPoint};
use wallet_core::WalletService;
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

/// Create an unsigned PSBT for a send flow.
///
/// This is the first API orchestration step in the PSBT transaction pipeline.
pub async fn create(
    storage: &WalletStorage,
    name: &str,
    to_address: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: create start name={} to={} amount_sat={} fee_rate_sat_per_vb={}",
        name, to_address, amount_sat, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let amount_sat = AmountSat::new(amount_sat)?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let to_address = to_address.to_string();
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_psbt(
                config.network,
                &to_address,
                amount_sat,
                fee_rate_sat_per_vb,
                true,
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create failed name={} to={} amount_sat={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    to_address,
                    amount_sat.as_u64(),
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
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

/// Create an unsigned PSBT for a send flow using explicit coin control.
///
/// This mirrors `create(...)`, but allows the caller to explicitly include or
/// exclude wallet UTXOs during transaction construction.
pub async fn create_with_coin_control(
    storage: &WalletStorage,
    name: &str,
    to_address: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
    coin_control: WalletCoinControlDto,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: create_with_coin_control start name={} to={} amount_sat={} fee_rate_sat_per_vb={} include_outpoints={} exclude_outpoints={} confirmed_only={} selection_mode={:?}",
        name,
        to_address,
        amount_sat,
        fee_rate_sat_per_vb,
        coin_control.include_outpoints.len(),
        coin_control.exclude_outpoints.len(),
        coin_control.confirmed_only,
        coin_control.selection_mode,
    );

    let config = load_wallet_config(storage, name).await?;
    let amount_sat = AmountSat::new(amount_sat)?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let to_address = to_address.to_string();
    let coin_control = coin_control.try_into_core()?;
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_psbt_with_coin_control(
                config.network,
                &to_address,
                amount_sat,
                fee_rate_sat_per_vb,
                true,
                Some(coin_control),
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_with_coin_control failed name={} to={} amount_sat={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    to_address,
                    amount_sat.as_u64(),
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_with_coin_control success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
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

/// Create an unsigned PSBT for a send-max flow.
///
/// This sends the maximum available amount (after fees) to the destination.
pub async fn create_send_max(
    storage: &WalletStorage,
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: create_send_max start name={} to={} fee_rate_sat_per_vb={}",
        name, to_address, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let to_address = to_address.to_string();
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_send_max_psbt(
                config.network,
                &to_address,
                fee_rate_sat_per_vb,
                true,
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_send_max failed name={} to={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    to_address,
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_send_max success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
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

/// Create an unsigned PSBT for a send-max flow using explicit coin control.
pub async fn create_send_max_with_coin_control(
    storage: &WalletStorage,
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
    coin_control: WalletCoinControlDto,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: create_send_max_with_coin_control start name={} to={} fee_rate_sat_per_vb={} include_outpoints={} exclude_outpoints={} confirmed_only={} selection_mode={:?}",
        name,
        to_address,
        fee_rate_sat_per_vb,
        coin_control.include_outpoints.len(),
        coin_control.exclude_outpoints.len(),
        coin_control.confirmed_only,
        coin_control.selection_mode,
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let to_address = to_address.to_string();
    let coin_control = coin_control.try_into_core()?;
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_send_max_psbt_with_coin_control(
                config.network,
                &to_address,
                fee_rate_sat_per_vb,
                true,
                Some(coin_control),
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_send_max_with_coin_control failed name={} to={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    to_address,
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_send_max_with_coin_control success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
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

/// Create an unsigned PSBT for a sweep flow using explicit coin control.
///
/// Sweep is implemented as strict send-max with an explicit include set.
pub async fn create_sweep(
    storage: &WalletStorage,
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
    coin_control: WalletCoinControlDto,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: create_sweep start name={} to={} fee_rate_sat_per_vb={} include_outpoints={} exclude_outpoints={} confirmed_only={} selection_mode={:?}",
        name,
        to_address,
        fee_rate_sat_per_vb,
        coin_control.include_outpoints.len(),
        coin_control.exclude_outpoints.len(),
        coin_control.confirmed_only,
        coin_control.selection_mode,
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let to_address = to_address.to_string();
    let coin_control = coin_control.try_into_core()?;
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_sweep_psbt(
                config.network,
                &to_address,
                fee_rate_sat_per_vb,
                true,
                coin_control,
            )
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_sweep failed name={} to={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    to_address,
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_sweep success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
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

/// Create an unsigned PSBT for a wallet-internal consolidation flow.
///
/// Consolidation spends multiple wallet UTXOs into a smaller number of
/// wallet-owned outputs, usually one internal output, to reduce fragmentation.
pub async fn create_consolidation(
    storage: &WalletStorage,
    name: &str,
    fee_rate_sat_per_vb: u64,
    consolidation: WalletConsolidationDto,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: create_consolidation start name={} fee_rate_sat_per_vb={} include_outpoints={} exclude_outpoints={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?} selection_mode={:?}",
        name,
        fee_rate_sat_per_vb,
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

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let consolidation = consolidation.try_into_core()?;
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet
            .create_consolidation_psbt(fee_rate_sat_per_vb, true, Some(consolidation))
            .map_err(|e| {
                tracing::error!(
                    "api psbt: create_consolidation failed name={} fee_rate_sat_per_vb={} error={}",
                    name_for_error,
                    fee_rate_sat_per_vb.as_u64(),
                    e
                );
                e
            })
    })
    .await?;

    info!(
        "api psbt: create_consolidation success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
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

pub async fn sign(
    storage: &WalletStorage,
    name: &str,
    psbt_base64: &str,
) -> WalletApiResult<WalletSignedPsbtDto> {
    debug!("api psbt: sign start name={}", name);

    let config = load_wallet_config(storage, name).await?;
    let psbt_base64 = PsbtBase64::from(psbt_base64);
    let name_for_error = name.to_string();

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
    name: &str,
    psbt_base64: &str,
) -> WalletApiResult<TxBroadcastResultDto> {
    debug!("api psbt: publish start name={}", name);

    let config = load_wallet_config(storage, name).await?;
    let psbt_base64 = PsbtBase64::from(psbt_base64);
    let name_for_error = name.to_string();

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
    name: &str,
    to_address: &str,
    fee_rate_sat_per_vb: u64,
    coin_control: WalletCoinControlDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    debug!(
        "api psbt: sweep start name={} to={} fee_rate_sat_per_vb={} include_outpoints={} exclude_outpoints={} confirmed_only={}",
        name,
        to_address,
        fee_rate_sat_per_vb,
        coin_control.include_outpoints.len(),
        coin_control.exclude_outpoints.len(),
        coin_control.confirmed_only,
    );

    let created =
        create_sweep(storage, name, to_address, fee_rate_sat_per_vb, coin_control).await?;

    let signed = sign(storage, name, &created.psbt_base64).await?;

    if !signed.finalized {
        return Err(crate::WalletApiError::SendNotFinalized);
    }

    publish(storage, name, &signed.psbt_base64).await
}

/// Create, sign, and publish a wallet-internal consolidation transaction.
pub async fn consolidate(
    storage: &WalletStorage,
    name: &str,
    fee_rate_sat_per_vb: u64,
    consolidation: WalletConsolidationDto,
) -> WalletApiResult<TxBroadcastResultDto> {
    debug!(
        "api psbt: consolidate start name={} fee_rate_sat_per_vb={} include_outpoints={} exclude_outpoints={} confirmed_only={} max_input_count={:?} min_input_count={:?} min_utxo_value_sat={:?} max_utxo_value_sat={:?} max_fee_pct={:?} strategy={:?}",
        name,
        fee_rate_sat_per_vb,
        consolidation.include_outpoints.len(),
        consolidation.exclude_outpoints.len(),
        consolidation.confirmed_only,
        consolidation.max_input_count,
        consolidation.min_input_count,
        consolidation.min_utxo_value_sat,
        consolidation.max_utxo_value_sat,
        consolidation.max_fee_pct_of_input_value,
        consolidation.strategy,
    );

    let created = create_consolidation(storage, name, fee_rate_sat_per_vb, consolidation).await?;

    let signed = sign(storage, name, &created.psbt_base64).await?;

    if !signed.finalized {
        return Err(crate::WalletApiError::SendNotFinalized);
    }

    publish(storage, name, &signed.psbt_base64).await
}

/// Build a replacement PSBT for an existing unconfirmed RBF transaction.
///
/// This mirrors `create(...)` but targets an existing replaceable transaction
/// identified by `txid` and requests a higher fee rate.
pub async fn bump_fee_psbt(
    storage: &WalletStorage,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api psbt: bump_fee_psbt start name={} txid={} fee_rate_sat_per_vb={}",
        name, txid, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let txid = txid.to_string();
    let txid_for_log = txid.clone();
    let name_for_error = name.to_string();

    let psbt = spawn_wallet_blocking(move || {
        let mut wallet = WalletService::load_or_create(&config)?;

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
        "api psbt: bump_fee_psbt success name={} original_txid={} replacement_txid={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} selected_inputs={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        txid_for_log,
        psbt.txid,
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
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletCpfpPsbtDto> {
    debug!(
        "api psbt: cpfp_psbt start name={} parent_txid={} selected_outpoint={} fee_rate_sat_per_vb={}",
        name,
        parent_txid,
        selected_outpoint,
        fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let parent_txid = parent_txid.to_string();
    let selected_outpoint_str = selected_outpoint.to_string();
    let selected_outpoint = WalletOutPoint::parse(selected_outpoint).map_err(|e| {
        crate::WalletApiError::InvalidInput(format!(
            "invalid selected_outpoint '{}': {}",
            selected_outpoint_str, e
        ))
    })?;
    let name_for_error = name.to_string();

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
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<TxBroadcastResultDto> {
    debug!(
        "api psbt: bump_fee start name={} txid={} fee_rate_sat_per_vb={}",
        name, txid, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let txid = txid.to_string();
    let txid_for_log = txid.clone();
    let name_for_error = name.to_string();

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
    name: &str,
    parent_txid: &str,
    selected_outpoint: &str,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<TxBroadcastResultDto> {
    debug!(
        "api psbt: cpfp start name={} parent_txid={} selected_outpoint={} fee_rate_sat_per_vb={}",
        name, parent_txid, selected_outpoint, fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let parent_txid = parent_txid.to_string();
    let selected_outpoint_str = selected_outpoint.to_string();
    let selected_outpoint = WalletOutPoint::parse(selected_outpoint).map_err(|e| {
        crate::WalletApiError::InvalidInput(format!(
            "invalid selected_outpoint '{}': {}",
            selected_outpoint_str, e
        ))
    })?;
    let parent_txid_for_log = parent_txid.clone();
    let selected_outpoint_for_log = selected_outpoint_str.clone();
    let fee_rate_sat_per_vb_for_log = fee_rate_sat_per_vb.as_u64();
    let name_for_error = name.to_string();

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
    fn wallet_coin_control_dto_can_be_constructed_for_psbt_create_calls() {
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
    }

    #[test]
    fn wallet_psbt_dto_can_carry_selected_inputs() {
        let dto = WalletPsbtDto {
            psbt_base64: "dummy_psbt".to_string(),
            txid: "dummy_txid".to_string(),
            original_txid: None,
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
    fn wallet_psbt_dto_can_represent_send_max_result() {
        let dto = WalletPsbtDto {
            psbt_base64: "dummy_send_max_psbt".to_string(),
            txid: "dummy_send_max_txid".to_string(),
            original_txid: None,
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
    fn wallet_consolidation_dto_can_be_constructed_for_consolidation_calls() {
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
    }
}
