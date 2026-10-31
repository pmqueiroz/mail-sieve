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

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct StillSending {
    pub from_addr: String,
    pub name: Option<String>,
    pub method: String,
    pub unsubscribed_at: i64,
    pub since_count: i64,
    pub last_date: i64,
}

pub fn still_sending(
    conn: &Connection,
    account_id: i64,
    grace_seconds: i64,
) -> Result<Vec<StillSending>> {
    let mut stmt = conn.prepare_cached(
        "SELECT u.from_addr, s.from_name, u.method, u.at, COUNT(*), MAX(m.date) FROM unsubscribes u \
         JOIN messages m ON m.account_id = u.account_id AND m.from_addr = u.from_addr AND m.date > u.at + ?3 \
         LEFT JOIN senders s ON s.account_id = u.account_id AND s.from_addr = u.from_addr \
         WHERE u.account_id = ?1 AND u.status = ?2 \
         GROUP BY u.from_addr ORDER BY COUNT(*) DESC, u.from_addr",
    )?;
    let rows = stmt
        .query_map(params![account_id, COMPLETED, grace_seconds], |row| {
            Ok(StillSending {
                from_addr: row.get(0)?,
                name: row.get(1)?,
                method: row.get(2)?,
                unsubscribed_at: row.get(3)?,
                since_count: row.get(4)?,
                last_date: row.get(5)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
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

    #[test]
    fn finds_senders_that_keep_mailing_after_the_grace_period() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let inbox = test_support::mailbox(&store, account, "INBOX");
        let unsubscribed_at = now();
        let grace = 2 * 86_400;
        let message = |uid: u32, from: &str, date: i64| mail_sieve_core::MessageMeta {
            uid,
            from: Some(mail_sieve_core::Address {
                name: None,
                email: from.into(),
            }),
            date: Some(date),
            ..Default::default()
        };
        store
            .write(|tx| {
                crate::messages::upsert_batch(
                    tx,
                    account,
                    inbox,
                    &[
                        message(1, "a@news.com", unsubscribed_at - 86_400),
                        message(2, "a@news.com", unsubscribed_at + 86_400),
                        message(3, "a@news.com", unsubscribed_at + 3 * 86_400),
                        message(4, "a@news.com", unsubscribed_at + 4 * 86_400),
                        message(5, "b@news.com", unsubscribed_at + 5 * 86_400),
                        message(6, "c@news.com", unsubscribed_at + 5 * 86_400),
                    ],
                )?;
                record(tx, account, "a@news.com", "one_click", COMPLETED, None)?;
                record(tx, account, "b@news.com", "link", LINK_OPENED, None)?;
                record(tx, account, "c@news.com", "mailto", "failed", None)
            })
            .unwrap();
        let found = store.read(|c| still_sending(c, account, grace)).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].from_addr, "a@news.com");
        assert_eq!(found[0].since_count, 2);
        assert_eq!(found[0].last_date, unsubscribed_at + 4 * 86_400);
    }
}
