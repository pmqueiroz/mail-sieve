use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::{Error, Result, now};

#[derive(Debug, Clone, Serialize)]
pub struct Mailbox {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub delimiter: Option<String>,
    pub special_use: Option<String>,
    pub sync_enabled: bool,
    pub uidvalidity: Option<u32>,
    pub uidnext: Option<u32>,
    pub highestmodseq: Option<u64>,
    pub last_synced_at: Option<i64>,
    pub message_count: u64,
}

#[derive(Debug, Clone)]
pub struct ListedMailbox {
    pub name: String,
    pub delimiter: Option<String>,
    pub special_use: Option<String>,
    pub default_sync: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SyncState {
    pub uidvalidity: u32,
    pub uidnext: u32,
    pub highestmodseq: Option<u64>,
}

const COLUMNS: &str = "m.id, m.account_id, m.name, m.delimiter, m.special_use, m.sync_enabled, \
    m.uidvalidity, m.uidnext, m.highestmodseq, m.last_synced_at, \
    (SELECT COUNT(*) FROM messages WHERE mailbox_id = m.id)";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Mailbox> {
    let highestmodseq: Option<i64> = row.get(8)?;
    Ok(Mailbox {
        id: row.get(0)?,
        account_id: row.get(1)?,
        name: row.get(2)?,
        delimiter: row.get(3)?,
        special_use: row.get(4)?,
        sync_enabled: row.get(5)?,
        uidvalidity: row.get(6)?,
        uidnext: row.get(7)?,
        highestmodseq: highestmodseq.map(|value| value as u64),
        last_synced_at: row.get(9)?,
        message_count: row.get::<_, i64>(10)? as u64,
    })
}

pub fn sync_listing(conn: &Connection, account_id: i64, listed: &[ListedMailbox]) -> Result<()> {
    let mut upsert = conn.prepare_cached(
        "INSERT INTO mailboxes (account_id, name, delimiter, special_use, sync_enabled) \
         VALUES (?1, ?2, ?3, ?4, ?5) \
         ON CONFLICT (account_id, name) DO UPDATE SET \
         delimiter = excluded.delimiter, special_use = excluded.special_use",
    )?;
    for mailbox in listed {
        upsert.execute(params![
            account_id,
            mailbox.name,
            mailbox.delimiter,
            mailbox.special_use,
            mailbox.default_sync
        ])?;
    }
    let existing = list(conn, account_id)?;
    for mailbox in existing {
        if !listed.iter().any(|listed| listed.name == mailbox.name) {
            conn.execute("DELETE FROM mailboxes WHERE id = ?1", [mailbox.id])?;
        }
    }
    Ok(())
}

pub fn list(conn: &Connection, account_id: i64) -> Result<Vec<Mailbox>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM mailboxes m WHERE m.account_id = ?1 ORDER BY m.name"
    ))?;
    let rows = stmt
        .query_map([account_id], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn get(conn: &Connection, id: i64) -> Result<Mailbox> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM mailboxes m WHERE m.id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(Error::NotFound)
}

pub fn find_by_name(conn: &Connection, account_id: i64, name: &str) -> Result<Option<Mailbox>> {
    Ok(conn
        .query_row(
            &format!("SELECT {COLUMNS} FROM mailboxes m WHERE m.account_id = ?1 AND m.name = ?2"),
            params![account_id, name],
            from_row,
        )
        .optional()?)
}

pub fn find_by_special_use(
    conn: &Connection,
    account_id: i64,
    special_use: &str,
) -> Result<Option<Mailbox>> {
    Ok(conn
        .query_row(
            &format!(
                "SELECT {COLUMNS} FROM mailboxes m WHERE m.account_id = ?1 AND m.special_use = ?2 \
                 ORDER BY m.id LIMIT 1"
            ),
            params![account_id, special_use],
            from_row,
        )
        .optional()?)
}

pub fn set_sync_enabled(conn: &Connection, id: i64, enabled: bool) -> Result<()> {
    match conn.execute(
        "UPDATE mailboxes SET sync_enabled = ?2 WHERE id = ?1",
        params![id, enabled],
    )? {
        0 => Err(Error::NotFound),
        _ => Ok(()),
    }
}

pub fn reset(conn: &Connection, id: i64) -> Result<()> {
    conn.execute("DELETE FROM messages WHERE mailbox_id = ?1", [id])?;
    conn.execute(
        "UPDATE mailboxes SET uidvalidity = NULL, uidnext = NULL, highestmodseq = NULL WHERE id = ?1",
        [id],
    )?;
    Ok(())
}

pub fn save_sync_state(conn: &Connection, id: i64, state: SyncState) -> Result<()> {
    conn.execute(
        "UPDATE mailboxes SET uidvalidity = ?2, uidnext = ?3, highestmodseq = ?4, last_synced_at = ?5 \
         WHERE id = ?1",
        params![id, state.uidvalidity, state.uidnext, state.highestmodseq.map(|v| v as i64), now()],
    )?;
    Ok(())
}
