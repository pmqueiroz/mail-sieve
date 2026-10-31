CREATE TABLE pinned_senders (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('sender', 'domain')),
    key TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    PRIMARY KEY (account_id, kind, key)
) WITHOUT ROWID;
