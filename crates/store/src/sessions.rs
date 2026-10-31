use rusqlite::{Connection, params};

use crate::{Result, now};

pub fn insert(conn: &Connection, token_hash: &str, ttl_seconds: i64) -> Result<()> {
    let created = now();
    conn.execute(
        "INSERT INTO sessions (token_hash, created_at, expires_at) VALUES (?1, ?2, ?3)",
        params![token_hash, created, created + ttl_seconds],
    )?;
    Ok(())
}

pub fn is_valid(conn: &Connection, token_hash: &str) -> Result<bool> {
    Ok(conn.query_row(
        "SELECT EXISTS (SELECT 1 FROM sessions WHERE token_hash = ?1 AND expires_at > ?2)",
        params![token_hash, now()],
        |row| row.get(0),
    )?)
}

pub fn delete(conn: &Connection, token_hash: &str) -> Result<()> {
    conn.execute("DELETE FROM sessions WHERE token_hash = ?1", [token_hash])?;
    Ok(())
}

pub fn delete_all(conn: &Connection) -> Result<()> {
    conn.execute("DELETE FROM sessions", [])?;
    Ok(())
}

pub fn purge_expired(conn: &Connection) -> Result<usize> {
    Ok(conn.execute("DELETE FROM sessions WHERE expires_at <= ?1", [now()])?)
}
