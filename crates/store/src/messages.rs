use mail_sieve_core::{Flags, LABEL_SEPARATOR, MessageMeta, join_labels, normalize_label};
use rusqlite::{Connection, params};

use crate::Result;

pub fn upsert_batch(
    conn: &Connection,
    account_id: i64,
    mailbox_id: i64,
    messages: &[MessageMeta],
) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "INSERT INTO messages (account_id, mailbox_id, uid, message_id, from_addr, from_name, \
         from_domain, subject, date, size, flags, list_id, unsub_http, unsub_mailto, unsub_one_click, \
         labels) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15, ?16) \
         ON CONFLICT (mailbox_id, uid) DO UPDATE SET flags = excluded.flags, labels = excluded.labels \
         WHERE flags != excluded.flags OR labels IS NOT excluded.labels",
    )?;
    let mut written = 0;
    for message in messages {
        let (from_addr, from_name, from_domain) = match &message.from {
            Some(address) => (
                address.email.to_lowercase(),
                address.name.clone(),
                address.domain().to_lowercase(),
            ),
            None => (String::new(), None, String::new()),
        };
        written += stmt.execute(params![
            account_id,
            mailbox_id,
            message.uid,
            message.message_id,
            from_addr,
            from_name,
            from_domain,
            message.subject,
            message.date.or(message.internal_date).unwrap_or(0),
            message.size,
            message.flags.bits(),
            message.list_id,
            message.unsubscribe.http,
            message.unsubscribe.mailto,
            message.unsubscribe.one_click,
            join_labels(&message.labels),
        ])?;
    }
    Ok(written)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagUpdate {
    pub uid: u32,
    pub flags: Flags,
    pub labels: Option<Vec<String>>,
}

pub fn update_flags(conn: &Connection, mailbox_id: i64, updates: &[FlagUpdate]) -> Result<usize> {
    let mut flags_only = conn.prepare_cached(
        "UPDATE messages SET flags = ?3 WHERE mailbox_id = ?1 AND uid = ?2 AND flags != ?3",
    )?;
    let mut with_labels = conn.prepare_cached(
        "UPDATE messages SET flags = ?3, labels = ?4 WHERE mailbox_id = ?1 AND uid = ?2 \
         AND (flags != ?3 OR labels IS NOT ?4)",
    )?;
    let mut updated = 0;
    for update in updates {
        updated += match &update.labels {
            None => flags_only.execute(params![mailbox_id, update.uid, update.flags.bits()])?,
            Some(labels) => with_labels.execute(params![
                mailbox_id,
                update.uid,
                update.flags.bits(),
                join_labels(labels)
            ])?,
        };
    }
    Ok(updated)
}

pub fn remove_label(
    conn: &Connection,
    mailbox_id: i64,
    uids: &[u32],
    label: &str,
) -> Result<usize> {
    let label = normalize_label(label);
    let mut select =
        conn.prepare_cached("SELECT labels FROM messages WHERE mailbox_id = ?1 AND uid = ?2")?;
    let mut update =
        conn.prepare_cached("UPDATE messages SET labels = ?3 WHERE mailbox_id = ?1 AND uid = ?2")?;
    let mut updated = 0;
    for uid in uids {
        let current: Option<String> =
            match select.query_row(params![mailbox_id, uid], |row| row.get(0)) {
                Ok(labels) => labels,
                Err(rusqlite::Error::QueryReturnedNoRows) => continue,
                Err(error) => return Err(error.into()),
            };
        let Some(current) = current else { continue };
        let remaining: Vec<String> = current
            .split(LABEL_SEPARATOR)
            .filter(|existing| *existing != label)
            .map(str::to_string)
            .collect();
        updated += update.execute(params![mailbox_id, uid, join_labels(&remaining)])?;
    }
    Ok(updated)
}

pub fn apply_flag_change(
    conn: &Connection,
    mailbox_id: i64,
    uids: &[u32],
    add: Flags,
    remove: Flags,
) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "UPDATE messages SET flags = (flags | ?3) & ~?4 WHERE mailbox_id = ?1 AND uid = ?2",
    )?;
    let mut updated = 0;
    for uid in uids {
        updated += stmt.execute(params![mailbox_id, uid, add.bits(), remove.bits()])?;
    }
    Ok(updated)
}

pub fn delete_uids(conn: &Connection, mailbox_id: i64, uids: &[u32]) -> Result<usize> {
    let mut stmt =
        conn.prepare_cached("DELETE FROM messages WHERE mailbox_id = ?1 AND uid = ?2")?;
    let mut deleted = 0;
    for uid in uids {
        deleted += stmt.execute(params![mailbox_id, uid])?;
    }
    Ok(deleted)
}

pub fn local_uids(conn: &Connection, mailbox_id: i64) -> Result<Vec<u32>> {
    let mut stmt =
        conn.prepare_cached("SELECT uid FROM messages WHERE mailbox_id = ?1 ORDER BY uid")?;
    let uids = stmt
        .query_map([mailbox_id], |row| row.get(0))?
        .collect::<rusqlite::Result<_>>()?;
    Ok(uids)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;
    use mail_sieve_core::{Address, Unsubscribe};

    fn meta(uid: u32, from: &str, flags: Flags, size: u32) -> MessageMeta {
        MessageMeta {
            uid,
            from: Some(Address {
                name: Some("News".into()),
                email: from.into(),
            }),
            subject: Some(format!("subject {uid}")),
            date: Some(1_700_000_000 + i64::from(uid)),
            size,
            flags,
            unsubscribe: Unsubscribe {
                http: Some("https://x/u".into()),
                mailto: None,
                one_click: true,
            },
            ..Default::default()
        }
    }

    fn sender(conn: &Connection, addr: &str) -> Option<(i64, i64, i64)> {
        conn.query_row(
            "SELECT message_count, unread_count, total_size FROM senders WHERE from_addr = ?1",
            [addr],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        )
        .ok()
    }

    #[test]
    fn triggers_maintain_sender_aggregates() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let inbox = test_support::mailbox(&store, account, "INBOX");
        store
            .write(|tx| {
                upsert_batch(
                    tx,
                    account,
                    inbox,
                    &[
                        meta(1, "News@Shop.com", Flags::default(), 100),
                        meta(2, "news@shop.com", Flags::SEEN, 50),
                        meta(3, "bob@example.com", Flags::default(), 10),
                    ],
                )
            })
            .unwrap();
        store
            .read(|conn| {
                assert_eq!(sender(conn, "news@shop.com"), Some((2, 1, 150)));
                Ok(())
            })
            .unwrap();

        store
            .write(|tx| {
                let seen = |uid| FlagUpdate {
                    uid,
                    flags: Flags::SEEN,
                    labels: None,
                };
                assert_eq!(update_flags(tx, inbox, &[seen(1), seen(2)])?, 1);
                assert_eq!(delete_uids(tx, inbox, &[2])?, 1);
                Ok(())
            })
            .unwrap();
        store
            .read(|conn| {
                assert_eq!(sender(conn, "news@shop.com"), Some((1, 0, 100)));
                Ok(())
            })
            .unwrap();

        store.write(|tx| delete_uids(tx, inbox, &[1])).unwrap();
        store
            .read(|conn| {
                assert_eq!(sender(conn, "news@shop.com"), None);
                assert_eq!(local_uids(conn, inbox)?, vec![3]);
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn upsert_only_rewrites_changed_flags() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let inbox = test_support::mailbox(&store, account, "INBOX");
        let batch = [meta(1, "a@b.com", Flags::default(), 1)];
        assert_eq!(
            store
                .write(|tx| upsert_batch(tx, account, inbox, &batch))
                .unwrap(),
            1
        );
        assert_eq!(
            store
                .write(|tx| upsert_batch(tx, account, inbox, &batch))
                .unwrap(),
            0
        );
        let read = [meta(1, "a@b.com", Flags::SEEN, 1)];
        assert_eq!(
            store
                .write(|tx| upsert_batch(tx, account, inbox, &read))
                .unwrap(),
            1
        );
        store
            .read(|conn| {
                assert_eq!(sender(conn, "a@b.com"), Some((1, 0, 1)));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    fn tracks_and_removes_labels() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let all = test_support::mailbox(&store, account, "[Gmail]/All Mail");
        let mut message = meta(1, "a@b.com", Flags::default(), 1);
        message.labels = vec!["\\Inbox".into(), "Receipts".into()];
        store
            .write(|tx| upsert_batch(tx, account, all, &[message]))
            .unwrap();
        let labels = |store: &crate::Store| {
            store
                .read(|c| {
                    Ok(c.query_row("SELECT labels FROM messages", [], |r| {
                        r.get::<_, Option<String>>(0)
                    })?)
                })
                .unwrap()
        };
        assert_eq!(labels(&store).as_deref(), Some("inbox\u{1f}receipts"));
        store
            .write(|tx| remove_label(tx, all, &[1, 99], "\\Inbox"))
            .unwrap();
        assert_eq!(labels(&store).as_deref(), Some("receipts"));
        let update = FlagUpdate {
            uid: 1,
            flags: Flags::SEEN,
            labels: Some(vec![]),
        };
        assert_eq!(
            store.write(|tx| update_flags(tx, all, &[update])).unwrap(),
            1
        );
        assert_eq!(labels(&store), None);
    }

    #[test]
    fn apply_flag_change_and_cascade_delete() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let inbox = test_support::mailbox(&store, account, "INBOX");
        store
            .write(|tx| {
                upsert_batch(
                    tx,
                    account,
                    inbox,
                    &[meta(1, "a@b.com", Flags::default(), 1)],
                )
            })
            .unwrap();
        store
            .write(|tx| {
                apply_flag_change(
                    tx,
                    inbox,
                    &[1],
                    Flags::SEEN | Flags::FLAGGED,
                    Flags::default(),
                )
            })
            .unwrap();
        store
            .write(|tx| apply_flag_change(tx, inbox, &[1], Flags::default(), Flags::FLAGGED))
            .unwrap();
        store
            .read(|conn| {
                let flags: u8 = conn.query_row("SELECT flags FROM messages", [], |r| r.get(0))?;
                assert_eq!(Flags::from_bits(flags), Flags::SEEN);
                Ok(())
            })
            .unwrap();
        store
            .write(|tx| crate::accounts::delete(tx, account))
            .unwrap();
        store
            .read(|conn| {
                let count: i64 =
                    conn.query_row("SELECT COUNT(*) FROM senders", [], |r| r.get(0))?;
                assert_eq!(count, 0);
                Ok(())
            })
            .unwrap();
    }
}
