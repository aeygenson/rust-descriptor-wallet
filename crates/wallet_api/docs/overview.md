# Wallet API Overview

`wallet_api` is the application-facing boundary for the wallet workspace.

It gives callers a stable async facade over wallet storage, runtime wallet loading, sync, inspection, PSBT construction, signing, finalization, and publication. The crate is used by `wallet_cli`, integration tests, and the Tauri desktop UI.

## Responsibilities

`wallet_api` owns the caller-facing parts of wallet orchestration:

- wallet registry operations: import, list, get, and delete wallets
- runtime wallet operations: address generation, sync, balance, status, transaction listing, and UTXO listing
- address and inspection enrichment: persisted receive-address history, labels, keychain/index metadata, transaction graph inputs/outputs, and UTXO derivation metadata
- PSBT preview flows: fixed amount, coin control, send-max, sweep, consolidation, RBF, and CPFP
- one-shot transaction flows: build, sign, publish, and return the broadcast result
- DTO conversion: normalize caller input into canonical request DTOs, parse those into typed domain requests, and return stable response DTOs
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
- `list_receive_addresses`
- `label_receive_address`
- `clear_receive_address_label`
- `sync`
- `backend_health`
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

Internally, service calls are now routed through canonical request DTOs rather than ad hoc parameter lists. That keeps CLI, tests, and desktop code aligned on one caller-facing request shape even when the public `WalletApi` facade still offers convenience methods.

## Caller Model

Callers should treat `wallet_api` as the single integration layer.

CLI and UI code should collect user intent, call `WalletApi`, and render DTOs or `WalletApiError`. They should not duplicate wallet selection rules, parse PSBT internals, or call lower-level crates directly for normal wallet operations.

Important caller-facing DTOs include:

- `WalletReceiveAddressHistoryDto` with `address`, `keychain`, optional `index`, `bitcoin_uri`, optional `label`, `created_at`, and optional `updated_at`
- `WalletUtxoDto` with optional `derivation_index`
- `WalletPsbtDto` with optional `original_txid` and optional `replacement` lineage metadata
- `WalletBroadcastCandidateDto` for finalized transaction analysis before broadcast

Important canonical request DTOs include:

- `CreatePsbtRequestDto`
- `SendRequestDto`
- `SendMaxRequestDto`
- `SweepRequestDto`
- `ConsolidationRequestDto`
- `SignPsbtRequestDto`
- `PublishPsbtRequestDto`
- `BumpFeeRequestDto`
- `CpfpRequestDto`
- wallet-state request DTOs such as `WalletAddressRequestDto`, `WalletTransactionsRequestDto`, and `WalletUtxosRequestDto`
- receive-address request DTOs such as `WalletReceiveAddressesRequestDto`, `LabelReceiveAddressRequestDto`, and `ClearReceiveAddressLabelRequestDto`

## Test Coverage

The regtest integration suite exercises the API against a local Bitcoin Core and Electrum environment. Coverage is split across:

- `crates/wallet_api/tests/regtest_flow.rs`
- `crates/wallet_api/tests/psbt_coin_control.rs`
- `crates/wallet_api/tests/send_max.rs`
- `crates/wallet_api/tests/sweep.rs`
- `crates/wallet_api/tests/consolidation.rs`
- `crates/wallet_api/tests/rbf.rs`
- `crates/wallet_api/tests/cpfp.rs`

Together they cover wallet receive/send behavior, PSBT signing and publication, coin control, send-max, sweep, consolidation, RBF, CPFP, and invalid input handling.
