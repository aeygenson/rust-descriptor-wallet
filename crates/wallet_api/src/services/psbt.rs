use crate::dto::{WalletPsbtDto, WalletPublishedTxDto, WalletSignedPsbtDto};
use crate::WalletApiResult;

use wallet_core::types::{AmountSat, FeeRateSatPerVb};
use wallet_core::WalletService;
use wallet_storage::WalletStorage;

use super::wallet::load_wallet_config;

use tracing::{debug, info};

/// Create an unsigned PSBT for a future send flow.
///
/// This is the first send-side API orchestration entry point.
/// It currently delegates to the core PSBT builder scaffold.
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

    let mut wallet = WalletService::load_or_create(&config)?;

    let psbt = wallet
        .create_psbt(
            config.network,
            to_address,
            amount_sat,
            fee_rate_sat_per_vb,
        )
        .map_err(|e| {
            tracing::error!(
                "api psbt: create failed name={} to={} amount_sat={} fee_rate_sat_per_vb={} error={}",
                name,
                to_address,
                amount_sat.as_u64(),
                fee_rate_sat_per_vb.as_u64(),
                e
            );
            e
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
    let mut wallet = WalletService::load_or_create(&config)?;

    let signed = wallet.sign_psbt(psbt_base64).map_err(|e| {
        tracing::error!(
            "api psbt: sign failed name={} error={}",
            name,
            e
        );
        e
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
    let wallet = WalletService::load_or_create(&config)?;

    let published = wallet.publish_psbt(psbt_base64).map_err(|e| {
        tracing::error!(
            "api psbt: publish failed name={} error={}",
            name,
            e
        );
        e
    })?;

    info!(
        "api psbt: publish success name={} txid={}",
        name,
        published.txid
    );

    Ok(published.into())
}