use rusqlite::{Connection, OptionalExtension, params};
use serde::Serialize;

use crate::{Result, now};

#[derive(Debug, Clone, Serialize)]
pub struct Unsubscribe {
    pub from_addr: String,
    pub method: String,
    pub status: String,
    pub detail: Option<String>,
    pub at: i64,
}

pub fn record(
    conn: &Connection,
    account_id: i64,
    from_addr: &str,
    method: &str,
    status: &str,
    detail: Option<&str>,
) -> Result<()> {
    conn.execute(
        "INSERT INTO unsubscribes (account_id, from_addr, method, status, detail, at) VALUES (?1, ?2, ?3, ?4, ?5, ?6) \
         ON CONFLICT (account_id, from_addr) DO UPDATE SET \
         method = excluded.method, status = excluded.status, detail = excluded.detail, at = excluded.at",
        params![account_id, from_addr, method, status, detail, now()],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, account_id: i64, from_addr: &str) -> Result<Option<Unsubscribe>> {
    Ok(conn
        .query_row(
            "SELECT from_addr, method, status, detail, at FROM unsubscribes WHERE account_id = ?1 AND from_addr = ?2",
            params![account_id, from_addr],
            |row| {
                Ok(Unsubscribe {
                    from_addr: row.get(0)?,
                    method: row.get(1)?,
                    status: row.get(2)?,
                    detail: row.get(3)?,
                    at: row.get(4)?,
                })
            },
        )
        .optional()?)
}

pub struct SenderUnsubscribe {
    pub http: Option<String>,
    pub mailto: Option<String>,
    pub one_click: bool,
}

pub fn sender_options(
    conn: &Connection,
    account_id: i64,
    from_addr: &str,
) -> Result<Option<SenderUnsubscribe>> {
    Ok(conn
        .query_row(
            "SELECT unsub_http, unsub_mailto, unsub_one_click FROM senders WHERE account_id = ?1 AND from_addr = ?2",
            params![account_id, from_addr],
            |row| Ok(SenderUnsubscribe { http: row.get(0)?, mailto: row.get(1)?, one_click: row.get(2)? }),
        )
        .optional()?)
}
