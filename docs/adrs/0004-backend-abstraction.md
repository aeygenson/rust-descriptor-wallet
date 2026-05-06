# ADR 0004: Backend Abstraction

## Status

Accepted

## Context

The wallet needs external blockchain integration for two distinct concerns:

- synchronizing wallet state
- broadcasting fully signed transactions

Those concerns are not backed by the same transports in every environment.
Today, the implemented code supports:

- sync through Esplora or Electrum
- broadcast through Esplora or Bitcoin Core RPC
- a no-op/mock-style broadcaster fallback when no broadcast backend is
  configured

Wallet construction logic should not know about HTTP clients, Electrum clients,
or RPC rejection parsing.

## Decision

The repository isolates sync and broadcast integration inside `crates/wallet_sync`.

`WalletSyncService` is the application-facing facade:

- `sync()` dispatches by `SyncBackendConfig`
- `broadcast_tx_hex()` dispatches by `BroadcastBackendConfig`

The current backend split is:

- sync backends
  - Esplora
  - Electrum

- broadcast backends
  - Esplora
  - Core RPC
  - `NoopBroadcaster` fallback when broadcast config is absent

Backend-specific error classification stays inside backend modules.

## Rationale

### Keep wallet-core transport-agnostic

`wallet_core` should reason about wallet state and transaction construction, not
about backend protocols.

### Keep sync and broadcast independently configurable

The project supports different backend choices for syncing and broadcasting, so
the abstraction should reflect that split instead of assuming one backend does
everything.

### Normalize backend-specific failures at the edge

Esplora and Core RPC reject transactions differently. Parsing and classification
belong in `wallet_sync`, not in callers.

### Leave room for test doubles without making them the public model

The codebase contains mock broadcast implementations for tests, but the
production-facing dispatch remains centered on the real backends above.

## Consequences

### Positive

- chain integration is localized
- sync and broadcast can evolve independently
- higher layers stay focused on orchestration and wallet logic
- backend-specific rejection handling does not leak upward

### Negative

- another crate boundary has to be maintained
- backend capabilities are not perfectly symmetric
- some internal backend modules exist without being part of the current
  production dispatch path

## Alternatives Considered

### Direct backend usage from `wallet_api`

Rejected because it would couple orchestration to backend clients and transport
details.

### Single backend for every environment

Rejected because the repository already supports different sync and broadcast
backends and uses them in different contexts.

### Force one unified full-feature interface for all backends

Rejected because sync and broadcast capabilities are intentionally split, and not
every backend participates in every path.

## Summary

The repository uses `wallet_sync` as the dedicated integration boundary for
chain sync and transaction broadcast, with dispatch driven by wallet
configuration and backend-specific behavior isolated behind that facade.
