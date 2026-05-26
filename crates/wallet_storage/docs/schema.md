# Wallet Storage Schema

The current schema is embedded in `migrations/0001_init.sql`, `migrations/0002_receive_history.sql`, `migrations/0003_address_book.sql`, and `migrations/0004_locked_utxos.sql`, and is applied by `WalletStorage::migrate`.

The schema is intentionally small: it stores wallet registry metadata plus persisted receive-address history, wallet-scoped address-book entries, and wallet-scoped locked-UTXO rows.

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

## Table: `receive_address_history`

```sql
CREATE TABLE IF NOT EXISTS receive_address_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_name TEXT NOT NULL,
    address TEXT NOT NULL,
    keychain TEXT NOT NULL,
    address_index INTEGER,
    bitcoin_uri TEXT NOT NULL,
    label TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT,

    FOREIGN KEY(wallet_name)
        REFERENCES wallets(name)
        ON DELETE CASCADE
);
```

## Columns

`wallet_name`

Owning wallet record name. This is a foreign key to `wallets(name)`.

`address`

Persisted encoded receive address string.

`keychain`

Current string keychain name returned by runtime derivation, typically `external` for receive flow.

`address_index`

Nullable derivation index returned by wallet derivation.

`bitcoin_uri`

Persisted `bitcoin:<address>` URI string returned to CLI/UI callers.

`label`

Nullable human label managed by caller-facing receive-address workflows.

`created_at`

Timestamp for initial persistence.

`updated_at`

Nullable timestamp updated when the receive-address label changes.

## Table: `locked_utxos`

```sql
CREATE TABLE IF NOT EXISTS locked_utxos (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_name TEXT NOT NULL,
    outpoint TEXT NOT NULL,
    reason TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT,

    CONSTRAINT fk_locked_utxos_wallet
        FOREIGN KEY (wallet_name)
        REFERENCES wallets(name)
        ON DELETE CASCADE,

    CONSTRAINT uq_locked_utxos_wallet_outpoint
        UNIQUE (wallet_name, outpoint)
);
```

## Columns

`wallet_name`

Owning wallet record name. This is a foreign key to `wallets(name)`.

`outpoint`

Persisted `<txid>:<vout>` string identifying the locked coin.

`reason`

Nullable operator-supplied lock reason.

`created_at`

Timestamp for initial lock persistence.

`updated_at`

Nullable timestamp reserved for future lock-row updates.

## Indexes

The migration creates:

```sql
CREATE INDEX IF NOT EXISTS idx_wallets_name ON wallets(name);
```

The `name` column is already unique, but the explicit index is kept by the initial migration.

The receive-history migration adds:

```sql
CREATE INDEX IF NOT EXISTS idx_receive_address_history_wallet_name
    ON receive_address_history(wallet_name);

CREATE UNIQUE INDEX IF NOT EXISTS idx_receive_address_history_wallet_address
    ON receive_address_history(wallet_name, address);
```

The locked-UTXO migration adds:

```sql
CREATE INDEX IF NOT EXISTS idx_locked_utxos_wallet_name
    ON locked_utxos(wallet_name);

CREATE INDEX IF NOT EXISTS idx_locked_utxos_outpoint
    ON locked_utxos(outpoint);
```

## Table: `address_book_entries`

```sql
CREATE TABLE IF NOT EXISTS address_book_entries (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    wallet_name TEXT NOT NULL,
    network TEXT NOT NULL,
    label TEXT NOT NULL,
    address TEXT NOT NULL,
    notes TEXT,
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT,

    CONSTRAINT fk_address_book_wallet
        FOREIGN KEY (wallet_name)
        REFERENCES wallets(name)
        ON DELETE CASCADE,

    CONSTRAINT uq_address_book_wallet_address
        UNIQUE (wallet_name, address),

    CONSTRAINT uq_address_book_wallet_label
        UNIQUE (wallet_name, label)
);
```

## Columns

`wallet_name`

Owning wallet record name. This is a foreign key to `wallets(name)`.

`network`

Text copy of the wallet network at creation time. This lets callers render recipient network metadata without reloading the wallet record.

`label`

Wallet-scoped recipient label. `(wallet_name, label)` is unique.

`address`

Persisted external destination address string. `(wallet_name, address)` is unique.

`notes`

Nullable caller-supplied free text.

`created_at`

Timestamp for initial persistence.

`updated_at`

Nullable timestamp reserved for later row updates. The current address-book flow only creates and deletes entries.

The address-book migration adds:

```sql
CREATE INDEX IF NOT EXISTS idx_address_book_wallet_name
    ON address_book_entries(wallet_name);

CREATE INDEX IF NOT EXISTS idx_address_book_network
    ON address_book_entries(network);

CREATE INDEX IF NOT EXISTS idx_address_book_label
    ON address_book_entries(label);
```

## Repository Queries

`get_wallet_by_name` selects all wallet columns by `name` and returns `WalletStorageError::NotFound` when absent.

`list_wallets` selects all wallet columns ordered by `id ASC`.

`create_wallet` inserts the metadata row and maps unique-name database failures to `WalletStorageError::AlreadyExists`.

`delete_wallet` deletes by name and returns `WalletStorageError::NotFound` when no row is affected.

`import_wallet_from_file` deserializes `ImportWalletFile`, serializes backend config fields into JSON strings, and delegates to `create_wallet`.

## Receive History Queries

`record_receive_address` inserts a receive-history row. If the wallet/address pair already exists, the repository returns the existing row instead of duplicating it.

`list_receive_addresses` selects all rows for a wallet ordered by `created_at DESC`.

`label_receive_address` updates `label` and `updated_at`, and returns `WalletStorageError::NotFound` when the row is missing.

`clear_receive_address_label` sets `label = NULL`, updates `updated_at`, and returns `WalletStorageError::NotFound` when the row is missing.

`get_receive_address_by_wallet_and_address` selects one row by `(wallet_name, address)`.

## Address Book Queries

`create_address_book_entry` inserts one wallet-scoped recipient row and maps duplicate wallet-local labels and addresses into dedicated storage errors.

`list_address_book_entries` selects all address-book rows for a wallet ordered by `label ASC`.

`get_address_book_entry_by_address` selects one row by `(wallet_name, address)`.

`delete_address_book_entry` deletes one row by `(wallet_name, address)` and returns `false` when no row matched.

## Locked UTXO Queries

`lock_utxo` inserts one wallet-scoped lock row and maps duplicate `(wallet_name, outpoint)` conflicts into a dedicated storage error.

`list_locked_utxos` selects all lock rows for a wallet ordered by `created_at DESC`.

`get_locked_utxo` selects one row by `(wallet_name, outpoint)`.

`is_utxo_locked` resolves to `true` only when that wallet-scoped outpoint row exists.

`unlock_utxo` deletes one row by `(wallet_name, outpoint)` and returns `false` when no row matched.

## Not Stored Here

The schema does not store:

- transaction history
- UTXO set
- sync checkpoints
- transaction labels
- signing metadata
- raw PSBTs

Receive-address labels and address-book rows are stored here, but broader transaction-annotation systems are not.

Runtime wallet state is managed by the BDK wallet store at `db_path`.
