# Wallet API Overview

`wallet_api` is the application-facing boundary for the wallet workspace.

It gives callers a stable async facade over wallet storage, runtime wallet loading, sync, inspection, PSBT construction, signing, and publication. The crate is used by `wallet_cli`, integration tests, and the planned Tauri desktop UI.

## Responsibilities

`wallet_api` owns the caller-facing parts of wallet orchestration:

- wallet registry operations: import, list, get, and delete wallets
- runtime wallet operations: address generation, sync, balance, status, transaction listing, and UTXO listing
- PSBT preview flows: fixed amount, coin control, send-max, sweep, consolidation, RBF, and CPFP
- one-shot transaction flows: build, sign, publish, and return the broadcast result
- DTO conversion: parse caller-friendly strings and simple values into typed domain requests
- error normalization: convert storage, sync, broadcast, and core failures into `WalletApiError`

It does not implement low-level wallet rules itself. Transaction semantics live in `wallet_core`, chain access lives in `wallet_sync`, and persistence lives in `wallet_storage`.

## Public Surface

The main entry point is `WalletApi` in `src/api.rs`.

Wallet metadata:

- `list_wallets`
- `get_wallet`
- `import_wallet`
- `delete_wallet`

Wallet state:

- `address`
- `sync_wallet`
- `balance`
- `status`
- `txs`
- `utxos`

PSBT previews:

- `create_psbt`
- `create_psbt_with_coin_control`
- `create_send_max_psbt`
- `create_send_max_psbt_with_coin_control`
- `create_sweep_psbt`
- `create_consolidation_psbt`
- `bump_fee_psbt`
- `cpfp_psbt`

One-shot publish flows:

- `send_psbt`
- `send_psbt_with_coin_control`
- `send_max_psbt`
- `send_max_psbt_with_coin_control`
- `sweep_and_broadcast`
- `consolidate_and_broadcast`
- `bump_fee`
- `cpfp`

PSBT utilities:

- `sign_psbt`
- `publish_psbt`

## Relationship To Other Crates

`wallet_core` owns typed domain behavior: descriptor-backed wallet loading, address derivation, UTXO inspection, transaction building, PSBT signing, send-max, sweep, consolidation, RBF, CPFP, and coin-control policy.

`wallet_sync` owns backend integration: Esplora, Electrum, and Bitcoin Core RPC sync or broadcast paths.

`wallet_storage` owns the local wallet registry and imported wallet metadata.

`wallet_api` wires those crates together and exposes a stable boundary for user-facing apps.

## Caller Model

Callers should treat `wallet_api` as the single integration layer.

CLI and UI code should collect user intent, call `WalletApi`, and render DTOs or `WalletApiError`. They should not duplicate wallet selection rules, parse PSBT internals, or call lower-level crates directly for normal wallet operations.

## Test Coverage

`crates/wallet_api/tests/regtest_flow.rs` exercises the API against a local Bitcoin Core and Electrum regtest environment. It covers wallet receive/send behavior, PSBT signing and publication, coin control, send-max, sweep, consolidation, RBF, CPFP, and invalid input handling.
