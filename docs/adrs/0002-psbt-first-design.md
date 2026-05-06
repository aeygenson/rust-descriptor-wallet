# ADR 0002: PSBT-First Design

## Status

Accepted

## Context

The wallet does not only support simple fixed-amount sends. It also needs to
support:

- fixed-amount send
- send-max
- sweep
- consolidation
- RBF
- CPFP
- explicit coin control
- preview-before-sign and preview-before-broadcast behavior

Those flows need a representation that preserves transaction intent before final
authorization and before network submission.

## Decision

The project uses PSBTs as the canonical transaction-construction boundary.

The standard lifecycle is:

1. create PSBT
2. inspect PSBT-derived preview data
3. sign PSBT
4. finalize to raw transaction
5. publish / broadcast

`wallet_core` produces typed PSBT results. `wallet_api` exposes DTOs around that
lifecycle. CLI one-shot commands are convenience wrappers that still compose the
same PSBT-oriented steps internally.

## Rationale

### Preview is a first-class requirement

The system needs to expose authoritative preview information such as:

- selected inputs
- recipient count
- output count
- amount
- change
- fee
- fee rate
- estimated vsize
- replaceability

That preview should exist before signing and before publishing.

### Advanced flows need an editable intermediate form

RBF, CPFP, sweep, and consolidation all benefit from an intermediate form that
is explicit and inspectable. PSBT fits that role better than immediately
building and broadcasting a finalized transaction.

### Construction, signing, and publishing are different concerns

The project separates:

- input and output selection
- authorization via signing
- network submission

That keeps behavior easier to test and easier to surface in CLI and future UI
flows.

### One-shot helpers should not invent a separate model

Even when the CLI or API offers a single command that signs and broadcasts, the
system should still reuse PSBT creation and signing internally instead of
creating a second transaction lifecycle.

## Consequences

### Positive

- users and tests can inspect what the wallet built before publication
- advanced flows share one transaction-construction model
- signing stays separate from transaction selection logic
- hardware or external signer support stays feasible
- the API has a stable preview-oriented boundary

### Negative

- transaction handling involves more explicit stages
- the API surface needs PSBT serialization and DTOs
- one-shot commands still depend on the staged lifecycle under the hood

## Alternatives Considered

### Immediate finalized transaction construction

Rejected because it hides selected inputs, fee decisions, and intermediate
state, which makes advanced flows harder to reason about and test.

### Separate flow-specific transaction pipelines

Rejected because it would duplicate lifecycle behavior across send, sweep,
consolidation, RBF, and CPFP.

## Summary

The repository treats PSBT creation as the canonical boundary for transaction
construction. Signing and publishing build on top of that boundary rather than
replacing it.
