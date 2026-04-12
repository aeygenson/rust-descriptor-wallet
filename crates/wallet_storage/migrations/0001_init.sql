CREATE TABLE IF NOT EXISTS wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL UNIQUE,

    -- network (bitcoin, testnet, signet, regtest)
    network TEXT NOT NULL,

    -- descriptors
    external_descriptor TEXT NOT NULL,
    internal_descriptor TEXT NOT NULL,

    -- backend configuration (JSON)
    sync_backend TEXT NOT NULL,
    broadcast_backend TEXT,

    -- storage
    db_path TEXT NOT NULL,

    -- flags
    is_watch_only INTEGER NOT NULL DEFAULT 0,

    -- metadata
    created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP,
    updated_at TEXT
);

-- Optional: future-proof index for fast lookup
CREATE INDEX IF NOT EXISTS idx_wallets_name ON wallets(name);