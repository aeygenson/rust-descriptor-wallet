# Wallet API Service Flows

This document describes the main flows exposed through `WalletApi`.

Internally, these flows are implemented with canonical request DTOs. `WalletApi` convenience methods are thin wrappers that build those request objects and hand them to the service layer.

## Wallet Registry

Registry methods use `service/registry.rs` and `wallet_storage`:

- `import_wallet(file_path)` imports a wallet JSON file into storage via `ImportWalletRequestDto`.
- `list_wallets()` returns wallet names, networks, and watch-only flags.
- `get_wallet(name)` returns descriptors, sync backend, broadcast backend, and watch-only status via `GetWalletRequestDto`.
- `descriptor_info(name)` returns a safe redacted descriptor inspection DTO for external and internal branches. It intentionally avoids returning raw private descriptor material, backend credentials, or wallet database paths.
- `delete_wallet(name)` removes the stored wallet record via `DeleteWalletRequestDto`.

Backend configuration is parsed when a wallet is loaded. Invalid stored backend metadata is surfaced as `WalletApiError::InvalidInput`.

## Wallet State

Wallet state methods use `service/wallet.rs` and `service/addresses.rs`.

`sync(name)` loads the wallet configuration, opens or creates the runtime wallet store, and calls `WalletSyncService::sync`. It returns `()` on success.

`address(name)` loads the runtime wallet, derives the next receive address via `WalletAddressRequestDto`, persists that row into storage, renders a QR SVG from the canonical Bitcoin URI, and returns a `WalletReceiveAddressHistoryDto`. The response includes `address`, `keychain`, optional derivation `index`, `bitcoin_uri`, optional `qr_svg`, optional `label`, and timestamps.

`list_receive_addresses(name)` reads persisted receive-address history for the wallet through `WalletReceiveAddressesRequestDto` and attaches QR SVG payloads to each returned row.

`label_receive_address(name, address, label)` updates the stored label for an existing persisted receive-address row through `LabelReceiveAddressRequestDto` and returns the QR-backed row.

`clear_receive_address_label(name, address)` clears the stored label for an existing persisted receive-address row through `ClearReceiveAddressLabelRequestDto` and returns the QR-backed row.

`create_address_book_entry(name, label, address, notes)` validates the wallet exists, derives the wallet network from stored config, persists a wallet-scoped external-recipient row through `CreateAddressBookEntryRequestDto`, and returns an `AddressBookEntryDto`.

`list_address_book_entries(name)` reads persisted wallet-scoped external-recipient rows through `ListAddressBookEntriesRequestDto`.

`get_address_book_entry(name, address)` looks up one persisted address-book row through `GetAddressBookEntryRequestDto`.

`delete_address_book_entry(name, address)` removes one persisted address-book row through `DeleteAddressBookEntryRequestDto` and returns a `bool` indicating whether a row was deleted.

`lock_utxos(name, outpoints, reason)` persists wallet-scoped spend locks through `WalletLockUtxosRequestDto` and returns the affected locked rows.

`unlock_utxos(name, outpoints)` removes persisted wallet-scoped spend locks through `WalletUnlockUtxosRequestDto` and returns the remaining locked rows.

`list_locked_utxos(name)` reads persisted wallet-scoped locked-outpoint rows through `WalletLockedUtxosRequestDto`.

`backend_health(name)` checks configured backend reachability and reports tip visibility without mutating wallet state.

`balance(name)` returns the current persisted wallet balance in satoshis. It does not perform a network sync.

`status(name)` returns `WalletStatusDto` with balance, UTXO count, and the highest known confirmation height from current wallet state. It does not perform a network sync.

## Inspection

Inspection methods use `service/inspect.rs`.

`txs(name)` returns `Vec<WalletTxDto>` from current synced wallet state via `WalletTransactionsRequestDto`.

`utxos(name)` returns `Vec<WalletUtxoDto>` from current synced wallet state via `WalletUtxosRequestDto`.

UTXO inspection is enriched with persisted lock state. `WalletUtxoDto` now includes:

- `is_locked`
- `lock_reason`
- `locked_at`

These methods intentionally avoid network calls. Run `sync(name)` first when the caller needs fresh chain state.

## PSBT Preview

Preview methods use `service/psbt.rs` and return transaction details before signing or broadcasting.

Each preview or publish workflow has a canonical request DTO inside `model.rs`, even when `WalletApi` still exposes an ergonomic convenience signature.

Fixed amount:

- `create_psbt(name, to, amount_sat, fee_rate_sat_per_vb, replaceable, confirmed_only)`
- `create_psbt_with_coin_control(name, to, amount_sat, fee_rate_sat_per_vb, replaceable, coin_control)`

Both normalize into `CreatePsbtRequestDto`.

Send-max and sweep:

- `create_send_max_psbt(name, to, fee_rate_sat_per_vb, replaceable)`
- `create_send_max_psbt_with_coin_control(name, to, fee_rate_sat_per_vb, replaceable, coin_control)`
- `create_sweep_psbt(name, to, fee_rate_sat_per_vb, replaceable, coin_control)`

These normalize into `SendMaxRequestDto` and `SweepRequestDto`.

Maintenance:

- `create_consolidation_psbt(name, fee_rate_sat_per_vb, consolidation)`
- `bump_fee_psbt(name, txid, fee_rate_sat_per_vb)`
- `cpfp_psbt(name, parent_txid, selected_outpoint, fee_rate_sat_per_vb)`

These normalize into `ConsolidationRequestDto`, `BumpFeeRequestDto`, and `CpfpRequestDto`.

Preview DTOs expose selected inputs, input and output counts, fee, fee rate, change amount when applicable, txid, estimated virtual size, and replaceability. Replacement-aware previews can also expose `original_txid` and replacement lineage metadata.

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

Internally, signing and publishing use `SignPsbtRequestDto` and `PublishPsbtRequestDto`.

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

Locked outpoints are enforced below the DTO layer:

- explicit include/select requests fail with `WalletApiError::LockedUtxo`
- automatic selection paths merge locked outpoints into the effective exclude set
- this applies to fixed sends, send-max, sweep, consolidation, and CPFP input selection

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
