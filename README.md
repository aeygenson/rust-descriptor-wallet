# Rust Descriptor Wallet

![Rust](https://img.shields.io/badge/Rust-2021-orange)
![BDK](https://img.shields.io/badge/BDK-2.x-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-actively--developing-yellow)

A modular Bitcoin descriptor wallet in Rust, designed around clean crate boundaries, BDK-based wallet logic, and a path toward a desktop wallet with a clear separation between core logic, sync, storage, API, and UI.

This repository is being built as a production-style architecture project: the design is already laid out, the workspace is in place, and the missing wallet functionality is actively being filled in.

Current milestone: persisted wallets now support runtime inspection plus unsigned PSBT creation, backed by stronger wallet-domain types in `wallet_core`.

## Vision

The goal is to build a descriptor-first Bitcoin wallet that demonstrates:

- clean Rust workspace architecture
- explicit separation of wallet logic, sync, storage, and presentation
- a practical PSBT-oriented transaction flow
- a codebase that can evolve from CLI-first development into a desktop application

## Architecture

![Architecture](docs/architecture.svg)

### Components

- `wallet_core (BDK)`: descriptor handling, wallet state, address derivation, transaction construction, and PSBT flow
- `wallet_sync`: blockchain synchronization layer
- `wallet_storage`: local persistence layer
- `wallet_api`: orchestration boundary shared by apps
- `wallet_cli`: command-line entry point
- `wallet_desktop`: desktop app entry point

## Project Structure

![Project Structure](docs/project-structure.svg)

## Current Progress

### Implemented

- Rust workspace with separate crates and app entry points
- `wallet_cli`, `wallet_desktop`, `wallet_api`, `wallet_core`, `wallet_sync`, and `wallet_storage` crates wired into the workspace
- architecture and project-structure documentation
- SQLite-backed wallet registry in `wallet_storage`
- automatic storage initialization and migration on API startup
- wallet import, listing, lookup, and deletion through `wallet_api`
- CLI commands for wallet metadata management
- runtime wallet loading and creation backed by per-wallet BDK file stores
- receive-address generation for stored wallets
- Esplora-based wallet sync
- balance queries over persisted wallet state
- wallet status reporting with balance, UTXO count, and latest observed block height
- transaction history inspection from synced wallet state
- UTXO inspection from synced wallet state
- unsigned PSBT creation through the runtime wallet flow
- stronger domain types for wallet amounts, fee rates, keychains, and transaction direction

### In Progress

- descriptor validation and richer domain logic inside `wallet_core`
- PSBT signing and finalization flow
- richer command surface in `wallet_api`
- desktop integration on top of the same runtime API

### Expected Shortly

- signed send flow on top of the created PSBT
- transaction signing and broadcast flow
- first end-to-end wallet flow across the workspace layers

## Planned Capabilities

The intended feature set includes:

- descriptor wallets with `wpkh` and later `tr`
- external and internal derivation paths
- blockchain sync through Esplora
- persisted wallet metadata and per-wallet database paths
- runtime address derivation and balance tracking
- UTXO tracking
- transaction history inspection
- transaction building
- unsigned PSBT creation
- PSBT signing flow
- watch-only support
- hardware signer support
- desktop UI built on the same API boundary

## PSBT Flow

![PSBT Flow](docs/psbt-flow.svg)

The intended transaction flow is:

1. wallet state and descriptors define spendable coins and change policy
2. the builder selects inputs and constructs outputs
3. a PSBT is created as the signing handoff format
4. a signer adds signatures without owning the full wallet application layer
5. the finalized transaction is broadcast to the network

## Getting Started

### Prerequisites

- Rust toolchain
- Cargo

### Build

```bash
cargo build
```

### Run the Current CLI

```bash
cargo run -p wallet_cli -- --help
```

Current output:

```text
Rust Descriptor Wallet CLI
Usage: wallet_cli <COMMAND>
```

Current wallet-management commands:

```bash
cargo run -p wallet_cli -- import-wallet --file wallet.json
cargo run -p wallet_cli -- list-wallets
cargo run -p wallet_cli -- get-wallet signet-dev
cargo run -p wallet_cli -- delete-wallet signet-dev
cargo run -p wallet_cli -- address --name signet-dev
cargo run -p wallet_cli -- sync --name signet-dev
cargo run -p wallet_cli -- balance --name signet-dev
cargo run -p wallet_cli -- status --name signet-dev
cargo run -p wallet_cli -- txs --name signet-dev
cargo run -p wallet_cli -- utxos --name signet-dev
cargo run -p wallet_cli -- create-psbt --name signet-dev --to tb1... --amount 5000 --fee-rate 2
```

What is stored right now:

- wallet name
- network
- external descriptor
- internal descriptor
- Esplora URL
- watch-only flag
- derived per-wallet database path

What works at runtime now:

- load or create a persisted BDK wallet from the stored descriptors
- reveal the next external receive address and persist the derivation state
- sync wallet state through the configured Esplora endpoint
- read total balance from the persisted wallet state
- inspect a high-level wallet status view
- inspect wallet transaction history from the current synced state
- inspect spendable UTXOs from the current synced state
- create an unsigned PSBT with destination, amount, fee, and selected input summary

Core domain types introduced:

- `AmountSat` for satoshi-denominated values
- `FeeRateSatPerVb` for fee-rate validation
- `WalletKeychain` for external vs internal wallet branches
- `TxDirection` for received, sent, and self-transfer transaction classification

Storage location:

- app database: `~/.rust-descriptor-wallet/app.db`
- per-wallet db path pattern: `~/.rust-descriptor-wallet/<wallet-name>.wallet.db`

The CLI now covers wallet metadata management, read-oriented runtime operations, and unsigned PSBT creation. Signing and broadcast are the next major step.

## Why Descriptor Wallets

Descriptor-based wallets make wallet behavior explicit and easier to reason about:

- script structure is declared directly
- derivation paths are clearer
- watch-only and signing roles can be separated more cleanly
- wallet policy becomes easier to evolve over time

## Example Descriptor Shape

External:

```text
wpkh([fingerprint/84'/1'/0']tpub.../0/*)
```

Internal:

```text
wpkh([fingerprint/84'/1'/0']tpub.../1/*)
```

## Example Import File

```json
{
  "name": "signet-dev",
  "network": "signet",
  "esplora_url": "https://blockstream.info/signet/api/",
  "external_descriptor": "tr([fingerprint/86'/1'/0']tpub.../0/*)#checksum",
  "internal_descriptor": "tr([fingerprint/86'/1'/0']tpub.../1/*)#checksum",
  "is_watch_only": true
}
```

## Development Roadmap

1. implement wallet primitives in `wallet_core`
2. integrate sync in `wallet_sync`
3. add persistence in `wallet_storage`
4. expose real operations through `wallet_api`
5. expand `wallet_cli` into a usable development interface
6. build out `wallet_desktop`

## Development Notes

- workspace edition: Rust 2021
- resolver: Cargo resolver v2
- BDK dependencies are already declared at the workspace level
- the repository is currently in active build-out rather than feature-complete state

## Author

Alex Eygenson
