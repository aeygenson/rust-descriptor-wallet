

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

CREATE INDEX IF NOT EXISTS idx_receive_address_history_wallet_name
    ON receive_address_history(wallet_name);

CREATE UNIQUE INDEX IF NOT EXISTS idx_receive_address_history_wallet_address
    ON receive_address_history(wallet_name, address);