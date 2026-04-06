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

/// Domain status representing PSBT signing progress.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PsbtSigningStatus {
    /// The signing attempt did not change the PSBT.
    Unchanged,
    /// The PSBT was updated but is not yet fully finalized.
    PartiallySigned,
    /// The PSBT is finalized and ready for extraction/broadcast.
    Finalized,
}

impl PsbtSigningStatus {
    /// Convert signing status into a stable string representation for DTO/API layers.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Unchanged => "unchanged",
            Self::PartiallySigned => "partial",
            Self::Finalized => "finalized",
        }
    }
}

impl std::fmt::Display for PsbtSigningStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Core model describing the result of attempting to sign a PSBT.
///
/// This remains a domain model in `wallet_core` and is independent from
/// API/CLI formatting.
#[derive(Debug, Clone)]
pub struct WalletSignedPsbtInfo {
    /// Base64-encoded signed PSBT payload.
    pub psbt_base64: String,

    /// Whether the wallet modified the PSBT during signing.
    pub modified: bool,

    /// Whether the wallet finalized the PSBT during signing.
    pub finalized: bool,

    /// Transaction id derived from the PSBT unsigned transaction.
    pub txid: String,
}

impl WalletSignedPsbtInfo {
    /// Classify signing result into a domain status.
    pub fn signing_status(&self) -> PsbtSigningStatus {
        match (self.modified, self.finalized) {
            (_, true) => PsbtSigningStatus::Finalized,
            (true, false) => PsbtSigningStatus::PartiallySigned,
            (false, false) => PsbtSigningStatus::Unchanged,
        }
    }
}

/// Core model describing a successfully published (broadcast) transaction.
///
/// This is a domain model and represents the result of broadcasting a
/// finalized transaction to the network.
#[derive(Debug, Clone)]
pub struct WalletPublishedTxInfo {
    /// Transaction id of the broadcasted transaction.
    pub txid: String,
}