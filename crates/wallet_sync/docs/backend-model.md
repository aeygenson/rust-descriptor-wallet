# Backend Model

`wallet_sync` models backends in two layers:

- `wallet_core::config::{SyncBackendConfig, BroadcastBackendConfig}` carries configured backend values.
- `wallet_sync::model` defines small backend-kind summaries used for service-level dispatch and logging.

## Sync Backend Kinds

`SyncBackendKind` currently supports:

- `Esplora`
- `Electrum`

These correspond to `WalletConfig.backend.sync`.

## Broadcast Backend Kinds

`BroadcastBackendKind` currently supports:

- `Esplora`
- `CoreRpc`
- `Mock`

`Mock` is used by the model layer and test backends. The main sync service uses a no-op broadcaster when no broadcast backend is configured.

## Backend Profile

`BackendProfile` is a small summary of the configured sync and broadcast pair:

- `sync: SyncBackendKind`
- `broadcast: Option<BroadcastBackendKind>`

It exists mainly for logs and diagnostics:

- `sync_label()`
- `broadcast_label()`

## Dispatch Rules

`WalletSyncService::sync` matches on `config.backend.sync`:

- `SyncBackendConfig::Esplora` -> `backend::esplora::sync::sync_wallet_esplora`
- `SyncBackendConfig::Electrum` -> `backend::electrum::sync::sync_wallet_electrum`

`WalletSyncService::broadcast_tx_hex` matches on `config.backend.broadcast`:

- `BroadcastBackendConfig::Esplora` -> `EsploraBroadcaster`
- `BroadcastBackendConfig::Rpc` -> `CoreRpcBroadcaster`
- `None` -> `NoopBroadcaster`

## Backend Traits And Types

Broadcast behavior is abstracted by `TxBroadcaster`:

- `broadcast_tx_hex(&self, tx_hex: &str) -> WalletSyncResult<()>`

Concrete broadcasters live in backend modules:

- `backend/esplora/broadcast.rs`
- `backend/core_rpc/broadcast.rs`
- `backend/mock/broadcast.rs`

Sync does not currently use a trait object boundary; the facade dispatches directly to backend-specific async functions.

## Capability Reality

The crate is not fully capability-generic yet.

Actual behavior today:

- Electrum: sync only
- Esplora: sync and broadcast
- Core RPC: broadcast only
- Mock: broadcast test doubles

That is the model the docs and callers should assume.
