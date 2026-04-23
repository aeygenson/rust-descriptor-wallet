# Regtest Setup

This document describes the actual local regtest profile under `infra/regtest`.

For the most up-to-date operational details, `infra/regtest/README.md` is the source of truth. This document is the higher-level infrastructure overview.

## Components

`bitcoind`

- runs in regtest mode
- exposes RPC on the configured local port
- holds chain state and mempool
- provides the local `miner` wallet used by helper scripts

`electrs`

- indexes the local regtest node
- exposes an Electrum-compatible server for wallet sync
- writes logs under `infra/regtest/electrs/electrs.log`

`scripts`

- `start.sh`
- `stop.sh`
- `reset.sh`
- `mine.sh`
- `fund.sh`

`configuration`

- `infra/regtest/bitcoin/bitcoin.conf`
- `infra/regtest/electrs/electrs.toml`

## Real Directory Layout

```text
infra/regtest/
├── README.md
├── bitcoin/
│   └── bitcoin.conf
├── electrs/
│   ├── electrs.log
│   └── electrs.toml
└── scripts/
    ├── fund.sh
    ├── mine.sh
    ├── reset.sh
    ├── start.sh
    └── stop.sh
```

The runtime data and generated files are created beneath the regtest tree when the scripts run.

## Requirements

The local profile expects these tools to be installed and discoverable through `PATH`, unless overridden explicitly:

- `bitcoind`
- `bitcoin-cli`
- `electrs`

Typical installation example on macOS:

```bash
brew install bitcoin
cargo install electrs
```

## Startup Behavior

`infra/regtest/scripts/start.sh` is the main entry point.

It:

- starts or reuses the configured regtest `bitcoind`
- waits for RPC readiness
- creates or loads the `miner` wallet
- mines the initial 101 blocks when the chain is empty
- starts or reuses `electrs`
- waits for the Electrum port to become ready

The script is designed to be idempotent for the configured local profile.

## Ports And Defaults

The current local profile uses these defaults:

- Bitcoin Core RPC: `18443`
- Bitcoin Core P2P: `18444`
- Electrum: `60401`
- Electrs monitoring: `24224`

Wallet configuration for this profile typically uses:

```text
NETWORK=regtest
ELECTRUM_URL=tcp://127.0.0.1:60401
BITCOIN_RPC_URL=http://127.0.0.1:18443
BITCOIN_RPC_USER=bitcoin
BITCOIN_RPC_PASS=bitcoin
```

## Script Overrides

The scripts rely on `PATH` by default, but support explicit overrides such as:

```bash
BITCOIND_BIN=/path/to/bitcoind
BITCOIN_CLI_BIN=/path/to/bitcoin-cli
ELECTRS_BIN=/path/to/electrs
BITCOIN_RPC_PORT=18443
BITCOIN_P2P_PORT=18444
ELECTRUM_PORT=60401
ELECTRS_MONITORING_PORT=24224
```

## Why This Setup Exists

This profile gives the codebase deterministic control over:

- funding wallets
- mining confirmations on demand
- testing confirmed-only behavior
- testing strict coin control against real UTXOs
- testing send-max, sweep, consolidation, RBF, and CPFP with real chain state

That is why regtest is used instead of relying on public networks or only mock backends.
