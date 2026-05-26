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

CREATE INDEX IF NOT EXISTS idx_locked_utxos_wallet_name
    ON locked_utxos(wallet_name);

CREATE INDEX IF NOT EXISTS idx_locked_utxos_outpoint
    ON locked_utxos(outpoint);
