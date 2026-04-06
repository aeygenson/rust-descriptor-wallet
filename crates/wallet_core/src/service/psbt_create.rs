use std::str::FromStr;

use bitcoin::{Address, Amount, Network};
use bitcoin::FeeRate;
use bdk_wallet::KeychainKind;
use tracing::{debug, info};

use crate::model::WalletPsbtInfo;
use crate::types::{AmountSat, FeeRateSatPerVb};
use crate::WalletCoreResult;
use super::*;

impl WalletService {
    /// Create an unsigned PSBT scaffold for a future send flow.
    ///
    /// This is the first core entry point for transaction construction.
    /// For now it performs basic validation and defines the stable service API,
    /// while the actual BDK transaction builder logic will be added next.
    ///
    /// Planned responsibilities of this method:
    /// - validate destination address
    /// - validate amount
    /// - ensure destination network matches wallet network
    /// - build unsigned PSBT with BDK
    /// - return fee/change/input selection summary
    pub fn create_psbt(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        amount_sat: AmountSat,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        debug!(
            "wallet_service: create_psbt start to={} amount_sat={} fee_rate_sat_per_vb={}",
            to_address,
            amount_sat.as_u64(),
            fee_rate_sat_per_vb.as_u64()
        );

        // Keep defensive validation in core even though wrapper constructors
        // normally reject invalid values. Tests can intentionally construct
        // zero-valued wrappers directly, and the core method should still
        // return precise domain errors.
        if amount_sat.as_u64() == 0 {
            return Err(crate::WalletCoreError::InvalidAmount);
        }

        if fee_rate_sat_per_vb.as_u64() == 0 {
            return Err(crate::WalletCoreError::InvalidFeeRate);
        }

        let parsed = Address::from_str(to_address)
            .map_err(|e| crate::WalletCoreError::InvalidDestinationAddress(e.to_string()))?;

        let checked = parsed
            .require_network(wallet_network)
            .map_err(|e| crate::WalletCoreError::DestinationNetworkMismatch(e.to_string()))?;

        let fee_rate = FeeRate::from_sat_per_vb(fee_rate_sat_per_vb.as_u64())
            .ok_or_else(|| crate::WalletCoreError::InvalidFeeRate)?;

        let recipient_script = checked.script_pubkey();
        let recipient_amount = Amount::from_sat(amount_sat.as_u64());

        let mut builder = self.wallet.build_tx();
        builder.add_recipient(recipient_script.clone(), recipient_amount);
        builder.fee_rate(fee_rate);

        let psbt = builder
            .finish()
            .map_err(|e| crate::WalletCoreError::PsbtBuildFailed(e.to_string()))?;

        for (idx, txout) in psbt.unsigned_tx.output.iter().enumerate() {
            let derivation = self.wallet.derivation_of_spk(txout.script_pubkey.clone());
            debug!(
                "wallet_service: psbt output idx={} value={} recipient_match={} derivation={:?}",
                idx,
                txout.value.to_sat(),
                txout.script_pubkey == recipient_script,
                derivation
            );
        }

        let selected_utxo_count = psbt.inputs.len();

        let total_input_sat: u64 = psbt
            .inputs
            .iter()
            .filter_map(|input| input.witness_utxo.as_ref().map(|txout| txout.value.to_sat()))
            .sum();

        let total_output_sat: u64 = psbt
            .unsigned_tx
            .output
            .iter()
            .map(|txout| txout.value.to_sat())
            .sum();

        let fee_sat = total_input_sat
            .checked_sub(total_output_sat)
            .ok_or_else(|| crate::WalletCoreError::FeeCalculationFailed)?;

        // Detect change primarily by looking for outputs that the wallet
        // recognizes as belonging to its internal (change) keychain.
        //
        // If that lookup does not find anything, use a pragmatic fallback for
        // the current single-recipient flow: when there are exactly two outputs
        // and one matches the recipient script, treat the other one as change.
        let change_amount_sat = psbt
            .unsigned_tx
            .output
            .iter()
            .filter_map(|txout| {
                self.wallet
                    .derivation_of_spk(txout.script_pubkey.clone())
                    .and_then(|(keychain, _)| {
                        if keychain == KeychainKind::Internal {
                            Some(txout.value.to_sat())
                        } else {
                            None
                        }
                    })
            })
            .next()
            .or_else(|| {
                let outputs = &psbt.unsigned_tx.output;
                if outputs.len() == 2 {
                    outputs
                        .iter()
                        .find(|txout| txout.script_pubkey != recipient_script)
                        .map(|txout| txout.value.to_sat())
                } else {
                    None
                }
            });

        let psbt_base64 = psbt.to_string();

        info!(
            "wallet_service: create_psbt success to={} amount_sat={} fee_sat={} change_amount_sat={:?} selected_utxos={}",
            to_address,
            amount_sat.as_u64(),
            fee_sat,
            change_amount_sat,
            selected_utxo_count
        );

        Ok(WalletPsbtInfo {
            psbt_base64,
            to_address: to_address.to_string(),
            amount_sat,
            fee_sat: AmountSat(fee_sat),
            change_amount_sat: change_amount_sat.map(AmountSat),
            selected_utxo_count,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::WalletConfig;
    use crate::types::{AmountSat, FeeRateSatPerVb};

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            external_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
            internal_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            db_path: unique_test_db_path("wallet_core_psbt"),
            esplora_url: "https://mempool.space/signet/api".to_string(),
            is_watch_only: true,
        }
    }

    fn load_test_wallet() -> (WalletConfig, WalletService) {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");
        (config, wallet)
    }

    fn create_psbt_with(
        wallet: &mut WalletService,
        network: Network,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        wallet.create_psbt(
            network,
            to_address,
            AmountSat(amount_sat),
            FeeRateSatPerVb(fee_rate_sat_per_vb),
        )
    }

    fn assert_create_psbt_err(
        result: WalletCoreResult<WalletPsbtInfo>,
        matcher: impl FnOnce(crate::WalletCoreError) -> bool,
    ) {
        match result {
            Err(err) => assert!(matcher(err), "unexpected error variant"),
            Ok(_) => panic!("expected create_psbt to fail"),
        }
    }

    fn unique_test_db_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX_EPOCH")
            .as_nanos();
        let seq = TEST_DB_COUNTER.fetch_add(1, Ordering::Relaxed);

        std::env::temp_dir().join(format!(
            "{}_{}_{}_{}.db",
            prefix,
            std::process::id(),
            nanos,
            seq
        ))
    }

    fn valid_signet_address() -> &'static str {
        "tb1pckmj4jv3z4399h0se8stn0f5c39eq6266hv296w00ysds0gkc79srg7udu"
    }

    fn valid_mainnet_address() -> &'static str {
        "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"
    }

    #[test]
    fn create_psbt_fails_for_zero_amount() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_signet_address(), 0, 1);

        assert_create_psbt_err(result, |err| matches!(err, crate::WalletCoreError::InvalidAmount));
    }

    #[test]
    fn create_psbt_fails_for_zero_fee_rate() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_signet_address(), 1000, 0);

        assert_create_psbt_err(result, |err| matches!(err, crate::WalletCoreError::InvalidFeeRate));
    }

    #[test]
    fn create_psbt_returns_invalid_destination_address_error() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, "invalid_address", 1000, 1);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::InvalidDestinationAddress(_))
        });
    }

    #[test]
    fn create_psbt_returns_destination_network_mismatch_error() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_mainnet_address(), 1000, 1);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::DestinationNetworkMismatch(_))
        });
    }

    #[test]
    fn create_psbt_fails_for_insufficient_funds() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_signet_address(), 1000, 1);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::PsbtBuildFailed(_))
        });
    }
}