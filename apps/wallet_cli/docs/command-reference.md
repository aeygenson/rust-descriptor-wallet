# Wallet CLI Command Reference

This document describes the user-facing commands exposed by `wallet_cli`.

The CLI is a thin layer over `wallet_api`. Commands keep user input as strings and DTO values at the boundary; parsing into typed wallet-core values happens below the CLI.

## General Usage

```bash
cargo run -p wallet_cli -- <command> [options]
```

Most runtime commands require:

```bash
--name <wallet>
```

Values:
- fee rates are expressed as sat/vB
- amounts are expressed as satoshis
- outpoints use `<txid>:<vout>`
- PSBT values use base64 strings

## Wallet Management

### List wallets

```bash
cargo run -p wallet_cli -- list-wallets
```

### Import wallet

```bash
cargo run -p wallet_cli -- import-wallet --file wallet-regtest-local.json
```

### Get wallet details

```bash
cargo run -p wallet_cli -- get-wallet --name <wallet>
```

### Delete wallet

```bash
cargo run -p wallet_cli -- delete-wallet --name <wallet>
```

## Sync and State

### Sync wallet

```bash
cargo run -p wallet_cli -- sync --name <wallet>
```

### Show balance

```bash
cargo run -p wallet_cli -- balance --name <wallet>
```

### Show status

```bash
cargo run -p wallet_cli -- status --name <wallet>
```

Status includes wallet balance, UTXO count, and latest observed block height.

### Generate receive address

```bash
cargo run -p wallet_cli -- address --name <wallet>
```

### List UTXOs

```bash
cargo run -p wallet_cli -- utxos --name <wallet>
```

UTXO output includes outpoint, value, confirmation state, address when available, and keychain.

### List transactions

```bash
cargo run -p wallet_cli -- txs --name <wallet>
```

Transaction output includes txid, direction, net value, fee when known, confirmation state, and replaceability.

## PSBT Lifecycle

### Create fixed-amount PSBT

```bash
cargo run -p wallet_cli -- create-psbt \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb>
```

### Create fixed-amount PSBT with coin control

```bash
cargo run -p wallet_cli -- create-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --exclude <txid:vout> \
  --confirmed-only \
  --selection-mode strict-manual
```

`--include` and `--exclude` can be repeated.

Selection modes:
- `strict-manual`
- `manual-with-auto-completion`
- `automatic-only`

### Sign PSBT

```bash
cargo run -p wallet_cli -- sign-psbt \
  --name <wallet> \
  --psbt-base64 '<base64>'
```

### Publish finalized PSBT

```bash
cargo run -p wallet_cli -- publish-psbt \
  --name <wallet> \
  --psbt-base64 '<base64>'
```

## One-Shot Fixed Send

Despite the command name, `send-psbt` is the one-shot fixed-send flow: create, sign, and publish.

```bash
cargo run -p wallet_cli -- send-psbt \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb>
```

### One-shot fixed send with coin control

```bash
cargo run -p wallet_cli -- send-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --amount <sat> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --exclude <txid:vout> \
  --confirmed-only \
  --selection-mode strict-manual
```

## Send-Max

### Create send-max PSBT

```bash
cargo run -p wallet_cli -- create-send-max-psbt \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb>
```

### Create send-max PSBT with coin control

```bash
cargo run -p wallet_cli -- create-send-max-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --exclude <txid:vout> \
  --confirmed-only \
  --selection-mode strict-manual
```

### One-shot send-max

```bash
cargo run -p wallet_cli -- send-max-psbt \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb>
```

### One-shot send-max with coin control

```bash
cargo run -p wallet_cli -- send-max-psbt-with-coin-control \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --exclude <txid:vout> \
  --confirmed-only \
  --selection-mode strict-manual
```

## Sweep

Sweep drains selected UTXOs to a destination after fees.

### Create sweep PSBT

```bash
cargo run -p wallet_cli -- sweep-psbt \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --selection-mode strict-manual
```

### One-shot sweep

```bash
cargo run -p wallet_cli -- sweep \
  --name <wallet> \
  --to <address> \
  --fee-rate <sat/vb> \
  --include <txid:vout> \
  --selection-mode strict-manual
```

Sweep also accepts `--exclude`, `--confirmed-only`, and `--selection-mode`.

## Consolidation

Consolidation is wallet-internal maintenance. It spends multiple wallet UTXOs into a wallet-owned output.

### Create consolidation PSBT

```bash
cargo run -p wallet_cli -- create-consolidation-psbt \
  --name <wallet> \
  --fee-rate <sat/vb> \
  --confirmed-only \
  --min-input-count 2 \
  --strategy smallest-first \
  --selection-mode automatic-only
```

### One-shot consolidation

```bash
cargo run -p wallet_cli -- consolidate-psbt \
  --name <wallet> \
  --fee-rate <sat/vb> \
  --confirmed-only \
  --min-input-count 2 \
  --strategy smallest-first \
  --selection-mode automatic-only
```

Consolidation options:
- `--include <txid:vout>` can be repeated
- `--exclude <txid:vout>` can be repeated
- `--confirmed-only`
- `--max-input-count <n>`
- `--min-input-count <n>`
- `--min-utxo-value-sat <sat>`
- `--max-utxo-value-sat <sat>`
- `--max-fee-pct <percent>`
- `--strategy smallest-first|largest-first|oldest-first`
- `--selection-mode strict-manual|manual-with-auto-completion|automatic-only`

## RBF Fee Bump

### Create bump-fee PSBT

```bash
cargo run -p wallet_cli -- bump-fee-psbt \
  --name <wallet> \
  --txid <txid> \
  --fee-rate <sat/vb>
```

### One-shot bump fee

```bash
cargo run -p wallet_cli -- bump-fee \
  --name <wallet> \
  --txid <txid> \
  --fee-rate <sat/vb>
```

## CPFP

CPFP spends a wallet-owned unconfirmed parent output in a child transaction.

### Create CPFP PSBT

```bash
cargo run -p wallet_cli -- cpfp-psbt \
  --name <wallet> \
  --parent-txid <txid> \
  --outpoint <txid:vout> \
  --fee-rate <sat/vb>
```

### One-shot CPFP

```bash
cargo run -p wallet_cli -- cpfp \
  --name <wallet> \
  --parent-txid <txid> \
  --outpoint <txid:vout> \
  --fee-rate <sat/vb>
```

## Notes

- All commands are deterministic wrappers over `wallet_api`.
- CLI output should be treated as backend-authoritative preview data.
- The CLI does not reimplement wallet selection, signing, or broadcast logic.
- Invalid outpoints and txids should surface as API errors, not panics.
