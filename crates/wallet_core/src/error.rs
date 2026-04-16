use bitcoin::psbt::PsbtParseError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum WalletCoreError {
    #[error("invalid state: {0}")]
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

    #[error("coin control include set is empty while exact selection is required")]
    CoinControlEmptySelection,

    #[error("coin control strict mode violation: selected inputs do not fully fund the transaction and automatic additional inputs are not allowed")]
    CoinControlStrictModeViolation,

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

    #[error("fee calculation failed")]
    FeeCalculationFailed,

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

impl From<PsbtParseError> for WalletCoreError {
    fn from(e: PsbtParseError) -> Self {
        WalletCoreError::InvalidPsbt(e.to_string())
    }
}
