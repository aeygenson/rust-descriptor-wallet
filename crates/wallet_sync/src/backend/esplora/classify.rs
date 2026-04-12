use crate::WalletSyncError;
use tracing::{debug, warn};

/// Classify an Esplora HTTP rejection into a backend-agnostic sync error.
///
/// This keeps backend-specific message parsing out of the broadcaster flow and
/// centralizes the mapping from Esplora rejection text to `WalletSyncError`.
pub(super) fn classify_esplora_rejection(
    status: reqwest::StatusCode,
    body: &str,
) -> WalletSyncError {
    let normalized = body.to_ascii_lowercase();

    debug!("classifying esplora rejection: status = {}, body = {}", status, body);

    if normalized.contains("txn-mempool-conflict")
        || normalized.contains("mempool conflict")
    {
        warn!("classified as mempool conflict: {}", body);
        return WalletSyncError::BroadcastMempoolConflict(body.to_string());
    }

    if normalized.contains("already in block chain")
        || normalized.contains("already confirmed")
        || normalized.contains("transaction already in block chain")
    {
        warn!("classified as already confirmed: {}", body);
        return WalletSyncError::BroadcastAlreadyConfirmed(body.to_string());
    }

    if normalized.contains("missing inputs") {
        warn!("classified as missing inputs: {}", body);
        return WalletSyncError::BroadcastMissingInputs(body.to_string());
    }

    if normalized.contains("non-bip68-final") || normalized.contains("non-final") {
        warn!("classified as non-final transaction: {}", body);
        return WalletSyncError::PsbtNotFinalized;
    }

    if normalized.contains("min relay fee")
        || normalized.contains("insufficient fee")
        || normalized.contains("fee not met")
    {
        warn!("classified as insufficient fee: {}", body);
        return WalletSyncError::BroadcastInsufficientFee(body.to_string());
    }

    warn!("classified as unknown esplora failure: status = {}, body = {}", status, body);
    WalletSyncError::BroadcastFailed(format!(
        "esplora rejected transaction: status={} body={}",
        status, body
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn classifies_mempool_conflict_error() {
        let err = classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"txn-mempool-conflict\"}",
        );

        match err {
            WalletSyncError::BroadcastMempoolConflict(msg) => {
                assert!(msg.contains("txn-mempool-conflict"));
            }
            other => panic!("expected BroadcastMempoolConflict, got {:?}", other),
        }
    }

    #[test]
    fn classifies_already_confirmed_error() {
        let err = classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-27,\"message\":\"transaction already in block chain\"}",
        );

        match err {
            WalletSyncError::BroadcastAlreadyConfirmed(msg) => {
                assert!(msg.contains("already in block chain"));
            }
            other => panic!("expected BroadcastAlreadyConfirmed, got {:?}", other),
        }
    }

    #[test]
    fn classifies_missing_inputs_error() {
        let err = classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-25,\"message\":\"missing inputs\"}",
        );

        match err {
            WalletSyncError::BroadcastMissingInputs(msg) => {
                assert!(msg.contains("missing inputs"));
            }
            other => panic!("expected BroadcastMissingInputs, got {:?}", other),
        }
    }

    #[test]
    fn classifies_insufficient_fee_error() {
        let err = classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "sendrawtransaction RPC error: {\"code\":-26,\"message\":\"min relay fee not met\"}",
        );

        match err {
            WalletSyncError::BroadcastInsufficientFee(msg) => {
                assert!(msg.contains("min relay fee"));
            }
            other => panic!("expected BroadcastInsufficientFee, got {:?}", other),
        }
    }

    #[test]
    fn classifies_unknown_rejection_as_broadcast_failed() {
        let err = classify_esplora_rejection(
            reqwest::StatusCode::BAD_REQUEST,
            "some unexpected rejection",
        );

        match err {
            WalletSyncError::BroadcastFailed(msg) => {
                assert!(msg.contains("status=400"));
                assert!(msg.contains("some unexpected rejection"));
            }
            other => panic!("expected BroadcastFailed, got {:?}", other),
        }
    }
}
