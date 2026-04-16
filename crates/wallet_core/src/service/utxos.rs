use super::*;
use bdk_chain::ChainPosition;
use bdk_wallet::KeychainKind;
use tracing::debug;

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
                confirmation_height,
                address,
                keychain,
            });
        }

        debug!("wallet_service: utxos count={}", result.len());
        result
    }

    /// Return the transaction id portion of an outpoint string (`txid:vout`).
    pub fn outpoint_txid(outpoint: &str) -> &str {
        outpoint.split(':').next().unwrap_or("")
    }

    /// Return all wallet UTXOs belonging to the given parent transaction id.
    pub fn utxos_for_txid(&self, txid: &str) -> Vec<WalletUtxoInfo> {
        self.utxos()
            .into_iter()
            .filter(|u| Self::outpoint_txid(&u.outpoint) == txid)
            .collect()
    }

    /// Return unconfirmed wallet UTXOs belonging to the given parent transaction id.
    pub fn unconfirmed_utxos_for_txid(&self, txid: &str) -> Vec<WalletUtxoInfo> {
        self.utxos_for_txid(txid)
            .into_iter()
            .filter(|u| !u.confirmed)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{
        BroadcastBackendConfig, SyncBackendConfig, WalletBackendConfig, WalletDescriptors,
    };
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
            db_path: unique_test_db_path("wallet_core_utxos"),
            is_watch_only: true,
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
            if let Some(address) = &u.address {
                assert!(!address.is_empty(), "derived address should not be empty");
            }
            // value uses AmountSat, so non-negativity is enforced by type
            assert!(
                matches!(
                    u.keychain,
                    WalletKeychain::External | WalletKeychain::Internal
                ),
                "unexpected keychain"
            );
        }
    }

    #[test]
    fn outpoint_txid_extracts_txid_prefix() {
        let txid = WalletService::outpoint_txid(
            "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee:1",
        );
        assert_eq!(
            txid,
            "b09f4f973fdc20fdad67ee670572037a1e8fec94848bca9293f78e89e26667ee"
        );
    }

    #[test]
    fn outpoint_txid_returns_whole_string_when_separator_missing() {
        let txid = WalletService::outpoint_txid("not_an_outpoint");
        assert_eq!(txid, "not_an_outpoint");
    }
}
