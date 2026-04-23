# Transaction Flows

This document describes the main transaction flows exposed by `wallet_cli` and backed by `wallet_api` and `wallet_core`.

The goal is to explain how wallet operations are modeled and how the CLI maps onto the underlying transaction lifecycle.

---

## Design Principles

The transaction flows in this project follow a few core principles:

- wallet logic lives in Rust backend layers, not in the CLI
- the CLI exposes explicit operations rather than hiding behavior
- PSBT creation and preview are first-class steps
- coin control is treated as a real wallet feature, not a special-case hack
- advanced flows like sweep, consolidation, RBF, and CPFP are modeled explicitly

---

## Shared Transaction Lifecycle

Most flows follow the same high-level lifecycle:

1. collect user intent
2. build a backend request
3. create a PSBT or transaction preview
4. inspect backend-selected inputs and derived values
5. sign the PSBT
6. publish the transaction

In one-shot commands, some of these steps are combined.

Even in those cases, the backend still conceptually performs the same sequence.

---

## Standard Fixed-Amount Send

This is the default payment flow.

User provides:
- destination address
- fixed amount
- fee rate
- optional coin-control settings

Backend behavior:
- selects inputs according to the request and selection mode
- computes fee, change, and estimated vsize
- produces PSBT preview data

Typical lifecycle:
1. create PSBT
2. inspect preview
3. sign
4. publish

Important notes:
- final selected inputs come from the backend
- change output may be created
- replaceability depends on request and transaction settings

---

## Send-Max Flow

Send-max is a wallet send flow where the recipient receives the maximum spendable value after fees.

User provides:
- destination address
- fee rate
- optional coin-control settings

Backend behavior:
- determines maximum spendable amount from eligible wallet inputs
- subtracts fee from spendable value
- builds a PSBT with the final send amount computed internally

Typical lifecycle:
1. create send-max PSBT
2. inspect preview
3. sign
4. publish

Important notes:
- the amount is not user-fixed
- selected inputs still depend on input-selection mode
- change output may or may not exist depending on the chosen inputs and fee outcome

---

## Sweep Flow

Sweep is modeled intentionally and explicitly.

In backend terms, sweep is effectively:
- `WalletSendAmountMode::Max`
- plus an explicit include set

The purpose of sweep is to drain explicitly selected UTXOs to a destination.

User provides:
- destination address
- fee rate
- explicit included outpoints

Backend behavior:
- uses the explicitly provided inputs
- computes the maximum transferable amount from those inputs after fees
- preserves sweep semantics instead of treating it as a generic send-max

Typical lifecycle:
1. select one or more UTXOs
2. create sweep PSBT
3. inspect preview
4. sign
5. publish

Important notes:
- sweep should be treated as explicit-input behavior
- in strict sweep-like behavior, additional wallet inputs should not be silently added
- sweep-like flows are expected to avoid wallet-internal change when fully draining selected inputs
- preview output should be used to verify no-change expectations

---

## Consolidation Flow

Consolidation is a wallet-maintenance flow, not an external payment flow.

Its purpose is to reduce wallet fragmentation by combining multiple UTXOs into a smaller number of wallet-internal outputs.

User provides:
- fee rate
- optional input filters or selected UTXOs
- optional input-selection mode

Backend behavior:
- selects candidate inputs according to request
- creates a wallet-internal transaction
- returns PSBT preview data describing the resulting transaction

Typical lifecycle:
1. choose candidate UTXOs or use automatic selection
2. create consolidation PSBT
3. inspect preview
4. sign
5. publish

Important notes:
- the output remains inside the same wallet
- this should not be presented as a normal recipient payment
- consolidation can be used to simplify future spending and reduce UTXO set complexity inside the wallet

---

## Coin Control Flow

Coin control is not a separate transaction type, but a selection policy applied to multiple flows.

It can affect:
- fixed sends
- send-max
- sweep
- consolidation

Coin control request components may include:
- explicit include set
- explicit exclude set
- confirmed-only behavior
- input-selection mode

The backend resolves the request and produces the final selected input set.

This distinction is important:
- user-requested inputs are not always the same as final backend-selected inputs
- the preview step is authoritative

---

## Input-Selection Modes

The project exposes three input-selection modes.

### 1. `strict-manual`

Meaning:
- only explicitly selected inputs may be used
- transaction creation fails if they are insufficient

Best for:
- deterministic coin control
- sweep-like behavior
- testing exact input selection

---

### 2. `manual-with-auto-completion`

Meaning:
- selected inputs are pinned
- backend may add more inputs if needed

Best for:
- guided coin control
- partial manual selection with wallet assistance

---

### 3. `automatic-only`

Meaning:
- backend selects inputs automatically
- no manual inclusion is required

Best for:
- default wallet send flows
- simple UX paths

---

## RBF Flow (Replace-By-Fee)

RBF is used to replace an unconfirmed replaceable transaction with a higher-fee version.

User provides:
- original transaction id
- new fee rate

Backend behavior:
- locates the original transaction
- verifies it is eligible for replacement
- creates a replacement PSBT or full replacement flow

Typical lifecycle:
1. choose unconfirmed replaceable transaction
2. create bump-fee PSBT
3. inspect replacement preview
4. sign
5. publish

Important notes:
- the replacement transaction should clearly reference the original txid in preview/output
- replaceability must be surfaced clearly to the user
- this is a first-class maintenance action, not a normal send

---

## CPFP Flow (Child Pays For Parent)

CPFP is used to accelerate an unconfirmed transaction by spending one of its outputs in a child transaction with a higher fee package.

User provides:
- parent transaction id
- spendable parent output outpoint
- fee rate

Backend behavior:
- validates the spendable output from the parent transaction
- constructs a child transaction
- returns preview information for the child transaction

Typical lifecycle:
1. choose eligible parent/output
2. create CPFP PSBT
3. inspect preview
4. sign
5. publish

Important notes:
- CPFP is different from RBF
- RBF replaces the original transaction
- CPFP adds a child transaction that improves package confirmation incentives

---

## Preview as Source of Truth

Across all flows, the preview result is the most important user-facing checkpoint.

Preview data may include:
- txid
- original txid
- destination
- amount
- fee
- fee rate
- replaceable flag
- selected inputs
- selected input count
- estimated vsize
- change amount
- output count
- recipient count

The preview should be treated as authoritative because:
- it reflects actual backend resolution
- it shows final input selection
- it reveals whether automatic completion occurred
- it makes sweep and consolidation behavior visible before signing

---

## Regtest-Backed Confidence

These flows are not just theoretical CLI paths.

The project backend is designed around regtest-backed correctness for flows such as:
- receiving funds after sync
- self-send with change
- strict manual input selection
- manual-with-auto-completion selection
- automatic-only selection
- sweep-like max flows
- consolidation
- RBF replacement
- CPFP construction and broadcast

This means the CLI transaction flows should be interpreted as real user-facing paths over tested backend behavior.

---

## Common Failure Cases

Some flows may fail before signing or publication.

Typical reasons include:
- insufficient selected inputs in `strict-manual`
- excluded inputs conflicting with required selection
- invalid destination address
- fee rate too low or otherwise rejected
- attempting RBF on a non-replaceable transaction
- attempting CPFP without a suitable spendable parent output
- confirmed-only filtering removing all usable inputs

The CLI should report these backend failures clearly rather than trying to recover with hidden heuristics.

---

## Summary

The project treats transaction construction as an explicit, inspectable process.

Core ideas:
- standard send, send-max, sweep, and consolidation are distinct flows
- coin control is first-class
- preview-before-signing is central
- backend-selected inputs are authoritative
- RBF and CPFP are explicit maintenance tools

This design keeps wallet behavior understandable, testable, and consistent across CLI and future desktop UI.
