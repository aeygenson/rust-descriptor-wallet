use bdk_chain::ChainPosition;
use bdk_wallet::KeychainKind;
use tracing::debug;
use super::*;
use crate::model::WalletUtxoInfo;
use crate::types::{AmountSat, WalletKeychain};
impl WalletService {
    /// Return list of wallet UTXOs (basic view).
    ///
    /// This reads spendable outputs from the underlying BDK wallet.
    /// No network calls are performed — data must be synced beforehand.
    ///
    /// Currently also includes:
    /// - address (when derivation data is available)
    /// - keychain kind (`external` / `internal`)
    ///
    /// Future improvements may include:
    /// - spendability flags
    pub fn utxos(&self) -> Vec<WalletUtxoInfo> {
        debug!("wallet_service: utxos start");

        let mut result = Vec::new();

        for utxo in self.wallet.list_unspent() {
            let outpoint = utxo.outpoint.to_string();
            let value = AmountSat(utxo.txout.value.to_sat());

            let address = Some(
                self.wallet
                    .peek_address(utxo.keychain, utxo.derivation_index)
                    .address
                    .to_string(),
            );

            let keychain = match utxo.keychain {
                KeychainKind::External => WalletKeychain::External,
                KeychainKind::Internal => WalletKeychain::Internal,
            };

            let (confirmed, confirmation_height) = match utxo.chain_position {
                ChainPosition::Confirmed { anchor, .. } => (true, Some(anchor.block_id.height)),
                ChainPosition::Unconfirmed { .. } => (false, None),
            };

            result.push(WalletUtxoInfo {
                outpoint,
                value,
                confirmed,
                confirmation_height,                address,
                keychain,
            });
        }

        debug!("wallet_service: utxos count={}", result.len());
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use bitcoin::Network;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU64, Ordering};
    use std::time::{SystemTime, UNIX_EPOCH};

    use crate::types::WalletKeychain;
    use crate::WalletConfig;

    static TEST_DB_COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_config() -> WalletConfig {
        WalletConfig {
            network: Network::Signet,
            external_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/0/*)#z3x5097m".to_string(),
            internal_descriptor: "tr([12071a7c/86'/1'/0']tpubDCaLkqfh67Qr7ZuRrUNrCYQ54sMjHfsJ4yQSGb3aBr1yqt3yXpamRBUwnGSnyNnxQYu7rqeBiPfw3mjBcFNX4ky2vhjj9bDrGstkfUbLB9T/1/*)#n9r4jswr".to_string(),
            db_path: unique_test_db_path("wallet_core_utxos"),
            esplora_url: "https://mempool.space/signet/api".to_string(),
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

    #[test]
    fn utxos_empty_for_fresh_wallet() {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let utxos = wallet.utxos();

        assert!(utxos.is_empty(), "fresh wallet should have no utxos");
    }

    #[test]
    fn utxos_have_consistent_fields() {
        let config = test_config();
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let utxos = wallet.utxos();

        for u in utxos {
            assert!(!u.outpoint.is_empty(), "outpoint should not be empty");
            // value uses AmountSat, so non-negativity is enforced by type
            assert!(
                matches!(u.keychain, WalletKeychain::External | WalletKeychain::Internal),
                "unexpected keychain"
            );
        }
    }
}