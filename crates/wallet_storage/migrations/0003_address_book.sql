

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

CREATE INDEX IF NOT EXISTS idx_address_book_wallet_name
    ON address_book_entries(wallet_name);

CREATE INDEX IF NOT EXISTS idx_address_book_network
    ON address_book_entries(network);

CREATE INDEX IF NOT EXISTS idx_address_book_label
    ON address_book_entries(label);