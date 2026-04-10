use crate::dto::{WalletPsbtDto, WalletPublishedTxDto, WalletSignedPsbtDto};
use crate::WalletApiResult;

use wallet_core::types::{AmountSat, FeeRateSatPerVb};
use wallet_core::WalletService;
use wallet_storage::WalletStorage;
use wallet_sync::{WalletSyncError, WalletSyncService};

use super::wallet::load_wallet_config;

use tracing::{debug, info};
use tokio::task;

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
            tracing::error!(
                "api psbt: publish not_finalized name={}",
                name,
            );
        }
        _ => {
            tracing::error!(
                "api psbt: publish failed name={} error={}",
                name,
                error
            );
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
        name,
        to_address,
        amount_sat,
        fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let amount_sat = AmountSat::new(amount_sat)?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let to_address = to_address.to_string();
    let name_for_error = name.to_string();

    let psbt = task::block_in_place(|| {
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
    })?;

    info!(
        "api psbt: create success name={} txid={} to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        psbt.txid,
        psbt.to_address,
        psbt.amount_sat,
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.len()
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
    let psbt_base64 = psbt_base64.to_string();
    let name_for_error = name.to_string();

    let signed = task::block_in_place(|| {
        let mut wallet = WalletService::load_or_create(&config)?;

        wallet.sign_psbt(&psbt_base64).map_err(|e| {
            tracing::error!(
                "api psbt: sign failed name={} error={}",
                name_for_error,
                e
            );
            e
        })
    })?;

    info!(
        "api psbt: sign status={} name={} modified={} finalized={} txid={} psbt_len={}",
        signed.signing_status(),
        name,
        signed.modified,
        signed.finalized,
        signed.txid,
        signed.psbt_base64.len()
    );

    Ok(signed.into())
}

pub async fn publish(
    storage: &WalletStorage,
    name: &str,
    psbt_base64: &str,
) -> WalletApiResult<WalletPublishedTxDto> {
    debug!("api psbt: publish start name={}", name);

    let config = load_wallet_config(storage, name).await?;
    let psbt_base64 = psbt_base64.to_string();
    let name_for_error = name.to_string();

    let published = task::block_in_place(|| -> WalletApiResult<wallet_core::model::WalletPublishedTxInfo> {
        let wallet = WalletService::load_or_create(&config)?;
        let sync_service = WalletSyncService::new();

        let finalized = wallet.finalize_psbt_for_broadcast(&psbt_base64)?;

        sync_service
            .broadcast_tx_hex(&config, &finalized.tx_hex)
            .map_err(|e| {
                log_publish_error(&name_for_error, &e);
                e
            })?;

        Ok(wallet_core::model::WalletPublishedTxInfo {
            txid: finalized.txid,
            replaceable: Some(finalized.replaceable),
        })
    })?;

    info!(
        "api psbt: publish success name={} txid={} replaceable={:?}",
        name,
        published.txid,
        published.replaceable,
    );

    Ok(published.into())
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
        name,
        txid,
        fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let txid = txid.to_string();
    let name_for_error = name.to_string();

    let psbt = task::block_in_place(|| {
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
    })?;

    info!(
        "api psbt: bump_fee_psbt success name={} original_txid={} replacement_txid={} fee_sat={} fee_rate_sat_per_vb={} replaceable={} selected_utxos={} inputs={} outputs={} recipients={} estimated_vsize={} psbt_len={}",
        name,
        txid,
        psbt.txid,
        psbt.fee_sat,
        psbt.fee_rate_sat_per_vb,
        psbt.replaceable,
        psbt.selected_utxo_count,
        psbt.input_count,
        psbt.output_count,
        psbt.recipient_count,
        psbt.estimated_vsize,
        psbt.psbt_base64.len()
    );

    Ok(psbt.into())
}

/// Build, sign, and publish a replacement transaction for an existing
/// unconfirmed RBF transaction.
pub async fn bump_fee(
    storage: &WalletStorage,
    name: &str,
    txid: &str,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletPublishedTxDto> {
    debug!(
        "api psbt: bump_fee start name={} txid={} fee_rate_sat_per_vb={}",
        name,
        txid,
        fee_rate_sat_per_vb
    );

    let config = load_wallet_config(storage, name).await?;
    let fee_rate_sat_per_vb = FeeRateSatPerVb::new(fee_rate_sat_per_vb)?;

    let txid = txid.to_string();
    let name_for_error = name.to_string();

    let published = task::block_in_place(|| -> WalletApiResult<wallet_core::model::WalletPublishedTxInfo> {
        let mut wallet = WalletService::load_or_create(&config)?;
        let sync_service = WalletSyncService::new();

        let bumped = wallet.bump_fee_psbt(&txid, fee_rate_sat_per_vb).map_err(|e| {
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
            .broadcast_tx_hex(&config, &finalized.tx_hex)
            .map_err(|e| {
                log_publish_error(&name_for_error, &e);
                e
            })?;

        Ok(wallet_core::model::WalletPublishedTxInfo {
            txid: finalized.txid,
            replaceable: Some(finalized.replaceable),
        })
    })?;

    info!(
        "api psbt: bump_fee success name={} original_txid={} replacement_txid={} replaceable={:?}",
        name,
        txid,
        published.txid,
        published.replaceable,
    );

    Ok(published.into())
}