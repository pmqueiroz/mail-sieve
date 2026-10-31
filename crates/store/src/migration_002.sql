ALTER TABLE action_log ADD COLUMN bytes INTEGER NOT NULL DEFAULT 0;

CREATE INDEX action_log_account_at ON action_log (account_id, at);

CREATE TABLE job_undo (
    job_id INTEGER NOT NULL REFERENCES jobs(id) ON DELETE CASCADE,
    mailbox_id INTEGER NOT NULL,
    uid INTEGER NOT NULL,
    message_id TEXT,
    PRIMARY KEY (job_id, mailbox_id, uid)
) WITHOUT ROWID;
