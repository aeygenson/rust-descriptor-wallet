use crate::broadcast::TxBroadcaster;
use crate::{WalletSyncError, WalletSyncResult};
use std::sync::Arc;
use tracing::{debug, info, warn};

/// Dummy broadcaster that always succeeds.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopBroadcaster;

impl TxBroadcaster for NoopBroadcaster {
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()> {
        debug!("noop broadcaster called, tx_hex_len = {}", tx_hex.len());
        info!("mock broadcast success (noop)");
        Ok(())
    }
}

/// Dummy broadcaster that always fails using a configured error factory.
pub struct FailingBroadcaster {
    make_error: Arc<dyn Fn() -> WalletSyncError + Send + Sync>,
}

impl FailingBroadcaster {
    /// Construct a broadcaster that fails with a transport error.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        debug!("creating failing broadcaster (generic): {}", message);
        Self {
            make_error: Arc::new(move || WalletSyncError::BroadcastTransport(message.clone())),
        }
    }

    /// Construct a broadcaster that fails with a transport error.
    pub fn transport(message: impl Into<String>) -> Self {
        let message = message.into();
        debug!("creating failing broadcaster (transport): {}", message);
        Self {
            make_error: Arc::new(move || WalletSyncError::BroadcastTransport(message.clone())),
        }
    }

    /// Construct a broadcaster that fails with a mempool conflict error.
    pub fn mempool_conflict(message: impl Into<String>) -> Self {
        let message = message.into();
        debug!(
            "creating failing broadcaster (mempool conflict): {}",
            message
        );
        Self {
            make_error: Arc::new(move || {
                WalletSyncError::BroadcastMempoolConflict(message.clone())
            }),
        }
    }

    /// Construct a broadcaster that fails with an explicit error factory.
    pub fn from_factory<F>(factory: F) -> Self
    where
        F: Fn() -> WalletSyncError + Send + Sync + 'static,
    {
        debug!("creating failing broadcaster from custom factory");
        Self {
            make_error: Arc::new(factory),
        }
    }
}

impl TxBroadcaster for FailingBroadcaster {
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()> {
        debug!("failing broadcaster called, tx_hex_len = {}", tx_hex.len());
        let err = (self.make_error)();
        warn!("mock broadcast failure: {:?}", err);
        Err(err)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn noop_broadcaster_succeeds() {
        let broadcaster = NoopBroadcaster;
        let result = broadcaster.broadcast_tx_hex("deadbeef");
        assert!(result.is_ok());
    }

    #[test]
    fn failing_broadcaster_returns_transport_error() {
        let broadcaster = FailingBroadcaster::transport("network down");
        let result = broadcaster.broadcast_tx_hex("deadbeef");
        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastTransport(_))
        ));
    }

    #[test]
    fn failing_broadcaster_returns_mempool_conflict_error() {
        let broadcaster = FailingBroadcaster::mempool_conflict("txn-mempool-conflict");
        let result = broadcaster.broadcast_tx_hex("deadbeef");
        assert!(matches!(
            result,
            Err(WalletSyncError::BroadcastMempoolConflict(_))
        ));
    }
}
