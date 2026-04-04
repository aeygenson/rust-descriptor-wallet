use bdk_chain::ChainPosition;
use tracing::debug;

use crate::model::WalletTxInfo;
use crate::types::{AmountSat, TxDirection};
use super::*;
impl WalletService {
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
            let txid = tx.tx_node.txid.to_string();

            let (sent, received) = self.wallet.sent_and_received(&tx.tx_node.tx);
            let sent_sat = sent.to_sat();
            let received_sat = received.to_sat();
            let net_value = received_sat as i64 - sent_sat as i64;

            let has_wallet_inputs = sent_sat > 0;
            let has_wallet_outputs = received_sat > 0;

            let direction = if !has_wallet_inputs && has_wallet_outputs {
                TxDirection::Received
            } else if has_wallet_inputs && has_wallet_outputs {
                TxDirection::SelfTransfer
            } else {
                TxDirection::Sent
            };

            let fee = if direction == TxDirection::Received {
                None
            } else {
                self
                    .wallet
                    .calculate_fee(&tx.tx_node.tx)
                    .ok()
                    .map(|amount| AmountSat(amount.to_sat()))
            };

            // Determine confirmation status and height from chain position
            let (confirmed, confirmation_height) = match tx.chain_position {
                ChainPosition::Confirmed { anchor, .. } => (true, Some(anchor.block_id.height)),
                ChainPosition::Unconfirmed { .. } => (false, None),
            };

            result.push(WalletTxInfo {
                txid,
                confirmed,
                confirmation_height,
                direction,
                net_value,
                fee,
            });
        }

        debug!("wallet_service: transactions count={}", result.len());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use crate::WalletConfig;
    
    fn test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            external_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
            internal_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            db_path: unique_test_db_path("wallet_core_txs"),
            esplora_url: "https://mempool.space/signet/api".to_string(),
        }
    }

    fn unique_test_db_path(prefix: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time before UNIX_EPOCH")
            .as_nanos();

        std::env::temp_dir().join(format!("{}_{}_{}.db", prefix, std::process::id(), nanos))
    }

    #[test]
    fn transactions_empty_for_fresh_wallet() {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let txs = wallet.transactions();

        assert!(txs.is_empty(), "fresh wallet should have no transactions");
    }

    #[test]
    fn transactions_have_consistent_fields() {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let txs = wallet.transactions();

        for tx in txs {
            assert!(!tx.txid.is_empty(), "txid should not be empty");
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