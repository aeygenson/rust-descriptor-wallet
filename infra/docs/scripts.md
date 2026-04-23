# Regtest Scripts

The helper scripts live in:

```text
infra/regtest/scripts/
```

They provide the operational interface for the local regtest profile.

## `start.sh`

Starts or reuses the local regtest services.

Actual behavior:

- start `bitcoind` if it is not already running for this datadir
- wait for RPC readiness
- create or load the `miner` wallet
- mine the initial 101 blocks when the chain is empty
- start or reuse `electrs`
- wait for the Electrum server port

Use it at the beginning of a manual session or before local integration testing.

## `stop.sh`

Stops the local regtest profile.

Actual behavior:

- stop `electrs`
- release configured Electrum and monitoring ports when needed
- stop the regtest `bitcoind` instance associated with this datadir

## `reset.sh`

Resets the local regtest data.

Actual behavior:

- stop running services
- delete local bitcoind chain data
- delete the Electrs index/state

By default it asks for confirmation. For automation, use:

```bash
FORCE=1 ./reset.sh
```

## `mine.sh`

Mines blocks to a new address from the local `miner` wallet.

Usage:

```bash
./mine.sh [block_count]
```

Behavior:

- defaults to 1 block
- uses Bitcoin Core RPC
- advances chain state and confirms pending transactions

Examples:

```bash
./mine.sh
./mine.sh 6
```

## `fund.sh`

Funds a destination address from the local `miner` wallet and mines 1 confirmation block.

Usage:

```bash
./fund.sh <address> [amount_btc]
```

Behavior:

- defaults to `1` BTC when amount is omitted
- sends funds through Bitcoin Core RPC
- mines 1 block after sending

Examples:

```bash
./fund.sh bcrt1...
./fund.sh bcrt1... 0.25
```

## Binary Discovery

The scripts use `PATH` by default and support these overrides:

```bash
BITCOIND_BIN=/path/to/bitcoind
BITCOIN_CLI_BIN=/path/to/bitcoin-cli
ELECTRS_BIN=/path/to/electrs
```

This matters when local installs are not on the default shell path.

## Relationship To Rust Helpers

These shell scripts are for manual environment control.

Automated Rust integration tests should generally prefer the Rust helpers in `crates/test_support`, which use the same regtest profile but provide a programmatic interface.
