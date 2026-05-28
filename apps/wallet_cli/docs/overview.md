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
- safe descriptor inspection
- coin control (explicit include/exclude inputs)
- locked UTXOs (wallet-scoped spend suppression with optional reason)
- persisted receive-address history
- receive-address labeling
- wallet-scoped address book entries
- receive-address QR SVG export

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

Caller-visible output is intentionally structured rather than prose-only. The CLI surfaces receive-address metadata, persisted receive history rows, optional QR SVG payloads, address labels, locked-UTXO metadata, transaction parent/output inspection data, and richer PSBT preview fields such as selected inputs and replacement lineage.

The address-book commands follow the same pattern: create, list, get, and delete all return or render the persisted wallet-scoped entry shape instead of inventing a separate CLI-only format.

The descriptor-inspection command follows the same principle: it renders a redacted backend-produced descriptor view instead of exposing raw private descriptor material or import/export payloads.

---

## Relationship to Desktop UI

The CLI acts as:

- a reference for command naming
- a reference for transaction flows
- a correctness baseline for UI behavior

The Tauri UI should **not diverge semantically** from CLI operations.
