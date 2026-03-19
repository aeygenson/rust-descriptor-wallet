# Rust Descriptor Wallet

![Rust](https://img.shields.io/badge/Rust-2021-orange)
![BDK](https://img.shields.io/badge/BDK-2.x-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-actively--developing-yellow)

A modular Bitcoin descriptor wallet in Rust, designed around clean crate boundaries, BDK-based wallet logic, and a path toward a desktop wallet with a clear separation between core logic, sync, storage, API, and UI.

This repository is being built as a production-style architecture project: the design is already laid out, the workspace is in place, and the missing wallet functionality is actively being filled in.

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
- CLI executable that builds and runs
- API-to-core integration scaffold

### In Progress

- wallet logic inside `wallet_core`
- sync integration inside `wallet_sync`
- persistence layer in `wallet_storage`
- richer command surface in `wallet_api`
- CLI feature expansion

### Expected Shortly

- real wallet actions exposed through the CLI
- descriptor-driven wallet state
- sync and storage integration
- first end-to-end wallet flow across the workspace layers

## Planned Capabilities

The intended feature set includes:

- descriptor wallets with `wpkh` and later `tr`
- external and internal derivation paths
- blockchain sync through Esplora
- balance and UTXO tracking
- transaction building
- PSBT creation and signing flow
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
cargo run -p wallet_cli
```

Current output:

```text
Welcome to rust-descriptor-wallet
```

The CLI is still at the scaffold stage, so the workspace structure and architecture are ahead of the implemented wallet commands.

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
