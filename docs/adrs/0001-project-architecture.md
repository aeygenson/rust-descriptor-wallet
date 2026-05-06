# ADR 0001: Project Architecture

## Status

Accepted

## Context

The repository implements a Bitcoin wallet stack with:

- descriptor-backed wallet loading
- PSBT-based transaction creation
- advanced maintenance flows such as sweep, consolidation, RBF, and CPFP
- multiple chain backends for sync and broadcast
- deterministic regtest-backed integration testing

The codebase needs clear boundaries between:

- domain wallet logic
- API and DTO normalization
- chain sync and broadcast integration
- wallet registry storage
- regtest infrastructure and integration helpers

Without those boundaries, Bitcoin-specific logic, backend selection, and
storage concerns would leak into each other and become harder to test.

## Decision

The repository uses a multi-crate architecture with explicit responsibilities.

- `crates/wallet_core`
  - owns the wallet domain model and `WalletService`
  - wraps BDK wallet behavior
  - implements PSBT creation, signing, publish preparation, RBF, CPFP, sweep,
    consolidation, lifecycle, transaction, and UTXO queries

- `crates/wallet_api`
  - exposes the application-facing orchestration API
  - converts DTOs into wallet-core types
  - coordinates wallet loading, sync, PSBT creation, signing, and publishing
  - maps core and sync errors into a stable application layer

- `crates/wallet_sync`
  - owns sync and broadcast integration
  - dispatches sync through Electrum or Esplora
  - dispatches broadcast through Esplora or Bitcoin Core RPC
  - keeps backend-specific transport and rejection parsing out of `wallet_core`

- `crates/wallet_storage`
  - owns the SQLite wallet registry
  - stores wallet definitions, descriptors, backend config, and the path to the
    BDK database

- `crates/test_support`
  - provides Rust helpers around the local regtest environment
  - wraps RPC access, mining, funding, wallet setup, and test environment boot

- `apps/wallet_cli`
  - is the current user-facing application
  - exposes API flows through a command-line interface

- `infra/regtest`
  - contains the local `bitcoind` plus `electrs` environment and helper scripts

## Rationale

### Keep wallet logic isolated from transport and persistence

`wallet_core` can reason about descriptors, PSBTs, inputs, fee rates, and BDK
wallet state without knowing how sync happens or how wallet definitions are
stored.

### Keep the application boundary explicit

`wallet_api` is the place where strings and DTOs are normalized into typed
wallet-core values. This prevents CLI- or UI-driven parsing concerns from
spreading into the domain crate.

### Keep backend integration replaceable

`wallet_sync` isolates Electrum, Esplora, and Core RPC behavior so that chain
integration changes do not force changes in wallet construction logic.

### Keep registry storage separate from BDK wallet state

The project stores wallet definitions in `wallet_storage`, while chain-derived
wallet content remains in the BDK-managed database pointed to by `db_path`.

### Support realistic testing

`infra/regtest` and `crates/test_support` let the repository test real Bitcoin
flows without coupling tests directly to shell scripts or external networks.

## Consequences

### Positive

- responsibilities are easier to reason about
- wallet logic is testable without directly embedding backend clients
- backend changes stay localized
- wallet registry storage stays small and focused
- CLI and future frontends can reuse the same API boundary

### Negative

- the system requires explicit wiring across crate boundaries
- some data types exist in both core and DTO form
- developers need discipline to avoid leaking concerns between layers

## Alternatives Considered

### Monolithic crate

Rejected because it would mix Bitcoin domain logic, sync integration, storage,
and CLI concerns in one place.

### Core plus everything-else split

Rejected because it would still leave sync orchestration, DTO normalization, and
storage coupled together.

## Summary

The repository uses a multi-crate architecture so wallet logic, API
normalization, chain integration, registry storage, and regtest support remain
separate and testable.
