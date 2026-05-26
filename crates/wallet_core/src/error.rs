use bitcoin::psbt::PsbtParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletCoreError {
    #[error("invariant violation: {0}")]
    InvalidState(String),

    #[error("not implemented: {0}")]
    NotImplemented(String),

    #[error("invalid configuration: {0}")]
    InvalidConfig(String),

    #[error(transparent)]
    Store(#[from] bdk_file_store::StoreError),

    #[error(transparent)]
    StoreWithDump(#[from] bdk_file_store::StoreErrorWithDump<bdk_wallet::ChangeSet>),

    #[error("wallet load error: {0}")]
    Load(String),

    #[error("wallet create error: {0}")]
    Create(String),

    #[error("wallet persist error: {0}")]
    Persist(String),
    #[error("invalid fee rate")]
    InvalidFeeRate,

    #[error("invalid txid: {0}")]
    InvalidTxid(String),

    #[error("invalid outpoint: {0}")]
    InvalidOutpoint(String),

    #[error("invalid virtual size: {0}")]
    InvalidVsize(String),

    #[error("invalid block height: {0}")]
    InvalidBlockHeight(String),

    #[error("invalid percent: {0}")]
    InvalidPercent(String),

    #[error("invalid psbt base64: {0}")]
    InvalidPsbtBase64(String),

    #[error("invalid transaction hex: {0}")]
    InvalidTxHex(String),

    #[error("transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("transaction is already confirmed: {0}")]
    TransactionAlreadyConfirmed(String),

    #[error("transaction is not replaceable (RBF disabled): {0}")]
    TransactionNotReplaceable(String),

    #[error(
        "requested fee rate for tx {txid} must be greater than original fee rate (original: {original_sat_per_vb}, requested: {requested_sat_per_vb})"
    )]
    FeeRateTooLowForBump {
        txid: String,
        original_sat_per_vb: crate::types::FeeRateSatPerVb,
        requested_sat_per_vb: crate::types::FeeRateSatPerVb,
    },

    #[error("fee bump build failed for tx {txid}: {reason}")]
    FeeBumpBuildFailed { txid: String, reason: String },

    #[error("transaction fee unavailable for tx {txid}: {reason}")]
    TransactionFeeUnavailable { txid: String, reason: String },

    #[error("transaction virtual size unavailable for tx {0}")]
    TransactionVsizeUnavailable(String),

    #[error("psbt conversion failed for tx {txid}: {reason}")]
    PsbtConversionFailed { txid: String, reason: String },

    #[error("invalid amount")]
    InvalidAmount,

    #[error("invalid destination address: {0}")]
    InvalidDestinationAddress(String),

    #[error("destination address network mismatch: {0}")]
    DestinationNetworkMismatch(String),

    #[error("psbt build failed: {0}")]
    PsbtBuildFailed(String),

    #[error("parent transaction id cannot be empty")]
    CpfpEmptyParentTxid,

    #[error("no suitable unconfirmed utxo found for parent transaction {0}")]
    CpfpNoCandidateUtxo(String),

    #[error("parent transaction not found: {0}")]
    CpfpParentNotFound(String),

    #[error("parent transaction already confirmed: {0}")]
    CpfpParentAlreadyConfirmed(String),

    #[error("insufficient value in selected utxo for cpfp: {0}")]
    CpfpInsufficientValue(String),

    #[error("cpfp transaction build failed for parent {parent_txid}: {reason}")]
    CpfpBuildFailed { parent_txid: String, reason: String },

    #[error("coin control outpoint not found in wallet: {0}")]
    CoinControlOutpointNotFound(String),

    #[error("coin control outpoint is invalid: {0}")]
    CoinControlInvalidOutpoint(String),

    #[error("coin control outpoint is not spendable: {0}")]
    CoinControlOutpointNotSpendable(String),

    #[error("coin control requested outpoint is not confirmed: {0}")]
    CoinControlOutpointNotConfirmed(String),

    #[error("coin control conflict: outpoint present in both include and exclude: {0}")]
    CoinControlConflict(String),

    #[error("utxo is locked and cannot be spent: {0}")]
    LockedUtxo(String),

    #[error("coin control include set is empty while exact selection is required")]
    CoinControlEmptySelection,

    #[error("coin control strict mode violation: selected inputs do not fully fund the transaction and automatic additional inputs are not allowed")]
    CoinControlStrictModeViolation,

    #[error("input selection failed: {0}")]
    SelectionFailed(String),

    #[error("coin control insufficient selected funds: selected={selected_sat}, required={required_sat}, fee_estimate={fee_estimate_sat}")]
    CoinControlInsufficientSelectedFunds {
        selected_sat: u64,
        required_sat: u64,
        fee_estimate_sat: u64,
    },

    #[error("send-max/sweep amount is too small after fees")]
    SendMaxAmountTooSmall,

    #[error("consolidation requires at least two eligible UTXOs")]
    ConsolidationTooFewInputs,

    #[error("consolidation amount is too small after fees")]
    ConsolidationAmountTooSmall,

    #[error(
        "consolidation does not meet minimum input count: required={required}, actual={actual}"
    )]
    ConsolidationMinInputNotMet { required: usize, actual: usize },

    #[error("consolidation input value outside allowed range")]
    ConsolidationValueFilterMismatch,

    #[error("consolidation fee exceeds allowed percentage: fee={fee_sat}, total_inputs={total_input_sat}, max_pct={max_pct}")]
    ConsolidationFeeTooHigh {
        fee_sat: u64,
        total_input_sat: u64,
        max_pct: u8,
    },

    #[error("consolidation produced no eligible UTXOs after applying filters")]
    ConsolidationNoEligibleUtxos,

    #[error("fee calculation failed: {0}")]
    FeeCalculationFailedWithReason(String),

    #[error("fee calculation failed")]
    FeeCalculationFailed,

    #[error("invalid psbt encoding: {0}")]
    InvalidPsbtEncoding(String),

    #[error("invalid psbt structure: {0}")]
    InvalidPsbtStructure(String),

    #[error("invalid psbt semantic state: {0}")]
    InvalidPsbtSemantic(String),

    #[error("invalid psbt: {0}")]
    InvalidPsbt(String),

    #[allow(deprecated)]
    #[error(transparent)]
    SignPsbtFailed(#[from] bdk_wallet::signer::SignerError),

    #[error("wallet is watch-only and cannot sign")]
    WatchOnlyCannotSign,

    #[error("psbt is not finalized")]
    PsbtNotFinalized,

    #[error("failed to extract transaction from psbt: {0}")]
    ExtractTxFailed(String),
}

/// Broad category for wallet-core errors.
///
/// This is intended for API/UI grouping, telemetry, logging policy,
/// and future retry/recovery decisions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletCoreErrorCategory {
    /// Configuration, descriptor, or invariant/state problem.
    Configuration,
    /// Wallet persistence or file-store problem.
    Persistence,
    /// Invalid user input or boundary parsing failure.
    Validation,
    /// Transaction lookup or graph/projection problem.
    Transaction,
    /// Fee calculation, fee policy, or fee bump problem.
    Fee,
    /// PSBT encoding, structure, signing, finalization, or extraction problem.
    Psbt,
    /// Coin-control or input-selection problem.
    Selection,
    /// CPFP workflow problem.
    Cpfp,
    /// Consolidation workflow problem.
    Consolidation,
    /// RBF/replacement workflow problem.
    Replacement,
    /// Feature has not been implemented yet.
    Unsupported,
}

impl WalletCoreErrorCategory {
    /// Stable string representation for API/UI/logging layers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Configuration => "configuration",
            Self::Persistence => "persistence",
            Self::Validation => "validation",
            Self::Transaction => "transaction",
            Self::Fee => "fee",
            Self::Psbt => "psbt",
            Self::Selection => "selection",
            Self::Cpfp => "cpfp",
            Self::Consolidation => "consolidation",
            Self::Replacement => "replacement",
            Self::Unsupported => "unsupported",
        }
    }
}

impl std::fmt::Display for WalletCoreErrorCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Recovery classification for wallet-core errors.
///
/// This tells higher layers whether retrying, changing user input,
/// resyncing/reloading state, or fixing configuration is the likely recovery path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletCoreErrorRecovery {
    /// Retrying the same operation may succeed later.
    Retry,
    /// User can resolve by changing request input or selected wallet data.
    UserAction,
    /// Caller should refresh/reload wallet state before trying again.
    RefreshState,
    /// Configuration or wallet setup must be corrected.
    FixConfiguration,
    /// Operation is unsupported or not available in the current wallet mode.
    Unsupported,
    /// Error is not expected to be recoverable by retrying unchanged input.
    Fatal,
}

impl WalletCoreErrorRecovery {
    /// Stable string representation for API/UI/logging layers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Retry => "retry",
            Self::UserAction => "user-action",
            Self::RefreshState => "refresh-state",
            Self::FixConfiguration => "fix-configuration",
            Self::Unsupported => "unsupported",
            Self::Fatal => "fatal",
        }
    }
}

impl std::fmt::Display for WalletCoreErrorRecovery {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Suggested logging severity for wallet-core errors.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WalletCoreErrorSeverity {
    /// Expected user-correctable validation/policy failure.
    Info,
    /// Recoverable but notable problem.
    Warning,
    /// Serious failure that should be surfaced prominently.
    Error,
}

impl WalletCoreErrorSeverity {
    /// Stable string representation for API/UI/logging layers.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Info => "info",
            Self::Warning => "warning",
            Self::Error => "error",
        }
    }
}

impl std::fmt::Display for WalletCoreErrorSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<PsbtParseError> for WalletCoreError {
    fn from(e: PsbtParseError) -> Self {
        WalletCoreError::InvalidPsbtEncoding(e.to_string())
    }
}

impl WalletCoreError {
    /// Return a broad category for this error.
    pub fn category(&self) -> WalletCoreErrorCategory {
        match self {
            Self::InvalidState(_) | Self::InvalidConfig(_) => WalletCoreErrorCategory::Configuration,
            Self::Store(_) | Self::StoreWithDump(_) | Self::Load(_) | Self::Create(_) | Self::Persist(_) => {
                WalletCoreErrorCategory::Persistence
            }
            Self::InvalidTxid(_)
            | Self::InvalidOutpoint(_)
            | Self::InvalidVsize(_)
            | Self::InvalidBlockHeight(_)
            | Self::InvalidPercent(_)
            | Self::InvalidPsbtBase64(_)
            | Self::InvalidTxHex(_)
            | Self::InvalidAmount
            | Self::InvalidDestinationAddress(_)
            | Self::DestinationNetworkMismatch(_) => WalletCoreErrorCategory::Validation,
            Self::TransactionNotFound(_)
            | Self::TransactionAlreadyConfirmed(_)
            | Self::TransactionFeeUnavailable { .. }
            | Self::TransactionVsizeUnavailable(_) => WalletCoreErrorCategory::Transaction,
            Self::InvalidFeeRate
            | Self::FeeRateTooLowForBump { .. }
            | Self::FeeCalculationFailedWithReason(_)
            | Self::FeeCalculationFailed => WalletCoreErrorCategory::Fee,
            Self::InvalidPsbtEncoding(_)
            | Self::InvalidPsbtStructure(_)
            | Self::InvalidPsbtSemantic(_)
            | Self::InvalidPsbt(_)
            | Self::SignPsbtFailed(_)
            | Self::WatchOnlyCannotSign
            | Self::PsbtNotFinalized
            | Self::ExtractTxFailed(_)
            | Self::PsbtConversionFailed { .. }
            | Self::PsbtBuildFailed(_) => WalletCoreErrorCategory::Psbt,
            Self::CoinControlOutpointNotFound(_)
            | Self::CoinControlInvalidOutpoint(_)
            | Self::CoinControlOutpointNotSpendable(_)
            | Self::CoinControlOutpointNotConfirmed(_)
            | Self::CoinControlConflict(_)
            | Self::LockedUtxo(_)
            | Self::CoinControlEmptySelection
            | Self::CoinControlStrictModeViolation
            | Self::SelectionFailed(_)
            | Self::CoinControlInsufficientSelectedFunds { .. }
            | Self::SendMaxAmountTooSmall => WalletCoreErrorCategory::Selection,
            Self::CpfpEmptyParentTxid
            | Self::CpfpNoCandidateUtxo(_)
            | Self::CpfpParentNotFound(_)
            | Self::CpfpParentAlreadyConfirmed(_)
            | Self::CpfpInsufficientValue(_)
            | Self::CpfpBuildFailed { .. } => WalletCoreErrorCategory::Cpfp,
            Self::ConsolidationTooFewInputs
            | Self::ConsolidationAmountTooSmall
            | Self::ConsolidationMinInputNotMet { .. }
            | Self::ConsolidationValueFilterMismatch
            | Self::ConsolidationFeeTooHigh { .. }
            | Self::ConsolidationNoEligibleUtxos => WalletCoreErrorCategory::Consolidation,
            Self::TransactionNotReplaceable(_)
            | Self::FeeBumpBuildFailed { .. } => WalletCoreErrorCategory::Replacement,
            Self::NotImplemented(_) => WalletCoreErrorCategory::Unsupported,
        }
    }

    /// Return the recommended recovery class for this error.
    pub fn recovery(&self) -> WalletCoreErrorRecovery {
        match self {
            Self::Store(_) | Self::StoreWithDump(_) | Self::Load(_) | Self::Create(_) | Self::Persist(_) => {
                WalletCoreErrorRecovery::Retry
            }
            Self::TransactionNotFound(_)
            | Self::TransactionFeeUnavailable { .. }
            | Self::TransactionVsizeUnavailable(_)
            | Self::CpfpParentNotFound(_)
            | Self::CpfpNoCandidateUtxo(_) => WalletCoreErrorRecovery::RefreshState,
            Self::InvalidTxid(_)
            | Self::InvalidOutpoint(_)
            | Self::InvalidVsize(_)
            | Self::InvalidBlockHeight(_)
            | Self::InvalidPercent(_)
            | Self::InvalidPsbtBase64(_)
            | Self::InvalidTxHex(_)
            | Self::InvalidFeeRate
            | Self::InvalidAmount
            | Self::InvalidDestinationAddress(_)
            | Self::DestinationNetworkMismatch(_)
            | Self::FeeRateTooLowForBump { .. }
            | Self::CoinControlOutpointNotFound(_)
            | Self::CoinControlInvalidOutpoint(_)
            | Self::CoinControlOutpointNotSpendable(_)
            | Self::CoinControlOutpointNotConfirmed(_)
            | Self::CoinControlConflict(_)
            | Self::LockedUtxo(_)
            | Self::CoinControlEmptySelection
            | Self::CoinControlStrictModeViolation
            | Self::SelectionFailed(_)
            | Self::CoinControlInsufficientSelectedFunds { .. }
            | Self::SendMaxAmountTooSmall
            | Self::ConsolidationTooFewInputs
            | Self::ConsolidationAmountTooSmall
            | Self::ConsolidationMinInputNotMet { .. }
            | Self::ConsolidationValueFilterMismatch
            | Self::ConsolidationFeeTooHigh { .. }
            | Self::ConsolidationNoEligibleUtxos
            | Self::CpfpEmptyParentTxid
            | Self::CpfpParentAlreadyConfirmed(_)
            | Self::CpfpInsufficientValue(_)
            | Self::TransactionAlreadyConfirmed(_)
            | Self::TransactionNotReplaceable(_)
            | Self::PsbtNotFinalized => WalletCoreErrorRecovery::UserAction,
            Self::InvalidConfig(_) | Self::WatchOnlyCannotSign => WalletCoreErrorRecovery::FixConfiguration,
            Self::NotImplemented(_) => WalletCoreErrorRecovery::Unsupported,
            Self::InvalidState(_)
            | Self::FeeCalculationFailedWithReason(_)
            | Self::FeeCalculationFailed
            | Self::InvalidPsbtEncoding(_)
            | Self::InvalidPsbtStructure(_)
            | Self::InvalidPsbtSemantic(_)
            | Self::InvalidPsbt(_)
            | Self::SignPsbtFailed(_)
            | Self::ExtractTxFailed(_)
            | Self::PsbtConversionFailed { .. }
            | Self::PsbtBuildFailed(_)
            | Self::CpfpBuildFailed { .. }
            | Self::FeeBumpBuildFailed { .. } => WalletCoreErrorRecovery::Fatal,
        }
    }

    /// Returns true when retrying the same operation may succeed later.
    pub fn is_retryable(&self) -> bool {
        matches!(self.recovery(), WalletCoreErrorRecovery::Retry)
    }

    /// Returns true when refreshing or resyncing wallet state is a sensible next step.
    pub fn should_refresh_state(&self) -> bool {
        matches!(self.recovery(), WalletCoreErrorRecovery::RefreshState)
    }

    /// Returns true when the user can probably fix the request by changing input.
    pub fn is_user_correctable(&self) -> bool {
        matches!(self.recovery(), WalletCoreErrorRecovery::UserAction)
    }

    /// Returns true when the likely fix is changing wallet configuration or wallet mode.
    pub fn requires_configuration_change(&self) -> bool {
        matches!(self.recovery(), WalletCoreErrorRecovery::FixConfiguration)
    }

    /// Return suggested logging severity for this error.
    pub fn severity(&self) -> WalletCoreErrorSeverity {
        match self.recovery() {
            WalletCoreErrorRecovery::UserAction | WalletCoreErrorRecovery::Unsupported => {
                WalletCoreErrorSeverity::Info
            }
            WalletCoreErrorRecovery::Retry | WalletCoreErrorRecovery::RefreshState => {
                WalletCoreErrorSeverity::Warning
            }
            WalletCoreErrorRecovery::FixConfiguration | WalletCoreErrorRecovery::Fatal => {
                WalletCoreErrorSeverity::Error
            }
        }
    }

    /// Build a fee-calculation error with contextual detail.
    pub fn fee_calculation_failed(reason: impl Into<String>) -> Self {
        Self::FeeCalculationFailedWithReason(reason.into())
    }

    /// Build an invalid-PSBT encoding error.
    pub fn invalid_psbt_encoding(reason: impl Into<String>) -> Self {
        Self::InvalidPsbtEncoding(reason.into())
    }

    /// Build an invalid-PSBT structure error.
    pub fn invalid_psbt_structure(reason: impl Into<String>) -> Self {
        Self::InvalidPsbtStructure(reason.into())
    }

    /// Build an invalid-PSBT semantic-state error.
    pub fn invalid_psbt_semantic(reason: impl Into<String>) -> Self {
        Self::InvalidPsbtSemantic(reason.into())
    }

    /// Build an invalid-outpoint error.
    pub fn invalid_outpoint(reason: impl Into<String>) -> Self {
        Self::InvalidOutpoint(reason.into())
    }

    /// Build an invalid-virtual-size error.
    pub fn invalid_vsize(reason: impl Into<String>) -> Self {
        Self::InvalidVsize(reason.into())
    }

    /// Build an invalid-block-height error.
    pub fn invalid_block_height(reason: impl Into<String>) -> Self {
        Self::InvalidBlockHeight(reason.into())
    }

    /// Build an invalid-percent error.
    pub fn invalid_percent(reason: impl Into<String>) -> Self {
        Self::InvalidPercent(reason.into())
    }

    /// Build an invalid-PSBT-base64 error.
    pub fn invalid_psbt_base64(reason: impl Into<String>) -> Self {
        Self::InvalidPsbtBase64(reason.into())
    }

    /// Build an invalid-transaction-hex error.
    pub fn invalid_tx_hex(reason: impl Into<String>) -> Self {
        Self::InvalidTxHex(reason.into())
    }
}

#[cfg(test)]
mod tests {
    use super::{
        WalletCoreError, WalletCoreErrorCategory, WalletCoreErrorRecovery, WalletCoreErrorSeverity,
    };

    #[test]
    fn invalid_destination_is_user_correctable_validation_error() {
        let err = WalletCoreError::InvalidDestinationAddress("bad address".to_string());

        assert_eq!(err.category(), WalletCoreErrorCategory::Validation);
        assert_eq!(err.recovery(), WalletCoreErrorRecovery::UserAction);
        assert!(err.is_user_correctable());
        assert!(!err.is_retryable());
        assert_eq!(err.severity(), WalletCoreErrorSeverity::Info);
    }

    #[test]
    fn missing_transaction_should_refresh_state() {
        let err = WalletCoreError::TransactionNotFound("txid".to_string());

        assert_eq!(err.category(), WalletCoreErrorCategory::Transaction);
        assert_eq!(err.recovery(), WalletCoreErrorRecovery::RefreshState);
        assert!(err.should_refresh_state());
        assert_eq!(err.severity(), WalletCoreErrorSeverity::Warning);
    }

    #[test]
    fn persistence_errors_are_retryable() {
        let err = WalletCoreError::Persist("temporary filesystem problem".to_string());

        assert_eq!(err.category(), WalletCoreErrorCategory::Persistence);
        assert_eq!(err.recovery(), WalletCoreErrorRecovery::Retry);
        assert!(err.is_retryable());
        assert_eq!(err.severity(), WalletCoreErrorSeverity::Warning);
    }

    #[test]
    fn watch_only_signing_requires_configuration_change() {
        let err = WalletCoreError::WatchOnlyCannotSign;

        assert_eq!(err.category(), WalletCoreErrorCategory::Psbt);
        assert_eq!(err.recovery(), WalletCoreErrorRecovery::FixConfiguration);
        assert!(err.requires_configuration_change());
        assert_eq!(err.severity(), WalletCoreErrorSeverity::Error);
    }

    #[test]
    fn locked_utxo_is_user_correctable_selection_error() {
        let err = WalletCoreError::LockedUtxo("txid:0".to_string());

        assert_eq!(err.category(), WalletCoreErrorCategory::Selection);
        assert_eq!(err.recovery(), WalletCoreErrorRecovery::UserAction);
        assert!(err.is_user_correctable());
        assert!(!err.is_retryable());
        assert_eq!(err.severity(), WalletCoreErrorSeverity::Info);
    }

    #[test]
    fn error_metadata_has_stable_string_representations() {
        assert_eq!(WalletCoreErrorCategory::Validation.as_str(), "validation");
        assert_eq!(WalletCoreErrorCategory::Validation.to_string(), "validation");

        assert_eq!(WalletCoreErrorRecovery::RefreshState.as_str(), "refresh-state");
        assert_eq!(
            WalletCoreErrorRecovery::RefreshState.to_string(),
            "refresh-state"
        );

        assert_eq!(WalletCoreErrorSeverity::Warning.as_str(), "warning");
        assert_eq!(WalletCoreErrorSeverity::Warning.to_string(), "warning");
    }
}
