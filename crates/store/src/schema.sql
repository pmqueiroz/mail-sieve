CREATE TABLE settings (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

CREATE TABLE accounts (
    id INTEGER PRIMARY KEY,
    label TEXT NOT NULL,
    email TEXT NOT NULL,
    imap_host TEXT NOT NULL,
    imap_port INTEGER NOT NULL,
    imap_tls INTEGER NOT NULL DEFAULT 1,
    smtp_host TEXT,
    smtp_port INTEGER,
    username TEXT NOT NULL,
    auth_kind TEXT NOT NULL,
    oauth_provider TEXT,
    credentials BLOB NOT NULL,
    provider TEXT NOT NULL DEFAULT 'generic',
    max_connections INTEGER NOT NULL DEFAULT 4,
    capabilities TEXT NOT NULL DEFAULT '',
    sync_interval_minutes INTEGER NOT NULL DEFAULT 15,
    created_at INTEGER NOT NULL,
    last_synced_at INTEGER,
    last_error TEXT
);

CREATE TABLE mailboxes (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    delimiter TEXT,
    special_use TEXT,
    sync_enabled INTEGER NOT NULL DEFAULT 1,
    uidvalidity INTEGER,
    uidnext INTEGER,
    highestmodseq INTEGER,
    last_synced_at INTEGER,
    UNIQUE (account_id, name)
);

CREATE TABLE messages (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL,
    mailbox_id INTEGER NOT NULL REFERENCES mailboxes(id) ON DELETE CASCADE,
    uid INTEGER NOT NULL,
    message_id TEXT,
    from_addr TEXT NOT NULL DEFAULT '',
    from_name TEXT,
    from_domain TEXT NOT NULL DEFAULT '',
    subject TEXT,
    date INTEGER,
    size INTEGER NOT NULL DEFAULT 0,
    flags INTEGER NOT NULL DEFAULT 0,
    list_id TEXT,
    unsub_http TEXT,
    unsub_mailto TEXT,
    unsub_one_click INTEGER NOT NULL DEFAULT 0,
    UNIQUE (mailbox_id, uid)
);

CREATE INDEX messages_account_from ON messages (account_id, from_addr);
CREATE INDEX messages_account_domain ON messages (account_id, from_domain);
CREATE INDEX messages_account_list ON messages (account_id, list_id) WHERE list_id IS NOT NULL;
CREATE INDEX messages_account_date ON messages (account_id, date DESC, id DESC);

CREATE VIRTUAL TABLE messages_fts USING fts5 (
    subject, from_name, from_addr,
    content = 'messages', content_rowid = 'id',
    tokenize = 'unicode61 remove_diacritics 2',
    prefix = '2 3'
);

CREATE TABLE senders (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    from_addr TEXT NOT NULL,
    from_name TEXT,
    from_domain TEXT NOT NULL,
    message_count INTEGER NOT NULL DEFAULT 0,
    unread_count INTEGER NOT NULL DEFAULT 0,
    total_size INTEGER NOT NULL DEFAULT 0,
    last_date INTEGER,
    list_id TEXT,
    unsub_http TEXT,
    unsub_mailto TEXT,
    unsub_one_click INTEGER NOT NULL DEFAULT 0,
    PRIMARY KEY (account_id, from_addr)
) WITHOUT ROWID;

CREATE INDEX senders_account_count ON senders (account_id, message_count DESC);

CREATE TRIGGER messages_after_insert AFTER INSERT ON messages BEGIN
    INSERT INTO senders (
        account_id, from_addr, from_name, from_domain, message_count, unread_count,
        total_size, last_date, list_id, unsub_http, unsub_mailto, unsub_one_click
    ) VALUES (
        new.account_id, new.from_addr, new.from_name, new.from_domain, 1, (new.flags & 1) = 0,
        new.size, new.date, new.list_id, new.unsub_http, new.unsub_mailto, new.unsub_one_click
    )
    ON CONFLICT (account_id, from_addr) DO UPDATE SET
        message_count = message_count + 1,
        unread_count = unread_count + excluded.unread_count,
        total_size = total_size + excluded.total_size,
        from_name = CASE
            WHEN excluded.last_date >= COALESCE(last_date, 0) THEN COALESCE(excluded.from_name, from_name)
            ELSE COALESCE(from_name, excluded.from_name)
        END,
        last_date = MAX(COALESCE(last_date, excluded.last_date), COALESCE(excluded.last_date, last_date)),
        list_id = COALESCE(excluded.list_id, list_id),
        unsub_http = COALESCE(excluded.unsub_http, unsub_http),
        unsub_mailto = COALESCE(excluded.unsub_mailto, unsub_mailto),
        unsub_one_click = MAX(unsub_one_click, excluded.unsub_one_click);
    INSERT INTO messages_fts (rowid, subject, from_name, from_addr)
    VALUES (new.id, new.subject, new.from_name, new.from_addr);
END;

CREATE TRIGGER messages_after_delete AFTER DELETE ON messages BEGIN
    UPDATE senders SET
        message_count = message_count - 1,
        unread_count = unread_count - ((old.flags & 1) = 0),
        total_size = total_size - old.size
    WHERE account_id = old.account_id AND from_addr = old.from_addr;
    DELETE FROM senders
    WHERE account_id = old.account_id AND from_addr = old.from_addr AND message_count <= 0;
    INSERT INTO messages_fts (messages_fts, rowid, subject, from_name, from_addr)
    VALUES ('delete', old.id, old.subject, old.from_name, old.from_addr);
END;

CREATE TRIGGER messages_after_flags AFTER UPDATE OF flags ON messages
WHEN (old.flags & 1) != (new.flags & 1) BEGIN
    UPDATE senders SET
        unread_count = unread_count + ((new.flags & 1) = 0) - ((old.flags & 1) = 0)
    WHERE account_id = new.account_id AND from_addr = new.from_addr;
END;

CREATE TABLE jobs (
    id INTEGER PRIMARY KEY,
    account_id INTEGER REFERENCES accounts(id) ON DELETE CASCADE,
    kind TEXT NOT NULL,
    state TEXT NOT NULL,
    payload TEXT NOT NULL DEFAULT '{}',
    total INTEGER NOT NULL DEFAULT 0,
    done INTEGER NOT NULL DEFAULT 0,
    error TEXT,
    created_at INTEGER NOT NULL,
    started_at INTEGER,
    finished_at INTEGER
);

CREATE INDEX jobs_state ON jobs (state, id);

CREATE TABLE job_targets (
    job_id INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    mailbox_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    PRIMARY KEY (job_id, mailbox_id, uid)
) WITHOUT ROWID;

CREATE TABLE rules (
    id INTEGER PRIMARY KEY,
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name TEXT NOT NULL,
    query TEXT NOT NULL,
    action TEXT NOT NULL,
    enabled INTEGER NOT NULL DEFAULT 1,
    interval_minutes INTEGER,
    run_after_sync INTEGER NOT NULL DEFAULT 0,
    last_run_at INTEGER,
    created_at INTEGER NOT NULL
);

CREATE TABLE action_log (
    id INTEGER PRIMARY KEY,
    job_id INTEGER,
    account_id INTEGER NOT NULL,
    action TEXT NOT NULL,
    mailbox TEXT NOT NULL,
    uids TEXT NOT NULL,
    count INTEGER NOT NULL,
    at INTEGER NOT NULL
);

CREATE TABLE unsubscribes (
    account_id INTEGER NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    from_addr TEXT NOT NULL,
    method TEXT NOT NULL,
    status TEXT NOT NULL,
    detail TEXT,
    at INTEGER NOT NULL,
    PRIMARY KEY (account_id, from_addr)
) WITHOUT ROWID;

CREATE TABLE sessions (
    token_hash TEXT PRIMARY KEY,
    created_at INTEGER NOT NULL,
    expires_at INTEGER NOT NULL
) WITHOUT ROWID;
