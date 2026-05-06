# ADR 0005: Storage Separation

## Status

Accepted

## Context

The system persists two different classes of data:

1. wallet registry metadata
   - wallet name
   - network
   - descriptors
   - sync backend configuration
   - broadcast backend configuration
   - wallet database path
   - watch-only flag

2. wallet content and chain-derived state
   - transactions
   - UTXOs
   - indexes and checkpoints
   - BDK-managed wallet state

Those two classes have different ownership and change for different reasons.

## Decision

The project keeps registry storage and wallet content storage separate.

- `crates/wallet_storage`
  - owns the SQLite wallet registry
  - uses `sqlx`
  - stores backend configs as JSON strings
  - records `db_path` for the BDK database

- `crates/wallet_core` / BDK
  - own the wallet content derived from chain sync and transaction activity
  - use the database located at the configured `db_path`

`wallet_api` bridges the two by loading registry metadata, constructing
`WalletConfig`, and opening the wallet service against the BDK database.

## Rationale

### Avoid duplicate sources of truth

If the project stored transactions and UTXOs in both a custom registry schema
and the BDK database, it would create consistency and migration problems.

### Keep the wallet registry small and stable

The registry only needs to answer:

- which wallets exist
- how they are configured
- where their BDK data lives

That scope is much smaller than full wallet state.

### Let BDK remain responsible for wallet content

Wallet content is already managed by the wallet engine. The project should not
re-implement that persistence layer without a separate explicit decision.

### Make import/export practical

The repository already has JSON import/export structures for wallet definitions.
Those map naturally to registry metadata, not to full chain-derived content.

## Consequences

### Positive

- only one place owns chain-derived wallet state
- registry schema stays simpler
- backend config and descriptor metadata can evolve without rewriting wallet
  history
- tests can recreate wallet definitions without duplicating content storage

### Negative

- debugging sometimes requires inspecting both the registry DB and the BDK DB
- callers need to understand the difference between registry metadata and wallet
  content

## Alternatives Considered

### One database for everything

Rejected because it would duplicate BDK-owned state and create unnecessary
schema complexity.

### File-only wallet registry

Rejected because the repository already benefits from a queryable structured
wallet registry and stable import/export boundaries.

## Summary

The repository stores wallet definitions in `wallet_storage` and keeps
chain-derived wallet content in the BDK database referenced by `db_path`. That
separation preserves a clear source of truth for each class of data.
