pub(crate) use crate::{
    types::{
        AmountSat, BlockHeight, FeeRateSatPerVb, PsbtBase64, TxDirection, TxHex, VSize,
        WalletKeychain, WalletOutPoint, WalletTxid,
    },
    WalletCoreResult,
};
use bdk_wallet::bitcoin::{psbt::Psbt, Sequence};
use serde::{Deserialize, Serialize};

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
    pub txid: WalletTxid,
    pub confirmed: bool,
    pub confirmation_height: Option<BlockHeight>,
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
    pub fee_rate_sat_per_vb: Option<FeeRateSatPerVb>,
}

/// Core wallet UTXO model used inside wallet_core
///
/// Represents a spendable output belonging to the wallet.
/// This is a domain model (not API DTO) and should not depend on wallet_api.
#[derive(Debug, Clone)]
pub struct WalletUtxoInfo {
    pub outpoint: WalletOutPoint,
    pub value: AmountSat,
    pub confirmed: bool,
    pub confirmation_height: Option<BlockHeight>,
    pub address: Option<String>,
    pub keychain: WalletKeychain,
}

/// Shared typed input-selection configuration used inside wallet-core.
///
/// Include/exclude outpoints stay in the wallet-core domain model as
/// `WalletOutPoint` values. They are converted into raw Bitcoin outpoints only
/// when matching against BDK `LocalOutput` values.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WalletInputSelectionConfig {
    pub include_outpoints: Vec<WalletOutPoint>,
    pub exclude_outpoints: Vec<WalletOutPoint>,
    pub confirmed_only: bool,
    pub selection_mode: Option<WalletInputSelectionMode>,

    pub max_input_count: Option<usize>,
    pub min_input_count: Option<usize>,
    pub min_utxo_value_sat: Option<u64>,
    pub max_utxo_value_sat: Option<u64>,
    pub strategy: Option<WalletConsolidationStrategy>,
}

impl WalletInputSelectionConfig {
    pub fn is_noop(&self) -> bool {
        self.include_outpoints.is_empty()
            && self.exclude_outpoints.is_empty()
            && !self.confirmed_only
            && self.selection_mode.is_none()
            && self.max_input_count.is_none()
            && self.min_input_count.is_none()
            && self.min_utxo_value_sat.is_none()
            && self.max_utxo_value_sat.is_none()
            && self.strategy.is_none()
    }

    pub fn is_empty(&self) -> bool {
        self.is_noop()
    }

    pub fn has_explicit_include_set(&self) -> bool {
        !self.include_outpoints.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WalletCoinControlInfo {
    pub selection: WalletInputSelectionConfig,
}

impl WalletCoinControlInfo {
    /// Returns true when this configuration has no effect on input selection.
    pub fn is_noop(&self) -> bool {
        self.selection.is_noop()
    }

    /// Backward-compatible alias for `is_noop()`.
    pub fn is_empty(&self) -> bool {
        self.selection.is_empty()
    }

    /// Returns true when an explicit include set is present.
    pub fn has_explicit_include_set(&self) -> bool {
        self.selection.has_explicit_include_set()
    }
}

/// Core model describing the resolved coin control state after validation.
///
/// This represents the normalized and validated result of applying
/// `WalletCoinControlInfo` to wallet UTXOs.
///
/// This is a domain model (not API DTO) and should not depend on wallet_api.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WalletCoinControlResolutionInfo {
    /// Outpoints explicitly selected for inclusion.
    pub included_outpoints: Vec<WalletOutPoint>,

    /// Outpoints explicitly excluded from selection.
    pub excluded_outpoints: Vec<WalletOutPoint>,

    /// Whether only confirmed UTXOs are allowed.
    pub confirmed_only: bool,

    /// Effective selection mode after normalization.
    pub selection_mode: Option<WalletInputSelectionMode>,

    /// Whether the caller provided an explicit include set.
    pub has_explicit_include_set: bool,
}

impl WalletCoinControlResolutionInfo {
    /// Returns true when no constraints are applied.
    pub fn is_noop(&self) -> bool {
        self.included_outpoints.is_empty()
            && self.excluded_outpoints.is_empty()
            && !self.confirmed_only
            && self.selection_mode.is_none()
    }

    /// Returns true when explicit manual input selection is active.
    pub fn has_manual_selection(&self) -> bool {
        self.has_explicit_include_set
    }

    /// Returns true when any explicit exclusions are present.
    pub fn has_exclusions(&self) -> bool {
        !self.excluded_outpoints.is_empty()
    }

    /// Returns the number of explicitly included outpoints.
    pub fn included_count(&self) -> usize {
        self.included_outpoints.len()
    }

    /// Returns the number of explicitly excluded outpoints.
    pub fn excluded_count(&self) -> usize {
        self.excluded_outpoints.len()
    }

    /// Returns true when explicit include or exclude constraints are present.
    pub fn has_constraints(&self) -> bool {
        !self.is_noop()
    }
}

/// Domain mode describing how manual input selection should interact with
/// automatic candidate selection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum WalletInputSelectionMode {
    /// Only explicitly included inputs may be used.
    StrictManual,

    /// Explicitly included inputs are pinned, and additional eligible inputs may
    /// be added automatically if needed.
    #[default]
    ManualWithAutoCompletion,

    /// Ignore explicit include sets and rely only on automatic selection.
    AutomaticOnly,
}

/// Strategy used when automatically selecting UTXOs for consolidation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WalletConsolidationStrategy {
    /// Prefer smaller UTXOs first to reduce fragmentation and dust.
    SmallestFirst,

    /// Prefer larger UTXOs first to reduce input overhead quickly.
    LargestFirst,

    /// Prefer older/earlier wallet UTXOs first when ordering information is available.
    ///
    /// When precise age ordering is unavailable, implementations may fall back to
    /// a stable deterministic order.
    OldestFirst,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct WalletConsolidationInfo {
    pub selection: WalletInputSelectionConfig,

    /// Optional cap on fee as a percentage of total selected input value.
    pub max_fee_pct_of_input_value: Option<crate::types::Percent>,
}

impl WalletConsolidationInfo {
    /// Returns true when this configuration has no effect on consolidation behavior.
    pub fn is_noop(&self) -> bool {
        self.selection.is_noop() && self.max_fee_pct_of_input_value.is_none()
    }

    /// Backward-compatible alias for `is_noop()`.
    pub fn is_empty(&self) -> bool {
        self.is_noop()
    }

    /// Returns true when an explicit include set is present.
    pub fn has_explicit_include_set(&self) -> bool {
        self.selection.has_explicit_include_set()
    }
}

/// Core model describing an unsigned PSBT created by the wallet.
///
/// This represents the result of transaction construction and is independent
/// from API/CLI formatting.
///
/// Unit-bearing and identifier-bearing fields use strong domain types such as
/// `WalletTxid`, `FeeRateSatPerVb`, `VSize`, and `PsbtBase64` to reduce mixups
/// inside wallet-core logic.
#[derive(Debug, Clone)]
pub struct WalletPsbtInfo {
    /// Base64-encoded PSBT payload.
    pub psbt_base64: PsbtBase64,

    /// Transaction id of the PSBT.
    pub txid: WalletTxid,

    /// Original transaction id (used for fee bump flows).
    pub original_txid: Option<WalletTxid>,

    /// Destination address the wallet is paying to.
    pub to_address: String,

    /// Requested payment amount in satoshis.
    pub amount_sat: AmountSat,

    /// Transaction fee in satoshis.
    pub fee_sat: AmountSat,

    /// Requested fee rate in sat/vB used when constructing the PSBT.
    pub fee_rate_sat_per_vb: FeeRateSatPerVb,

    /// Whether the transaction was created as replaceable via RBF.
    pub replaceable: bool,

    /// Change amount in satoshis, if a change output was created.
    pub change_amount_sat: Option<AmountSat>,

    /// Number of wallet UTXOs selected for this PSBT.
    pub selected_utxo_count: usize,

    /// Exact selected input outpoints used for this PSBT.
    pub selected_inputs: Vec<WalletOutPoint>,

    /// Total number of inputs in the transaction.
    pub input_count: usize,

    /// Total number of outputs in the transaction.
    pub output_count: usize,

    /// Number of recipient outputs (non-change).
    pub recipient_count: usize,

    /// Estimated virtual size of the transaction.
    pub estimated_vsize: VSize,
}

impl WalletPsbtInfo {
    /// Build a conservative minimal `WalletPsbtInfo` from a raw PSBT.
    ///
    /// This is primarily used by flows such as fee-bump, where a fresh PSBT
    /// must be returned from wallet_core even when full UI-oriented metadata is
    /// not readily available without additional wallet context.
    ///
    /// The conversion guarantees:
    /// - base64 PSBT payload is preserved
    /// - RBF flag is derived from input sequence numbers
    /// - selected input count is preserved
    /// - selected input outpoints are preserved
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
    pub fn from_psbt_minimal(psbt: Psbt) -> WalletCoreResult<Self> {
        let replaceable = psbt
            .unsigned_tx
            .input
            .iter()
            .any(|txin| txin.sequence.0 < Sequence::ENABLE_LOCKTIME_NO_RBF.0);

        let txid = WalletTxid::from(psbt.unsigned_tx.compute_txid());
        let input_count = psbt.unsigned_tx.input.len();
        let selected_inputs = psbt
            .unsigned_tx
            .input
            .iter()
            .map(|txin| WalletOutPoint::from(txin.previous_output))
            .collect::<Vec<_>>();
        let output_count = psbt.unsigned_tx.output.len();
        // Conservative assumption: if there is more than one output, one is likely change
        // This avoids overcounting recipients in minimal mode without wallet context
        let recipient_count = output_count.saturating_sub(1);
        let estimated_vsize = VSize::from(psbt.unsigned_tx.vsize() as u64);

        Ok(Self {
            psbt_base64: PsbtBase64::from(psbt.to_string()),
            txid,
            original_txid: None,
            to_address: String::new(),
            amount_sat: AmountSat::default(),
            fee_sat: AmountSat::default(),
            fee_rate_sat_per_vb: FeeRateSatPerVb::ZERO,
            replaceable,
            change_amount_sat: None,
            selected_utxo_count: input_count,
            selected_inputs,
            input_count,
            output_count,
            recipient_count,
            estimated_vsize,
        })
    }

    /// Returns true when the PSBT has at least one selected input.
    pub fn has_selected_inputs(&self) -> bool {
        !self.selected_inputs.is_empty()
    }

    /// Returns true when the PSBT appears to contain change.
    pub fn has_change(&self) -> bool {
        self.change_amount_sat.is_some()
    }

    /// Returns true when the PSBT represents a likely self-transfer.
    pub fn is_likely_self_transfer(&self) -> bool {
        self.recipient_count == 0 && self.has_change()
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
    pub psbt_base64: PsbtBase64,

    /// Whether the wallet modified the PSBT during signing.
    pub modified: bool,

    /// Whether the wallet finalized the PSBT during signing.
    pub finalized: bool,

    /// Transaction id of the resulting transaction represented by the PSBT.
    pub txid: WalletTxid,
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

/// Core model describing a finalized transaction extracted from a PSBT.
///
/// This represents the output of core logic after a PSBT has been fully
/// signed and finalized, but before it is broadcast to the network.
///
/// This model is intentionally independent of any broadcasting mechanism
/// and is used as the boundary between `wallet_core` and `wallet_sync`.
#[derive(Debug, Clone)]
pub struct WalletFinalizedTxInfo {
    /// Transaction id of the finalized transaction.
    pub txid: WalletTxid,

    /// Raw transaction hex ready for broadcast.
    pub tx_hex: TxHex,

    /// Whether the transaction is replaceable via RBF.
    pub replaceable: bool,
}
/// Core model describing the build plan for a CPFP (Child Pays For Parent)
/// child transaction before the PSBT is created.
///
/// This is an internal domain model in `wallet_core` and keeps CPFP planning
/// data structured in the same style as the other wallet models.
#[derive(Debug, Clone)]
pub struct WalletCpfpBuildPlanInfo {
    /// Selected input outpoint used for the CPFP child transaction.
    pub input_outpoint: WalletOutPoint,

    /// Input value in satoshis.
    pub input_value_sat: AmountSat,

    /// Planned child output value in satoshis.
    pub child_output_value_sat: AmountSat,

    /// Planned transaction fee in satoshis.
    pub fee_sat: AmountSat,

    /// Estimated virtual size of the child transaction.
    pub estimated_vsize: VSize,
}

/// Core model describing a CPFP (Child Pays For Parent) PSBT created by the wallet.
///
/// This is a domain model (not API DTO) and mirrors the style of `WalletPsbtInfo`.
#[derive(Debug, Clone)]
pub struct WalletCpfpPsbtInfo {
    /// Base64-encoded PSBT payload.
    pub psbt_base64: PsbtBase64,

    /// Transaction id of the CPFP child transaction.
    pub txid: WalletTxid,

    /// Transaction id of the parent transaction being accelerated via CPFP.
    pub parent_txid: WalletTxid,

    /// Selected outpoint (txid:vout) used for CPFP.
    pub selected_outpoint: WalletOutPoint,

    /// Input value in satoshis used for the CPFP child transaction.
    pub input_value_sat: AmountSat,

    /// Child output value in satoshis after subtracting the fee.
    pub child_output_value_sat: AmountSat,

    /// Transaction fee in satoshis for the CPFP child transaction.
    pub fee_sat: AmountSat,

    /// Requested fee rate in sat/vB used for CPFP.
    pub fee_rate_sat_per_vb: FeeRateSatPerVb,

    /// Whether the transaction is replaceable via RBF.
    pub replaceable: bool,

    /// Estimated virtual size of the transaction.
    pub estimated_vsize: VSize,
}

/// Domain mode describing how a send amount should be interpreted when building a PSBT.
///
/// This stays in `wallet_core` because it is part of transaction-construction behavior,
/// not API formatting.
///
/// Behavior notes:
/// - In `Max` mode with strict coin control (when `include_outpoints` is provided),
///   only the selected inputs are used and the transaction will fail if they do not
///   fully fund the transaction including fees.
/// - In `Fixed` mode, the provided amount is respected and additional inputs are
///   not automatically added when strict coin control is active.
/// - Sweep flows are represented as `Max` mode combined with an explicit include
///   set in `WalletCoinControlInfo`.
/// - Consolidation is not represented by `WalletSendAmountMode`; it is a
///   separate wallet-maintenance strategy modeled by `WalletConsolidationInfo`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletSendAmountMode {
    /// Build a transaction for an explicit fixed amount.
    Fixed(AmountSat),

    /// Build a transaction that drains the selected or otherwise eligible inputs
    /// to the recipient, after subtracting fees.
    ///
    /// When combined with an explicit include set, this represents a sweep.
    Max,
}
