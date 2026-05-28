# Screen Map

This document describes the screens that are implemented today.

## Navigation

Current sidebar routes:

- Overview
- Receive
- Address Book
- UTXOs
- Send
- Transactions

There is no Settings or separate Maintenance route in the current app.

## Overview

File:

- [src/pages/OverviewPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/OverviewPage.tsx)

Current responsibilities:

- verify the Tauri backend connection
- load wallet status for the selected wallet
- load backend health for the selected wallet
- load descriptor inspection metadata for the selected wallet
- show balance, UTXO count, and last synced block height
- show a safe redacted descriptor card with branch metadata

This screen is operational status, not a marketing dashboard.

## UTXOs

File:

- [src/pages/UtxosPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/UtxosPage.tsx)

Current responsibilities:

- load wallet UTXOs
- maintain local selection state
- summarize confirmed, pending, locked, and spendable UTXO value
- let the user filter rows by confirmation and lock state
- show persisted lock reason and lock timestamp per row
- forward selected outpoints into Send flows

Current actions:

- lock selected spendable UTXOs
- unlock selected locked UTXOs
- send fixed amount with selected UTXOs
- send max with selected UTXOs
- sweep selected UTXOs
- consolidate selected UTXOs

## Receive

File:

- [src/pages/ReceivePage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/ReceivePage.tsx)

Current responsibilities:

- show the currently selected wallet in a receive-specific header
- request the next receive address from the Tauri backend
- render address metadata returned by Rust
- render the QR image for the active receive address
- load persisted receive-address history for the selected wallet
- let the user select a historical receive address as the active card
- let the user save or clear the label for the active receive-history row
- let the user generate a fresh address on demand
- let the user copy the raw address or Bitcoin URI

Current UI states:

- no wallet selected
- ready to generate
- address generated
- receive history loaded
- backend/request error

Current boundary note:

- the Tauri backend exposes generate, list, label, clear-label, and QR-backed receive rows
- the current rendered page uses generation, history browsing, QR rendering, and visible label editing

## Send

File:

- [src/pages/SendPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/SendPage.tsx)

Current modes:

- fixed
- send max
- sweep
- consolidate

Current responsibilities:

- load available UTXOs for coin control
- accept router-provided preselected outpoints
- build PSBT previews
- sign PSBTs
- publish PSBTs
- refresh wallet/UTXO state after broadcast

Current UI features:

- manual vs automatic coin selection handling
- strict-manual behavior for sweep and consolidation
- PSBT preview panel with selected input and fee information

## Address Book

File:

- [src/pages/AddressBookPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/AddressBookPage.tsx)

Current responsibilities:

- load persisted wallet-scoped address-book entries
- create a new external recipient entry with label, address, and optional notes
- delete an existing entry
- render network, label, address, notes, and timestamps

Current UI states:

- no wallet selected
- loading entries
- create form ready
- non-empty address-book list
- request/validation error

Current boundary note:

- this page is for external destinations, not wallet-owned receive derivations
- it uses the dedicated address-book Tauri commands, not the receive-history commands

## Transactions

File:

- [src/pages/TransactionsPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/TransactionsPage.tsx)

Current responsibilities:

- load transaction history
- derive transaction graph relationships
- resolve transaction intent labels from stored metadata or fallback heuristics
- filter transactions
- open transaction details
- trigger RBF PSBT flow
- trigger CPFP PSBT flow

Current action surface:

- copy txid
- inspect details
- inspect transaction intent badge in table and details modal
- create/sign/publish RBF replacement PSBT
- create/sign/publish CPFP PSBT

## Workflow Relationships

Implemented relationships:

- Overview -> operational state and health
- UTXOs -> Send with preselected outpoints
- Address Book -> external recipient management
- Send fixed/send max/sweep/consolidate -> PSBT preview -> sign -> publish
- Transactions -> RBF/CPFP workflows

The Send screen should be understood as four transaction-entry flows that share one downstream workflow, not as one fixed-send screen with a few optional extras.

The Receive screen should be understood as a dedicated address-generation workflow, not as a hidden sub-action inside Overview or UTXOs.

The Address Book screen should be understood as an external-recipient registry, not as an alias for receive history.

## Missing Screens

Still not implemented:

- wallet management page
- settings/configuration page
- standalone maintenance page

Those should stay documented as future work, not current surface.
