# Regtest Environment

This directory contains a fully local Bitcoin regtest setup using:

- Bitcoin Core (`bitcoind`)
- Electrs (Electrum server)

This environment is used for:

- integration testing
- explicit coin control testing
- wallet-internal UTXO consolidation testing
- RBF (Replace-By-Fee) testing
- CPFP (Child Pays For Parent)
- controlled mempool behavior

---

## Structure

```
regtest/
  bitcoin/
    bitcoin.conf
    data/

  electrs/
    electrs.toml
    db/

  scripts/
    start.sh
    stop.sh
    reset.sh
    mine.sh
    fund.sh
```

---

## Requirements

You must have installed locally:

```bash
brew install bitcoin
cargo install electrs
```

Verify:

```bash
which bitcoind
which electrs
```

---

## Start environment

```bash
cd infra/regtest/scripts
./start.sh
```

This will:

- start `bitcoind` in regtest mode
- wait for RPC readiness
- create or load the local `miner` wallet
- mine the initial 101 blocks when the chain is empty
- start `electrs`
- wait for the Electrum port to listen

The script is intentionally idempotent: if `bitcoind` or `electrs` already owns the configured ports for this regtest profile, startup reuses the running services instead of starting duplicates.

Binary discovery uses `PATH` by default. Override these variables when needed:

```bash
BITCOIND_BIN=/path/to/bitcoind \
BITCOIN_CLI_BIN=/path/to/bitcoin-cli \
ELECTRS_BIN=/path/to/electrs \
./start.sh
```

Electrs output is written to:

```text
infra/regtest/electrs/electrs.log
```

---

## Initialize blockchain

Run once after startup:

```bash
./mine.sh
```

This mines 101 blocks and unlocks coinbase funds.

---

## Fund a wallet address

```bash
./fund.sh <ADDRESS>
```

This will:

- send 1 BTC to the address
- mine 1 block to confirm it

---

## Stop environment

```bash
./stop.sh
```

This stops electrs first, frees the Electrum and monitoring ports if needed, then stops the regtest `bitcoind` instance for this datadir. The default Electrum port is `60401`.

---

## Reset environment

```bash
./reset.sh
```

This deletes:

- blockchain data
- electrs index

Use when you want a clean chain.

---

## Wallet configuration

Use these settings in your wallet:

```env
NETWORK=regtest
ELECTRUM_URL=tcp://127.0.0.1:60401
BITCOIN_RPC_URL=http://127.0.0.1:18443
BITCOIN_RPC_USER=bitcoin
BITCOIN_RPC_PASS=bitcoin
```

---

## Testing flows

You can now reliably test:

- send transactions without immediate confirmation
- send-max wallet drains
- sweep flows over explicitly selected outpoints
- wallet-internal consolidation flows
- coin control with explicit include/exclude outpoints
- strict coin control where included outpoints must fully fund the spend
- RBF (bump-fee)
- CPFP
- mempool behavior

Unlike Signet, regtest allows full control over block production.

---

## Notes

- Do not run multiple regtest instances on the same ports
- Always reset if you see inconsistent state
- Scripts are for manual control; automated tests should prefer Rust `test_support`
- The sample wallet config lives at `wallet-regtest-local.json`
- This local profile uses Electrum for sync and Bitcoin Core RPC for broadcast

---

## Current Coverage

Current automated regtest coverage includes:

- receive funds and observe balance after sync
- self-send flows with change output tracking
- send-max PSBT creation and one-shot max-send flows
- sweep PSBT creation and one-shot sweep flows
- wallet-internal consolidation PSBT creation and one-shot consolidation flows
- coin-control PSBT creation and send flows
- strict manual, manual-with-auto-completion, and automatic-only input selection behavior
- RBF replacement and confirmation checks
- CPFP child build, publish, and confirmation checks

Run the full API regtest suite serially:

```bash
cargo test -p wallet_api --test regtest_flow -- --nocapture --test-threads=1
```

The test file uses current-thread Tokio tests plus `serial_test`, which keeps RustRover and Cargo runs aligned with the shared local regtest services.
