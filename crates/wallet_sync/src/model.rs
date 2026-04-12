

/// Shared backend-facing models for the `wallet_sync` crate.
///
/// These are intentionally small, backend-agnostic data structures used by the
/// sync facade and backend implementations. They should not contain transport
/// logic, HTTP clients, or backend-specific parsing code.

/// Backend selection for blockchain synchronization.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyncBackendKind {
    Esplora,
    Electrum,
}

/// Backend selection for transaction broadcast.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BroadcastBackendKind {
    Esplora,
    CoreRpc,
    Mock,
}

/// Small summary of the configured backend pair for a wallet.
///
/// This is useful in logs, diagnostics, and service-level dispatch without
/// exposing transport details everywhere.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackendProfile {
    pub sync: SyncBackendKind,
    pub broadcast: Option<BroadcastBackendKind>,
}

impl BackendProfile {
    pub fn new(sync: SyncBackendKind, broadcast: Option<BroadcastBackendKind>) -> Self {
        Self { sync, broadcast }
    }

    pub fn sync_label(&self) -> &'static str {
        match self.sync {
            SyncBackendKind::Esplora => "esplora",
            SyncBackendKind::Electrum => "electrum",
        }
    }

    pub fn broadcast_label(&self) -> Option<&'static str> {
        self.broadcast.as_ref().map(|kind| match kind {
            BroadcastBackendKind::Esplora => "esplora",
            BroadcastBackendKind::CoreRpc => "core_rpc",
            BroadcastBackendKind::Mock => "mock",
        })
    }
}

/// Minimal result returned by broadcast backends after a transaction was
/// accepted for relay.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TxBroadcastResult {
    pub txid: String,
    pub replaceable: Option<bool>,
}

impl TxBroadcastResult {
    pub fn new(txid: impl Into<String>, replaceable: Option<bool>) -> Self {
        Self {
            txid: txid.into(),
            replaceable,
        }
    }
}