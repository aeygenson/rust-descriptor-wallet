use crate::types::{AmountSat, WalletKeychain, TxDirection};


/// Core wallet transaction model used inside wallet_core
///
/// This is a domain model (not API DTO) and should not depend on wallet_api.
///
/// `net_value` is expressed in satoshis:
/// - positive => net funds received by the wallet
/// - negative => net funds sent from the wallet
/// - zero     => likely a self-transfer / reshuffle
#[derive(Debug, Clone)]
pub struct WalletTxInfo {
    pub txid: String,
    pub confirmed: bool,
    pub confirmation_height: Option<u32>,
    pub direction: TxDirection,
    /// Net value in satoshis:
    /// positive => received
    /// negative => sent
    /// zero => self-transfer
    pub net_value: i64, // keep signed for direction semantics

    /// Transaction fee in satoshis (always positive when present)
    pub fee: Option<AmountSat>,
}

/// Core wallet UTXO model used inside wallet_core
///
/// Represents a spendable output belonging to the wallet.
/// This is a domain model (not API DTO) and should not depend on wallet_api.
#[derive(Debug, Clone)]
pub struct WalletUtxoInfo {
    pub outpoint: String,
    pub value: AmountSat,
    pub confirmed: bool,
    pub confirmation_height: Option<u32>,
    pub address: Option<String>,
    pub keychain: WalletKeychain,
}

/// Core model describing an unsigned PSBT created by the wallet.
///
/// This represents the result of transaction construction and is independent
/// from API/CLI formatting.
#[derive(Debug, Clone)]
pub struct WalletPsbtInfo {
    /// Base64-encoded PSBT payload.
    pub psbt_base64: String,

    /// Destination address the wallet is paying to.
    pub to_address: String,

    /// Requested payment amount in satoshis.
    pub amount_sat: AmountSat,

    /// Transaction fee in satoshis.
    pub fee_sat: AmountSat,

    /// Change amount in satoshis, if a change output was created.
    pub change_amount_sat: Option<AmountSat>,

    /// Number of wallet UTXOs selected for this PSBT.
    pub selected_utxo_count: usize,
}