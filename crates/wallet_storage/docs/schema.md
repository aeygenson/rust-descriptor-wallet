# Wallet Storage Schema

The current schema is embedded in `migrations/0001_init.sql` and applied by `WalletStorage::migrate`.

The schema is intentionally small: it stores wallet registry metadata only.

## Table: `wallets`

```sql
CREATE TABLE IF NOT EXISTS wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,
    network TEXT NOT NULL,
    external_descriptor TEXT NOT NULL,
    internal_descriptor TEXT NOT NULL,
    sync_backend TEXT NOT NULL,
    broadcast_backend TEXT,
    db_path TEXT NOT NULL,
    is_watch_only INTEGER NOT NULL DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT
);
```

## Columns

`id`

Autoincrementing primary key.

`name`

Unique wallet name used by `wallet_api` and callers.

`network`

Stored as text. Current expected values are `bitcoin`, `testnet`, `signet`, and `regtest`.

`external_descriptor`

External receive descriptor.

`internal_descriptor`

Internal change descriptor.

`sync_backend`

JSON string serialized from `SyncBackendFile`.

Examples:

```json
{"kind":"electrum","url":"tcp://127.0.0.1:60401"}
```

```json
{"kind":"esplora","url":"https://example.invalid"}
```

`broadcast_backend`

Optional JSON string serialized from `BroadcastBackendFile`.

Examples:

```json
{"kind":"rpc","url":"http://127.0.0.1:18443","rpc_user":"bitcoin","rpc_pass":"bitcoin"}
```

```json
{"kind":"esplora","url":"https://example.invalid"}
```

`db_path`

Filesystem path to the per-wallet BDK database file, normally:

```text
~/.rust-descriptor-wallet/wallets/<wallet-name>/wallet.db
```

`is_watch_only`

Stored as SQLite integer boolean through sqlx. `0` means false and `1` means true.

`created_at`

Text timestamp defaulted by SQLite with `CURRENT_TIMESTAMP`.

`updated_at`

Nullable text timestamp. The current code does not update this field yet.

## Indexes

The migration creates:

```sql
CREATE INDEX IF NOT EXISTS idx_wallets_name ON wallets(name);
```

The `name` column is already unique, but the explicit index is kept by the initial migration.

## Repository Queries

`get_wallet_by_name` selects all wallet columns by `name` and returns `WalletStorageError::NotFound` when absent.

`list_wallets` selects all wallet columns ordered by `id ASC`.

`create_wallet` inserts the metadata row and maps unique-name database failures to `WalletStorageError::AlreadyExists`.

`delete_wallet` deletes by name and returns `WalletStorageError::NotFound` when no row is affected.

`import_wallet_from_file` deserializes `ImportWalletFile`, serializes backend config fields into JSON strings, and delegates to `create_wallet`.

## Not Stored Here

The schema does not store:

- transaction history
- UTXO set
- sync checkpoints
- address labels
- transaction labels
- signing metadata
- raw PSBTs

Runtime wallet state is managed by the BDK wallet store at `db_path`.
