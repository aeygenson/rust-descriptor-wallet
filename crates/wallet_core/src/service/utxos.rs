use super::common_outpoint::outpoint_txid;
use super::*;
use bdk_chain::ChainPosition;
use bdk_wallet::KeychainKind;
use tracing::debug;

use crate::model::WalletUtxoInfo;
use crate::types::{AmountSat, BlockHeight, WalletKeychain, WalletOutPoint, WalletTxid};

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
            let outpoint = WalletOutPoint::from(utxo.outpoint);
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
                ChainPosition::Confirmed { anchor, .. } => {
                    (true, Some(BlockHeight::from(anchor.block_id.height)))
                }
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

    /// Return all wallet UTXOs belonging to the given parent transaction id.
    pub fn utxos_for_txid(&self, txid: &str) -> Vec<WalletUtxoInfo> {
        let txid = WalletTxid::parse(txid);
        self.utxos()
            .into_iter()
            .filter(|u| match txid {
                Ok(ref parsed) => outpoint_txid(&u.outpoint) == *parsed,
                Err(_) => false,
            })
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
    use crate::service::common_test_util::test_support::test_config_with_db_prefix;
    use crate::types::WalletKeychain;

    #[test]
    fn utxos_empty_for_fresh_wallet() {
        let config = test_config_with_db_prefix("wallet_core_utxos");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let utxos = wallet.utxos();

        assert!(utxos.is_empty(), "fresh wallet should have no utxos");
    }

    #[test]
    fn utxos_have_consistent_fields() {
        let config = test_config_with_db_prefix("wallet_core_utxos");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let utxos = wallet.utxos();

        for u in utxos {
            assert!(
                u.outpoint.to_string().contains(':'),
                "outpoint should be in txid:vout form"
            );
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
}
