# Test Architecture

The project uses regtest-backed integration tests to validate wallet behavior against a real local Bitcoin node.

The purpose of `test_support` is to make those tests deterministic and readable without moving wallet logic into the test harness.

## Architecture Layers

### Infrastructure Layer

The local infrastructure lives under `infra/regtest`.

It provides:
- `bitcoind` on regtest
- a miner wallet used for funding and block generation
- electrs for Electrum-backed wallet sync
- scripts for start, stop, reset, funding, and mining

`test_support::paths` resolves those scripts and data directories from the repository root.

### RPC Layer

`test_support::bitcoind` and `test_support::rpc` wrap Bitcoin Core RPC access.

This layer handles:
- RPC configuration from environment variables
- base node client creation
- miner wallet client creation
- miner wallet loading
- block-height queries
- mining
- funding
- mempool inspection

Tests should prefer these helpers over open-coded `bitcoincore_rpc` setup.

### Scenario Setup Layer

`RegtestEnv` is the common scenario setup facade.

Typical use:

```rust
let env = RegtestEnv::new();
env.start()?;
env.mine(1)?;
```

For wallet funding:

```rust
let address = parse_regtest_address(&address_string)?;
env.fund_sats(&address, 100_000)?;
env.mine(1)?;
```

The test then calls production APIs such as `WalletApi::sync`, `WalletApi::create_psbt`, `WalletApi::sign_psbt`, or `WalletApi::publish_psbt`.

### Assertion Helper Layer

Small helpers in `test_support::wallet` support precise assertions.

Examples:
- `parse_regtest_address` rejects non-regtest addresses
- `parse_txid` validates txid strings
- `outpoint_txid` extracts the txid portion of `txid:vout`
- `decode_psbt_inputs` returns unsigned transaction input outpoints from a PSBT

These helpers are intentionally narrow. They should not decide wallet policy.

## Typical Integration Test Flow

A regtest scenario normally follows this shape:

1. Create `RegtestEnv`.
2. Start or reuse the local regtest stack.
3. Create or load a wallet through `wallet_api`.
4. Generate a wallet receive address.
5. Fund the address through the miner wallet.
6. Mine blocks if confirmed funds are required.
7. Sync the wallet through the configured backend.
8. Execute a production wallet operation.
9. Inspect preview data, mempool state, or wallet state.
10. Mine and sync again when confirmation behavior matters.

The important rule is that tests should make chain transitions explicit. Funding, mining, and sync are separate steps because each step represents real wallet state movement.

## Concurrency Model

Regtest scenarios share local infrastructure, ports, miner wallet state, and wallet databases.

For that reason, integration tests should run serially when they interact with shared regtest resources. The current wallet API regtest suite uses `serial_test` for this reason, and each async test uses the Tokio `current_thread` flavor.

Running the test binary with one test thread is still useful:

```bash
cargo test -p wallet_api --test regtest_flow -- --test-threads=1
```

Serial execution avoids:
- mempool races
- wallet database interference
- confirmation-order ambiguity
- concurrent script start/reset conflicts

The current-thread Tokio flavor keeps each test deterministic from the async runtime side. It does not replace `serial_test`; both are useful because they solve different problems.

## Determinism Rules

Good regtest tests should:
- use explicit wallet names or isolated fixture state
- fund exact amounts
- mine intentionally when confirmation is required
- call wallet sync before state assertions
- inspect final backend-selected inputs instead of assuming requested inputs are final
- avoid hidden sleeps unless waiting for infrastructure readiness

`test_support` helps with these rules, but the test still owns the scenario.

## What to Assert

Useful assertions include:
- wallet balance after funding, spending, and confirmation
- PSBT selected inputs and input count
- change amount and output count
- recipient count consistency
- fee-rate and virtual-size metadata
- mempool membership before and after RBF or CPFP
- replacement transaction visibility
- child transaction visibility
- consolidation input and output behavior
- consolidation fee percentage limits
- consolidation min/max UTXO value filters
- consolidation strategy ordering
- wallet-core invariant preservation
- rejection of invalid outpoints, txids, or impossible coin-control policies

Less useful assertions are simple success checks that do not verify wallet behavior.

## Boundaries

`test_support` should remain a helper crate.

It should not:
- construct wallet policies
- select transaction inputs
- decide fee-bump behavior
- sign transactions
- publish through anything except explicit RPC helper calls used by tests

Those responsibilities belong to production crates.

## Summary

The test architecture is intentionally backend-first: tests use real local Bitcoin infrastructure and production wallet APIs, while `test_support` provides deterministic setup and inspection tools.

This gives higher confidence than mocks for transaction behavior such as coin control, send-max, sweep, consolidation, RBF, and CPFP.
