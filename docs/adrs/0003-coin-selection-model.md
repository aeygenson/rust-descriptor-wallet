# ADR 0003: Coin Selection Model

## Status

Accepted

## Context

The wallet needs to fund different transaction families with different input
constraints:

- fixed-amount send
- send-max
- sweep
- consolidation
- RBF and CPFP preparation

The system also needs:

- explicit include and exclude sets
- confirmed-only filtering
- deterministic regtest behavior
- optional consolidation-specific limits such as min/max input count and min/max
  UTXO value

Fully automatic funding is not sufficient because some flows are driven by
explicit requested outpoints, and strict failures are part of the expected
behavior.

## Decision

The project uses a typed input-selection model with three public modes:

1. `StrictManual`
   - only explicitly included inputs may be used
   - automatic top-up is not allowed
   - insufficient selections fail

2. `ManualWithAutoCompletion`
   - explicitly included inputs are pinned
   - additional eligible inputs may be added automatically

3. `AutomaticOnly`
   - automatic candidate selection is used
   - no exact-manual guarantee is implied

The shared selection config supports:

- `include_outpoints`
- `exclude_outpoints`
- `confirmed_only`
- `min_input_count`
- `max_input_count`
- `min_utxo_value_sat`
- `max_utxo_value_sat`
- consolidation ordering strategy

When callers pass explicit include outpoints and do not specify a mode, the
effective default resolves to `StrictManual`. Without explicit includes, the
effective default resolves to `AutomaticOnly`.

## Rationale

### Explicit inputs must remain explicit

Sweep-like flows and deterministic tests often require that the wallet either
uses the requested outpoints or fails. Silent auto-completion would make those
flows ambiguous.

### Common semantics should be shared across flows

The same selection model is reused by send, send-max, sweep, and consolidation
instead of inventing a different rule set per feature.

### Filters belong with selection, not as ad hoc special cases

Confirmed-only checks, include/exclude conflicts, and min/max constraints are
all part of the same funding decision and should be validated together.

### Integration tests depend on deterministic failure semantics

The regtest suite verifies exact input selection, missing outpoint failures,
insufficient strict-manual failures, and consolidation filters. The model must
support those cases directly.

## Consequences

### Positive

- advanced users and tests can request exact input behavior
- ordinary automatic funding remains available
- failures are explicit instead of silently corrected
- consolidation can reuse the same typed selection config with extra filters

### Negative

- the API surface is more complex than a simple automatic-funding wallet
- callers need to understand the difference between exact and hybrid modes
- selection validation has more edge cases to enforce

## Alternatives Considered

### Automatic-only funding

Rejected because it cannot express exact sweep or deterministic explicit-input
behavior.

### Manual-only funding

Rejected because it would make ordinary sends unnecessarily cumbersome.

### Flow-specific one-off input rules

Rejected because it would fragment semantics across features and make failures
less predictable.

## Summary

The repository uses a shared typed coin-selection model that supports strict
manual funding, manual-plus-automatic completion, and automatic-only selection,
with explicit filters and deterministic failure semantics.
