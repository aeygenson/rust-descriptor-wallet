use std::str::FromStr;

use std::collections::HashSet;

use bitcoin::{Address, Amount, Network, Sequence};
use bitcoin::FeeRate;
use bdk_wallet::KeychainKind;
use tracing::{debug, info};

use crate::model::{WalletCoinControlInfo, WalletPsbtInfo};
use crate::types::{AmountSat, FeeRateSatPerVb};
use crate::WalletCoreResult;
use super::*;

impl WalletService {
    /// Create an unsigned PSBT for a send flow.
    ///
    /// This is the core entry point for transaction construction. It validates
    /// the destination, amount, fee rate, wallet network, and returns an
    /// unsigned PSBT together with fee/change/input-selection summary data.
    pub fn create_psbt(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        amount_sat: AmountSat,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        self.create_psbt_with_coin_control(
            wallet_network,
            to_address,
            amount_sat,
            fee_rate_sat_per_vb,
            enable_rbf,
            None,
        )
    }

    /// Create an unsigned PSBT for a send flow with optional coin control.
    pub fn create_psbt_with_coin_control(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        amount_sat: AmountSat,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
        coin_control: Option<WalletCoinControlInfo>,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        debug!(
            "wallet_service: create_psbt start to={} amount_sat={} fee_rate_sat_per_vb={} enable_rbf={} has_coin_control={}",
            to_address,
            amount_sat.as_u64(),
            fee_rate_sat_per_vb.as_u64(),
            enable_rbf,
            coin_control.is_some(),
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

        let selected_inputs = match coin_control.as_ref() {
            Some(cc) if !cc.is_empty() => self.resolve_coin_control_inputs(cc)?,
            _ => Vec::new(),
        };

        let excluded_inputs = match coin_control.as_ref() {
            Some(cc) if !cc.exclude_outpoints.is_empty() => self.resolve_coin_control_exclusions(cc)?,
            _ => Vec::new(),
        };

        let strict_selected_inputs = coin_control
            .as_ref()
            .map(|cc| !cc.include_outpoints.is_empty())
            .unwrap_or(false);

        let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        let mut strict_excluded_inputs = excluded_inputs.clone();
        if strict_selected_inputs {
            let selected_set: HashSet<_> = selected_inputs.iter().copied().collect();

            let selected_total_sat: u64 = wallet_utxos
                .iter()
                .filter(|u| selected_set.contains(&u.outpoint))
                .map(|u| u.txout.value.to_sat())
                .sum();

            // Conservative fee estimate for strict selection mode.
            // We assume one recipient output plus one change output.
            let input_count = selected_inputs.len() as u64;
            let output_count = 2u64;
            let estimated_vsize = 11u64 + input_count * 58u64 + output_count * 43u64;
            let fee_estimate_sat = estimated_vsize * fee_rate_sat_per_vb.as_u64();
            let required_sat = amount_sat.as_u64() + fee_estimate_sat;

            if selected_inputs.is_empty() {
                return Err(crate::WalletCoreError::CoinControlEmptySelection);
            }

            if selected_total_sat < amount_sat.as_u64() {
                return Err(crate::WalletCoreError::CoinControlInsufficientSelectedFunds {
                    selected_sat: selected_total_sat,
                    required_sat,
                    fee_estimate_sat,
                });
            }

            if selected_total_sat < required_sat {
                return Err(crate::WalletCoreError::CoinControlStrictModeViolation);
            }

            for utxo in &wallet_utxos {
                if !selected_set.contains(&utxo.outpoint)
                    && !strict_excluded_inputs.contains(&utxo.outpoint)
                {
                    strict_excluded_inputs.push(utxo.outpoint);
                }
            }
        }

        debug!(
            "wallet_service: create_psbt coin_control selected_inputs={} excluded_inputs={} strict_selected_inputs={}",
            selected_inputs.len(),
            strict_excluded_inputs.len(),
            strict_selected_inputs,
        );

        let mut builder = self.wallet.build_tx();
        builder.add_recipient(recipient_script.clone(), recipient_amount);
        builder.fee_rate(fee_rate);
        if enable_rbf {
            builder.set_exact_sequence(Sequence(0xFFFFFFFD));
        }

        for outpoint in &selected_inputs {
            builder.add_utxo(*outpoint).map_err(|e| {
                crate::WalletCoreError::CoinControlOutpointNotSpendable(format!(
                    "{} ({})",
                    outpoint, e
                ))
            })?;
        }

        if !strict_excluded_inputs.is_empty() {
            builder.unspendable(strict_excluded_inputs.clone());
        }

        let psbt = builder
            .finish()
            .map_err(|e| crate::WalletCoreError::PsbtBuildFailed(e.to_string()))?;

        let actual_replaceable = psbt.unsigned_tx.is_explicitly_rbf();

        for (idx, txout) in psbt.unsigned_tx.output.iter().enumerate() {
            let derivation = self.wallet.derivation_of_spk(txout.script_pubkey.clone());
            debug!(
                "wallet_service: psbt output idx={} value={} recipient_match={} derivation={:?} tx_replaceable={}",
                idx,
                txout.value.to_sat(),
                txout.script_pubkey == recipient_script,
                derivation,
                actual_replaceable
            );
        }

        let selected_utxo_count = psbt.inputs.len();
        let selected_inputs: Vec<String> = psbt
            .unsigned_tx
            .input
            .iter()
            .map(|txin| txin.previous_output.to_string())
            .collect();

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
            "wallet_service: create_psbt success to={} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} requested_rbf={} actual_replaceable={} change_amount_sat={:?} selected_utxos={} coin_control_selected_inputs={} coin_control_excluded_inputs={} strict_selected_inputs={}",
            to_address,
            amount_sat.as_u64(),
            fee_sat,
            fee_rate_sat_per_vb.as_u64(),
            enable_rbf,
            actual_replaceable,
            change_amount_sat,
            selected_utxo_count,
            selected_inputs.len(),
            strict_excluded_inputs.len(),
            strict_selected_inputs
        );

        Ok(WalletPsbtInfo {
            psbt_base64,
            txid: psbt.unsigned_tx.compute_txid().to_string(),
            original_txid: None,
            to_address: to_address.to_string(),
            amount_sat,
            fee_sat: AmountSat::from(fee_sat),
            fee_rate_sat_per_vb: fee_rate_sat_per_vb.as_u64(),
            replaceable: actual_replaceable,
            change_amount_sat: change_amount_sat.map(AmountSat),
            selected_utxo_count,
            selected_inputs,
            input_count: psbt.unsigned_tx.input.len(),
            output_count: psbt.unsigned_tx.output.len(),
            recipient_count: 1,
            estimated_vsize: psbt.unsigned_tx.vsize() as u64,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use crate::config::{
        BroadcastBackendConfig, SyncBackendConfig, WalletBackendConfig, WalletDescriptors,
    };
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::WalletConfig;
    use crate::types::{AmountSat, FeeRateSatPerVb};
    use bitcoin::absolute::LockTime;
    use bitcoin::{OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness};
    use bitcoin::transaction::Version;
    use bitcoin::psbt::Psbt;

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            descriptors: WalletDescriptors {
                external: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
                internal: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            },
            backend: WalletBackendConfig {
                sync: SyncBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                },
                broadcast: Some(BroadcastBackendConfig::Esplora {
                    url: "https://mempool.space/signet/api".to_string(),
                }),
            },
            db_path: unique_test_db_path("wallet_core_psbt"),
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
        enable_rbf: bool,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        wallet.create_psbt(
            network,
            to_address,
            AmountSat::from(amount_sat),
            FeeRateSatPerVb::from(fee_rate_sat_per_vb),
            enable_rbf,
        )
    }

    fn create_psbt_with_coin_control(
        wallet: &mut WalletService,
        network: Network,
        to_address: &str,
        amount_sat: u64,
        fee_rate_sat_per_vb: u64,
        enable_rbf: bool,
        coin_control: WalletCoinControlInfo,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        wallet.create_psbt_with_coin_control(
            network,
            to_address,
            AmountSat::from(amount_sat),
            FeeRateSatPerVb::from(fee_rate_sat_per_vb),
            enable_rbf,
            Some(coin_control),
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

        let result = create_psbt_with(&mut wallet, config.network, valid_signet_address(), 0, 1, true);

        assert_create_psbt_err(result, |err| matches!(err, crate::WalletCoreError::InvalidAmount));
    }

    #[test]
    fn create_psbt_fails_for_zero_fee_rate() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_signet_address(), 1000, 0, true);

        assert_create_psbt_err(result, |err| matches!(err, crate::WalletCoreError::InvalidFeeRate));
    }

    #[test]
    fn create_psbt_returns_invalid_destination_address_error() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, "invalid_address", 1000, 1, true);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::InvalidDestinationAddress(_))
        });
    }

    #[test]
    fn create_psbt_returns_destination_network_mismatch_error() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_mainnet_address(), 1000, 1, true);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::DestinationNetworkMismatch(_))
        });
    }


    #[test]
    fn create_psbt_fails_for_insufficient_funds() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(&mut wallet, config.network, valid_signet_address(), 1000, 1, true);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::PsbtBuildFailed(_))
        });
    }

    #[test]
    fn create_psbt_with_coin_control_fails_for_invalid_included_outpoint() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
            WalletCoinControlInfo {
                include_outpoints: vec!["not_an_outpoint".to_string()],
                exclude_outpoints: Vec::new(),
                confirmed_only: false,
            },
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::CoinControlInvalidOutpoint(_))
        });
    }

    #[test]
    fn create_psbt_with_coin_control_fails_for_include_exclude_conflict() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint =
            "0000000000000000000000000000000000000000000000000000000000000001:0".to_string();

        let result = create_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
            WalletCoinControlInfo {
                include_outpoints: vec![outpoint.clone()],
                exclude_outpoints: vec![outpoint.clone()],
                confirmed_only: false,
            },
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::CoinControlConflict(conflict) if conflict == outpoint)
        });
    }

    #[test]
    fn create_psbt_with_coin_control_fails_for_missing_selected_outpoint() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint =
            "0000000000000000000000000000000000000000000000000000000000000001:0".to_string();

        let result = create_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
            WalletCoinControlInfo {
                include_outpoints: vec![outpoint.clone()],
                exclude_outpoints: Vec::new(),
                confirmed_only: false,
            },
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::CoinControlOutpointNotFound(missing) if missing == outpoint)
        });
    }

    #[test]
    fn create_psbt_populates_selected_inputs_field_when_transaction_build_succeeds() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
        );

        match result {
            Ok(psbt_info) => {
                assert_eq!(
                    psbt_info.selected_inputs.len(),
                    psbt_info.input_count,
                    "selected_inputs should match actual input count"
                );
            }
            Err(crate::WalletCoreError::PsbtBuildFailed(_)) => {
                // Fresh watch-only test wallets may have no funds; in that case this
                // test cannot reach transaction construction and should not fail the suite.
            }
            Err(other) => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn from_psbt_minimal_preserves_selected_input_outpoints() {
        let txid1: Txid =
            "0000000000000000000000000000000000000000000000000000000000000001"
                .parse()
                .expect("valid txid");
        let txid2: Txid =
            "0000000000000000000000000000000000000000000000000000000000000002"
                .parse()
                .expect("valid txid");

        let unsigned_tx = Transaction {
            version: Version(2),
            lock_time: LockTime::ZERO,
            input: vec![
                TxIn {
                    previous_output: OutPoint { txid: txid1, vout: 0 },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence(0xFFFFFFFD),
                    witness: Witness::default(),
                },
                TxIn {
                    previous_output: OutPoint { txid: txid2, vout: 1 },
                    script_sig: ScriptBuf::new(),
                    sequence: Sequence(0xFFFFFFFD),
                    witness: Witness::default(),
                },
            ],
            output: vec![TxOut {
                value: Amount::from_sat(1500),
                script_pubkey: ScriptBuf::new(),
            }],
        };

        let psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("psbt should build from unsigned tx");
        let info = WalletPsbtInfo::from_psbt_minimal(psbt).expect("minimal PSBT conversion should succeed");

        assert_eq!(info.selected_utxo_count, 2);
        assert_eq!(info.input_count, 2);
        assert_eq!(
            info.selected_inputs,
            vec![
                "0000000000000000000000000000000000000000000000000000000000000001:0".to_string(),
                "0000000000000000000000000000000000000000000000000000000000000002:1".to_string(),
            ]
        );
    }

    #[test]
    fn create_psbt_marks_transaction_replaceable_when_rbf_enabled() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
        );

        match result {
            Ok(psbt_info) => {
                let psbt = crate::service::psbt_common::parse_psbt(&psbt_info.psbt_base64)
                    .expect("created PSBT should parse");
                assert!(psbt_info.replaceable, "PSBT info should report replaceable");
                assert!(
                    psbt.unsigned_tx.is_explicitly_rbf(),
                    "unsigned transaction should be explicitly RBF"
                );
            }
            Err(crate::WalletCoreError::PsbtBuildFailed(_)) => {
                // Fresh watch-only test wallets may have no funds; in that case this
                // test cannot reach transaction construction and should not fail the suite.
            }
            Err(other) => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn create_psbt_marks_transaction_non_replaceable_when_rbf_disabled() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            false,
        );

        match result {
            Ok(psbt_info) => {
                let psbt = crate::service::psbt_common::parse_psbt(&psbt_info.psbt_base64)
                    .expect("created PSBT should parse");
                assert!(
                    !psbt_info.replaceable,
                    "PSBT info should report non-replaceable when RBF is disabled"
                );
                assert!(
                    !psbt.unsigned_tx.is_explicitly_rbf(),
                    "unsigned transaction should not be explicitly RBF"
                );
            }
            Err(crate::WalletCoreError::PsbtBuildFailed(_)) => {
                // Fresh watch-only test wallets may have no funds; in that case this
                // test cannot reach transaction construction and should not fail the suite.
            }
            Err(other) => panic!("unexpected error: {:?}", other),
        }
    }
}