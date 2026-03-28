CREATE TABLE IF NOT EXISTS wallets (
                                       id INTEGER PRIMARY KEY AUTOINCREMENT,
                                       name TEXT NOT NULL UNIQUE,
                                       network TEXT NOT NULL,
                                       external_descriptor TEXT NOT NULL,
                                       internal_descriptor TEXT NOT NULL,
                                       db_path TEXT NOT NULL,
                                       esplora_url TEXT NOT NULL,
                                       is_watch_only INTEGER NOT NULL DEFAULT 1,
                                       created_at TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
);