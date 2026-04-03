use bdk_file_store::Store;
use bdk_wallet::{ChangeSet, KeychainKind, PersistedWallet, Wallet};
use bdk_chain::ChainPosition;

use crate::{WalletConfig, WalletCoreResult};
use crate::model::{WalletTxInfo, WalletUtxoInfo};

use tracing::{debug, info, error};

/// Magic bytes used by `bdk_file_store` to identify our database format.
///
/// Changing this will make existing databases incompatible.
const MAGIC_BYTES: &[u8] = b"rust-descriptor-wallet-v1";

#[derive(Debug)]
pub struct WalletService {
    /// BDK persisted wallet wrapper.
    wallet: PersistedWallet<Store<ChangeSet>>,

    /// File-backed store used by the persisted wallet.
    db: Store<ChangeSet>,
}

impl WalletService {
    /// Load existing wallet or create a new one if it does not exist.
    pub fn load_or_create(config: &WalletConfig) -> WalletCoreResult<Self> {
        info!("wallet_service: load_or_create start path={}", config.db_path.display());
        let (mut db, _changeset) =
            Store::<ChangeSet>::load_or_create(MAGIC_BYTES, &config.db_path)?;

        let external_descriptor = config.external_descriptor.clone();
        let internal_descriptor = config.internal_descriptor.clone();

        let wallet = match Wallet::load()
            .descriptor(KeychainKind::External, Some(external_descriptor.clone()))
            .descriptor(KeychainKind::Internal, Some(internal_descriptor.clone()))
            .check_network(config.network)
            .load_wallet(&mut db)
            .map_err(|e| {
                error!("wallet_service: load_wallet error: {}", e);
                crate::WalletCoreError::Load(e.to_string())
            })?
        {
            None => Wallet::create(
                external_descriptor,
                internal_descriptor,
            )
            .network(config.network)
            .create_wallet(&mut db)
            .map_err(|e| {
                error!("wallet_service: create_wallet error: {}", e);
                crate::WalletCoreError::Create(e.to_string())
            })?,
            Some(wallet) => wallet,
        };

        info!("wallet_service: load_or_create success");
        Ok(Self { wallet, db })
    }

    /// Reveal next receive address.
    ///
    /// Important:
    /// - Advances internal derivation index
    /// - MUST be persisted to avoid address reuse
    pub fn next_receive_address(&mut self) -> WalletCoreResult<String> {
        debug!("wallet_service: next_receive_address start");
        let address_info = self.wallet.reveal_next_address(KeychainKind::External);
        self.persist()?;
        info!("wallet_service: next_receive_address generated address={}", address_info.address);
        Ok(address_info.address.to_string())
    }

    /// Return total wallet balance in satoshis.
    pub fn balance_sat(&self) -> WalletCoreResult<u64> {
        debug!("wallet_service: balance_sat queried");
        Ok(self.wallet.balance().total().to_sat())
    }

    /// Return list of wallet transactions (basic view).
    ///
    /// This reads transaction data from the underlying BDK wallet.
    /// No network calls are performed — data must be synced beforehand.
    ///
    /// Currently returns:
    /// - txid
    /// - confirmation status
    /// - confirmation height (if available)
    /// - direction (`received`, `sent`, `self`, `unknown`)
    /// - net value in satoshis
    /// - optional fee in satoshis
    ///
    /// Future improvements may include:
    /// - timestamps
    /// - more precise self-transfer classification
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
        // - sent: wallet spends funds and sends value externally
        // - self: wallet spends funds but also receives wallet-owned outputs back
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
                "received".to_string()
            } else if has_wallet_inputs && has_wallet_outputs {
                "self".to_string()
            } else if has_wallet_inputs {
                "sent".to_string()
            } else {
                "unknown".to_string()
            };

            let fee = if direction == "received" {
                None
            } else {
                self
                    .wallet
                    .calculate_fee(&tx.tx_node.tx)
                    .ok()
                    .map(|amount| amount.to_sat())
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

        // BDK exposes wallet-owned spendable outputs via `list_unspent()`.
        // We map them into our core domain model (WalletUtxoInfo).
        let mut result = Vec::new();

        for utxo in self.wallet.list_unspent() {
            let outpoint = utxo.outpoint.to_string();
            let value = utxo.txout.value.to_sat();

            let address = Some(
                self.wallet
                    .peek_address(utxo.keychain, utxo.derivation_index)
                    .address
                    .to_string(),
            );

            let keychain = match utxo.keychain {
                KeychainKind::External => "external".to_string(),
                KeychainKind::Internal => "internal".to_string(),
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

    pub fn wallet(&self) -> &Wallet {
        &self.wallet
    }

    /// Mutable access to underlying BDK wallet.
    ///
    /// Used by sync layer to apply blockchain updates.
    pub fn wallet_mut(&mut self) -> &mut Wallet {
        &mut self.wallet
    }

    /// Persist staged wallet changes to disk.
    pub fn persist(&mut self) -> WalletCoreResult<()> {
        debug!("wallet_service: persist start");
        let _ = self
            .wallet
            .persist(&mut self.db)
            .map_err(|e| {
                error!("wallet_service: persist error: {}", e);
                crate::WalletCoreError::Persist(e.to_string())
            })?;
        debug!("wallet_service: persist done");
        Ok(())
    }
}