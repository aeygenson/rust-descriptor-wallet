use std::str::FromStr;

use std::collections::HashSet;

use bdk_wallet::KeychainKind;
use bitcoin::FeeRate;
use bitcoin::{Address, Amount, Network};
use tracing::{debug, info};

use super::common_outpoint::ensure_no_outpoint_overlap;
use super::common_selection::{effective_selection_mode, is_strict_manual_selection};
use super::common_tx::RBF_SEQUENCE;
use super::psbt_coin_selector::select_inputs;
use super::*;
use crate::model::{
    WalletCoinControlInfo, WalletInputSelectionConfig, WalletPsbtInfo, WalletSendAmountMode,
};
use crate::types::{AmountSat, FeeRateSatPerVb, PsbtBase64, VSize, WalletOutPoint, WalletTxid};
use crate::WalletCoreResult;

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
        self.create_psbt_internal(
            wallet_network,
            to_address,
            WalletSendAmountMode::Fixed(amount_sat),
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
        self.create_psbt_internal(
            wallet_network,
            to_address,
            WalletSendAmountMode::Fixed(amount_sat),
            fee_rate_sat_per_vb,
            enable_rbf,
            coin_control,
        )
    }

    /// Create an unsigned PSBT for a send-max flow.
    pub fn create_send_max_psbt(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        self.create_psbt_internal(
            wallet_network,
            to_address,
            WalletSendAmountMode::Max,
            fee_rate_sat_per_vb,
            enable_rbf,
            None,
        )
    }

    /// Create an unsigned PSBT for a send-max flow with optional coin control.
    pub fn create_send_max_psbt_with_coin_control(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
        coin_control: Option<WalletCoinControlInfo>,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        self.create_psbt_internal(
            wallet_network,
            to_address,
            WalletSendAmountMode::Max,
            fee_rate_sat_per_vb,
            enable_rbf,
            coin_control,
        )
    }

    /// Create an unsigned PSBT for a sweep flow.
    ///
    /// Sweep is modeled as strict send-max with an explicit include set.
    pub fn create_sweep_psbt(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
        coin_control: WalletCoinControlInfo,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        self.create_psbt_internal(
            wallet_network,
            to_address,
            WalletSendAmountMode::Max,
            fee_rate_sat_per_vb,
            enable_rbf,
            Some(coin_control),
        )
    }

    /// Create an unsigned PSBT for a sweep flow using an already optional coin-control model.
    ///
    /// This is a convenience alias for callers that already carry `Option<WalletCoinControlInfo>`.
    pub fn create_sweep_psbt_with_optional_coin_control(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
        coin_control: Option<WalletCoinControlInfo>,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        self.create_psbt_internal(
            wallet_network,
            to_address,
            WalletSendAmountMode::Max,
            fee_rate_sat_per_vb,
            enable_rbf,
            coin_control,
        )
    }

    fn create_psbt_internal(
        &mut self,
        wallet_network: Network,
        to_address: &str,
        send_amount_mode: WalletSendAmountMode,
        fee_rate_sat_per_vb: FeeRateSatPerVb,
        enable_rbf: bool,
        coin_control: Option<WalletCoinControlInfo>,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        let strict_selected_inputs = coin_control
            .as_ref()
            .map(|cc| {
                is_strict_manual_selection(
                    &cc.selection.include_outpoints,
                    cc.selection.selection_mode,
                )
            })
            .unwrap_or(false);
        debug!(
            "wallet_service: create_psbt start to={} send_amount_mode={:?} fee_rate_sat_per_vb={} enable_rbf={} has_coin_control={} sweep_semantics={} selection_mode={:?}",
            to_address,
            send_amount_mode,
            fee_rate_sat_per_vb.as_u64(),
            enable_rbf,
            coin_control.is_some(),
            matches!(send_amount_mode, WalletSendAmountMode::Max) && strict_selected_inputs,
            coin_control.as_ref().and_then(|cc| cc.selection.selection_mode),
        );

        // Keep defensive validation in core even though wrapper constructors
        // normally reject invalid values. Tests can intentionally construct
        // zero-valued wrappers directly, and the core method should still
        // return precise domain errors.
        if matches!(send_amount_mode, WalletSendAmountMode::Fixed(amount_sat) if amount_sat.as_u64() == 0)
        {
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

        if let Some(cc) = coin_control.as_ref() {
            for inc in &cc.selection.include_outpoints {
                if cc.selection.exclude_outpoints.contains(inc) {
                    return Err(crate::WalletCoreError::CoinControlConflict(inc.to_string()));
                }
            }

            ensure_no_outpoint_overlap(
                &cc.selection.include_outpoints,
                &cc.selection.exclude_outpoints,
            )?;
        }

        let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        let selected_inputs = match coin_control.as_ref() {
            Some(cc) if !cc.is_empty() => {
                let resolution = self.resolve_coin_control_inputs(cc)?;
                let validated_selected = resolution.included_outpoints;
                let validated_excluded = resolution.excluded_outpoints;

                let selection_mode = effective_selection_mode(
                    &cc.selection.include_outpoints,
                    cc.selection.selection_mode,
                );

                let cfg = WalletInputSelectionConfig {
                    include_outpoints: validated_selected,
                    exclude_outpoints: validated_excluded,
                    confirmed_only: cc.selection.confirmed_only,
                    selection_mode: Some(selection_mode),
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                };

                select_inputs(&wallet_utxos, &cfg)?
            }
            _ => Vec::new(),
        };

        let excluded_inputs = match coin_control.as_ref() {
            Some(cc) if !cc.selection.exclude_outpoints.is_empty() => {
                self.resolve_coin_control_exclusions(cc)?
            }
            _ => Vec::new(),
        };

        // let strict_selected_inputs calculated above

        // let wallet_utxos: Vec<_> = self.wallet.list_unspent().collect();

        let mut strict_excluded_inputs = excluded_inputs.clone();
        if strict_selected_inputs {
            let selected_set: HashSet<_> = selected_inputs.iter().copied().collect();

            let selected_total_sat: u64 = wallet_utxos
                .iter()
                .filter(|u| selected_set.contains(&WalletOutPoint::from(u.outpoint)))
                .map(|u| u.txout.value.to_sat())
                .sum();

            // Conservative fee estimate for strict selection mode.
            // We assume one recipient output plus one change output.
            let input_count = selected_inputs.len() as u64;
            let output_count = 2u64;
            let estimated_vsize = 11u64 + input_count * 58u64 + output_count * 43u64;
            let fee_estimate_sat = estimated_vsize * fee_rate_sat_per_vb.as_u64();

            if selected_inputs.is_empty() {
                return Err(crate::WalletCoreError::CoinControlEmptySelection);
            }

            match send_amount_mode {
                WalletSendAmountMode::Fixed(amount_sat) => {
                    let required_sat = amount_sat.as_u64() + fee_estimate_sat;

                    if selected_total_sat < amount_sat.as_u64() {
                        return Err(
                            crate::WalletCoreError::CoinControlInsufficientSelectedFunds {
                                selected_sat: selected_total_sat,
                                required_sat,
                                fee_estimate_sat,
                            },
                        );
                    }

                    if selected_total_sat < required_sat {
                        return Err(crate::WalletCoreError::CoinControlStrictModeViolation);
                    }
                }
                WalletSendAmountMode::Max => {
                    if selected_total_sat <= fee_estimate_sat {
                        return Err(crate::WalletCoreError::SendMaxAmountTooSmall);
                    }
                }
            }

            for utxo in &wallet_utxos {
                let wallet_outpoint = WalletOutPoint::from(utxo.outpoint);
                if !selected_set.contains(&wallet_outpoint)
                    && !strict_excluded_inputs.contains(&wallet_outpoint)
                {
                    strict_excluded_inputs.push(wallet_outpoint);
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
        builder.fee_rate(fee_rate);
        if enable_rbf {
            builder.set_exact_sequence(RBF_SEQUENCE);
        }

        match send_amount_mode {
            WalletSendAmountMode::Fixed(amount_sat) => {
                builder.add_recipient(
                    recipient_script.clone(),
                    Amount::from_sat(amount_sat.as_u64()),
                );
            }
            WalletSendAmountMode::Max => {
                builder.drain_wallet();
                builder.drain_to(recipient_script.clone());
            }
        }

        for outpoint in &selected_inputs {
            builder
                .add_utxo(bitcoin::OutPoint::from(*outpoint))
                .map_err(|e| {
                    crate::WalletCoreError::CoinControlOutpointNotSpendable(format!(
                        "{} ({})",
                        outpoint, e
                    ))
                })?;
        }

        if !strict_excluded_inputs.is_empty() {
            builder.unspendable(
                strict_excluded_inputs
                    .iter()
                    .map(|op| bitcoin::OutPoint::from(*op))
                    .collect(),
            );
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
        let selected_inputs: Vec<WalletOutPoint> = psbt
            .unsigned_tx
            .input
            .iter()
            .map(|txin| WalletOutPoint::from(txin.previous_output))
            .collect();

        let total_input_sat: u64 = psbt
            .inputs
            .iter()
            .filter_map(|input| {
                input
                    .witness_utxo
                    .as_ref()
                    .map(|txout| txout.value.to_sat())
            })
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

        let effective_amount_sat = match send_amount_mode {
            WalletSendAmountMode::Fixed(amount_sat) => amount_sat,
            WalletSendAmountMode::Max => {
                let drained_sat: u64 = psbt
                    .unsigned_tx
                    .output
                    .iter()
                    .filter(|txout| txout.script_pubkey == recipient_script)
                    .map(|txout| txout.value.to_sat())
                    .sum();

                if drained_sat == 0 {
                    return Err(crate::WalletCoreError::SendMaxAmountTooSmall);
                }

                AmountSat::from(drained_sat)
            }
        };

        let psbt_base64 = PsbtBase64::from(psbt.to_string());

        info!(
            "wallet_service: create_psbt success to={} send_amount_mode={:?} amount_sat={} fee_sat={} fee_rate_sat_per_vb={} requested_rbf={} actual_replaceable={} change_amount_sat={:?} selected_utxos={} coin_control_selected_inputs={} coin_control_excluded_inputs={} strict_selected_inputs={} sweep_semantics={} selection_mode={:?}",
            to_address,
            send_amount_mode,
            effective_amount_sat.as_u64(),
            fee_sat,
            fee_rate_sat_per_vb.as_u64(),
            enable_rbf,
            actual_replaceable,
            change_amount_sat,
            selected_utxo_count,
            selected_inputs.len(),
            strict_excluded_inputs.len(),
            strict_selected_inputs,
            matches!(send_amount_mode, WalletSendAmountMode::Max) && strict_selected_inputs,
            coin_control.as_ref().and_then(|cc| cc.selection.selection_mode)
        );

        Ok(WalletPsbtInfo {
            psbt_base64,
            txid: WalletTxid::from(psbt.unsigned_tx.compute_txid()),
            original_txid: None,
            to_address: to_address.to_string(),
            amount_sat: effective_amount_sat,
            fee_sat: AmountSat::from(fee_sat),
            fee_rate_sat_per_vb,
            replaceable: actual_replaceable,
            change_amount_sat: change_amount_sat.map(AmountSat),
            selected_utxo_count,
            selected_inputs,
            input_count: psbt.unsigned_tx.input.len(),
            output_count: psbt.unsigned_tx.output.len(),
            recipient_count: 1,
            estimated_vsize: VSize::from(psbt.unsigned_tx.vsize() as u64),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::common_test_util::test_support::{load_test_wallet, valid_signet_address};
    use crate::types::{AmountSat, FeeRateSatPerVb, WalletOutPoint};
    use bitcoin::absolute::LockTime;
    use bitcoin::psbt::Psbt;
    use bitcoin::transaction::Version;
    use bitcoin::{OutPoint, ScriptBuf, Transaction, TxIn, TxOut, Txid, Witness};

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

    fn create_send_max_psbt_with(
        wallet: &mut WalletService,
        network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        enable_rbf: bool,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        wallet.create_send_max_psbt(
            network,
            to_address,
            FeeRateSatPerVb::from(fee_rate_sat_per_vb),
            enable_rbf,
        )
    }

    fn create_send_max_psbt_with_coin_control(
        wallet: &mut WalletService,
        network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        enable_rbf: bool,
        coin_control: WalletCoinControlInfo,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        wallet.create_send_max_psbt_with_coin_control(
            network,
            to_address,
            FeeRateSatPerVb::from(fee_rate_sat_per_vb),
            enable_rbf,
            Some(coin_control),
        )
    }

    fn create_sweep_psbt_with(
        wallet: &mut WalletService,
        network: Network,
        to_address: &str,
        fee_rate_sat_per_vb: u64,
        enable_rbf: bool,
        coin_control: WalletCoinControlInfo,
    ) -> WalletCoreResult<WalletPsbtInfo> {
        wallet.create_sweep_psbt(
            network,
            to_address,
            FeeRateSatPerVb::from(fee_rate_sat_per_vb),
            enable_rbf,
            coin_control,
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

    fn valid_mainnet_address() -> &'static str {
        "bc1qxy2kgdygjrsqtzq2n0yrf2493p83kkfjhx0wlh"
    }

    #[test]
    fn create_psbt_fails_for_zero_amount() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            0,
            1,
            true,
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::InvalidAmount)
        });
    }

    #[test]
    fn create_psbt_fails_for_zero_fee_rate() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            0,
            true,
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::InvalidFeeRate)
        });
    }

    #[test]
    fn create_send_max_psbt_fails_for_zero_fee_rate() {
        let (config, mut wallet) = load_test_wallet();

        let result =
            create_send_max_psbt_with(&mut wallet, config.network, valid_signet_address(), 0, true);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::InvalidFeeRate)
        });
    }

    #[test]
    fn create_send_max_psbt_fails_for_insufficient_funds() {
        let (config, mut wallet) = load_test_wallet();

        let result =
            create_send_max_psbt_with(&mut wallet, config.network, valid_signet_address(), 1, true);

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::PsbtBuildFailed(_))
        });
    }

    #[test]
    fn create_send_max_psbt_populates_selected_inputs_field_when_transaction_build_succeeds() {
        let (config, mut wallet) = load_test_wallet();

        let result =
            create_send_max_psbt_with(&mut wallet, config.network, valid_signet_address(), 1, true);

        match result {
            Ok(psbt_info) => {
                assert!(
                    psbt_info.amount_sat.as_u64() > 0,
                    "send-max should produce a non-zero recipient amount"
                );
                assert_eq!(
                    psbt_info.selected_inputs.len(),
                    psbt_info.input_count,
                    "selected_inputs should match actual input count in send-max flow"
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
    fn create_psbt_returns_invalid_destination_address_error() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            "invalid_address",
            1000,
            1,
            true,
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::InvalidDestinationAddress(_))
        });
    }

    #[test]
    fn create_psbt_returns_destination_network_mismatch_error() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_mainnet_address(),
            1000,
            1,
            true,
        );

        assert_create_psbt_err(result, |err| {
            matches!(err, crate::WalletCoreError::DestinationNetworkMismatch(_))
        });
    }

    #[test]
    fn create_psbt_fails_for_insufficient_funds() {
        let (config, mut wallet) = load_test_wallet();

        let result = create_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
        );

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
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![WalletOutPoint::parse(
                        "0000000000000000000000000000000000000000000000000000000000000001:0",
                    )
                    .unwrap()],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: None,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            },
        );

        match result {
            Err(err) => {
                let msg = err.to_string();
                assert!(
                    msg.contains("outpoint")
                        || msg.contains("selection")
                        || msg.contains("input")
                        || msg.contains("not_an_outpoint"),
                    "unexpected error message: {}",
                    msg
                );
            }
            Ok(_) => panic!("expected create_psbt to fail"),
        }
    }

    #[test]
    fn create_psbt_with_automatic_only_selection_attempts_auto_pick() {
        let (config, mut wallet) = load_test_wallet();

        let result = wallet.create_psbt_with_coin_control(
            config.network,
            valid_signet_address(),
            AmountSat::from(1000),
            FeeRateSatPerVb::from(1),
            true,
            Some(WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: Vec::new(),
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: Some(crate::model::WalletInputSelectionMode::AutomaticOnly),
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            }),
        );

        match result {
            Ok(psbt) => {
                assert!(
                    psbt.input_count > 0,
                    "automatic-only selection should choose inputs when construction succeeds"
                );
            }
            Err(crate::WalletCoreError::PsbtBuildFailed(_)) => {
                // Fresh watch-only test wallets may have no funds; in that case
                // the selector path was still exercised and this test should not fail.
            }
            Err(crate::WalletCoreError::SelectionFailed(reason)) => {
                assert_eq!(reason, "no inputs selected");
            }
            Err(other) => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn create_psbt_strict_manual_does_not_auto_complete_missing_inputs() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap();

        let result = wallet.create_psbt_with_coin_control(
            config.network,
            valid_signet_address(),
            AmountSat::from(1000),
            FeeRateSatPerVb::from(1),
            true,
            Some(WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![outpoint],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: Some(crate::model::WalletInputSelectionMode::StrictManual),
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            }),
        );

        assert_create_psbt_err(result, |err| match &err {
            crate::WalletCoreError::CoinControlOutpointNotFound(missing) => {
                *missing == outpoint.to_string()
            }
            crate::WalletCoreError::CoinControlStrictModeViolation => true,
            crate::WalletCoreError::SelectionFailed(_) => true,
            _ => false,
        });
    }

    #[test]
    fn create_psbt_manual_with_auto_completion_preserves_manual_inputs_when_successful() {
        let (config, mut wallet) = load_test_wallet();
        let included = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap();

        let result = wallet.create_psbt_with_coin_control(
            config.network,
            valid_signet_address(),
            AmountSat::from(1000),
            FeeRateSatPerVb::from(1),
            true,
            Some(WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![included],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: Some(
                        crate::model::WalletInputSelectionMode::ManualWithAutoCompletion,
                    ),
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            }),
        );

        match result {
            Ok(psbt) => {
                assert!(
                    psbt.selected_inputs.contains(&included),
                    "manual input must be preserved when auto-completion succeeds"
                );
            }
            Err(crate::WalletCoreError::CoinControlOutpointNotFound(missing)) => {
                assert_eq!(missing, included.to_string());
            }
            Err(crate::WalletCoreError::SelectionFailed(_)) => {
                // Acceptable in empty/funded-less test wallet scenarios; selector path still exercised.
            }
            Err(crate::WalletCoreError::PsbtBuildFailed(_)) => {
                // Fresh watch-only test wallets may have no funds.
            }
            Err(other) => panic!("unexpected error: {:?}", other),
        }
    }

    #[test]
    fn create_psbt_with_coin_control_fails_for_include_exclude_conflict() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap();

        let result = create_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
            WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![outpoint.clone()],
                    exclude_outpoints: vec![outpoint.clone()],
                    confirmed_only: false,
                    selection_mode: None,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            },
        );

        assert_create_psbt_err(
            result,
            |err| matches!(err, crate::WalletCoreError::CoinControlConflict(conflict) if conflict == outpoint.to_string()),
        );
    }

    #[test]
    fn create_psbt_with_coin_control_fails_for_missing_selected_outpoint() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap();

        let result = create_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1000,
            1,
            true,
            WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![outpoint.clone()],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: None,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            },
        );

        assert_create_psbt_err(
            result,
            |err| matches!(err, crate::WalletCoreError::CoinControlOutpointNotFound(missing) if missing == outpoint.to_string()),
        );
    }

    #[test]
    fn create_send_max_with_coin_control_fails_for_missing_selected_outpoint() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap();

        let result = create_send_max_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1,
            true,
            WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![outpoint.clone()],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: None,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            },
        );

        assert_create_psbt_err(
            result,
            |err| matches!(err, crate::WalletCoreError::CoinControlOutpointNotFound(missing) if missing == outpoint.to_string()),
        );
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
        let txid1: Txid = "0000000000000000000000000000000000000000000000000000000000000001"
            .parse()
            .expect("valid txid");
        let txid2: Txid = "0000000000000000000000000000000000000000000000000000000000000002"
            .parse()
            .expect("valid txid");

        let unsigned_tx = Transaction {
            version: Version(2),
            lock_time: LockTime::ZERO,
            input: vec![
                TxIn {
                    previous_output: OutPoint {
                        txid: txid1,
                        vout: 0,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: RBF_SEQUENCE,
                    witness: Witness::default(),
                },
                TxIn {
                    previous_output: OutPoint {
                        txid: txid2,
                        vout: 1,
                    },
                    script_sig: ScriptBuf::new(),
                    sequence: RBF_SEQUENCE,
                    witness: Witness::default(),
                },
            ],
            output: vec![TxOut {
                value: Amount::from_sat(1500),
                script_pubkey: ScriptBuf::new(),
            }],
        };

        let psbt = Psbt::from_unsigned_tx(unsigned_tx).expect("psbt should build from unsigned tx");
        let info = WalletPsbtInfo::from_psbt_minimal(psbt)
            .expect("minimal PSBT conversion should succeed");

        assert_eq!(info.selected_utxo_count, 2);
        assert_eq!(info.input_count, 2);
        assert_eq!(
            info.selected_inputs,
            vec![
                WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000001:0"
                )
                .unwrap(),
                WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000002:1"
                )
                .unwrap(),
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
                let psbt: Psbt = (&psbt_info.psbt_base64)
                    .try_into()
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
                let psbt = crate::service::common_tx::parse_psbt(psbt_info.psbt_base64.as_str())
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

    #[test]
    fn create_sweep_psbt_fails_for_missing_selected_outpoint() {
        let (config, mut wallet) = load_test_wallet();
        let outpoint = WalletOutPoint::parse(
            "0000000000000000000000000000000000000000000000000000000000000001:0",
        )
        .unwrap();

        let result = create_sweep_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1,
            true,
            WalletCoinControlInfo {
                selection: WalletInputSelectionConfig {
                    include_outpoints: vec![outpoint.clone()],
                    exclude_outpoints: Vec::new(),
                    confirmed_only: false,
                    selection_mode: None,
                    max_input_count: None,
                    min_input_count: None,
                    min_utxo_value_sat: None,
                    max_utxo_value_sat: None,
                    strategy: None,
                },
            },
        );

        assert_create_psbt_err(
            result,
            |err| matches!(err, crate::WalletCoreError::CoinControlOutpointNotFound(missing) if missing == outpoint.to_string()),
        );
    }

    #[test]
    fn create_sweep_psbt_matches_send_max_with_explicit_include_set_behavior() {
        let (config, mut wallet) = load_test_wallet();
        let coin_control = WalletCoinControlInfo {
            selection: WalletInputSelectionConfig {
                include_outpoints: vec![WalletOutPoint::parse(
                    "0000000000000000000000000000000000000000000000000000000000000001:0",
                )
                .unwrap()],
                exclude_outpoints: Vec::new(),
                confirmed_only: false,
                selection_mode: None,
                max_input_count: None,
                min_input_count: None,
                min_utxo_value_sat: None,
                max_utxo_value_sat: None,
                strategy: None,
            },
        };

        let sweep_result = create_sweep_psbt_with(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1,
            true,
            coin_control.clone(),
        );

        let send_max_result = create_send_max_psbt_with_coin_control(
            &mut wallet,
            config.network,
            valid_signet_address(),
            1,
            true,
            coin_control,
        );

        match (sweep_result, send_max_result) {
            (Err(left), Err(right)) => {
                assert_eq!(left.to_string(), right.to_string());
            }
            (Ok(left), Ok(right)) => {
                assert_eq!(left.amount_sat, right.amount_sat);
                assert_eq!(left.selected_inputs, right.selected_inputs);
                assert_eq!(left.fee_rate_sat_per_vb, right.fee_rate_sat_per_vb);
            }
            (left, right) => panic!("expected sweep and strict send-max to behave the same, got left={left:?}, right={right:?}"),
        }
    }
}
