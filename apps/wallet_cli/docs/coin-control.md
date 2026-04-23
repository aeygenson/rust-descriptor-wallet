# Coin Control

This document explains how coin control works in `rust-descriptor-wallet` and how it is exposed through `wallet_cli`.

Coin control is treated as a first-class wallet capability. It is not an afterthought or a frontend-only convenience feature.

In this project, coin control is part of the transaction-construction model and is backed by `wallet_core`, surfaced through `wallet_api`, and exposed by `wallet_cli`.

---

## What Coin Control Means

Coin control is the ability to influence which wallet UTXOs are used or avoided when building a transaction.

Instead of always letting the wallet select inputs automatically, the user may:
- explicitly include certain outpoints
- explicitly exclude certain outpoints
- require confirmed-only selection
- choose how strictly the backend must follow manual input choices

This matters for real wallet use cases such as:
- deterministic input selection
- privacy-sensitive spending decisions
- draining selected UTXOs
- preparing sweep transactions
- testing exact transaction-building behavior
- UTXO maintenance and consolidation workflows

---

## Why Coin Control Is Important in This Project

This wallet is not designed as a toy send-only interface.

It explicitly supports realistic transaction behavior such as:
- standard sends
- send-max
- sweep
- consolidation
- RBF
- CPFP

Coin control is especially important because these flows depend on clear and inspectable input-selection behavior.

Examples:
- sweep should be based on explicit selected inputs
- consolidation may need precise candidate control
- strict manual tests should fail when selected inputs are insufficient
- preview results should reveal when the backend added extra inputs

---

## Core Concepts

### UTXO / Outpoint

A spendable wallet input is represented by an outpoint:

- `txid:vout`

This identifies a specific transaction output that can be spent.

Example:

```text
7f3d...9b2a:0
```

---

### Include Set

The include set contains outpoints the user explicitly wants to spend.

CLI example:

```bash
--include <txid:vout>
```

Depending on selection mode, included inputs may be:
- the only allowed inputs
- mandatory starting inputs that the wallet may extend

---

### Exclude Set

The exclude set contains outpoints the user explicitly does not want to spend.

CLI example:

```bash
--exclude <txid:vout>
```

Excluded inputs should never be selected by the backend for that transaction.

---

### Confirmed-Only

Confirmed-only selection restricts eligible inputs to confirmed UTXOs.

CLI example:

```bash
--confirmed-only
```

This is useful when the user wants to avoid:
- spending unconfirmed change
- chaining new transactions on top of unconfirmed ones
- accidental use of immature wallet state during testing

---

## Input-Selection Modes

The most important coin-control behavior in this project comes from the input-selection mode.

These modes define how much freedom the backend has when resolving final inputs.

---

### 1. `strict-manual`

Meaning:
- only explicitly included inputs may be used
- if those inputs are insufficient, transaction creation fails

This is the most deterministic mode.

Use it when you want:
- exact input selection
- sweep-like behavior
- explicit UTXO draining
- strict tests
- no backend auto-completion

Example mental model:

> Use only these selected UTXOs. If they are not enough, fail.

---

### 2. `manual-with-auto-completion`

Meaning:
- explicitly included inputs are pinned
- backend may add more inputs if needed

This mode preserves manual intent while still allowing the wallet to complete the transaction.

Use it when you want:
- partial manual guidance
- selected inputs to be mandatory
- wallet assistance if selected inputs alone are insufficient

Example mental model:

> Start with these selected UTXOs, but add more if required.

---

### 3. `automatic-only`

Meaning:
- backend selects inputs automatically
- no explicit manual input set is required

This is the simplest mode for normal wallet usage.

Use it when you want:
- convenience
- default sends
- minimal user control over exact inputs

Example mental model:

> Wallet, choose the inputs yourself.

---

## Requested Inputs vs Final Inputs

This distinction is critical.

Coin control does **not** mean that the user-facing request is automatically the final transaction input set.

There are two separate ideas:

### 1. Requested input policy
What the user asked for:
- included outpoints
- excluded outpoints
- confirmed-only
- selection mode

### 2. Final backend-selected inputs
What the backend actually used after resolving the request.

These may differ when:
- selection mode is `manual-with-auto-completion`
- selection mode is `automatic-only`
- some requested inputs are ineligible
- fee or amount requirements require broader selection

This is why preview data is so important.

---

## Preview Is the Source of Truth

The wallet should treat preview output as authoritative.

Useful preview fields include:
- selected inputs
- selected input count
- fee
- fee rate
- estimated vsize
- change amount
- replaceable flag

Preview is the checkpoint where the user can verify:
- whether the requested inputs were honored
- whether extra inputs were added
- whether a sweep behaves like a true drain
- whether consolidation stays internal

This applies to both CLI output and future GUI UX.

---

## How Coin Control Applies to Different Flows

Coin control is shared across multiple transaction types.

---

### Standard Send

Coin control may be used to:
- force a specific input
- avoid a specific input
- spend confirmed inputs only
- test exact input selection

---

### Send-Max

Coin control may be used to:
- limit which UTXOs contribute to the max-spend amount
- prevent undesired UTXOs from being used
- control whether the wallet may supplement manual selection

---

### Sweep

Coin control is central to sweep semantics.

Sweep behavior is effectively:
- max-style send amount
- explicit included inputs

This means sweep should usually behave like:
- selected inputs only
- no silent input expansion
- drain selected value after fees

In practice, `strict-manual` is the most natural sweep mode.

---

### Consolidation

Coin control helps define which UTXOs should be merged.

This is useful when the user wants to:
- consolidate only small UTXOs
- consolidate only selected inputs
- avoid touching specific coins
- control whether the backend may broaden candidate selection

---

## Typical CLI Patterns

### Exact manual selection

```bash
cargo run -p wallet_cli -- create-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --selection-mode strict-manual
```

Expected behavior:
- use only included input(s)
- fail if insufficient

---

### Manual selection with wallet completion

```bash
cargo run -p wallet_cli -- create-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --selection-mode manual-with-auto-completion
```

Expected behavior:
- included input must be used
- wallet may add more inputs

---

### Automatic selection with exclusions

```bash
cargo run -p wallet_cli -- create-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb> \
  --exclude <txid:vout> \
  --selection-mode automatic-only
```

Expected behavior:
- wallet chooses inputs
- excluded input is not allowed

---

### Sweep selected inputs

```bash
cargo run -p wallet_cli -- sweep-psbt \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --selection-mode strict-manual
```

Expected behavior:
- explicitly selected inputs drive the transaction
- preview should confirm sweep-like no-change expectations when applicable

---

## Common Failure Cases

Coin-control requests may fail for valid reasons.

Typical examples:
- selected inputs are insufficient in `strict-manual`
- included and excluded sets conflict
- included inputs are unconfirmed while `--confirmed-only` is active
- excluded inputs remove all viable candidates
- requested inputs cannot satisfy fee and amount constraints
- invalid outpoint format is supplied

These failures are expected and useful.

They show that the backend is honoring explicit policy rather than silently changing transaction behavior.

---

## UX Guidance for the Future Desktop App

The future Tauri GUI should expose coin control clearly.

Recommended user-facing language:
- `strict-manual` means **Selected only**
- `manual-with-auto-completion` means **Selected + wallet completion**
- `automatic-only` means **Wallet chooses automatically**

Important UX rule:
- never hide the final selected inputs
- always show whether backend auto-completion occurred
- keep backend vocabulary visible in advanced/details views

This will make the desktop app both educational and correct.

---

## Testing Value

Coin control is also important for backend validation.

It supports regtest-backed scenarios such as:
- strict manual selection
- insufficient selected-input failure
- selected include/exclude conflicts
- manual-with-auto-completion behavior
- automatic-only selection
- sweep-like no-change behavior
- consolidation candidate control

This makes coin control a correctness feature, not just a UX feature.

---

## Summary

In this project, coin control means explicit and inspectable control over transaction input selection.

Key ideas:
- include/exclude sets shape candidate inputs
- confirmed-only narrows eligibility
- selection mode determines backend freedom
- requested inputs and final inputs are not always the same
- preview output is authoritative
- sweep and consolidation depend heavily on clear coin-control semantics

This design keeps transaction construction understandable, testable, and consistent across CLI, backend, and future desktop UI.
