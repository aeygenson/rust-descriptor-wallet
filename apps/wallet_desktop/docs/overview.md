# Wallet Desktop Overview

`wallet_desktop` is the repository's first desktop user interface built on Tauri, React, and TypeScript.

The current implementation is not a wireframe anymore. It already drives real wallet flows through `wallet_api` and the same regtest-backed runtime used by the CLI.

## Role In The Repository

The desktop app is a presentation layer over the existing Rust wallet stack:

```text
React UI
  -> Tauri invoke
  -> Tauri Rust command modules
  -> wallet_api
  -> wallet_core / wallet_sync / wallet_storage
```

The desktop app does not implement wallet logic itself. It renders wallet state, collects user intent, and forwards requests to Rust.

## Implemented Product Surface

Current routed screens:

- Overview
- Receive
- UTXOs
- Send
- Transactions

Current desktop capabilities:

- load and switch wallets through a shared wallet provider
- show wallet status and backend health
- generate the next receive address for the active wallet
- list persisted receive-address history for the active wallet
- display receive-address keychain, derivation-index, timestamp, and label metadata
- copy the raw receive address and Bitcoin URI from a dedicated receive surface
- inspect UTXOs and carry selected outpoints into send flows
- build PSBT previews for fixed send, send-max, sweep, and consolidation
- sign and publish PSBTs
- inspect transaction history with derived parent/child graph data
- classify transactions by user-visible intent and show intent badges in history/details
- create and broadcast RBF and CPFP flows from the Transactions screen

## Transaction Creation Flows

The current GUI does not only support a basic fixed-amount send path. Four transaction-building modes are implemented and all of them converge into the same PSBT lifecycle:

```mermaid
flowchart TD
    A["Send Screen"] --> B["Fixed Send Form"]
    A --> C["Send Max Form"]
    A --> D["Sweep Form"]
    A --> E["Consolidation Form"]

    B --> F["Coin Control + Request Shaping"]
    C --> F
    D --> F
    E --> F

    F --> G["Tauri send command"]
    G --> H["wallet_api PSBT builder"]
    H --> I["PSBT Preview"]
    I --> J["Sign PSBT"]
    J --> K["Publish PSBT"]
```

That shared downstream pipeline is the important product fact. The forms differ, but preview, signing, and publishing are intentionally unified.

## Receive Flow

Receive is now a dedicated first-class screen rather than an implicit wallet helper.

```mermaid
flowchart LR
    A["Receive Route"] --> B["Generate address action"]
    B --> C["feature/receive api.ts"]
    C --> D["Tauri get_receive_address"]
    D --> E["wallet_api address()"]
    E --> F["Persist receive history row"]
    F --> G["WalletReceiveAddressHistoryDto"]
    G --> H["Receive card"]
    G --> I["Receive history list"]
    H --> J["Copy address"]
    H --> K["Copy bitcoin URI"]
```

The receive page is still intentionally lightweight in the rendered UI: it asks Rust for the next wallet-controlled address, persists that row, then renders the canonical address string plus history-backed metadata returned by the backend. The backend command surface already supports listing and labeling persisted receive rows, but the current page only uses generation plus history browsing.

## Transaction Intent Layer

The Transactions screen now exposes more than raw wallet history. The desktop client resolves a transaction intent for display:

- `fixed`
- `send_max`
- `sweep`
- `consolidation`
- `rbf`
- `cpfp`
- `unknown`

Intent resolution is hybrid:

- the Send screen stores explicit intent after publishing fixed/send-max/sweep/consolidation flows
- the Transactions screen stores explicit intent after broadcasting RBF and CPFP flows
- if no stored intent exists, the frontend falls back to structural inference from wallet-owned outputs and input/output shape

This is a desktop presentation feature. It improves operator understanding without changing Rust-side transaction semantics.

## What Is Actually Running

The frontend is under [src](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src).

The Tauri host is under [src-tauri](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri).

The Rust side initializes `WalletApi` once at app startup in [src-tauri/src/lib.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/lib.rs) and exposes wallet, UTXO, transaction, send, RBF, and CPFP commands.

## Design Direction That Survived Contact With Code

Three earlier design ideas are now implemented and still correct:

- Rust remains the source of truth for transaction construction.
- The send experience is PSBT-preview-first.
- Coin control in the UI expresses user intent, while final selection stays backend-owned.

## Gaps And Near-Term Limits

The app is still a first version.

Notable current limits:

- no settings or wallet import management UI
- no query library; data loading is handled with local `useEffect`/`useState` patterns
- no generalized component library yet

## Summary

`wallet_desktop` has moved from design draft to a working first GUI over the real wallet runtime. The documentation for this app should now describe the implemented command surface and screens, not an aspirational future shell.
