use crate::WalletSyncError;

use tracing::{debug, warn};

/// Classify a Bitcoin Core JSON-RPC rejection into a backend-agnostic sync
/// error.
///
/// This keeps Bitcoin Core-specific error code/message parsing out of the
/// broadcaster flow and centralizes the mapping from RPC rejection details to
/// `WalletSyncError`.
pub(super) fn classify_rpc_rejection(code: i64, message: &str) -> WalletSyncError {
    let normalized = message.to_ascii_lowercase();

    debug!(
        "classifying rpc rejection: code = {}, message = {}",
        code, message
    );

    if normalized.contains("txn-mempool-conflict")
        || normalized.contains("mempool conflict")
        || (code == -26 && normalized.contains("conflict"))
    {
        warn!("classified as mempool conflict: {}", message);
        return WalletSyncError::BroadcastMempoolConflict(message.to_string());
    }

    if normalized.contains("already in block chain")
        || normalized.contains("already confirmed")
        || normalized.contains("already in blockchain")
        || code == -27
    {
        warn!("classified as already confirmed: {}", message);
        return WalletSyncError::BroadcastAlreadyConfirmed(message.to_string());
    }

    if normalized.contains("missing inputs") || code == -25 {
        warn!("classified as missing inputs: {}", message);
        return WalletSyncError::BroadcastMissingInputs(message.to_string());
    }

    if normalized.contains("non-bip68-final")
        || normalized.contains("non-final")
        || normalized.contains("non-bip125-replaceable")
    {
        warn!("classified as non-final transaction: {}", message);
        return WalletSyncError::PsbtNotFinalized;
    }

    if normalized.contains("min relay fee")
        || normalized.contains("insufficient fee")
        || normalized.contains("fee not met")
        || normalized.contains("relay fee")
    {
        warn!("classified as insufficient fee: {}", message);
        return WalletSyncError::BroadcastInsufficientFee(message.to_string());
    }

    warn!(
        "classified as unknown broadcast failure: code = {}, message = {}",
        code, message
    );
    WalletSyncError::BroadcastFailed(format!(
        "bitcoin core rejected transaction: code={} message={}",
        code, message
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mempool_conflict_error() {
        let err = classify_rpc_rejection(-26, "txn-mempool-conflict");

        match err {
            WalletSyncError::BroadcastMempoolConflict(msg) => {
                assert!(msg.contains("txn-mempool-conflict"));
            }
            other => panic!("expected BroadcastMempoolConflict, got {:?}", other),
        }
    }

    #[test]
    fn classifies_already_confirmed_error() {
        let err = classify_rpc_rejection(-27, "transaction already in block chain");

        match err {
            WalletSyncError::BroadcastAlreadyConfirmed(msg) => {
                assert!(msg.contains("already in block chain"));
            }
            other => panic!("expected BroadcastAlreadyConfirmed, got {:?}", other),
        }
    }

    #[test]
    fn classifies_missing_inputs_error() {
        let err = classify_rpc_rejection(-25, "missing inputs");

        match err {
            WalletSyncError::BroadcastMissingInputs(msg) => {
                assert!(msg.contains("missing inputs"));
            }
            other => panic!("expected BroadcastMissingInputs, got {:?}", other),
        }
    }

    #[test]
    fn classifies_insufficient_fee_error() {
        let err = classify_rpc_rejection(-26, "min relay fee not met");

        match err {
            WalletSyncError::BroadcastInsufficientFee(msg) => {
                assert!(msg.contains("min relay fee"));
            }
            other => panic!("expected BroadcastInsufficientFee, got {:?}", other),
        }
    }

    #[test]
    fn classifies_non_final_error() {
        let err = classify_rpc_rejection(-26, "non-BIP68-final");
        assert!(matches!(err, WalletSyncError::PsbtNotFinalized));
    }

    #[test]
    fn classifies_unknown_rejection_as_broadcast_failed() {
        let err = classify_rpc_rejection(-26, "some unexpected rejection");

        match err {
            WalletSyncError::BroadcastFailed(msg) => {
                assert!(msg.contains("code=-26"));
                assert!(msg.contains("some unexpected rejection"));
            }
            other => panic!("expected BroadcastFailed, got {:?}", other),
        }
    }
}
