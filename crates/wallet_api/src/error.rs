use thiserror::Error;
use wallet_core::error::WalletCoreError;
use wallet_storage::error::WalletStorageError;
use wallet_sync::error::WalletSyncError;

#[derive(Debug, Error)]
pub enum WalletApiError {
    #[error("broadcast transport error: {0}")]
    BroadcastTransport(String),

    #[error("transaction broadcast failed: {0}")]
    BroadcastFailed(String),

    #[error("mempool conflict: {0}")]
    BroadcastMempoolConflict(String),

    #[error("transaction already confirmed: {0}")]
    BroadcastAlreadyConfirmed(String),

    #[error("missing inputs: {0}")]
    BroadcastMissingInputs(String),

    #[error("insufficient relay fee: {0}")]
    BroadcastInsufficientFee(String),

    #[error("sync error: {0}")]
    Sync(String),

    #[error("invalid backend: {0}")]
    InvalidBackend(String),

    #[error("QR generation failed: {0}")]
    QrGeneration(String),

    #[error("address book label already exists: {0}")]
    DuplicateAddressBookLabel(String),

    #[error("address book address already exists: {0}")]
    DuplicateAddressBookAddress(String),

    #[error("locked utxo already exists: {0}")]
    DuplicateLockedUtxo(String),

    #[error("locked utxo not found: {0}")]
    LockedUtxoNotFound(String),

    #[error("utxo is locked and cannot be spent: {0}")]
    LockedUtxo(String),

    #[error("invalid address book address: {0}")]
    InvalidAddressBookAddress(String),

    #[error("backend unavailable: {0}")]
    BackendUnavailable(String),

    #[error("backend health check failed: {0}")]
    BackendHealth(String),

    #[error(transparent)]
    Storage(WalletStorageError),

    #[error(transparent)]
    Core(WalletCoreError),

    #[error("not found: {0}")]
    NotFound(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("invalid amount")]
    InvalidAmount,

    #[error("invalid fee rate")]
    InvalidFeeRate,

    #[error("transaction not found: {0}")]
    TransactionNotFound(String),

    #[error("transaction already confirmed: {0}")]
    TransactionAlreadyConfirmed(String),

    #[error("transaction is not replaceable (RBF disabled): {0}")]
    TransactionNotReplaceable(String),

    #[error(
        "fee rate too low for bump (original: {original_sat_per_vb} sat/vB, requested: {requested_sat_per_vb} sat/vB)"
    )]
    FeeRateTooLowForBump {
        original_sat_per_vb: u64,
        requested_sat_per_vb: u64,
    },

    #[error("fee bump build failed: {0}")]
    FeeBumpBuildFailed(String),

    #[error("selection failed: {0}")]
    SelectionFailed(String),

    #[error("unsupported operation: {0}")]
    NotImplemented(String),

    #[error("cpfp build failed: {0}")]
    CpfpBuildFailed(String),

    #[error("invalid destination address: {0}")]
    InvalidDestinationAddress(String),

    #[error("destination network mismatch: {0}")]
    DestinationNetworkMismatch(String),

    #[error("psbt build failed: {0}")]
    PsbtBuildFailed(String),

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

    #[error("psbt signing failed: {0}")]
    SignPsbtFailed(String),

    #[error("wallet is watch-only and cannot sign")]
    WatchOnlyCannotSign,

    #[error("psbt is not finalized")]
    PsbtNotFinalized,

    #[error("signed PSBT must be finalized before publish")]
    SendNotFinalized,

    #[error("failed to extract transaction from psbt: {0}")]
    ExtractTxFailed(String),
}

impl From<WalletStorageError> for WalletApiError {
    fn from(error: WalletStorageError) -> Self {
        match error {
            WalletStorageError::DuplicateAddressBookLabel(message) => {
                Self::DuplicateAddressBookLabel(message)
            }
            WalletStorageError::DuplicateAddressBookAddress(message) => {
                Self::DuplicateAddressBookAddress(message)
            }
            WalletStorageError::DuplicateLockedUtxo(message) => {
                Self::DuplicateLockedUtxo(message)
            }
            WalletStorageError::LockedUtxoNotFound(message) => {
                Self::LockedUtxoNotFound(message)
            }
            WalletStorageError::LockedUtxo(message) => {
                Self::LockedUtxo(message)
            }
            WalletStorageError::InvalidAddressBookAddress(message) => {
                Self::InvalidAddressBookAddress(message)
            }
            other => Self::Storage(other),
        }
    }
}

impl From<WalletSyncError> for WalletApiError {
    fn from(e: WalletSyncError) -> Self {
        match e {
            WalletSyncError::BroadcastTransport(s) => WalletApiError::BroadcastTransport(s),
            WalletSyncError::BroadcastFailed(s) => {
                let normalized = s.to_ascii_lowercase();

                if normalized.contains("non-final") {
                    WalletApiError::PsbtNotFinalized
                } else {
                    WalletApiError::BroadcastFailed(s)
                }
            }
            WalletSyncError::BroadcastMempoolConflict(s) => {
                WalletApiError::BroadcastMempoolConflict(s)
            }
            WalletSyncError::BroadcastAlreadyConfirmed(s) => {
                WalletApiError::BroadcastAlreadyConfirmed(s)
            }
            WalletSyncError::BroadcastMissingInputs(s) => WalletApiError::BroadcastMissingInputs(s),
            WalletSyncError::BroadcastInsufficientFee(s) => {
                WalletApiError::BroadcastInsufficientFee(s)
            }
            WalletSyncError::PsbtNotFinalized => WalletApiError::PsbtNotFinalized,
            WalletSyncError::InvalidBackend(s) => WalletApiError::InvalidBackend(s),
            WalletSyncError::BackendUnavailable(s) => WalletApiError::BackendUnavailable(s),
            WalletSyncError::Core(core) => WalletApiError::from(core),
            other => WalletApiError::Sync(other.to_string()),
        }
    }
}

impl From<WalletCoreError> for WalletApiError {
    fn from(e: WalletCoreError) -> Self {
        match e {
            WalletCoreError::InvalidAmount => WalletApiError::InvalidAmount,
            WalletCoreError::InvalidFeeRate => WalletApiError::InvalidFeeRate,
            WalletCoreError::TransactionNotFound(txid) => {
                WalletApiError::TransactionNotFound(txid)
            }
            WalletCoreError::TransactionAlreadyConfirmed(txid) => {
                WalletApiError::TransactionAlreadyConfirmed(txid)
            }
            WalletCoreError::TransactionNotReplaceable(txid) => {
                WalletApiError::TransactionNotReplaceable(txid)
            }
            WalletCoreError::FeeRateTooLowForBump {
                original_sat_per_vb,
                requested_sat_per_vb,
                ..
            } => WalletApiError::FeeRateTooLowForBump {
                original_sat_per_vb: original_sat_per_vb.as_u64(),
                requested_sat_per_vb: requested_sat_per_vb.as_u64(),
            },
            WalletCoreError::FeeBumpBuildFailed { reason, .. } => {
                WalletApiError::FeeBumpBuildFailed(reason)
            }
            WalletCoreError::CpfpBuildFailed { reason, .. } => {
                WalletApiError::CpfpBuildFailed(reason)
            }
            WalletCoreError::InvalidDestinationAddress(s) => {
                WalletApiError::InvalidDestinationAddress(s)
            }
            WalletCoreError::DestinationNetworkMismatch(s) => {
                WalletApiError::DestinationNetworkMismatch(s)
            }
            WalletCoreError::PsbtBuildFailed(s) => WalletApiError::PsbtBuildFailed(s),
            WalletCoreError::FeeCalculationFailedWithReason(reason) => {
                WalletApiError::FeeCalculationFailedWithReason(reason)
            }
            WalletCoreError::FeeCalculationFailed => WalletApiError::FeeCalculationFailed,
            WalletCoreError::InvalidPsbtEncoding(reason) => {
                WalletApiError::InvalidPsbtEncoding(reason)
            }
            WalletCoreError::InvalidPsbtStructure(reason) => {
                WalletApiError::InvalidPsbtStructure(reason)
            }
            WalletCoreError::InvalidPsbtSemantic(reason) => {
                WalletApiError::InvalidPsbtSemantic(reason)
            }
            WalletCoreError::InvalidPsbt(e) => {
                WalletApiError::InvalidPsbt(e.to_string())
            }
            WalletCoreError::PsbtConversionFailed { reason, .. } => {
                WalletApiError::InvalidPsbt(reason)
            }
            WalletCoreError::SignPsbtFailed(e) => {
                WalletApiError::SignPsbtFailed(e.to_string())
            }
            WalletCoreError::WatchOnlyCannotSign => {
                WalletApiError::WatchOnlyCannotSign
            }
            WalletCoreError::PsbtNotFinalized => {
                WalletApiError::PsbtNotFinalized
            }
            WalletCoreError::ExtractTxFailed(s) => {
                WalletApiError::ExtractTxFailed(s)
            }
            WalletCoreError::InvalidConfig(s) => {
                WalletApiError::InvalidInput(s)
            }
            WalletCoreError::InvalidState(s) => {
                WalletApiError::InvalidInput(s)
            }
            WalletCoreError::InvalidTxid(s) => {
                WalletApiError::InvalidInput(format!("invalid txid: {}", s))
            }
            WalletCoreError::InvalidOutpoint(s) => {
                WalletApiError::InvalidInput(format!("invalid outpoint: {}", s))
            }
            WalletCoreError::InvalidVsize(s) => {
                WalletApiError::InvalidInput(format!("invalid vsize: {}", s))
            }
            WalletCoreError::InvalidBlockHeight(s) => {
                WalletApiError::InvalidInput(format!("invalid block height: {}", s))
            }
            WalletCoreError::InvalidPercent(s) => {
                WalletApiError::InvalidInput(format!("invalid percent: {}", s))
            }
            WalletCoreError::InvalidPsbtBase64(s) => {
                WalletApiError::InvalidInput(format!("invalid psbt base64: {}", s))
            }
            WalletCoreError::InvalidTxHex(s) => {
                WalletApiError::InvalidInput(format!("invalid transaction hex: {}", s))
            }
            WalletCoreError::CoinControlOutpointNotFound(s) => {
                WalletApiError::InvalidInput(format!("coin control outpoint not found: {}", s))
            }
            WalletCoreError::CoinControlOutpointNotSpendable(s) => {
                WalletApiError::InvalidInput(format!("coin control outpoint not spendable: {}", s))
            }
            WalletCoreError::CoinControlOutpointNotConfirmed(s) => {
                WalletApiError::InvalidInput(format!("coin control outpoint not confirmed: {}", s))
            }
            WalletCoreError::CoinControlConflict(s) => {
                WalletApiError::InvalidInput(format!("coin control conflict: {}", s))
            }
            WalletCoreError::LockedUtxo(s) => {
                WalletApiError::LockedUtxo(s)
            }
            WalletCoreError::CoinControlEmptySelection => {
                WalletApiError::InvalidInput("coin control selection is empty".to_string())
            }
            WalletCoreError::CoinControlInvalidOutpoint(s) => {
                WalletApiError::InvalidInput(format!("invalid coin control outpoint: {}", s))
            }
            WalletCoreError::CoinControlStrictModeViolation => {
                WalletApiError::InvalidInput(
                    "coin control strict manual mode requires explicit selected inputs".to_string(),
                )
            }
            WalletCoreError::SelectionFailed(s) => {
                WalletApiError::SelectionFailed(s)
            }
            WalletCoreError::SendMaxAmountTooSmall => {
                WalletApiError::InvalidInput(
                    "send max amount is too small after fees".to_string(),
                )
            }
            WalletCoreError::CoinControlInsufficientSelectedFunds { selected_sat, required_sat, fee_estimate_sat } => {
                WalletApiError::InvalidInput(format!(
                    "coin control insufficient funds: selected={} required={} fee_estimate={}",
                    selected_sat, required_sat, fee_estimate_sat
                ))
            }
            WalletCoreError::ConsolidationTooFewInputs => {
                WalletApiError::InvalidInput(
                    "consolidation error: requires at least two eligible UTXOs after applying the current selection and filters".to_string(),
                )
            }
            WalletCoreError::ConsolidationAmountTooSmall => {
                WalletApiError::InvalidInput(
                    "consolidation error: selected inputs do not leave a usable consolidation amount after fees".to_string(),
                )
            }
            WalletCoreError::ConsolidationMinInputNotMet { required, actual } => {
                WalletApiError::InvalidInput(format!(
                    "consolidation error: minimum input count not met after applying selection and filters: required={} actual={}",
                    required, actual
                ))
            }
            WalletCoreError::ConsolidationValueFilterMismatch => {
                WalletApiError::InvalidInput(
                    "consolidation error: one or more selected inputs do not satisfy the configured value filters".to_string(),
                )
            }
            WalletCoreError::ConsolidationFeeTooHigh {
                fee_sat,
                total_input_sat,
                max_pct,
            } => {
                WalletApiError::InvalidInput(format!(
                    "consolidation error: estimated fee exceeds the configured limit: fee={} total_inputs={} max_pct={}",
                    fee_sat, total_input_sat, max_pct
                ))
            }
            WalletCoreError::ConsolidationNoEligibleUtxos => {
                WalletApiError::InvalidInput(
                    "consolidation error: no eligible UTXOs remain after applying the current filters and strategy".to_string(),
                )
            }
            WalletCoreError::CpfpEmptyParentTxid => {
                WalletApiError::InvalidInput("cpfp parent txid is empty".to_string())
            }
            WalletCoreError::CpfpNoCandidateUtxo(txid) => {
                WalletApiError::InvalidInput(format!(
                    "cpfp error: no candidate child UTXO found for parent {}",
                    txid
                ))
            }
            WalletCoreError::CpfpParentNotFound(txid) => {
                WalletApiError::TransactionNotFound(txid)
            }
            WalletCoreError::CpfpParentAlreadyConfirmed(txid) => {
                WalletApiError::TransactionAlreadyConfirmed(txid)
            }
            WalletCoreError::CpfpInsufficientValue(reason) => {
                WalletApiError::InvalidInput(format!("cpfp insufficient value: {}", reason))
            }
            WalletCoreError::TransactionFeeUnavailable { txid, reason } => {
                WalletApiError::InvalidInput(format!(
                    "transaction fee unavailable for {}: {}",
                    txid, reason
                ))
            }
            WalletCoreError::TransactionVsizeUnavailable(txid) => {
                WalletApiError::InvalidInput(format!(
                    "transaction vsize unavailable for {}",
                    txid
                ))
            }
            WalletCoreError::Store(s) => {
                WalletApiError::InvalidInput(format!("wallet persistence error: {}", s))
            }
            WalletCoreError::StoreWithDump(s) => {
                WalletApiError::InvalidInput(format!("wallet persistence error: {}", s))
            }
            WalletCoreError::Load(s) => {
                WalletApiError::InvalidInput(format!("wallet persistence load error: {}", s))
            }
            WalletCoreError::Create(s) => {
                WalletApiError::InvalidInput(format!("wallet persistence create error: {}", s))
            }
            WalletCoreError::Persist(s) => {
                WalletApiError::InvalidInput(format!("wallet persistence save error: {}", s))
            }
            WalletCoreError::NotImplemented(s) => {
                WalletApiError::NotImplemented(s)
            }
        }
    }
}

impl WalletApiError {
    pub fn category(&self) -> &'static str {
        match self {
            Self::Core(core) => core.category().as_str(),
            Self::Storage(_) => "persistence",
            Self::BroadcastTransport(_)
            | Self::Sync(_)
            | Self::InvalidBackend(_)
            | Self::BackendUnavailable(_)
            | Self::BackendHealth(_) => "backend",
            Self::BroadcastFailed(_)
            | Self::BroadcastMempoolConflict(_)
            | Self::BroadcastAlreadyConfirmed(_)
            | Self::BroadcastMissingInputs(_)
            | Self::BroadcastInsufficientFee(_) => "broadcast",
            Self::InvalidInput(_)
            | Self::InvalidAmount
            | Self::InvalidFeeRate
            | Self::InvalidDestinationAddress(_)
            | Self::DestinationNetworkMismatch(_)
            | Self::SelectionFailed(_) => "validation",
            Self::QrGeneration(_) => "receive",
            Self::DuplicateAddressBookLabel(_)
            | Self::DuplicateAddressBookAddress(_)
            | Self::InvalidAddressBookAddress(_) => "address-book",
            Self::DuplicateLockedUtxo(_)
            | Self::LockedUtxoNotFound(_)
            | Self::LockedUtxo(_) => "coin-control",
            Self::TransactionNotFound(_)
            | Self::TransactionAlreadyConfirmed(_)
            | Self::TransactionNotReplaceable(_) => "transaction",
            Self::FeeRateTooLowForBump { .. }
            | Self::FeeBumpBuildFailed(_)
            | Self::FeeCalculationFailedWithReason(_)
            | Self::FeeCalculationFailed => "fee",
            Self::CpfpBuildFailed(_) => "cpfp",
            Self::PsbtBuildFailed(_)
            | Self::InvalidPsbtEncoding(_)
            | Self::InvalidPsbtStructure(_)
            | Self::InvalidPsbtSemantic(_)
            | Self::InvalidPsbt(_)
            | Self::SignPsbtFailed(_)
            | Self::WatchOnlyCannotSign
            | Self::PsbtNotFinalized
            | Self::SendNotFinalized
            | Self::ExtractTxFailed(_) => "psbt",
            Self::NotFound(_) => "not-found",
            Self::NotImplemented(_) => "unsupported",
        }
    }

    pub fn recovery(&self) -> &'static str {
        match self {
            Self::Core(core) => core.recovery().as_str(),
            Self::BroadcastTransport(_)
            | Self::Sync(_)
            | Self::BackendUnavailable(_)
            | Self::BackendHealth(_)
            | Self::Storage(_)
            | Self::QrGeneration(_) => "retry",
            Self::TransactionNotFound(_)
            | Self::BroadcastMissingInputs(_) => "refresh-state",
            Self::InvalidInput(_)
            | Self::InvalidAmount
            | Self::InvalidFeeRate
            | Self::InvalidDestinationAddress(_)
            | Self::DestinationNetworkMismatch(_)
            | Self::FeeRateTooLowForBump { .. }
            | Self::TransactionAlreadyConfirmed(_)
            | Self::TransactionNotReplaceable(_)
            | Self::PsbtNotFinalized
            | Self::SendNotFinalized
            | Self::SelectionFailed(_)
            | Self::DuplicateAddressBookLabel(_)
            | Self::DuplicateAddressBookAddress(_)
            | Self::InvalidAddressBookAddress(_)
            | Self::DuplicateLockedUtxo(_)
            | Self::LockedUtxoNotFound(_)
            | Self::LockedUtxo(_) => "user-action",
            Self::WatchOnlyCannotSign | Self::InvalidBackend(_) => "fix-configuration",
            Self::NotImplemented(_) => "unsupported",
            _ => "fatal",
        }
    }

    pub fn severity(&self) -> &'static str {
        match self {
            Self::Core(core) => core.severity().as_str(),
            Self::InvalidInput(_)
            | Self::InvalidAmount
            | Self::InvalidFeeRate
            | Self::InvalidDestinationAddress(_)
            | Self::DestinationNetworkMismatch(_)
            | Self::SelectionFailed(_)
            | Self::DuplicateAddressBookLabel(_)
            | Self::DuplicateAddressBookAddress(_)
            | Self::InvalidAddressBookAddress(_)
            | Self::DuplicateLockedUtxo(_)
            | Self::LockedUtxoNotFound(_)
            | Self::LockedUtxo(_)
            | Self::NotImplemented(_) => "info",
            Self::BroadcastTransport(_)
            | Self::Sync(_)
            | Self::BackendUnavailable(_)
            | Self::BackendHealth(_)
            | Self::Storage(_)
            | Self::QrGeneration(_)
            | Self::TransactionNotFound(_)
            | Self::BroadcastMissingInputs(_) => "warning",
            _ => "error",
        }
    }
}
