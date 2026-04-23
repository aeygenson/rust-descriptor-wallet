# Wallet Core Overview

`wallet_core` is the domain engine of the wallet workspace.

It owns descriptor-backed runtime wallet behavior, typed domain models, transaction construction, PSBT signing/finalization, UTXO and transaction inspection, coin-control policy, send-max, sweep, consolidation, RBF, and CPFP.

## Public Boundary

The crate exports:

- `WalletConfig`
- `WalletCore`
- `WalletCoreError`
- `WalletService`
- `WalletKeychain`
- `WalletCoreResult<T>`

Most real wallet operations are methods on `WalletService`.

`WalletCore` currently provides lightweight domain helpers such as descriptor signing classification and PSBT signing-status classification.

## Main Modules

Top-level modules:

- `config.rs`: wallet descriptors, backend config, database path, watch-only flag.
- `core.rs`: small core helper facade.
- `error.rs`: `WalletCoreError` domain failures.
- `model.rs`: wallet transaction, UTXO, PSBT, coin-control, consolidation, signing, and CPFP domain models.
- `types.rs`: strongly typed wrappers for amounts, fee rates, txids, outpoints, PSBT base64, transaction hex, virtual size, block height, percentages, keychains, and transaction direction.
- `service/`: runtime wallet operations.

Service modules:

- `lifecycle.rs`: load/create persisted BDK wallet, derive addresses, read balance, persist state.
- `txs.rs`: transaction history model conversion.
- `utxos.rs`: UTXO model conversion and txid-filtered UTXO lookup.
- `psbt_create.rs`: fixed send, send-max, sweep, coin-control-aware PSBT construction.
- `psbt_consolidation.rs`: wallet-internal consolidation PSBT construction.
- `psbt_rbf.rs`: fee-bump replacement PSBT construction.
- `psbt_cpfp.rs`: CPFP child PSBT construction.
- `psbt_sign.rs`: software-wallet PSBT signing.
- `psbt_publish.rs`: finalized PSBT extraction into broadcast-ready raw transaction hex.
- `psbt_coin_control.rs`: include/exclude validation against wallet UTXOs.
- `psbt_coin_selector.rs`: typed candidate selection for manual and consolidation flows.
- `common_*`: shared helpers for outpoints, selection, transactions, and tests.

## What Core Owns

`wallet_core` owns:

- wallet loading and BDK file-store persistence
- receive-address derivation
- balance, transaction, and UTXO inspection from local wallet state
- destination address and network validation during PSBT creation
- positive amount and fee-rate validation
- selected input validation
- transaction builder setup
- fee, change, selected input, output count, replaceability, and vsize summary fields
- PSBT signing and finalized transaction extraction
- RBF and CPFP eligibility checks

## What Core Does Not Own

`wallet_core` does not own:

- CLI argument parsing
- GUI behavior
- API DTO formatting
- wallet registry storage
- chain sync orchestration
- transaction broadcast transport

Those responsibilities live in `wallet_api`, `wallet_storage`, and `wallet_sync`.

## Parsing Boundary

The core uses typed values internally, but it also contains parsing helpers in `types.rs` and `service/common_outpoint.rs`.

These helpers are domain-level validation tools. Higher layers should still prefer converting user input at the API boundary before invoking core operations.

## Testing Model

Unit tests in `wallet_core` validate typed conversions, selection helpers, transaction summaries, PSBT signing/finalization, RBF helpers, CPFP planning, and consolidation constraints.

End-to-end behavior with real regtest infrastructure is covered from `wallet_api` integration tests, which call into `wallet_core` through the production API boundary.
