# Screen Map

This document describes the screens that are implemented today.

## Navigation

Current sidebar routes:

- Overview
- UTXOs
- Send
- Transactions

There is no Receive, Settings, or separate Maintenance route in the current app.

## Overview

File:

- [src/pages/OverviewPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/OverviewPage.tsx)

Current responsibilities:

- verify the Tauri backend connection
- load wallet status for the selected wallet
- load backend health for the selected wallet
- show balance, UTXO count, and last synced block height

This screen is operational status, not a marketing dashboard.

## UTXOs

File:

- [src/pages/UtxosPage.tsx](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/pages/UtxosPage.tsx)

Current responsibilities:

- load wallet UTXOs
- maintain local selection state
- summarize confirmed and pending UTXO value
- forward selected outpoints into Send flows

Current actions:

- send fixed amount with selected UTXOs
- send max with selected UTXOs
- sweep selected UTXOs
- consolidate selected UTXOs

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
- Send fixed/send max/sweep/consolidate -> PSBT preview -> sign -> publish
- Transactions -> RBF/CPFP workflows

The Send screen should be understood as four transaction-entry flows that share one downstream workflow, not as one fixed-send screen with a few optional extras.

## Missing Screens

Still not implemented:

- dedicated receive page
- wallet management page
- settings/configuration page
- standalone maintenance page

Those should stay documented as future work, not current surface.
