# Rust Descriptor Wallet

![Rust](https://img.shields.io/badge/Rust-2021-orange)
![BDK](https://img.shields.io/badge/BDK-2.x-blue)
![License](https://img.shields.io/badge/license-MIT-green)
![Status](https://img.shields.io/badge/status-actively--developing-yellow)

A modular Bitcoin descriptor wallet in Rust, designed around clean crate boundaries, BDK-based wallet logic, and a path toward a desktop wallet with a clear separation between core logic, chain integration, storage, API, and UI.

This repository is being built as a production-style architecture project: the design is already laid out, the workspace is in place, and the missing wallet functionality is actively being filled in.

Current milestone: the project now supports CPFP transaction acceleration on top of the existing send and RBF flows, with regtest-backed integration coverage.

## Vision

The goal is to build a descriptor-first Bitcoin wallet that demonstrates:

- clean Rust workspace architecture
- explicit separation of wallet logic, chain integration, storage, and presentation
- a practical PSBT-oriented transaction flow
- a codebase that can evolve from CLI-first development into a desktop application

## Architecture

![Architecture](docs/architecture.svg)

### Components

- `wallet_core (BDK)`: descriptor handling, wallet state, address derivation, transaction construction, and PSBT flow
- `wallet_sync`: chain integration layer for Esplora, Electrum, and Bitcoin Core RPC backends
- `wallet_storage`: local persistence layer
- `wallet_api`: orchestration boundary shared by apps
- `test_support`: local regtest helpers for integration tests and scripted environment control
- `wallet_cli`: command-line entry point
- `wallet_desktop`: desktop app entry point

## Project Structure

![Project Structure](docs/project-structure.svg)

## Current Progress

### Implemented

- Rust workspace with separate crates and app entry points
- `wallet_cli`, `wallet_desktop`, `wallet_api`, `wallet_core`, `wallet_sync`, `wallet_storage`, and `test_support` crates wired into the workspace
- architecture and project-structure documentation
- SQLite-backed wallet registry in `wallet_storage`
- automatic storage initialization and migration on API startup
- wallet import, listing, lookup, and deletion through `wallet_api`
- CLI commands for wallet metadata management
- runtime wallet loading and creation backed by per-wallet BDK file stores
- receive-address generation for stored wallets
- backend-aware wallet sync through `wallet_sync`
- Electrum sync support for local and compatible deployments
- balance queries over persisted wallet state
- wallet status reporting with balance, UTXO count, and latest observed block height
- transaction history inspection from synced wallet state
- UTXO inspection from synced wallet state
- unsigned PSBT creation through the runtime wallet flow
- PSBT signing for software-signing wallets
- finalized-PSBT extraction and publish through `wallet_sync`
- Bitcoin Core RPC broadcast backend for local/regtest transaction publication
- end-to-end create/sign/publish orchestration in the API layer
- replacement PSBT creation for RBF-enabled transactions
- one-shot fee bump flow from replacement build through publish
- CPFP PSBT creation for unconfirmed parent transactions
- end-to-end CPFP flow through build, sign, publish, and confirmation in integration tests
- transaction inspection now surfaces fee rate and replaceability metadata
- stronger domain types for wallet amounts, fee rates, keychains, and transaction direction
- regtest support scripts under `infra/regtest`
- reusable `test_support` helpers for local node control, mining, funding, and mempool inspection
- local integration tests covering receive, self-send/change, RBF replacement, and CPFP flows

### In Progress

- descriptor validation and richer domain logic inside `wallet_core`
- richer send controls and policy handling
- richer command surface in `wallet_api`
- desktop integration on top of the same runtime API

### Expected Shortly

- broader fee management and transaction controls
- hardware-signing flow on top of the same PSBT pipeline
- first end-to-end wallet flow across the workspace layers

## Planned Capabilities

The intended feature set includes:

- descriptor wallets with `wpkh` and later `tr`
- external and internal derivation paths
- blockchain sync through Esplora and Electrum
- transaction broadcast through Esplora or Bitcoin Core RPC
- persisted wallet metadata and per-wallet database paths
- runtime address derivation and balance tracking
- UTXO tracking
- transaction history inspection
- transaction building
- unsigned PSBT creation
- PSBT signing flow
- finalized transaction broadcast
- one-shot send flow through create + sign + publish
- RBF fee bump flow
- CPFP child-transaction flow
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

Current code now covers the full software-wallet path: create PSBT, sign it, finalize it, and publish the resulting transaction through the shared chain backend, with local regtest using Electrum sync plus Bitcoin Core RPC broadcast.

For replaceable transactions, the code also supports a fee-bump path:

1. inspect an unconfirmed RBF transaction
2. build a replacement PSBT at a higher fee rate
3. sign the replacement transaction
4. publish the replacement through the configured broadcast backend

For stuck transactions with wallet-owned unconfirmed outputs, the code also supports a CPFP path:

1. inspect an unconfirmed parent transaction
2. select an eligible wallet-owned child input from the parent outputs
3. build a CPFP PSBT at the requested fee rate
4. sign and publish the child transaction through the configured backend

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
cargo run -p wallet_cli -- import-wallet --file wallet-mutiny-soft.json
cargo run -p wallet_cli -- import-wallet --file wallet-regtest-local.json
cargo run -p wallet_cli -- list-wallets
cargo run -p wallet_cli -- get-wallet mutiny-soft
cargo run -p wallet_cli -- delete-wallet mutiny-soft
cargo run -p wallet_cli -- address --name regtest-local
cargo run -p wallet_cli -- sync --name regtest-local
cargo run -p wallet_cli -- balance --name regtest-local
cargo run -p wallet_cli -- status --name regtest-local
cargo run -p wallet_cli -- txs --name regtest-local
cargo run -p wallet_cli -- utxos --name regtest-local
cargo run -p wallet_cli -- create-psbt --name regtest-local --to bcrt1... --amount 5000 --fee-rate 2
cargo run -p wallet_cli -- sign-psbt --name regtest-local --psbt '<base64>'
cargo run -p wallet_cli -- publish-psbt --name regtest-local --psbt '<base64>'
cargo run -p wallet_cli -- bump-fee-psbt --name regtest-local --txid <txid> --fee-rate 5
cargo run -p wallet_cli -- bump-fee --name regtest-local --txid <txid> --fee-rate 5
cargo run -p wallet_cli -- cpfp-psbt --name regtest-local --parent-txid <txid> --outpoint <txid:vout> --fee-rate 5
cargo run -p wallet_cli -- send-psbt --name regtest-local --to bcrt1... --amount 5000 --fee-rate 2
```

Current note on `cpfp-psbt`:

- use `txs` and `utxos` to choose a wallet-owned unconfirmed parent output
- pass that outpoint explicitly with `--outpoint`
- then run `cpfp-psbt`, `sign-psbt`, and `publish-psbt` for the full manual flow

What is stored right now:

- wallet name
- network
- external and internal descriptors
- sync backend configuration
- optional broadcast backend configuration
- watch-only flag
- derived per-wallet database path

What works at runtime now:

- load or create a persisted BDK wallet from the stored descriptors
- reveal the next external receive address and persist the derivation state
- sync wallet state through the configured backend via `wallet_sync`
- read total balance from the persisted wallet state
- inspect a high-level wallet status view
- inspect wallet transaction history from the current synced state
- inspect spendable UTXOs from the current synced state
- create an unsigned PSBT with destination, amount, fee, and selected input summary
- sign a PSBT using wallet-owned private descriptor material
- classify signing results as unchanged, partial, or finalized
- validate and extract a finalized PSBT into a raw transaction
- broadcast raw transaction hex through the configured backend via `wallet_sync`
- run an end-to-end send path through create, sign, and publish
- inspect fee rate and replaceability on wallet transactions
- build replacement PSBTs for eligible RBF transactions
- execute a full fee-bump flow through replacement build, sign, and publish
- build CPFP PSBTs for eligible unconfirmed parent transactions
- sign and publish CPFP child transactions through the same PSBT pipeline
- run local regtest-backed integration flows against real node services

Core domain types introduced:

- `AmountSat` for satoshi-denominated values
- `FeeRateSatPerVb` for fee-rate validation
- `WalletKeychain` for external vs internal wallet branches
- `TxDirection` for received, sent, and self-transfer transaction classification
- `PsbtSigningStatus` for stable signing-state classification

Storage location:

- app database: `~/.rust-descriptor-wallet/app.db`
- per-wallet db path pattern: `~/.rust-descriptor-wallet/<wallet-name>.wallet.db`

The CLI now covers wallet metadata management, read-oriented runtime operations, PSBT creation/signing/publish, one-shot send, RBF fee bumping, and CPFP PSBT creation. The workspace now also has a cleaner backend boundary where `wallet_sync` owns chain integration across Esplora, Electrum, and Bitcoin Core RPC. The next major step is broadening policy and signing options rather than just proving the core transaction lifecycle.

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
  "name": "mutiny-soft",
  "network": "signet",
  "descriptors": {
    "external": "tr([fingerprint/86'/1'/0']tpub.../0/*)#checksum",
    "internal": "tr([fingerprint/86'/1'/0']tpub.../1/*)#checksum"
  },
  "backend": {
    "sync": {
      "kind": "esplora",
      "url": "https://mutinynet.com/api"
    },
    "broadcast": {
      "kind": "esplora",
      "url": "https://mutinynet.com/api"
    }
  },
  "is_watch_only": true
}
```

## Local Regtest

The repository now includes a local regtest environment in [infra/regtest/README.md](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/infra/regtest/README.md).

It provides:

- `bitcoind` in regtest mode
- `electrs` for Electrum sync
- helper scripts for start, stop, reset, mine, and fund
- a sample local wallet config in [wallet-regtest-local.json](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/wallet-regtest-local.json)

This local profile uses:

- Electrum for sync
- Bitcoin Core RPC for broadcast

Current integration coverage in [crates/wallet_api/tests/regtest_flow.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/crates/wallet_api/tests/regtest_flow.rs):

- receive funds and observe balance after sync
- self-send with change output tracking
- RBF fee bump with mempool replacement and confirmation checks
- CPFP child transaction build, publish, and confirmation checks

## Development Roadmap

1. implement wallet primitives in `wallet_core`
2. continue expanding `wallet_sync` as the multi-backend chain boundary
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
