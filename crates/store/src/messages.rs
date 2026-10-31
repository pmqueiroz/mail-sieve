use mail_sieve_core::{Flags, MessageMeta};
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
         from_domain, subject, date, size, flags, list_id, unsub_http, unsub_mailto, unsub_one_click) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14, ?15) \
         ON CONFLICT (mailbox_id, uid) DO UPDATE SET flags = excluded.flags \
         WHERE flags != excluded.flags",
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
        ])?;
    }
    Ok(written)
}

pub fn update_flags(conn: &Connection, mailbox_id: i64, flags: &[(u32, Flags)]) -> Result<usize> {
    let mut stmt = conn.prepare_cached(
        "UPDATE messages SET flags = ?3 WHERE mailbox_id = ?1 AND uid = ?2 AND flags != ?3",
    )?;
    let mut updated = 0;
    for (uid, flags) in flags {
        updated += stmt.execute(params![mailbox_id, uid, flags.bits()])?;
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
                assert_eq!(
                    update_flags(tx, inbox, &[(1, Flags::SEEN), (2, Flags::SEEN)])?,
                    1
                );
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
