# Infra Overview

The `infra` directory contains the local Bitcoin infrastructure used for development and integration testing.

Today that means one concrete profile:

- `infra/regtest` for a fully local Bitcoin Core plus Electrs environment

This infrastructure exists to support realistic wallet testing without depending on public networks.

## What Infra Provides

The regtest profile provides:

- a local `bitcoind` node in regtest mode
- an Electrs server for Electrum-compatible wallet sync
- helper scripts for start, stop, reset, mine, and fund flows
- checked-in configuration files for Bitcoin Core and Electrs

## What Infra Does Not Provide

The infra layer does not own:

- wallet registry storage
- transaction construction
- PSBT signing
- API orchestration

It only provides the local chain environment that the higher-level crates use.

## Main Consumers

The local environment is consumed by:

- `wallet_sync` for Electrum sync and Bitcoin Core RPC broadcast
- `wallet_api` integration tests
- `test_support` helpers
- manual development workflows from the CLI

## Current Layout

```text
infra/
├── docs/
│   ├── overview.md
│   ├── regtest-setup.md
│   └── scripts.md
└── regtest/
    ├── README.md
    ├── bitcoin/
    │   └── bitcoin.conf
    ├── electrs/
    │   └── electrs.toml
    └── scripts/
        ├── start.sh
        ├── stop.sh
        ├── reset.sh
        ├── mine.sh
        └── fund.sh
```

## Operational Model

High-level flow:

```text
bitcoind (regtest)
  -> electrs
  -> wallet_sync
  -> wallet_api
  -> CLI / tests / desktop UI
```

The regtest profile mirrors the production-style architecture closely enough to test funding, sync, UTXO selection, broadcast, confirmation, RBF, and CPFP with real Bitcoin software.
