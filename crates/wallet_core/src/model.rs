use bdk_wallet::bitcoin::{psbt::Psbt, Sequence};

use crate::{
    WalletCoreResult,
    types::{AmountSat, WalletKeychain, TxDirection},
};


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
    /// Whether the transaction is replaceable via RBF.
    pub replaceable: bool,
    /// Net value in satoshis:
    /// positive => received
    /// negative => sent
    /// zero => self-transfer
    pub net_value: i64, // keep signed for direction semantics

    /// Transaction fee in satoshis (always positive when present)
    pub fee: Option<AmountSat>,

    /// Fee rate in sat/vB when known.
    pub fee_rate_sat_per_vb: Option<u64>,
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

    /// Transaction id of the PSBT.
    pub txid: String,

    /// Original transaction id (used for fee bump flows).
    pub original_txid: Option<String>,

    /// Destination address the wallet is paying to.
    pub to_address: String,

    /// Requested payment amount in satoshis.
    pub amount_sat: AmountSat,

    /// Transaction fee in satoshis.
    pub fee_sat: AmountSat,

    /// Requested fee rate in sat/vB used when constructing the PSBT.
    pub fee_rate_sat_per_vb: u64,

    /// Whether the transaction was created as replaceable via RBF.
    pub replaceable: bool,

    /// Change amount in satoshis, if a change output was created.
    pub change_amount_sat: Option<AmountSat>,

    /// Number of wallet UTXOs selected for this PSBT.
    pub selected_utxo_count: usize,

    /// Total number of inputs in the transaction.
    pub input_count: usize,

    /// Total number of outputs in the transaction.
    pub output_count: usize,

    /// Number of recipient outputs (non-change).
    pub recipient_count: usize,

    /// Estimated virtual size of the transaction.
    pub estimated_vsize: u64,
}

impl WalletPsbtInfo {
    /// Build a conservative `WalletPsbtInfo` from a raw PSBT.
    ///
    /// This is primarily used by flows such as fee-bump, where a fresh PSBT
    /// must be returned from wallet_core even when full UI-oriented metadata is
    /// not readily available without additional wallet context.
    ///
    /// The conversion guarantees:
    /// - base64 PSBT payload is preserved
    /// - RBF flag is derived from input sequence numbers
    /// - selected input count is preserved
    ///
    /// The remaining fields are populated conservatively:
    /// - `to_address` = empty string
    /// - `amount_sat` = 0
    /// - `fee_sat` = 0
    /// - `fee_rate_sat_per_vb` = 0
    /// - `change_amount_sat` = None
    ///
    /// If later you want richer output, add a more specific constructor such as
    /// `from_psbt_with_metadata(...)` and keep this method as the lowest-common-
    /// denominator fallback.
    pub fn from_psbt(psbt: Psbt) -> WalletCoreResult<Self> {
        let replaceable = psbt
            .unsigned_tx
            .input
            .iter()
            .any(|txin| txin.sequence.0 < Sequence::ENABLE_LOCKTIME_NO_RBF.0);

        let txid = psbt.unsigned_tx.compute_txid().to_string();
        let input_count = psbt.unsigned_tx.input.len();
        let output_count = psbt.unsigned_tx.output.len();
        let recipient_count = output_count;
        let estimated_vsize = psbt.unsigned_tx.vsize() as u64;

        Ok(Self {
            psbt_base64: psbt.to_string(),
            txid,
            original_txid: None,
            to_address: String::new(),
            amount_sat: AmountSat::default(),
            fee_sat: AmountSat::default(),
            fee_rate_sat_per_vb: 0,
            replaceable,
            change_amount_sat: None,
            selected_utxo_count: input_count,
            input_count,
            output_count,
            recipient_count,
            estimated_vsize,
        })
    }
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
            Self::PartiallySigned => "partially_signed",
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

    /// Transaction id of the resulting transaction represented by the PSBT.
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

    /// Whether the transaction was created as replaceable via RBF, when known.
    pub replaceable: Option<bool>,
}