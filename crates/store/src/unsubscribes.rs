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

pub const LINK_OPENED: &str = "link_opened";
pub const COMPLETED: &str = "completed";

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct PendingLink {
    pub from_addr: String,
    pub name: Option<String>,
    pub url: Option<String>,
    pub at: i64,
}

pub fn pending_links(conn: &Connection, account_id: i64) -> Result<Vec<PendingLink>> {
    let mut stmt = conn.prepare_cached(
        "SELECT u.from_addr, s.from_name, COALESCE(u.detail, s.unsub_http), u.at FROM unsubscribes u \
         LEFT JOIN senders s ON s.account_id = u.account_id AND s.from_addr = u.from_addr \
         WHERE u.account_id = ?1 AND u.status = ?2 ORDER BY u.at DESC",
    )?;
    let rows = stmt
        .query_map(params![account_id, LINK_OPENED], |row| {
            Ok(PendingLink {
                from_addr: row.get(0)?,
                name: row.get(1)?,
                url: row.get(2)?,
                at: row.get(3)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn mark_link_done(conn: &Connection, account_id: i64, from_addr: &str) -> Result<bool> {
    let changed = conn.execute(
        "UPDATE unsubscribes SET status = ?3, detail = 'finished on the sender site', at = ?4 \
         WHERE account_id = ?1 AND from_addr = ?2 AND status = ?5",
        params![account_id, from_addr, COMPLETED, now(), LINK_OPENED],
    )?;
    Ok(changed > 0)
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[test]
    fn lists_and_completes_pending_links() {
        let store = test_support::store();
        let account = test_support::account(&store);
        store
            .write(|tx| {
                record(
                    tx,
                    account,
                    "a@news.com",
                    "link",
                    LINK_OPENED,
                    Some("https://news.com/u"),
                )?;
                record(tx, account, "b@news.com", "one_click", COMPLETED, None)
            })
            .unwrap();
        let pending = store.read(|c| pending_links(c, account)).unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].url.as_deref(), Some("https://news.com/u"));
        assert!(
            store
                .write(|tx| mark_link_done(tx, account, "a@news.com"))
                .unwrap()
        );
        assert!(
            !store
                .write(|tx| mark_link_done(tx, account, "a@news.com"))
                .unwrap()
        );
        assert!(
            store
                .read(|c| pending_links(c, account))
                .unwrap()
                .is_empty()
        );
    }
}
