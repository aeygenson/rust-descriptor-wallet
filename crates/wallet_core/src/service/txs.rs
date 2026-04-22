use bdk_chain::ChainPosition;
use tracing::debug;

use super::common_tx::{
    classify_tx_direction, fee_rate_sat_per_vb_from_fee_and_vsize, is_rbf_enabled,
};
use super::*;
use crate::model::WalletTxInfo;
use crate::types::{AmountSat, BlockHeight, TxDirection, WalletTxid};
impl WalletService {
    /// This transaction view operates on wallet transactions already stored in
    /// the BDK graph and does not participate in typed outpoint selection.
    ///
    /// The recent `WalletOutPoint` migration affects coin-control, sweep,
    /// consolidation, and PSBT-input selection flows, but not this read-only
    /// transaction summary path.
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
    ///
    /// Future improvements may include:
    /// - timestamps
    /// - richer transaction classification
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
        }
    }
}
