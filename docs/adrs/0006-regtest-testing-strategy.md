# ADR 0006: Regtest Testing Strategy

## Status

Accepted

## Context

The repository tests wallet behavior that depends on real chain state and
transaction policy, including:

- confirmation vs mempool transitions
- explicit coin-control selection
- sweep and send-max semantics
- consolidation rules and fee limits
- RBF replacement
- CPFP child construction and publication

Mocks alone are not enough to validate those flows end to end, while public
networks are too slow and non-deterministic.

## Decision

The project uses a regtest-first integration-testing strategy.

The implemented setup combines:

- `infra/regtest`
  - local `bitcoind`
  - local `electrs`
  - helper scripts for start, stop, reset, mine, and fund

- `crates/test_support`
  - Rust wrappers for RPC access, mining, funding, wallet bootstrapping, and
    environment setup

- `crates/wallet_api/tests/regtest_flow.rs`
  - end-to-end flow validation against the local regtest environment

Unit tests still exist in individual crates, but regtest is the primary answer
when chain behavior materially affects correctness.

## Rationale

### Real Bitcoin behavior matters

Flows such as RBF, CPFP, sweep, and consolidation are sensitive to mempool
state, fee handling, and wallet-owned UTXO state. Regtest validates those flows
against real node behavior.

### Determinism matters

The local regtest environment provides deterministic mining, funding, and sync
control without relying on external infrastructure.

### Shell setup should be wrapped by Rust helpers

The infrastructure scripts are useful, but integration tests should consume a
stable Rust support layer instead of embedding shell assumptions everywhere.

### The project already depends on this environment operationally

The main integration suite in `regtest_flow.rs` is written around this local
environment, so the architectural record should describe that reality directly.

## Consequences

### Positive

- high confidence in end-to-end wallet correctness
- deterministic funding and confirmation control
- realistic validation of advanced transaction flows
- shared local environment for CLI and integration testing

### Negative

- integration testing requires local infrastructure
- failures can involve more moving parts than pure unit tests
- developers need the regtest environment available for the full suite

## Alternatives Considered

### Mock-only integration testing

Rejected because it cannot validate real node acceptance, mempool state, or
chain confirmation transitions.

### Public-network testing

Rejected because it is slow, non-deterministic, and unsuitable for reproducible
development workflows.

## Summary

The repository treats local regtest infrastructure as the primary integration
test environment whenever real chain behavior matters, with `infra/regtest`,
`crates/test_support`, and `regtest_flow.rs` forming one coherent testing
strategy.
