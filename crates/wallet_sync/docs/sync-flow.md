# Sync And Broadcast Flow

This document describes the real `wallet_sync` flow used by the current code.

## Sync Flow

High-level path:

```text
wallet_api
  -> WalletSyncService::sync
  -> configured sync backend
  -> backend full scan
  -> wallet.apply_update(...)
  -> wallet.persist()
```

The service receives:

- a mutable `WalletService`
- a `WalletConfig`

It does not load wallets from storage itself.

## Esplora Sync

`backend/esplora/sync.rs`:

- requires `SyncBackendConfig::Esplora`
- builds a blocking Esplora client with `bdk_esplora`
- starts a BDK full scan from the loaded wallet
- uses:
  - `PARALLEL_REQUESTS = 5`
  - `STOP_GAP = 25`
- applies the update to the wallet
- persists the wallet

Backend mismatch returns `WalletSyncError::InvalidBackend`.

Esplora scan failures map to `WalletSyncError::SyncFailed`.

## Electrum Sync

`backend/electrum/sync.rs`:

- requires `SyncBackendConfig::Electrum`
- when the `electrum` feature is enabled, builds a `bdk_electrum` client
- starts a BDK full scan from the loaded wallet
- uses:
  - `STOP_GAP = 25`
  - `BATCH_SIZE = 50`
  - `FETCH_PREV_TXOUTS = false`
- applies the update to the wallet
- persists the wallet

If the crate is built without the `electrum` feature, the same function returns `WalletSyncError::BackendUnavailable`.

## Broadcast Flow

High-level path:

```text
wallet_api
  -> wallet_core finalize_psbt_for_broadcast
  -> WalletSyncService::broadcast_tx_hex
  -> selected broadcaster
```

`wallet_sync` broadcasts raw transaction hex, not PSBTs.

## Esplora Broadcast

`backend/esplora/broadcast.rs`:

- POSTs raw hex to `<base_url>/tx`
- uses blocking `reqwest`
- trims trailing slashes from the configured base URL
- retries on server errors, `429`, and request timeout
- classifies backend rejections into structured `WalletSyncError` variants

Possible mappings include:

- mempool conflict
- PSBT not finalized style errors when backend response reports non-final semantics
- transport errors

## Core RPC Broadcast

`backend/core_rpc/broadcast.rs`:

- sends JSON-RPC `sendrawtransaction`
- uses blocking `reqwest`
- retries on retryable HTTP status and retryable RPC code `-28`
- classifies RPC rejection codes and messages into structured sync errors

Possible mappings include:

- `BroadcastMempoolConflict`
- `BroadcastAlreadyConfirmed`
- `BroadcastMissingInputs`
- `BroadcastInsufficientFee`
- generic `BroadcastFailed`
- `BroadcastTransport`

## No Broadcast Backend

If `WalletConfig.backend.broadcast` is `None`, `WalletSyncService` uses `NoopBroadcaster`.

That is mainly useful for tests and development. It reports success without sending the transaction anywhere.

## Error Surface

All backend failures map into `WalletSyncError`.

Important variants:

- `InvalidBackend`
- `BackendUnavailable`
- `SyncFailed`
- `BroadcastTransport`
- `BroadcastFailed`
- `BroadcastMempoolConflict`
- `BroadcastAlreadyConfirmed`
- `BroadcastMissingInputs`
- `BroadcastInsufficientFee`
- `PsbtNotFinalized`
- `Core`

`WalletSyncError::into_core()` can map some sync-layer failures back into `WalletCoreError` when needed.
