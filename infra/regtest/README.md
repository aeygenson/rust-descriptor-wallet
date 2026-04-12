# Regtest Environment

This directory contains a fully local Bitcoin regtest setup using:

- Bitcoin Core (`bitcoind`)
- Electrs (Electrum server)

This environment is used for:

- integration testing
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
- start `electrs`

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
ELECTRUM_URL=tcp://127.0.0.1:50001
BITCOIN_RPC_URL=http://127.0.0.1:18443
BITCOIN_RPC_USER=bitcoin
BITCOIN_RPC_PASS=bitcoin
```

---

## Testing flows

You can now reliably test:

- send transactions without immediate confirmation
- RBF (bump-fee)
- CPFP
- mempool behavior

Unlike Signet, regtest allows full control over block production.

---

## Notes

- Do not run multiple regtest instances on the same ports
- Always reset if you see inconsistent state
- Scripts are for manual control only (tests should use Rust `test_support`)

---

## Next steps

- integrate `test_support` crate
- add automated regtest tests
- implement CPFP
