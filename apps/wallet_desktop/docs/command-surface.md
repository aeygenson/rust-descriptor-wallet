# Command Surface

This document describes the command surface that is actually registered in [src-tauri/src/lib.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/lib.rs).

The React frontend talks to Rust through typed feature API wrappers and a shared invoke helper. Command names are snake_case on the Tauri side and are wrapped by camelCase helpers in the frontend.

## Wallet Commands

Rust module:

- [src-tauri/src/commands/wallet.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/commands/wallet.rs)

Registered commands:

- `get_app_info`
- `list_wallets`
- `get_wallet_status`
- `sync_wallet`
- `backend_health`
- `get_receive_address`
- `list_receive_addresses`
- `label_receive_address`
- `clear_receive_address_label`
- `create_address_book_entry`
- `list_address_book_entries`
- `get_address_book_entry`
- `delete_address_book_entry`

Frontend wrappers:

- [src/features/wallet/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/wallet/api.ts)
- [src/features/receive/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/receive/api.ts)
- [src/features/address-book/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/address-book/api.ts)

## UTXO Commands

Rust module:

- [src-tauri/src/commands/utxos.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/commands/utxos.rs)

Registered commands:

- `list_utxos`

Frontend wrappers:

- [src/features/utxos/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/utxos/api.ts)

## Transaction Commands

Rust module:

- [src-tauri/src/commands/transactions.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/commands/transactions.rs)

Registered commands:

- `list_transactions`

Frontend wrappers:

- [src/features/transactions/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/transactions/api.ts)

## Send / PSBT Commands

Rust module:

- [src-tauri/src/commands/send.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/commands/send.rs)

Request decoding:

- [src-tauri/src/commands/send_model.rs](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src-tauri/src/commands/send_model.rs)

Registered PSBT preview commands:

- `create_psbt`
- `create_psbt_with_coin_control`
- `create_send_max_psbt`
- `create_send_max_psbt_with_coin_control`
- `create_sweep_psbt`
- `create_consolidation_psbt`
- `bump_fee_psbt`
- `cpfp_psbt`

Registered signing/publish/direct-send commands:

- `sign_psbt`
- `publish_psbt`
- `send_psbt`
- `send_psbt_with_coin_control`
- `send_max_psbt`
- `send_max_psbt_with_coin_control`
- `send_sweep_psbt`
- `consolidate_psbt`
- `bump_fee`
- `cpfp`

Frontend wrappers:

- [src/features/send/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/send/api.ts)
- [src/features/transactions/api.ts](/Users/alexandereygenson/MyRust/rust-descriptor-wallet/apps/wallet_desktop/src/features/transactions/api.ts)

## Request Shape Convention

The implemented frontend does not pass flat positional parameters.

Instead, most commands receive a `request` object with camelCase fields on the TypeScript side. The Rust send-model layer decodes those into the canonical request DTO shapes expected by `wallet_api`.

Examples:

- `walletName`
- `address`
- `amountSat`
- `feeRateSatVb`
- `replaceable`
- `confirmedOnly`
- nested `coinControl`
- nested `consolidation`

The receive-address commands are the main exception here. They take flat arguments rather than a nested `request` object:

- `get_receive_address(walletName)` -> `WalletReceiveAddressHistoryDto`
- `list_receive_addresses(walletName)` -> `WalletReceiveAddressHistoryDto[]`
- `label_receive_address(walletName, address, label)` -> `WalletReceiveAddressHistoryDto`
- `clear_receive_address_label(walletName, address)` -> `WalletReceiveAddressHistoryDto`

The address-book commands follow the same flat-argument style:

- `create_address_book_entry(walletName, label, address, notes)` -> `AddressBookEntryDto`
- `list_address_book_entries(walletName)` -> `AddressBookEntryDto[]`
- `get_address_book_entry(walletName, address)` -> `AddressBookEntryDto | null`
- `delete_address_book_entry(walletName, address)` -> `boolean`

Receive DTOs now carry:

- `address`
- `keychain`
- `index`
- `bitcoin_uri`
- optional `qr_svg`
- optional `label`
- `created_at`
- optional `updated_at`

Address-book DTOs carry:

- `wallet_name`
- `network`
- `label`
- `address`
- optional `notes`
- `created_at`
- optional `updated_at`

## Important Boundary Rules

- command handlers call `wallet_api`; they do not reimplement wallet logic
- send command handlers now mostly call `wallet_api::service::*` through canonical request DTOs, not handwritten argument lists
- frontend code should use feature `api.ts` files, not raw `invoke` in components
- backend validation errors are surfaced as strings today

## What Is Not In The Command Surface

The current app does not expose:

- transaction-details-by-id command
- wallet import/create/delete commands
- settings/configuration mutation commands

Those may come later, but they are not part of the current desktop surface and should not be documented as if they already exist.
