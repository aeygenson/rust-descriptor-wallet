# Wallet Sync Overview

`wallet_sync` is the blockchain-integration crate for the workspace.

It owns wallet synchronization against configured chain backends and broadcasting of fully signed raw transactions. It does not build transactions and it does not persist wallet registry metadata.

## Public Boundary

The crate exports:

- `WalletSyncService`
- `WalletSyncError`
- `WalletSyncResult<T>`
- backend-facing `model` types
- the `TxBroadcaster` trait

The main entry point used by `wallet_api` is `WalletSyncService`.

## What The Service Does

`WalletSyncService` provides two top-level operations:

- `sync(&mut WalletService, &WalletConfig) -> WalletSyncResult<()>`
- `broadcast_tx_hex(&WalletConfig, &str) -> WalletSyncResult<()>`

Sync dispatches from `WalletConfig.backend.sync`.

Broadcast dispatches from `WalletConfig.backend.broadcast`.

The service also builds a small `BackendProfile` for logs and diagnostics.

## Supported Backends Today

Sync backends:

- Esplora
- Electrum

Broadcast backends:

- Esplora
- Bitcoin Core RPC
- No-op mock fallback when no broadcast backend is configured

There is also a `mock` backend module for testing and a placeholder `p2p` module, but production dispatch in `service.rs` currently routes only to Esplora, Electrum, and Core RPC.

## Relationship To Other Crates

`wallet_core` owns the wallet state machine and BDK wallet behavior.

`wallet_sync` updates a loaded `WalletService` from the chain and broadcasts finalized raw transaction hex.

`wallet_api` chooses when to call sync or broadcast and converts `WalletSyncError` into API-level errors.

`wallet_storage` stores backend configuration strings, but does not perform network operations.

## What This Crate Does Not Own

`wallet_sync` does not:

- construct PSBTs
- sign PSBTs
- finalize PSBTs
- choose wallet inputs
- parse caller DTOs
- store wallet registry data

It operates on a ready `WalletConfig`, a loaded `WalletService`, and fully signed raw transaction hex.
