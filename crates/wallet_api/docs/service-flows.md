# Wallet API Service Flows

This document describes the main flows exposed through `WalletApi`.

## Wallet Registry

Registry methods use `service/registry.rs` and `wallet_storage`:

- `import_wallet(file_path)` imports a wallet JSON file into storage.
- `list_wallets()` returns wallet names, networks, and watch-only flags.
- `get_wallet(name)` returns descriptors, sync backend, broadcast backend, and watch-only status.
- `delete_wallet(name)` removes the stored wallet record.

Backend configuration is parsed when a wallet is loaded. Invalid stored backend metadata is surfaced as `WalletApiError::InvalidInput`.

## Wallet State

Wallet state methods use `service/wallet.rs`.

`sync_wallet(name)` loads the wallet configuration, opens or creates the runtime wallet store, and calls `WalletSyncService::sync`. It returns `()` on success.

`address(name)` loads the runtime wallet and derives the next receive address.

`balance(name)` returns the current persisted wallet balance in satoshis. It does not perform a network sync.

`status(name)` returns `WalletStatusDto` with balance, UTXO count, and the highest known confirmation height from current wallet state. It does not perform a network sync.

## Inspection

Inspection methods use `service/inspect.rs`.

`txs(name)` returns `Vec<WalletTxDto>` from current synced wallet state.

`utxos(name)` returns `Vec<WalletUtxoDto>` from current synced wallet state.

These methods intentionally avoid network calls. Run `sync_wallet(name)` first when the caller needs fresh chain state.

## PSBT Preview

Preview methods use `service/psbt.rs` and return transaction details before signing or broadcasting.

Fixed amount:

- `create_psbt(name, to, amount_sat, fee_rate_sat_per_vb)`
- `create_psbt_with_coin_control(name, to, amount_sat, fee_rate_sat_per_vb, coin_control)`

Send-max and sweep:

- `create_send_max_psbt(name, to, fee_rate_sat_per_vb)`
- `create_send_max_psbt_with_coin_control(name, to, fee_rate_sat_per_vb, coin_control)`
- `create_sweep_psbt(name, to, fee_rate_sat_per_vb, coin_control)`

Maintenance:

- `create_consolidation_psbt(name, fee_rate_sat_per_vb, consolidation)`
- `bump_fee_psbt(name, txid, fee_rate_sat_per_vb)`
- `cpfp_psbt(name, parent_txid, selected_outpoint, fee_rate_sat_per_vb)`

Preview DTOs expose selected inputs, input and output counts, fee, fee rate, change amount when applicable, txid, estimated virtual size, and replaceability.

## One-Shot Publish

One-shot methods compose preview, signing, and publication:

- `send_psbt`
- `send_psbt_with_coin_control`
- `send_max_psbt`
- `send_max_psbt_with_coin_control`
- `sweep_and_broadcast`
- `consolidate_and_broadcast`
- `bump_fee`
- `cpfp`

These methods are convenience paths for software-signing wallets. They build the relevant PSBT, sign it, publish the finalized transaction through the configured broadcast backend, and return `TxBroadcastResultDto`.

Watch-only wallets can create preview PSBTs but cannot sign through the software-signing API path.

## Coin Control

Coin control applies to fixed sends, send-max, sweep, and consolidation.

`WalletCoinControlDto` supports:

- `include_outpoints`
- `exclude_outpoints`
- `confirmed_only`
- `selection_mode`

Selection modes:

- `strict-manual`: use only explicitly included inputs.
- `manual-with-auto-completion`: pin included inputs and allow extra eligible inputs when needed.
- `automatic-only`: ignore manual include sets and let the backend select.

Invalid outpoint strings are converted into `WalletApiError::InvalidInput`.

## Consolidation

`WalletConsolidationDto` adds consolidation-specific controls:

- input include and exclude sets
- confirmed-only filtering
- minimum and maximum input count
- minimum and maximum UTXO value
- maximum fee percentage of selected input value
- strategy: `smallest-first`, `largest-first`, or `oldest-first`
- selection mode

The output is wallet-internal. Consolidation is not treated as an external payment.

## RBF

`bump_fee_psbt` and `bump_fee` operate on an unconfirmed replaceable transaction.

The API validates that the original transaction exists, is unconfirmed, is replaceable, and that the requested fee rate is higher than the original effective fee rate.

## CPFP

`cpfp_psbt` and `cpfp` operate on an unconfirmed parent transaction and a selected wallet-owned outpoint from that parent.

The API builds a child transaction spending the selected parent output at the requested fee rate. This is used to accelerate confirmation of an unconfirmed parent transaction.
