use super::common_outpoint::outpoint_txid;
use super::*;
use bdk_chain::ChainPosition;
use bdk_wallet::KeychainKind;
use tracing::debug;

use crate::model::{WalletTxOutputInfo, WalletUtxoInfo};
use crate::types::{
    AddressIndex, AmountSat, BlockHeight, WalletKeychain, WalletOutPoint, WalletTxid,
};

impl WalletService {
    /// Return list of wallet UTXOs (basic view).
    ///
    /// This reads spendable outputs from the underlying BDK wallet.
    /// No network calls are performed — data must be synced beforehand.
    ///
    /// Currently also includes:
    /// - address derived from wallet keychain/index metadata
    /// - keychain kind (`external` / `internal`)
    /// - derivation index as a typed `AddressIndex`
    ///
    /// Future improvements may include:
    /// - spendability flags
    /// - freeze/lock state
    /// - label metadata
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
                derivation_index: Some(AddressIndex::from(utxo.derivation_index)),
            });
        }

        debug!(
            total = result.len(),
            external = result
                .iter()
                .filter(|u| u.keychain == WalletKeychain::External)
                .count(),
            internal = result
                .iter()
                .filter(|u| u.keychain == WalletKeychain::Internal)
                .count(),
            with_derivation_index = result
                .iter()
                .filter(|u| u.derivation_index.is_some())
                .count(),
            "wallet_service: utxos collected"
        );
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

    /// Return wallet-owned outputs belonging to the given transaction id.
    ///
    /// This is intentionally based on the current wallet UTXO set, so it only returns
    /// outputs that are still spendable by this wallet. That makes it suitable for
    /// CPFP candidate selection in the UI.
    pub fn wallet_owned_outputs_for_txid(&self, txid: &str) -> Vec<WalletTxOutputInfo> {
        self.utxos_for_txid(txid)
            .into_iter()
            .map(|utxo| WalletTxOutputInfo {
                outpoint: utxo.outpoint,
                value: utxo.value,
                address: utxo.address,
                is_mine: true,
                keychain: Some(utxo.keychain),
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

    /// Return unconfirmed wallet-owned outputs belonging to the given transaction id.
    ///
    /// These are the direct CPFP candidates for an unconfirmed parent transaction.
    pub fn unconfirmed_wallet_owned_outputs_for_txid(&self, txid: &str) -> Vec<WalletTxOutputInfo> {
        self.unconfirmed_utxos_for_txid(txid)
            .into_iter()
            .map(|utxo| WalletTxOutputInfo {
                outpoint: utxo.outpoint,
                value: utxo.value,
                address: utxo.address,
                is_mine: true,
                keychain: Some(utxo.keychain),
            })
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
            if let Some(index) = u.derivation_index {
                assert_eq!(index.as_u32(), u32::from(index));
            }
            assert!(
                u.derivation_index.is_some(),
                "wallet utxo projection should preserve derivation index"
            );
        }
    }

    #[test]
    fn wallet_owned_outputs_for_missing_txid_is_empty() {
        let config = test_config_with_db_prefix("wallet_core_utxos_outputs_missing");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let outputs = wallet.wallet_owned_outputs_for_txid(
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        assert!(
            outputs.is_empty(),
            "missing txid should have no owned outputs"
        );
    }

    #[test]
    fn unconfirmed_wallet_owned_outputs_for_missing_txid_is_empty() {
        let config = test_config_with_db_prefix("wallet_core_utxos_unconfirmed_outputs_missing");
        let wallet = WalletService::load_or_create(&config)
            .expect("wallet should load or create successfully");

        let outputs = wallet.unconfirmed_wallet_owned_outputs_for_txid(
            "0000000000000000000000000000000000000000000000000000000000000000",
        );

        assert!(
            outputs.is_empty(),
            "missing txid should have no unconfirmed owned outputs"
        );
    }
}
