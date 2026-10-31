use rusqlite::{Connection, params};
use serde::Serialize;

use crate::Result;
use crate::search::category_sql;

const DAY: i64 = 86_400;
const WEEK: i64 = 7 * DAY;
const YEAR: i64 = 365 * DAY;
const SEEN: i64 = 1;

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct NeverOpened {
    pub senders: u64,
    pub messages: u64,
    pub bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct CategoryCount {
    pub category: String,
    pub count: u64,
}

#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize)]
pub struct Overview {
    pub total: u64,
    pub total_bytes: u64,
    pub unread: u64,
    pub unread_week: u64,
    pub list_messages: u64,
    pub sender_count: u64,
    pub never_opened: NeverOpened,
    pub freed_week_bytes: u64,
    pub categories: Vec<CategoryCount>,
}

pub fn overview(conn: &Connection, account_id: i64, now: i64) -> Result<Overview> {
    let (total, total_bytes, unread, unread_week, list_messages) = conn.query_row(
        &format!(
            "SELECT COUNT(*), COALESCE(SUM(size), 0), \
             COALESCE(SUM((flags & {SEEN}) = 0), 0), \
             COALESCE(SUM((flags & {SEEN}) = 0 AND date >= ?2), 0), \
             COALESCE(SUM(list_id IS NOT NULL), 0) \
             FROM messages WHERE account_id = ?1"
        ),
        params![account_id, now - WEEK],
        |row| {
            Ok((
                row.get::<_, i64>(0)?,
                row.get::<_, i64>(1)?,
                row.get::<_, i64>(2)?,
                row.get::<_, i64>(3)?,
                row.get::<_, i64>(4)?,
            ))
        },
    )?;
    let (sender_count, never_opened) = conn.query_row(
        "SELECT COUNT(*), \
         COALESCE(SUM(unread_count = message_count), 0), \
         COALESCE(SUM(CASE WHEN unread_count = message_count THEN message_count END), 0), \
         COALESCE(SUM(CASE WHEN unread_count = message_count THEN total_size END), 0) \
         FROM senders WHERE account_id = ?1 AND message_count > 0",
        [account_id],
        |row| {
            Ok((
                row.get::<_, i64>(0)? as u64,
                NeverOpened {
                    senders: row.get::<_, i64>(1)? as u64,
                    messages: row.get::<_, i64>(2)? as u64,
                    bytes: row.get::<_, i64>(3)? as u64,
                },
            ))
        },
    )?;
    let freed_week_bytes: i64 = conn.query_row(
        "SELECT COALESCE(SUM(bytes), 0) FROM action_log WHERE account_id = ?1 AND at >= ?2",
        params![account_id, now - WEEK],
        |row| row.get(0),
    )?;
    let mut stmt = conn.prepare(&format!(
        "SELECT {} AS category, COUNT(*) FROM messages m \
         WHERE m.account_id = ?1 AND m.date >= ?2 GROUP BY category ORDER BY COUNT(*) DESC",
        category_sql()
    ))?;
    let categories = stmt
        .query_map(params![account_id, now - YEAR], |row| {
            Ok(CategoryCount {
                category: row.get(0)?,
                count: row.get::<_, i64>(1)? as u64,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(Overview {
        total: total as u64,
        total_bytes: total_bytes as u64,
        unread: unread as u64,
        unread_week: unread_week as u64,
        list_messages: list_messages as u64,
        sender_count,
        never_opened,
        freed_week_bytes: freed_week_bytes as u64,
        categories,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{messages, test_support};
    use mail_sieve_core::{Address, Flags, MessageMeta};

    const NOW: i64 = 1_800_000_000;

    fn meta(uid: u32, from: &str, flags: Flags, age_days: i64, list: Option<&str>) -> MessageMeta {
        MessageMeta {
            uid,
            from: Some(Address {
                name: None,
                email: from.into(),
            }),
            subject: Some("hello".into()),
            date: Some(NOW - age_days * DAY),
            size: 100,
            flags,
            list_id: list.map(Into::into),
            ..Default::default()
        }
    }

    #[test]
    fn summarizes_an_account() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let inbox = test_support::mailbox(&store, account, "INBOX");
        store
            .write(|tx| {
                messages::upsert_batch(
                    tx,
                    account,
                    inbox,
                    &[
                        meta(1, "news@list.com", Flags::default(), 2, Some("list.com")),
                        meta(2, "news@list.com", Flags::default(), 30, Some("list.com")),
                        meta(3, "alice@friends.org", Flags::SEEN, 1, None),
                        meta(4, "alice@friends.org", Flags::default(), 400, None),
                    ],
                )?;
                tx.execute(
                    "INSERT INTO action_log (account_id, action, mailbox, uids, count, bytes, at) \
                     VALUES (?1, 'trash', 'INBOX', '9', 1, 500, ?2), (?1, 'trash', 'INBOX', '8', 1, 700, ?3)",
                    params![account, NOW - DAY, NOW - 30 * DAY],
                )?;
                Ok(())
            })
            .unwrap();
        let summary = store.read(|c| overview(c, account, NOW)).unwrap();
        assert_eq!(
            (
                summary.total,
                summary.total_bytes,
                summary.unread,
                summary.unread_week
            ),
            (4, 400, 3, 1)
        );
        assert_eq!((summary.list_messages, summary.sender_count), (2, 2));
        assert_eq!(
            summary.never_opened,
            NeverOpened {
                senders: 1,
                messages: 2,
                bytes: 200
            }
        );
        assert_eq!(summary.freed_week_bytes, 500);
        assert_eq!(
            summary.categories,
            vec![
                CategoryCount {
                    category: "newsletters".into(),
                    count: 2
                },
                CategoryCount {
                    category: "people".into(),
                    count: 1
                },
            ]
        );
    }
}
