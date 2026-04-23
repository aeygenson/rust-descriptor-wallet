# Wallet Storage Overview

`wallet_storage` is the SQLite-backed wallet registry for the project.

It stores wallet metadata, descriptors, backend configuration, watch-only state, and the per-wallet BDK database path. It does not store synced transaction history or UTXO state; those live in the BDK wallet store referenced by each wallet record.

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

`repository.rs` owns SQL queries and filesystem side effects related to wallet directory creation.

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
