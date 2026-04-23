# Wallet API Architecture

`wallet_api` is a facade and orchestration crate. It sits between app callers and the lower-level wallet, storage, and chain crates.

## Module Layout

```text
crates/wallet_api/
├── src/
│   ├── api.rs
│   ├── error.rs
│   ├── factory.rs
│   ├── lib.rs
│   ├── model.rs
│   └── service/
│       ├── inspect.rs
│       ├── mod.rs
│       ├── psbt.rs
│       ├── registry.rs
│       └── wallet.rs
└── tests/
    ├── regtest_flow.rs
    └── support/
        └── mod.rs
```

## Module Responsibilities

`api.rs` exposes the public `WalletApi` facade. It keeps caller methods small and delegates to service modules.

`model.rs` defines caller-facing DTOs, backend DTOs, selection-mode DTOs, and conversion helpers into wallet-core request types.

`error.rs` defines `WalletApiError` and maps `wallet_core`, `wallet_sync`, and `wallet_storage` failures into API-level categories.

`factory.rs` builds a default `WalletApi` with shared `WalletCore`, `WalletStorage`, and `WalletSyncService` dependencies.

`service/registry.rs` handles imported wallet metadata through `wallet_storage`.

`service/wallet.rs` loads stored wallet configuration, converts backend settings into `WalletConfig`, and handles address, sync, balance, and status operations.

`service/inspect.rs` loads wallet state and returns transaction or UTXO DTOs. It does not perform network calls; callers should run `sync_wallet` first when they need fresh chain data.

`service/psbt.rs` owns transaction orchestration: fixed sends, coin control, send-max, sweep, consolidation, signing, publishing, RBF, and CPFP.

## Request Flow

Typical request flow:

```text
caller
  -> WalletApi method
  -> DTO parsing and validation
  -> service module
  -> wallet_storage / wallet_sync / wallet_core
  -> DTO result or WalletApiError
```

The public API stays stable while internals can evolve behind the service layer.

## Async And Blocking Work

The API is async because callers and backend services are async. Wallet operations that need blocking BDK/file-store work are isolated with Tokio blocking helpers inside the service layer.

The regtest integration tests run the API through a single-threaded test harness for deterministic shared regtest infrastructure, while production API calls still use async service methods.

## Boundaries

`wallet_api` owns orchestration, DTO conversion, and error normalization.

`wallet_core` owns transaction and wallet rules.

`wallet_sync` owns sync and broadcast backends.

`wallet_storage` owns persisted wallet metadata.

`wallet_cli` and future `wallet_desktop` should remain thin callers over this API.
