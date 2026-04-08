use crate::dto::{WalletPsbtDto, WalletPublishedTxDto, WalletSignedPsbtDto};
use crate::WalletApiResult;

use wallet_core::types::{AmountSat, FeeRateSatPerVb};
use crate::broadcaster::esplora::EsploraBroadcaster;
use wallet_core::{WalletCoreError, WalletService};
use wallet_storage::WalletStorage;

use super::wallet::load_wallet_config;

use tracing::{debug, info};
use tokio::task;

fn log_publish_error(name: &str, error: &WalletCoreError) {
    match error {
        WalletCoreError::BroadcastTransport(msg) => {
            tracing::error!(
                "api psbt: publish transport_failed name={} error={}",
                name,
                msg
            );
        }
        WalletCoreError::BroadcastMempoolConflict(msg) => {
            tracing::error!(
                "api psbt: publish mempool_conflict name={} error={}",
                name,
                msg
            );
        }
        WalletCoreError::BroadcastAlreadyConfirmed(msg) => {
            tracing::error!(
                "api psbt: publish already_confirmed name={} error={}",
                name,
                msg
            );
        }
        WalletCoreError::BroadcastMissingInputs(msg) => {
            tracing::error!(
                "api psbt: publish missing_inputs name={} error={}",
                name,
                msg
            );
        }
        WalletCoreError::BroadcastInsufficientFee(msg) => {
            tracing::error!(
                "api psbt: publish insufficient_fee name={} error={}",
                name,
                msg
            );
        }
        WalletCoreError::PsbtNotFinalized => {
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
        "api psbt: create success name={} to={} amount_sat={} fee_sat={} selected_utxos={} psbt_len={}",
        name,
        psbt.to_address,
        psbt.amount_sat.as_u64(),
        psbt.fee_sat.as_u64(),
        psbt.selected_utxo_count,
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

    let published = task::block_in_place(|| {
        let wallet = WalletService::load_or_create(&config)?;
        let broadcaster = EsploraBroadcaster::new(config.esplora_url.clone());

        wallet.publish_psbt(&psbt_base64, &broadcaster).map_err(|e| {
            log_publish_error(&name_for_error, &e);
            e
        })
    })?;

    info!(
        "api psbt: publish success name={} txid={}",
        name,
        published.txid
    );

    Ok(published.into())
}