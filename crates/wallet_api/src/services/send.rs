use crate::dto::WalletPsbtDto;
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
pub async fn create_psbt(
    storage: &WalletStorage,
    name: &str,
    to_address: &str,
    amount_sat: u64,
    fee_rate_sat_per_vb: u64,
) -> WalletApiResult<WalletPsbtDto> {
    debug!(
        "api send: create_psbt start name={} to={} amount_sat={} fee_rate_sat_per_vb={}",
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
                "api send: create_psbt failed name={} to={} amount_sat={} fee_rate_sat_per_vb={} error={}",
                name,
                to_address,
                amount_sat.as_u64(),
                fee_rate_sat_per_vb.as_u64(),
                e
            );
            e
        })?;

    info!(
        "api send: create_psbt success name={} to={} amount_sat={} fee_sat={} selected_utxos={} psbt_len={}",
        name,
        psbt.to_address,
        psbt.amount_sat.as_u64(),
        psbt.fee_sat.as_u64(),
        psbt.selected_utxo_count,
        psbt.psbt_base64.len()
    );

    Ok(psbt.into())
}