use mail_sieve_core::{Category, Query, TermKind, normalize_label};
use rusqlite::types::Value;
use rusqlite::{Connection, Row, params_from_iter};
use serde::{Deserialize, Serialize};

use crate::Result;
use crate::pins::{self, Pin};

const SEEN: i64 = 1;
const ANSWERED: i64 = 2;
const FLAGGED: i64 = 4;

#[derive(Debug, Default)]
struct Compiled {
    clauses: Vec<String>,
    params: Vec<Value>,
}

impl Compiled {
    fn push(&mut self, negated: bool, clause: String, params: impl IntoIterator<Item = Value>) {
        let clause = if negated {
            format!("NOT COALESCE(({clause}), 0)")
        } else {
            format!("COALESCE(({clause}), 0)")
        };
        self.clauses.push(clause);
        self.params.extend(params);
    }

    fn where_sql(&self) -> String {
        if self.clauses.is_empty() {
            "1".to_string()
        } else {
            self.clauses.join(" AND ")
        }
    }
}

const SOCIAL_DOMAINS: [&str; 14] = [
    "facebook.com",
    "facebookmail.com",
    "instagram.com",
    "linkedin.com",
    "twitter.com",
    "x.com",
    "reddit.com",
    "redditmail.com",
    "tiktok.com",
    "pinterest.com",
    "meetup.com",
    "strava.com",
    "discord.com",
    "mastodon.social",
];
const RECEIPT_WORDS: [&str; 7] = [
    "receipt",
    "invoice",
    "order",
    "payment",
    "purchase",
    "booking",
    "statement",
];
const NOTIFICATION_SENDERS: [&str; 6] = [
    "noreply",
    "no-reply",
    "donotreply",
    "do-not-reply",
    "notification",
    "alert",
];

fn any_like(column: &str, patterns: impl IntoIterator<Item = String>) -> String {
    patterns
        .into_iter()
        .map(|pattern| format!("{column} LIKE '{pattern}'"))
        .collect::<Vec<_>>()
        .join(" OR ")
}

pub fn category_sql() -> String {
    let receipts = any_like(
        "m.subject",
        RECEIPT_WORDS.iter().map(|word| format!("%{word}%")),
    );
    let social_domains = SOCIAL_DOMAINS
        .iter()
        .map(|domain| format!("'{domain}'"))
        .collect::<Vec<_>>()
        .join(", ");
    let social_subdomains = any_like(
        "m.from_domain",
        SOCIAL_DOMAINS.iter().map(|domain| format!("%.{domain}")),
    );
    let notifications = any_like(
        "m.from_addr",
        NOTIFICATION_SENDERS.iter().map(|word| format!("%{word}%")),
    );
    format!(
        "CASE \
         WHEN {receipts} THEN '{}' \
         WHEN m.from_domain IN ({social_domains}) OR {social_subdomains} THEN '{}' \
         WHEN m.list_id IS NOT NULL THEN '{}' \
         WHEN {notifications} THEN '{}' \
         WHEN m.unsub_http IS NOT NULL OR m.unsub_mailto IS NOT NULL THEN '{}' \
         ELSE '{}' END",
        Category::Receipts,
        Category::Social,
        Category::Newsletters,
        Category::Notifications,
        Category::Promotions,
        Category::People,
    )
}

fn like_escape(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('%', "\\%")
        .replace('_', "\\_")
}

fn contains(value: &str) -> Value {
    Value::Text(format!("%{}%", like_escape(value)))
}

fn fts_expression(text: &str) -> String {
    text.split_whitespace()
        .map(|word| format!("\"{}\"*", word.replace('"', "\"\"")))
        .collect::<Vec<_>>()
        .join(" ")
}

fn message_pinned_sql() -> String {
    pins::pinned_sql("m.account_id", "m.from_addr", "m.from_domain")
}

fn compile(account_id: i64, query: &Query, now: i64) -> Compiled {
    let mut compiled = Compiled::default();
    compiled.clauses.push("m.account_id = ?".into());
    compiled.params.push(Value::Integer(account_id));
    for term in &query.terms {
        let negated = term.negated;
        match &term.kind {
            TermKind::From(pattern) if pattern.contains('*') => {
                let like = like_escape(pattern).replace('*', "%");
                compiled.push(negated, "m.from_addr LIKE ? ESCAPE '\\'".into(), [Value::Text(like)]);
            }
            TermKind::From(address) if address.contains('@') => {
                compiled.push(negated, "m.from_addr = ?".into(), [Value::Text(address.clone())]);
            }
            TermKind::From(fragment) => compiled.push(
                negated,
                "m.from_addr LIKE ? ESCAPE '\\' OR m.from_name LIKE ? ESCAPE '\\'".into(),
                [contains(fragment), contains(fragment)],
            ),
            TermKind::Domain(domain) => compiled.push(
                negated,
                "m.from_domain = ? OR m.from_domain LIKE ? ESCAPE '\\'".into(),
                [Value::Text(domain.clone()), Value::Text(format!("%.{}", like_escape(domain)))],
            ),
            TermKind::Subject(text) => {
                compiled.push(negated, "m.subject LIKE ? ESCAPE '\\'".into(), [contains(text)])
            }
            TermKind::List(list) => {
                compiled.push(negated, "m.list_id LIKE ? ESCAPE '\\'".into(), [contains(list)])
            }
            TermKind::Mailbox(name) => compiled.push(
                negated,
                "m.mailbox_id IN (SELECT id FROM mailboxes WHERE account_id = ? AND name = ? COLLATE NOCASE) \
                 OR instr(char(31) || m.labels || char(31), char(31) || ? || char(31)) > 0"
                    .into(),
                [Value::Integer(account_id), Value::Text(name.clone()), Value::Text(normalize_label(name))],
            ),
            TermKind::Unread => compiled.push(negated, format!("(m.flags & {SEEN}) = 0"), []),
            TermKind::Flagged => compiled.push(negated, format!("(m.flags & {FLAGGED}) != 0"), []),
            TermKind::Answered => compiled.push(negated, format!("(m.flags & {ANSWERED}) != 0"), []),
            TermKind::Pinned => compiled.push(negated, message_pinned_sql(), []),
            TermKind::HasUnsubscribe => {
                compiled.push(negated, "m.unsub_http IS NOT NULL OR m.unsub_mailto IS NOT NULL".into(), [])
            }
            TermKind::HasList => compiled.push(negated, "m.list_id IS NOT NULL".into(), []),
            TermKind::OlderThan(seconds) => {
                compiled.push(negated, "m.date < ?".into(), [Value::Integer(now - seconds)])
            }
            TermKind::NewerThan(seconds) => {
                compiled.push(negated, "m.date >= ?".into(), [Value::Integer(now - seconds)])
            }
            TermKind::Before(timestamp) => compiled.push(negated, "m.date < ?".into(), [Value::Integer(*timestamp)]),
            TermKind::After(timestamp) => compiled.push(negated, "m.date >= ?".into(), [Value::Integer(*timestamp)]),
            TermKind::Larger(bytes) => compiled.push(negated, "m.size > ?".into(), [Value::Integer(*bytes as i64)]),
            TermKind::Smaller(bytes) => compiled.push(negated, "m.size < ?".into(), [Value::Integer(*bytes as i64)]),
            TermKind::Category(category) => compiled.push(
                negated,
                format!("({}) = ?", category_sql()),
                [Value::Text(category.to_string())],
            ),
            TermKind::Text(text) => compiled.push(
                negated,
                "m.id IN (SELECT rowid FROM messages_fts WHERE messages_fts MATCH ?)".into(),
                [Value::Text(fts_expression(text))],
            ),
        }
    }
    compiled
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
pub struct Totals {
    pub count: u64,
    pub bytes: u64,
    pub pinned: u64,
}

pub fn totals(conn: &Connection, account_id: i64, query: &Query, now: i64) -> Result<Totals> {
    let compiled = compile(account_id, query, now);
    let sql = format!(
        "SELECT COUNT(*), COALESCE(SUM(m.size), 0), COALESCE(SUM({}), 0) FROM messages m WHERE {}",
        message_pinned_sql(),
        compiled.where_sql()
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let (count, bytes, pinned): (i64, i64, i64) = stmt
        .query_row(params_from_iter(compiled.params), |row| {
            Ok((row.get(0)?, row.get(1)?, row.get(2)?))
        })?;
    Ok(Totals {
        count: count as u64,
        bytes: bytes as u64,
        pinned: pinned as u64,
    })
}

#[derive(Debug, Clone, Serialize)]
pub struct MessageRow {
    pub id: i64,
    pub mailbox_id: i64,
    pub mailbox: String,
    pub uid: u32,
    pub from_addr: String,
    pub from_name: Option<String>,
    pub subject: Option<String>,
    pub date: i64,
    pub size: u64,
    pub unread: bool,
    pub flagged: bool,
    pub list_id: Option<String>,
    pub has_unsubscribe: bool,
    pub pinned: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Cursor {
    pub date: i64,
    pub id: i64,
}

fn message_row(row: &Row<'_>) -> rusqlite::Result<MessageRow> {
    let flags: i64 = row.get(10)?;
    Ok(MessageRow {
        id: row.get(0)?,
        mailbox_id: row.get(1)?,
        mailbox: row.get(2)?,
        uid: row.get(3)?,
        from_addr: row.get(4)?,
        from_name: row.get(5)?,
        subject: row.get(6)?,
        date: row.get(7)?,
        size: row.get::<_, i64>(8)? as u64,
        list_id: row.get(9)?,
        unread: flags & SEEN == 0,
        flagged: flags & FLAGGED != 0,
        has_unsubscribe: row.get(11)?,
        pinned: row.get(12)?,
    })
}

pub fn messages(
    conn: &Connection,
    account_id: i64,
    query: &Query,
    now: i64,
    after: Option<Cursor>,
    limit: u32,
) -> Result<Vec<MessageRow>> {
    let mut compiled = compile(account_id, query, now);
    if let Some(cursor) = after {
        compiled.clauses.push("(m.date, m.id) < (?, ?)".into());
        compiled
            .params
            .extend([Value::Integer(cursor.date), Value::Integer(cursor.id)]);
    }
    compiled.params.push(Value::Integer(i64::from(limit)));
    let sql = format!(
        "SELECT m.id, m.mailbox_id, b.name, m.uid, m.from_addr, m.from_name, m.subject, m.date, m.size, \
         m.list_id, m.flags, (m.unsub_http IS NOT NULL OR m.unsub_mailto IS NOT NULL), {} \
         FROM messages m JOIN mailboxes b ON b.id = m.mailbox_id \
         WHERE {} ORDER BY m.date DESC, m.id DESC LIMIT ?",
        message_pinned_sql(),
        compiled.where_sql()
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let rows = stmt
        .query_map(params_from_iter(compiled.params), message_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Target {
    pub mailbox_id: i64,
    pub uid: u32,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Resolved {
    pub targets: Vec<Target>,
    pub skipped: Vec<Target>,
}

impl Resolved {
    pub fn merge(&mut self, other: Resolved) {
        self.targets.extend(other.targets);
        self.skipped.extend(other.skipped);
        self.normalize();
    }

    fn normalize(&mut self) {
        for list in [&mut self.targets, &mut self.skipped] {
            list.sort_by_key(|target| (target.mailbox_id, target.uid));
            list.dedup();
        }
    }

    fn collect(&mut self, mut rows: rusqlite::Rows<'_>, include_pinned: bool) -> Result<()> {
        while let Some(row) = rows.next()? {
            let target = Target {
                mailbox_id: row.get(0)?,
                uid: row.get(1)?,
            };
            let pinned: bool = row.get(2)?;
            if pinned && !include_pinned {
                self.skipped.push(target);
            } else {
                self.targets.push(target);
            }
        }
        Ok(())
    }
}

pub fn resolve(
    conn: &Connection,
    account_id: i64,
    query: &Query,
    now: i64,
    include_pinned: bool,
) -> Result<Resolved> {
    let compiled = compile(account_id, query, now);
    let sql = format!(
        "SELECT m.mailbox_id, m.uid, {} FROM messages m WHERE {} ORDER BY m.mailbox_id, m.uid",
        message_pinned_sql(),
        compiled.where_sql()
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let mut resolved = Resolved::default();
    resolved.collect(
        stmt.query(params_from_iter(compiled.params))?,
        include_pinned,
    )?;
    Ok(resolved)
}

pub fn resolve_ids(
    conn: &Connection,
    account_id: i64,
    ids: &[i64],
    include_pinned: bool,
) -> Result<Resolved> {
    let sql = format!(
        "SELECT m.mailbox_id, m.uid, {} FROM messages m WHERE m.account_id = ?1 AND m.id = ?2",
        message_pinned_sql()
    );
    let mut stmt = conn.prepare_cached(&sql)?;
    let mut resolved = Resolved::default();
    for id in ids {
        resolved.collect(stmt.query([account_id, *id])?, include_pinned)?;
    }
    resolved.normalize();
    Ok(resolved)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    pub account_id: i64,
    pub mailbox: String,
    pub uidvalidity: Option<u32>,
    pub uid: u32,
}

pub fn locate(conn: &Connection, message_id: i64) -> Result<Location> {
    use rusqlite::OptionalExtension;
    conn.query_row(
        "SELECT m.account_id, b.name, b.uidvalidity, m.uid FROM messages m \
         JOIN mailboxes b ON b.id = m.mailbox_id WHERE m.id = ?1",
        [message_id],
        |row| {
            Ok(Location {
                account_id: row.get(0)?,
                mailbox: row.get(1)?,
                uidvalidity: row.get(2)?,
                uid: row.get(3)?,
            })
        },
    )
    .optional()?
    .ok_or(crate::Error::NotFound)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum GroupBy {
    #[default]
    Sender,
    Domain,
    List,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SortBy {
    #[default]
    Count,
    Size,
    Unread,
    Recent,
}

impl SortBy {
    fn column(self) -> &'static str {
        match self {
            SortBy::Count => "message_count",
            SortBy::Size => "total_size",
            SortBy::Unread => "unread_count",
            SortBy::Recent => "last_date",
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Group {
    pub key: String,
    pub name: Option<String>,
    pub sender_count: u64,
    pub message_count: u64,
    pub unread_count: u64,
    pub total_size: u64,
    pub last_date: Option<i64>,
    pub list_id: Option<String>,
    pub unsub_http: Option<String>,
    pub unsub_mailto: Option<String>,
    pub unsub_one_click: bool,
    pub unsubscribe_status: Option<String>,
    pub pinned_count: u64,
    pub pin: Option<Pin>,
}

fn group_row(row: &Row<'_>) -> rusqlite::Result<Group> {
    Ok(Group {
        key: row.get(0)?,
        name: row.get(1)?,
        sender_count: row.get::<_, i64>(2)? as u64,
        message_count: row.get::<_, i64>(3)? as u64,
        unread_count: row.get::<_, i64>(4)? as u64,
        total_size: row.get::<_, i64>(5)? as u64,
        last_date: row.get(6)?,
        list_id: row.get(7)?,
        unsub_http: row.get(8)?,
        unsub_mailto: row.get(9)?,
        unsub_one_click: row.get(10)?,
        unsubscribe_status: row.get(11)?,
        pinned_count: row.get::<_, i64>(12)? as u64,
        pin: pins::decode(row.get(13)?),
    })
}

pub struct GroupRequest<'a> {
    pub account_id: i64,
    pub query: &'a Query,
    pub now: i64,
    pub group_by: GroupBy,
    pub sort_by: SortBy,
    pub limit: u32,
    pub offset: u32,
}

pub fn groups(conn: &Connection, request: &GroupRequest<'_>) -> Result<Vec<Group>> {
    let use_materialized = request.query.is_empty() && request.group_by != GroupBy::List;
    let (source, mut params) = if use_materialized {
        (
            "SELECT account_id, from_addr, from_name, from_domain, message_count, unread_count, total_size, \
             last_date, list_id, unsub_http, unsub_mailto, unsub_one_click FROM senders WHERE account_id = ?"
                .to_string(),
            vec![Value::Integer(request.account_id)],
        )
    } else {
        let compiled = compile(request.account_id, request.query, request.now);
        (
            format!(
                "SELECT m.account_id, m.from_addr, MAX(m.from_name) AS from_name, m.from_domain, \
                 COUNT(*) AS message_count, SUM((m.flags & {SEEN}) = 0) AS unread_count, \
                 SUM(m.size) AS total_size, MAX(m.date) AS last_date, MAX(m.list_id) AS list_id, \
                 MAX(m.unsub_http) AS unsub_http, MAX(m.unsub_mailto) AS unsub_mailto, \
                 MAX(m.unsub_one_click) AS unsub_one_click \
                 FROM messages m WHERE {} GROUP BY m.from_addr",
                compiled.where_sql()
            ),
            compiled.params,
        )
    };
    let (key, extra_filter, pin, pinned_first) = match request.group_by {
        GroupBy::Sender => (
            "s.from_addr",
            "",
            pins::pin_sql("s.account_id", "s.from_addr", "s.from_domain"),
            "pin IS NOT NULL DESC, ",
        ),
        GroupBy::Domain => (
            "s.from_domain",
            "",
            pins::pin_sql("s.account_id", "NULL", "s.from_domain"),
            "pin IS NOT NULL DESC, ",
        ),
        GroupBy::List => (
            "s.list_id",
            "WHERE s.list_id IS NOT NULL",
            "NULL".to_string(),
            "",
        ),
    };
    let sender_pinned = pins::pinned_sql("s.account_id", "s.from_addr", "s.from_domain");
    let sort = match request.sort_by {
        SortBy::Recent => "MAX(s.last_date)".to_string(),
        other => format!("SUM(s.{})", other.column()),
    };
    let sql = format!(
        "SELECT {key}, MAX(s.from_name), COUNT(*), SUM(s.message_count), SUM(s.unread_count), \
         SUM(s.total_size), MAX(s.last_date), MAX(s.list_id), MAX(s.unsub_http), MAX(s.unsub_mailto), \
         MAX(s.unsub_one_click), MAX(u.status), \
         SUM(CASE WHEN {sender_pinned} THEN s.message_count ELSE 0 END), {pin} AS pin \
         FROM ({source}) s \
         LEFT JOIN unsubscribes u ON u.account_id = s.account_id AND u.from_addr = s.from_addr \
         {extra_filter} GROUP BY {key} ORDER BY {pinned_first}{sort} DESC, {key} LIMIT ? OFFSET ?"
    );
    params.extend([
        Value::Integer(i64::from(request.limit)),
        Value::Integer(i64::from(request.offset)),
    ]);
    let mut stmt = conn.prepare_cached(&sql)?;
    let rows = stmt
        .query_map(params_from_iter(params), group_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{messages, test_support};
    use mail_sieve_core::{Address, Flags, MessageMeta, Unsubscribe};

    const NOW: i64 = 1_800_000_000;
    const DAY: i64 = 86_400;

    fn seed() -> (test_support::TestStore, i64) {
        let store = test_support::store();
        let account = test_support::account(&store);
        let inbox = test_support::mailbox(&store, account, "INBOX");
        let archive = test_support::mailbox(&store, account, "Archive");
        let message = |uid: u32,
                       from: &str,
                       subject: &str,
                       age_days: i64,
                       flags: Flags,
                       list: Option<&str>| MessageMeta {
            uid,
            from: Some(Address {
                name: Some(from.split('@').next().unwrap().to_string()),
                email: from.into(),
            }),
            subject: Some(subject.into()),
            date: Some(NOW - age_days * DAY),
            size: 1000 * uid,
            flags,
            list_id: list.map(Into::into),
            unsubscribe: Unsubscribe {
                http: list.map(|_| "https://unsub".to_string()),
                mailto: None,
                one_click: list.is_some(),
            },
            ..Default::default()
        };
        store
            .write(|tx| {
                messages::upsert_batch(
                    tx,
                    account,
                    inbox,
                    &[
                        message(
                            1,
                            "deals@shop.com",
                            "Big sale today",
                            400,
                            Flags::default(),
                            Some("deals.shop.com"),
                        ),
                        MessageMeta {
                            labels: vec!["Receipts".into()],
                            ..message(
                                2,
                                "deals@shop.com",
                                "Weekly deals",
                                10,
                                Flags::SEEN,
                                Some("deals.shop.com"),
                            )
                        },
                        message(
                            3,
                            "news@mail.shop.com",
                            "Newsletter",
                            5,
                            Flags::default(),
                            Some("news.shop.com"),
                        ),
                        message(4, "alice@friends.org", "Dinner?", 1, Flags::FLAGGED, None),
                    ],
                )?;
                messages::upsert_batch(
                    tx,
                    account,
                    archive,
                    &[message(
                        5,
                        "alice@friends.org",
                        "Old trip photos",
                        800,
                        Flags::SEEN,
                        None,
                    )],
                )
            })
            .unwrap();
        (store, account)
    }

    fn uids(store: &crate::Store, account: i64, query: &str) -> Vec<u32> {
        let query = Query::parse(query).unwrap();
        let mut found: Vec<u32> = store
            .read(|conn| resolve(conn, account, &query, NOW, true))
            .unwrap()
            .targets
            .into_iter()
            .map(|t| t.uid)
            .collect();
        found.sort_unstable();
        found
    }

    #[test]
    fn filters_by_each_term() {
        let (store, account) = seed();
        assert_eq!(uids(&store, account, ""), vec![1, 2, 3, 4, 5]);
        assert_eq!(uids(&store, account, "from:deals@shop.com"), vec![1, 2]);
        assert_eq!(uids(&store, account, "from:*@shop.com"), vec![1, 2]);
        assert_eq!(uids(&store, account, "from:alice"), vec![4, 5]);
        assert_eq!(uids(&store, account, "domain:shop.com"), vec![1, 2, 3]);
        assert_eq!(uids(&store, account, "is:unread"), vec![1, 3, 4]);
        assert_eq!(uids(&store, account, "is:read"), vec![2, 5]);
        assert_eq!(uids(&store, account, "is:flagged"), vec![4]);
        assert_eq!(
            uids(&store, account, "has:unsubscribe older_than:1y"),
            vec![1]
        );
        assert_eq!(uids(&store, account, "-has:unsubscribe"), vec![4, 5]);
        assert_eq!(uids(&store, account, "newer_than:7d"), vec![3, 4]);
        assert_eq!(uids(&store, account, "in:archive"), vec![5]);
        assert_eq!(uids(&store, account, "-in:INBOX"), vec![5]);
        assert_eq!(uids(&store, account, "in:receipts"), vec![2]);
        assert_eq!(uids(&store, account, "larger:3kb"), vec![4, 5]);
        assert_eq!(uids(&store, account, "subject:deal"), vec![2]);
        assert_eq!(uids(&store, account, "list:news"), vec![3]);
        assert_eq!(uids(&store, account, "phot"), vec![5]);
        assert_eq!(uids(&store, account, "-sale shop"), vec![2, 3]);
        assert_eq!(uids(&store, account, "subject:100%"), Vec::<u32>::new());
        assert_eq!(uids(&store, account, "category:newsletters"), vec![1, 2, 3]);
        assert_eq!(uids(&store, account, "category:people"), vec![4, 5]);
        assert_eq!(uids(&store, account, "-category:people"), vec![1, 2, 3]);
    }

    #[test]
    fn totals_and_keyset_pagination() {
        let (store, account) = seed();
        let query = Query::parse("").unwrap();
        let totals = store
            .read(|conn| totals(conn, account, &query, NOW))
            .unwrap();
        assert_eq!(
            totals,
            Totals {
                count: 5,
                bytes: 15_000,
                pinned: 0
            }
        );
        let first = store
            .read(|conn| messages(conn, account, &query, NOW, None, 2))
            .unwrap();
        assert_eq!(first.iter().map(|m| m.uid).collect::<Vec<_>>(), vec![4, 3]);
        let last = first.last().unwrap();
        let cursor = Cursor {
            date: last.date,
            id: last.id,
        };
        let second = store
            .read(|conn| messages(conn, account, &query, NOW, Some(cursor), 10))
            .unwrap();
        assert_eq!(
            second.iter().map(|m| m.uid).collect::<Vec<_>>(),
            vec![2, 1, 5]
        );
        assert_eq!(second[2].mailbox, "Archive");
        let location = store.read(|conn| locate(conn, second[2].id)).unwrap();
        assert_eq!((location.mailbox.as_str(), location.uid), ("Archive", 5));
        assert!(matches!(
            store.read(|conn| locate(conn, 9999)),
            Err(crate::Error::NotFound)
        ));
    }

    fn group_keys(
        store: &crate::Store,
        account: i64,
        query: &str,
        group_by: GroupBy,
        sort_by: SortBy,
    ) -> Vec<(String, u64)> {
        let query = Query::parse(query).unwrap();
        let request = GroupRequest {
            account_id: account,
            query: &query,
            now: NOW,
            group_by,
            sort_by,
            limit: 50,
            offset: 0,
        };
        store
            .read(|conn| groups(conn, &request))
            .unwrap()
            .into_iter()
            .map(|g| (g.key, g.message_count))
            .collect()
    }

    #[test]
    fn groups_from_materialized_and_filtered_sources() {
        let (store, account) = seed();
        assert_eq!(
            group_keys(&store, account, "", GroupBy::Sender, SortBy::Count),
            vec![
                ("alice@friends.org".into(), 2),
                ("deals@shop.com".into(), 2),
                ("news@mail.shop.com".into(), 1)
            ]
        );
        assert_eq!(
            group_keys(&store, account, "", GroupBy::Domain, SortBy::Size),
            vec![
                ("friends.org".into(), 2),
                ("mail.shop.com".into(), 1),
                ("shop.com".into(), 2)
            ]
        );
        assert_eq!(
            group_keys(&store, account, "is:unread", GroupBy::Sender, SortBy::Count),
            vec![
                ("alice@friends.org".into(), 1),
                ("deals@shop.com".into(), 1),
                ("news@mail.shop.com".into(), 1)
            ]
        );
        assert_eq!(
            group_keys(&store, account, "", GroupBy::List, SortBy::Count),
            vec![("deals.shop.com".into(), 2), ("news.shop.com".into(), 1)]
        );
    }

    #[test]
    fn pins_protect_senders_and_domains() {
        let (store, account) = seed();
        store
            .write(|tx| {
                pins::add(tx, account, pins::PinKind::Sender, " Alice@Friends.org ")?;
                pins::add(tx, account, pins::PinKind::Domain, "@shop.com")?;
                pins::add(tx, account, pins::PinKind::Domain, "shop.com")
            })
            .unwrap();
        assert_eq!(store.read(|c| pins::list(c, account)).unwrap().len(), 2);
        assert_eq!(uids(&store, account, "is:pinned"), vec![1, 2, 3, 4, 5]);
        store
            .write(|tx| pins::remove(tx, account, pins::PinKind::Domain, "shop.com"))
            .unwrap();
        store
            .write(|tx| pins::add(tx, account, pins::PinKind::Domain, "mail.shop.com"))
            .unwrap();
        assert_eq!(uids(&store, account, "is:pinned"), vec![3, 4, 5]);
        assert_eq!(uids(&store, account, "-is:pinned"), vec![1, 2]);

        let query = Query::parse("").unwrap();
        let guarded = store
            .read(|c| resolve(c, account, &query, NOW, false))
            .unwrap();
        let uids_of = |targets: &[Target]| targets.iter().map(|t| t.uid).collect::<Vec<_>>();
        assert_eq!(uids_of(&guarded.targets), vec![1, 2]);
        assert_eq!(guarded.skipped.len(), 3);
        let ids: Vec<i64> = store
            .read(|c| messages(c, account, &query, NOW, None, 10))
            .unwrap()
            .into_iter()
            .filter(|m| m.pinned)
            .map(|m| m.id)
            .collect();
        assert_eq!(ids.len(), 3);
        let by_id = store
            .read(|c| resolve_ids(c, account, &ids, false))
            .unwrap();
        assert!(by_id.targets.is_empty());
        assert_eq!(by_id.skipped.len(), 3);
        let forced = store.read(|c| resolve_ids(c, account, &ids, true)).unwrap();
        assert_eq!(forced.targets.len(), 3);
        assert_eq!(
            store
                .read(|c| totals(c, account, &query, NOW))
                .unwrap()
                .pinned,
            3
        );

        let request = |group_by| GroupRequest {
            account_id: account,
            query: &query,
            now: NOW,
            group_by,
            sort_by: SortBy::Count,
            limit: 50,
            offset: 0,
        };
        let senders = store
            .read(|c| groups(c, &request(GroupBy::Sender)))
            .unwrap();
        let rows: Vec<_> = senders
            .iter()
            .map(|g| {
                (
                    g.key.as_str(),
                    g.pin.as_ref().map(|p| p.kind),
                    g.pinned_count,
                )
            })
            .collect();
        assert_eq!(
            rows,
            vec![
                ("alice@friends.org", Some(pins::PinKind::Sender), 2),
                ("news@mail.shop.com", Some(pins::PinKind::Domain), 1),
                ("deals@shop.com", None, 0),
            ]
        );
        let domains = store
            .read(|c| groups(c, &request(GroupBy::Domain)))
            .unwrap();
        let rows: Vec<_> = domains
            .iter()
            .map(|g| (g.key.as_str(), g.pin.is_some(), g.pinned_count))
            .collect();
        assert_eq!(
            rows,
            vec![
                ("mail.shop.com", true, 1),
                ("friends.org", false, 2),
                ("shop.com", false, 0),
            ]
        );
    }
}
