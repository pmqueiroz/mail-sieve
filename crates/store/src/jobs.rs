use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::search::Target;
use crate::{Error, Result, now};

pub const QUEUED: &str = "queued";
pub const RUNNING: &str = "running";
pub const DONE: &str = "done";
pub const FAILED: &str = "failed";
pub const CANCELLED: &str = "cancelled";

#[derive(Debug, Clone, Serialize)]
pub struct Job {
    pub id: i64,
    pub account_id: Option<i64>,
    pub kind: String,
    pub state: String,
    pub payload: serde_json::Value,
    pub total: u64,
    pub done: u64,
    pub error: Option<String>,
    pub created_at: i64,
    pub started_at: Option<i64>,
    pub finished_at: Option<i64>,
}

const COLUMNS: &str =
    "id, account_id, kind, state, payload, total, done, error, created_at, started_at, finished_at";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Job> {
    let payload: String = row.get(4)?;
    Ok(Job {
        id: row.get(0)?,
        account_id: row.get(1)?,
        kind: row.get(2)?,
        state: row.get(3)?,
        payload: serde_json::from_str(&payload).unwrap_or(serde_json::Value::Null),
        total: row.get::<_, i64>(5)? as u64,
        done: row.get::<_, i64>(6)? as u64,
        error: row.get(7)?,
        created_at: row.get(8)?,
        started_at: row.get(9)?,
        finished_at: row.get(10)?,
    })
}

pub fn create(
    conn: &Connection,
    account_id: Option<i64>,
    kind: &str,
    payload: &serde_json::Value,
) -> Result<i64> {
    conn.execute(
        "INSERT INTO jobs (account_id, kind, state, payload, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
        params![account_id, kind, QUEUED, payload.to_string(), now()],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn add_targets(conn: &Connection, job_id: i64, targets: &[Target]) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "INSERT OR IGNORE INTO job_targets (job_id, mailbox_id, uid) VALUES (?1, ?2, ?3)",
    )?;
    for target in targets {
        stmt.execute(params![job_id, target.mailbox_id, target.uid])?;
    }
    conn.execute(
        "UPDATE jobs SET total = (SELECT COUNT(*) FROM job_targets WHERE job_id = ?1) WHERE id = ?1",
        [job_id],
    )?;
    Ok(())
}

pub fn remaining_targets(conn: &Connection, job_id: i64) -> Result<Vec<Target>> {
    let mut stmt = conn.prepare_cached(
        "SELECT mailbox_id, uid FROM job_targets WHERE job_id = ?1 ORDER BY mailbox_id, uid",
    )?;
    let rows = stmt
        .query_map([job_id], |row| {
            Ok(Target {
                mailbox_id: row.get(0)?,
                uid: row.get(1)?,
            })
        })?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn complete_targets(
    conn: &Connection,
    job_id: i64,
    mailbox_id: i64,
    uids: &[u32],
) -> Result<()> {
    let mut stmt = conn.prepare_cached(
        "DELETE FROM job_targets WHERE job_id = ?1 AND mailbox_id = ?2 AND uid = ?3",
    )?;
    for uid in uids {
        stmt.execute(params![job_id, mailbox_id, uid])?;
    }
    conn.execute(
        "UPDATE jobs SET done = total - (SELECT COUNT(*) FROM job_targets WHERE job_id = ?1) WHERE id = ?1",
        [job_id],
    )?;
    Ok(())
}

pub fn get(conn: &Connection, id: i64) -> Result<Job> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM jobs WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(Error::NotFound)
}

pub fn list(conn: &Connection, limit: u32) -> Result<Vec<Job>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM jobs ORDER BY id DESC LIMIT ?1"
    ))?;
    let rows = stmt
        .query_map([limit], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn queued(conn: &Connection) -> Result<Vec<Job>> {
    let mut stmt = conn.prepare_cached(&format!(
        "SELECT {COLUMNS} FROM jobs WHERE state = ?1 ORDER BY id"
    ))?;
    let rows = stmt
        .query_map([QUEUED], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn has_active(conn: &Connection, account_id: i64, kind: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM jobs WHERE account_id = ?1 AND kind = ?2 AND state IN (?3, ?4))",
        params![account_id, kind, QUEUED, RUNNING],
        |row| row.get(0),
    )?)
}

pub fn requeue_interrupted(conn: &Connection) -> Result<usize> {
    Ok(conn.execute(
        "UPDATE jobs SET state = ?1 WHERE state = ?2",
        [QUEUED, RUNNING],
    )?)
}

pub fn mark_running(conn: &Connection, id: i64) -> Result<()> {
    conn.execute(
        "UPDATE jobs SET state = ?2, started_at = COALESCE(started_at, ?3) WHERE id = ?1",
        params![id, RUNNING, now()],
    )?;
    Ok(())
}

pub fn set_progress(conn: &Connection, id: i64, done: u64, total: u64) -> Result<()> {
    conn.execute(
        "UPDATE jobs SET done = ?2, total = ?3 WHERE id = ?1",
        params![id, done as i64, total as i64],
    )?;
    Ok(())
}

pub fn finish(conn: &Connection, id: i64, state: &str, error: Option<&str>) -> Result<()> {
    conn.execute(
        "UPDATE jobs SET state = ?2, error = ?3, finished_at = ?4 WHERE id = ?1 AND state != ?5",
        params![id, state, error, now(), CANCELLED],
    )?;
    if state != FAILED {
        conn.execute("DELETE FROM job_targets WHERE job_id = ?1", [id])?;
    }
    Ok(())
}

pub fn cancel(conn: &Connection, id: i64) -> Result<bool> {
    let changed = conn.execute(
        "UPDATE jobs SET state = ?2, finished_at = ?3 WHERE id = ?1 AND state IN (?4, ?5)",
        params![id, CANCELLED, now(), QUEUED, RUNNING],
    )?;
    Ok(changed > 0)
}

pub fn is_cancelled(conn: &Connection, id: i64) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT state = ?2 FROM jobs WHERE id = ?1",
        params![id, CANCELLED],
        |row| row.get(0),
    )?)
}

pub fn prune_finished(conn: &Connection, older_than: i64) -> Result<usize> {
    Ok(conn.execute(
        "DELETE FROM jobs WHERE state IN (?1, ?2, ?3) AND finished_at < ?4",
        params![DONE, FAILED, CANCELLED, older_than],
    )?)
}

pub struct ActionLogEntry<'a> {
    pub job_id: Option<i64>,
    pub account_id: i64,
    pub action: &'a str,
    pub mailbox: &'a str,
    pub uids: &'a str,
    pub count: usize,
}

pub fn log_action(conn: &Connection, entry: &ActionLogEntry<'_>) -> Result<()> {
    conn.execute(
        "INSERT INTO action_log (job_id, account_id, action, mailbox, uids, count, at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            entry.job_id,
            entry.account_id,
            entry.action,
            entry.mailbox,
            entry.uids,
            entry.count as i64,
            now()
        ],
    )?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    #[test]
    fn tracks_targets_and_progress() {
        let store = test_support::store();
        let account = test_support::account(&store);
        let job = store
            .write(|tx| {
                let job = create(
                    tx,
                    Some(account),
                    "action",
                    &serde_json::json!({"action": "trash"}),
                )?;
                add_targets(
                    tx,
                    job,
                    &[
                        Target {
                            mailbox_id: 1,
                            uid: 1,
                        },
                        Target {
                            mailbox_id: 1,
                            uid: 2,
                        },
                    ],
                )?;
                complete_targets(tx, job, 1, &[1])?;
                Ok(job)
            })
            .unwrap();
        let (job, remaining) = store
            .read(|conn| Ok((get(conn, job)?, remaining_targets(conn, job)?)))
            .unwrap();
        assert_eq!((job.total, job.done), (2, 1));
        assert_eq!(
            remaining,
            vec![Target {
                mailbox_id: 1,
                uid: 2
            }]
        );
        assert_eq!(job.payload["action"], "trash");

        store.write(|tx| mark_running(tx, job.id)).unwrap();
        assert_eq!(store.write(|tx| requeue_interrupted(tx)).unwrap(), 1);
        assert!(store.write(|tx| cancel(tx, job.id)).unwrap());
        assert!(store.read(|conn| is_cancelled(conn, job.id)).unwrap());
        assert!(!store.write(|tx| cancel(tx, job.id)).unwrap());
        store.write(|tx| finish(tx, job.id, DONE, None)).unwrap();
        assert_eq!(
            store.read(|conn| get(conn, job.id)).unwrap().state,
            CANCELLED
        );
    }
}
