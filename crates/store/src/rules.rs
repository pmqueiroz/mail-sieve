use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::{Error, Result, now};

#[derive(Debug, Clone, Serialize)]
pub struct Rule {
    pub id: i64,
    pub account_id: i64,
    pub name: String,
    pub query: String,
    pub action: String,
    pub enabled: bool,
    pub interval_minutes: Option<u32>,
    pub run_after_sync: bool,
    pub last_run_at: Option<i64>,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct RuleInput {
    pub account_id: i64,
    pub name: String,
    pub query: String,
    pub action: String,
    pub enabled: bool,
    pub interval_minutes: Option<u32>,
    pub run_after_sync: bool,
}

const COLUMNS: &str = "id, account_id, name, query, action, enabled, interval_minutes, run_after_sync, last_run_at, created_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Rule> {
    Ok(Rule {
        id: row.get(0)?,
        account_id: row.get(1)?,
        name: row.get(2)?,
        query: row.get(3)?,
        action: row.get(4)?,
        enabled: row.get(5)?,
        interval_minutes: row.get(6)?,
        run_after_sync: row.get(7)?,
        last_run_at: row.get(8)?,
        created_at: row.get(9)?,
    })
}

pub fn insert(conn: &Connection, rule: &RuleInput) -> Result<i64> {
    conn.execute(
        "INSERT INTO rules (account_id, name, query, action, enabled, interval_minutes, run_after_sync, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
        params![
            rule.account_id,
            rule.name,
            rule.query,
            rule.action,
            rule.enabled,
            rule.interval_minutes,
            rule.run_after_sync,
            now()
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn update(conn: &Connection, id: i64, rule: &RuleInput) -> Result<()> {
    let changed = conn.execute(
        "UPDATE rules SET account_id = ?2, name = ?3, query = ?4, action = ?5, enabled = ?6, \
         interval_minutes = ?7, run_after_sync = ?8 WHERE id = ?1",
        params![
            id,
            rule.account_id,
            rule.name,
            rule.query,
            rule.action,
            rule.enabled,
            rule.interval_minutes,
            rule.run_after_sync
        ],
    )?;
    if changed == 0 {
        Err(Error::NotFound)
    } else {
        Ok(())
    }
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    match conn.execute("DELETE FROM rules WHERE id = ?1", [id])? {
        0 => Err(Error::NotFound),
        _ => Ok(()),
    }
}

pub fn get(conn: &Connection, id: i64) -> Result<Rule> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM rules WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(Error::NotFound)
}

pub fn list(conn: &Connection) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare_cached(&format!("SELECT {COLUMNS} FROM rules ORDER BY id"))?;
    let rows = stmt
        .query_map([], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn due(conn: &Connection, at: i64) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM rules WHERE enabled = 1 AND interval_minutes IS NOT NULL \
         AND COALESCE(last_run_at, 0) + interval_minutes * 60 <= ?1 ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([at], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn after_sync(conn: &Connection, account_id: i64) -> Result<Vec<Rule>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM rules WHERE enabled = 1 AND run_after_sync = 1 AND account_id = ?1 ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([account_id], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn mark_run(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE rules SET last_run_at = ?2 WHERE id = ?1",
        params![id, now()],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[test]
    fn selects_due_rules() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let input = |name: &str, interval: Option<u32>, after_sync: bool| RuleInput {
            account_id: account,
            name: name.into(),
            query: "older_than:30d has:unsubscribe".into(),
            action: "archive".into(),
            enabled: true,
            interval_minutes: interval,
            run_after_sync: after_sync,
        };
        let hourly = store
            .write(|tx| insert(tx, &input("hourly", Some(60), false)))
            .unwrap();
        store
            .write(|tx| insert(tx, &input("on sync", None, true)))
            .unwrap();
        let names = |rules: Vec<Rule>| rules.into_iter().map(|r| r.name).collect::<Vec<_>>();
        assert_eq!(names(store.read(|c| due(c, now())).unwrap()), ["hourly"]);
        store.write(|tx| mark_run(tx, hourly)).unwrap();
        assert!(store.read(|c| due(c, now())).unwrap().is_empty());
        assert_eq!(
            names(store.read(|c| due(c, now() + 3600)).unwrap()),
            ["hourly"]
        );
        assert_eq!(
            names(store.read(|c| after_sync(c, account)).unwrap()),
            ["on sync"]
        );
    }
}
