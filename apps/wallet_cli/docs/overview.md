# Wallet CLI Overview

## Purpose

The `wallet_cli` is a thin command-line interface over the `wallet_api` layer.

It is designed to:
- expose real wallet operations
- mirror backend capabilities
- provide a deterministic interface for testing and debugging
- act as a reference model for the desktop UI

This CLI is **not a separate implementation** of wallet logic.

All business logic lives in:
- `wallet_core`
- `wallet_api`

---

## Design Principles

- thin wrapper over `wallet_api`
- no duplication of wallet logic
- explicit commands over implicit behavior
- predictable input/output
- consistent naming with backend concepts

---

## Key Concepts

The CLI exposes the following core concepts:

- PSBT (Partially Signed Bitcoin Transaction)
- coin control (explicit include/exclude inputs)

Input selection modes:
- `strict-manual`
- `manual-with-auto-completion`
- `automatic-only`

Send modes:
- fixed amount
- send-max
- sweep
- consolidation

Transaction lifecycle:
- create
- sign
- publish

---

## Relationship to Desktop UI

The CLI acts as:

- a reference for command naming
- a reference for transaction flows
- a correctness baseline for UI behavior

The Tauri UI should **not diverge semantically** from CLI operations.
