use crate::{WalletCoreError, WalletCoreResult};
use std::sync::Arc;

/// Abstraction for broadcasting a fully signed raw transaction to the network.
///
/// `wallet_core` owns only the behavior contract. Concrete implementations
/// (Esplora, Electrum, Bitcoin Core RPC, mocks, etc.) should live outside the
/// core crate and implement this trait.
pub trait TxBroadcaster {
    /// Broadcast a raw transaction serialized as hex.
    fn broadcast_tx_hex(&self, tx_hex: &str) -> WalletCoreResult<()>;
}

/// Test/dummy broadcaster that always succeeds.
#[derive(Debug, Default, Clone, Copy)]
pub struct NoopBroadcaster;

impl TxBroadcaster for NoopBroadcaster {
    fn broadcast_tx_hex(&self, _tx_hex: &str) -> WalletCoreResult<()> {
        Ok(())
    }
}

/// Test/dummy broadcaster that always fails using a configured error factory.
pub struct FailingBroadcaster {
    make_error: Arc<dyn Fn() -> WalletCoreError + Send + Sync>,
}

impl FailingBroadcaster {
    /// Construct a broadcaster that fails with a generic broadcast error.
    pub fn new(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            make_error: Arc::new(move || WalletCoreError::BroadcastFailed(message.clone())),
        }
    }

    /// Construct a broadcaster that fails with a transport error.
    pub fn transport(message: impl Into<String>) -> Self {
        let message = message.into();
        Self {
            make_error: Arc::new(move || WalletCoreError::BroadcastTransport(message.clone())),
        }
    }

    /// Construct a broadcaster that fails with an explicit core error factory.
    pub fn from_factory<F>(factory: F) -> Self
    where
        F: Fn() -> WalletCoreError + Send + Sync + 'static,
    {
        Self {
            make_error: Arc::new(factory),
        }
    }
}

impl TxBroadcaster for FailingBroadcaster {
    fn broadcast_tx_hex(&self, _tx_hex: &str) -> WalletCoreResult<()> {
        Err((self.make_error)())
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
    fn failing_broadcaster_returns_broadcast_failed() {
        let broadcaster = FailingBroadcaster::new("mock broadcast failure");
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(
            result,
            Err(WalletCoreError::BroadcastFailed(_))
        ));
    }

    #[test]
    fn failing_broadcaster_can_return_structured_transport_error() {
        let broadcaster = FailingBroadcaster::transport("network down");
        let result = broadcaster.broadcast_tx_hex("deadbeef");

        assert!(matches!(
            result,
            Err(WalletCoreError::BroadcastTransport(msg)) if msg.contains("network down")
        ));
    }
}