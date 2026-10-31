use std::fmt;
use std::str::FromStr;

use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};

use crate::{Error, Result, now};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PinKind {
    Sender,
    Domain,
}

impl PinKind {
    fn as_str(self) -> &'static str {
        match self {
            PinKind::Sender => "sender",
            PinKind::Domain => "domain",
        }
    }
}

impl fmt::Display for PinKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for PinKind {
    type Err = ();

    fn from_str(value: &str) -> std::result::Result<Self, ()> {
        match value {
            "sender" => Ok(PinKind::Sender),
            "domain" => Ok(PinKind::Domain),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Pin {
    pub kind: PinKind,
    pub key: String,
}

pub fn normalize_key(key: &str) -> String {
    key.trim().trim_start_matches('@').to_lowercase()
}

pub(crate) fn covering_sql(account: &str, addr: &str, domain: &str) -> String {
    format!(
        "p.account_id = {account} AND ((p.kind = 'sender' AND p.key = {addr}) \
         OR (p.kind = 'domain' AND ({domain} = p.key \
         OR substr({domain}, -length(p.key) - 1) = '.' || p.key)))"
    )
}

pub(crate) fn pinned_sql(account: &str, addr: &str, domain: &str) -> String {
    format!(
        "EXISTS (SELECT 1 FROM pinned_senders p WHERE {})",
        covering_sql(account, addr, domain)
    )
}

pub(crate) fn pin_sql(account: &str, addr: &str, domain: &str) -> String {
    format!(
        "(SELECT p.kind || char(31) || p.key FROM pinned_senders p WHERE {} \
         ORDER BY p.kind = 'sender' DESC, length(p.key) DESC LIMIT 1)",
        covering_sql(account, addr, domain)
    )
}

pub(crate) fn decode(raw: Option<String>) -> Option<Pin> {
    let raw = raw?;
    let (kind, key) = raw.split_once('\u{1f}')?;
    Some(Pin {
        kind: kind.parse().ok()?,
        key: key.to_string(),
    })
}

pub fn list(conn: &Connection, account_id: i64) -> Result<Vec<Pin>> {
    let mut stmt = conn.prepare_cached(
        "SELECT kind, key FROM pinned_senders WHERE account_id = ?1 ORDER BY kind DESC, key",
    )?;
    let rows = stmt
        .query_map([account_id], |row| {
            let kind: String = row.get(0)?;
            Ok(Pin {
                kind: kind.parse().unwrap_or(PinKind::Sender),
                key: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn add(conn: &Connection, account_id: i64, kind: PinKind, key: &str) -> Result<Pin> {
    let key = normalize_key(key);
    conn.execute(
        "INSERT OR IGNORE INTO pinned_senders (account_id, kind, key, created_at) VALUES (?1, ?2, ?3, ?4)",
        params![account_id, kind.as_str(), key, now()],
    )?;
    Ok(Pin { kind, key })
}

pub fn remove(conn: &Connection, account_id: i64, kind: PinKind, key: &str) -> Result<()> {
    match conn.execute(
        "DELETE FROM pinned_senders WHERE account_id = ?1 AND kind = ?2 AND key = ?3",
        params![account_id, kind.as_str(), normalize_key(key)],
    )? {
        0 => Err(Error::NotFound),
        _ => Ok(()),
    }
}
