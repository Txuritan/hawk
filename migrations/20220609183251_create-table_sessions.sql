CREATE TABLE IF NOT EXISTS sessions (
    id TEXT NOT NULL,
    token TEXT NOT NULL,
    created DATETIME DEFAULT (DATETIME('now')),
    PRIMARY KEY (id, token)
);