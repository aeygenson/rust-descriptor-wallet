# Test Support Overview

The `test_support` crate contains reusable helpers for local, regtest-backed integration tests.

It does not implement wallet behavior. Its job is to make real-chain test setup predictable: start the local infrastructure, talk to `bitcoind`, fund wallet addresses, mine blocks, inspect the mempool, and decode PSBT inputs when tests need exact assertions.

## What It Provides

### Regtest Environment Control

`RegtestEnv` is the high-level entry point for test setup.

It can:
- start the local regtest stack through `infra/regtest/scripts/start.sh`
- stop the stack through `infra/regtest/scripts/stop.sh`
- reset the stack through `infra/regtest/scripts/reset.sh`
- mine blocks through RPC or the project script
- fund regtest addresses in BTC or satoshis
- query chain height and miner-wallet balance
- return a ready miner wallet RPC client

### Bitcoin Core RPC Helpers

The RPC helpers wrap the local `bitcoind` node and miner wallet.

They support:
- base node RPC client creation
- miner wallet RPC client creation
- miner wallet loading
- block-height queries
- mining to a new or supplied regtest address
- funding an address from the miner wallet
- raw mempool snapshots
- mempool membership checks

### Path Helpers

The path helpers locate repository and regtest resources from the current working directory.

They resolve:
- repository root
- `infra/regtest`
- regtest scripts
- bitcoind data directory
- electrs database directory
- local wallet JSON fixtures

### Wallet Parsing Helpers

The wallet helpers provide small parsing and inspection utilities used by integration tests.

They include:
- regtest address parsing with network enforcement
- txid parsing
- outpoint txid extraction
- PSBT input outpoint decoding

## Runtime Configuration

`BitcoindConfig::from_env()` reads these environment variables and falls back to the project regtest defaults:

- `WALLET_REGTEST_RPC_URL`, default `http://127.0.0.1:18443`
- `WALLET_REGTEST_RPC_USER`, default `bitcoin`
- `WALLET_REGTEST_RPC_PASS`, default `bitcoin`
- `WALLET_REGTEST_MINER_WALLET`, default `miner`

These defaults match the local infrastructure under `infra/regtest`.

## Design Boundaries

`test_support` intentionally stays small.

It should:
- own local regtest plumbing
- expose deterministic test helpers
- keep setup code out of wallet crates
- make scenario tests easier to read

It should not:
- duplicate `wallet_core` transaction logic
- duplicate `wallet_api` orchestration
- hide important sync, mining, or confirmation steps
- behave like a mock chain backend

## Main Consumers

The main consumer today is the regtest integration suite under `crates/wallet_api/tests`.

Those tests use `test_support` to set up real wallet states before calling the production API. This keeps the tests close to real wallet behavior while avoiding repeated low-level RPC and script plumbing in each scenario.

The `wallet_api` tests also define a small local helper, `ensure_confirmed_wallet_utxos`, under `crates/wallet_api/tests/support`. That helper builds on `RegtestEnv` and `parse_regtest_address` to top up a wallet with confirmed UTXOs before scenario assertions.

## Summary

`test_support` is the local integration-test harness for the workspace.

It provides real regtest infrastructure control and small assertion helpers, while keeping wallet behavior inside `wallet_core`, `wallet_api`, and `wallet_sync`.
