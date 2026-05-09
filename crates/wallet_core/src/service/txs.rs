use bdk_chain::ChainPosition;
use tracing::debug;

use super::common_outpoint::{group_outpoints_by_txid, outpoint_txid};
use super::common_tx::{
    classify_tx_direction, fee_rate_sat_per_vb_from_fee_and_vsize, is_rbf_enabled,
};
use super::*;
use crate::model::{WalletTxInfo, WalletTxInputInfo};
use crate::types::{AmountSat, BlockHeight, TxDirection, WalletTxid};
impl WalletService {
    /// This transaction view operates on wallet transactions already stored in
    /// the BDK graph and exposes a read-only transaction summary for CLI/API/UI.
    ///
    /// It now includes typed inputs and wallet-owned spendable outputs so the UI
    /// can inspect parent/child transaction relationships and derive CPFP
    /// candidate outpoints without guessing.
    ///
    /// Return list of wallet transactions (basic view).
    ///
    /// This reads transaction data from the underlying BDK wallet.
    /// No network calls are performed — data must be synced beforehand.
    ///
    /// Currently returns:
    /// - txid
    /// - confirmation status
    /// - confirmation height (if available)
    /// - direction (`received`, `sent`, `self`)
    /// - net value in satoshis
    /// - optional fee in satoshis
    /// - optional fee rate in sat/vB
    /// - previous outpoints spent by the transaction inputs
    /// - wallet-owned spendable outputs currently visible in the wallet UTXO set
    ///
    /// Future improvements may include:
    /// - timestamps
    /// - richer transaction classification
    /// - all transaction outputs, including non-wallet recipient outputs
    pub fn transactions(&self) -> Vec<WalletTxInfo> {
        debug!("wallet_service: transactions start");

        // BDK stores transactions in its internal graph.
        // We iterate over all known transactions and map them
        // into our core domain model (WalletTxInfo).
        //
        // For each transaction we compute:
        // - sent amount from wallet-owned inputs
        // - received amount to wallet-owned outputs
        // - net value (received - sent)
        // - direction string for simple CLI display
        // - optional fee when BDK can calculate it from known inputs
        //
        // Direction rules for now:
        // - received: wallet only gains funds
        // - sent: wallet spends funds without any wallet-owned outputs coming back
        // - self: wallet spends funds and also receives wallet-owned outputs back
        let mut result = Vec::new();

        for tx in self.wallet.transactions() {
            let txid = WalletTxid::from(tx.tx_node.txid);

            let (sent, received) = self.wallet.sent_and_received(&tx.tx_node.tx);
            let sent_sat = sent.to_sat();
            let received_sat = received.to_sat();
            let net_value = received_sat as i64 - sent_sat as i64;

            let direction = classify_tx_direction(sent_sat, received_sat, net_value);

            let fee = if direction == TxDirection::Received {
                None
            } else {
                self.wallet
                    .calculate_fee(&tx.tx_node.tx)
                    .ok()
                    .map(|amount| AmountSat(amount.to_sat()))
            };

            let fee_rate_sat_per_vb = fee.as_ref().map(|fee_sat| {
                fee_rate_sat_per_vb_from_fee_and_vsize(
                    fee_sat.as_u64(),
                    tx.tx_node.tx.vsize() as u64,
                )
            });

            let replaceable = is_rbf_enabled(&tx.tx_node.tx);

            let inputs: Vec<WalletTxInputInfo> = tx
                .tx_node
                .tx
                .input
                .iter()
                .map(|input| WalletTxInputInfo {
                    previous_outpoint: input.previous_output.into(),
                })
                .collect();

            let previous_outpoints: Vec<_> = inputs
                .iter()
                .map(|input| input.previous_outpoint)
                .collect();
            let grouped_parent_outpoints = group_outpoints_by_txid(&previous_outpoints);
            debug!(
                txid = %txid,
                input_count = inputs.len(),
                parent_tx_count = grouped_parent_outpoints.len(),
                "wallet_service: transactions input parent grouping"
            );

            // Determine confirmation status and height from chain position
            let (confirmed, confirmation_height) = match tx.chain_position {
                ChainPosition::Confirmed { anchor, .. } => {
                    (true, Some(BlockHeight::from(anchor.block_id.height)))
                }
                ChainPosition::Unconfirmed { .. } => (false, None),
            };

            result.push(WalletTxInfo {
                txid,
                confirmed,
                confirmation_height,
                direction,
                net_value,
                fee,
                replaceable,
                fee_rate_sat_per_vb,
                inputs,
                outputs: self.wallet_owned_outputs_for_txid(&txid.to_string()),
            });
        }

        debug!("wallet_service: transactions count={}", result.len());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::service::common_test_util::test_support::test_config_with_db_prefix;

    #[test]
    fn transactions_empty_for_fresh_wallet() {
        let config = test_config_with_db_prefix("wallet_core_txs");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let txs = wallet.transactions();

        assert!(txs.is_empty(), "fresh wallet should have no transactions");
    }

    #[test]
    fn transactions_have_consistent_fields() {
        let config = test_config_with_db_prefix("wallet_core_txs");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let txs = wallet.transactions();

        for tx in txs {
            assert!(!tx.txid.to_string().is_empty(), "txid should not be empty");
            // direction must be one of expected values
            assert!(
                matches!(
                    tx.direction,
                    TxDirection::Received | TxDirection::Sent | TxDirection::SelfTransfer
                ),
                "unexpected direction"
            );

            if let Some(fee) = tx.fee {
                assert!(fee.as_u64() > 0, "fee should be positive");
            }

            if matches!(tx.direction, TxDirection::Received) {
                assert!(tx.net_value >= 0, "received tx should not be negative");
            }

            for input in &tx.inputs {
                assert!(
                    input.previous_outpoint.to_string().contains(':'),
                    "input previous outpoint should be in txid:vout form"
                );
                assert_eq!(
                    outpoint_txid(&input.previous_outpoint).to_string().len(),
                    64,
                    "input previous txid should be canonical hex"
                );
            }

            for output in &tx.outputs {
                assert!(
                    output.outpoint.to_string().contains(':'),
                    "output outpoint should be in txid:vout form"
                );
                assert!(output.value.as_u64() > 0, "output value should be positive");
                assert!(output.is_mine, "transaction outputs should be wallet-owned");
            }
        }
    }

    #[test]
    fn transaction_input_parent_grouping_handles_empty_projection() {
        let config = test_config_with_db_prefix("wallet_core_txs_grouping");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let txs = wallet.transactions();
        let all_inputs: Vec<_> = txs
            .iter()
            .flat_map(|tx| tx.inputs.iter().map(|input| input.previous_outpoint))
            .collect();
        let grouped = group_outpoints_by_txid(&all_inputs);

        assert!(grouped.is_empty());
    }
}
