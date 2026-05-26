# Wallet Storage Overview

`wallet_storage` is the SQLite-backed wallet registry and lightweight wallet metadata store for the project.

It stores wallet metadata, descriptors, backend configuration, watch-only state, the per-wallet BDK database path, persisted receive-address history rows with optional labels, wallet-scoped address-book entries for external recipients, and wallet-scoped locked UTXO rows. It does not store synced transaction history or live UTXO state; those live in the BDK wallet store referenced by each wallet record.

## What The Crate Exposes

The main type is `WalletStorage` from `src/lib.rs`.

Public methods:

- `connect()`: open the default SQLite database.
- `migrate()`: apply the embedded initial schema.
- `get_wallet_by_name(name)`: fetch one wallet record.
- `list_wallets()`: list wallet records ordered by insertion id.
- `create_wallet(...)`: insert a wallet record and create its wallet directory.
- `delete_wallet(name)`: delete a wallet record.
- `import_wallet_from_file(file_path)`: read a wallet JSON file and insert it.
- `record_receive_address(...)`: insert or reload a persisted receive-address history row.
- `list_receive_addresses(wallet_name)`: list persisted receive-address history rows for a wallet.
- `get_receive_address_by_wallet_and_address(wallet_name, address)`: fetch one persisted receive-address row.
- `label_receive_address(wallet_name, address, label)`: store or update a human label.
- `clear_receive_address_label(wallet_name, address)`: remove a stored label.
- `create_address_book_entry(...)`: persist one wallet-scoped external-recipient row.
- `list_address_book_entries(wallet_name)`: list persisted address-book rows for a wallet.
- `get_address_book_entry_by_address(wallet_name, address)`: fetch one persisted address-book row.
- `delete_address_book_entry(wallet_name, address)`: remove one persisted address-book row.
- `lock_utxo(wallet_name, outpoint, reason)`: persist one wallet-scoped locked-UTXO row.
- `list_locked_utxos(wallet_name)`: list persisted locked-UTXO rows for a wallet.
- `get_locked_utxo(wallet_name, outpoint)`: fetch one persisted locked-UTXO row.
- `is_utxo_locked(wallet_name, outpoint)`: check whether one outpoint is locked.
- `unlock_utxo(wallet_name, outpoint)`: remove one persisted locked-UTXO row.
- `pool()`: expose the underlying `SqlitePool`.

The crate also re-exports repository functions for direct use where needed.

## Module Responsibilities

`db.rs` owns default paths, SQLite pool construction, and migration execution.

`models.rs` owns row models and import/export file models:

- `WalletRecord`
- `ImportWalletFile`
- `WalletDescriptorsFile`
- `WalletBackendFile`
- `SyncBackendFile`
- `BroadcastBackendFile`

`repository/wallets.rs` owns SQL queries and filesystem side effects related to wallet directory creation.

`repository/receive_history.rs` owns SQL queries for persisted receive-address history and label updates.

`repository/address_book.rs` owns SQL queries for wallet-scoped address-book rows.

`repository/locked_utxos.rs` owns SQL queries for wallet-scoped locked-UTXO rows.

`error.rs` owns `WalletStorageError`.

## Default Paths

The default application directory is:

```text
~/.rust-descriptor-wallet
```

The app registry database is:

```text
~/.rust-descriptor-wallet/app.db
```

Each wallet gets a BDK database path:

```text
~/.rust-descriptor-wallet/wallets/<wallet-name>/wallet.db
```

`create_wallet` creates the wallet directory before inserting the database record.

## Import File Format

`import_wallet_from_file` reads an `ImportWalletFile` JSON document:

```json
{
  "name": "regtest-local",
  "network": "regtest",
  "descriptors": {
    "external": "wpkh(...)",
    "internal": "wpkh(...)"
  },
  "backend": {
    "sync": {
      "kind": "electrum",
      "url": "tcp://127.0.0.1:60401"
    },
    "broadcast": {
      "kind": "rpc",
      "url": "http://127.0.0.1:18443",
      "rpc_user": "bitcoin",
      "rpc_pass": "bitcoin"
    }
  },
  "is_watch_only": false
}
```

Supported sync backends:

- `esplora`
- `electrum`

Supported broadcast backends:

- `esplora`
- `rpc`

The backend values are serialized into JSON strings before being stored in SQLite.

## Receive History Model

Persisted receive-address rows store:

- `wallet_name`
- `address`
- `keychain`
- optional `address_index`
- `bitcoin_uri`
- optional `label`
- `created_at`
- optional `updated_at`

Address history is wallet-scoped. The `(wallet_name, address)` pair is unique, so generating an already-known address reuses the existing record instead of duplicating it.

## Address Book Model

Persisted address-book rows store:

- `wallet_name`
- `network`
- `label`
- `address`
- optional `notes`
- `created_at`
- optional `updated_at`

Address-book rows are wallet-scoped. Both `(wallet_name, label)` and `(wallet_name, address)` are unique.

## Locked UTXO Model

Persisted locked-UTXO rows store:

- `wallet_name`
- `outpoint`
- optional `reason`
- `created_at`
- optional `updated_at`

Locked UTXO rows are wallet-scoped. `(wallet_name, outpoint)` is unique.

## Storage Boundary

`wallet_storage` does not validate descriptor semantics, network compatibility, wallet signing policy, or backend reachability. It stores and retrieves records.

Validation and runtime conversion happen above it in `wallet_api` and below it in `wallet_core`/`wallet_sync`.

## Error Model

`WalletStorageError` wraps:

- `sqlx::Error`
- `serde_json::Error`
- `std::io::Error`

It also defines domain storage errors:

- `HomeDirNotFound`
- `NotFound`
- `AlreadyExists`
- `InvalidBackend`
- `InvalidConfig`
- `InvalidPath`

`AlreadyExists` is produced when SQLite reports a unique constraint failure on wallet name.

For receive-address history, duplicate `(wallet_name, address)` inserts are treated as an existing-row lookup rather than a hard failure.

For address-book entries, duplicate wallet-local labels and duplicate wallet-local addresses are surfaced as explicit storage errors so callers can render a precise user-facing validation message.

For locked UTXOs, duplicate wallet-local outpoints and missing unlock targets are surfaced as explicit storage errors so callers can render deterministic coin-control feedback.
