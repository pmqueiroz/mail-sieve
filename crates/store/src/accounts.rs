use rusqlite::{Connection, OptionalExtension, Row, params};
use serde::Serialize;

use crate::{Error, Result, now};

#[derive(Debug, Clone, Serialize)]
pub struct Account {
    pub id: i64,
    pub label: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub username: String,
    pub auth_kind: String,
    pub oauth_provider: Option<String>,
    #[serde(skip)]
    pub credentials: Vec<u8>,
    pub provider: String,
    pub max_connections: u32,
    pub capabilities: String,
    pub sync_interval_minutes: u32,
    pub created_at: i64,
    pub last_synced_at: Option<i64>,
    pub last_error: Option<String>,
}

#[derive(Debug, Clone)]
pub struct NewAccount {
    pub label: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub imap_tls: bool,
    pub smtp_host: Option<String>,
    pub smtp_port: Option<u16>,
    pub username: String,
    pub auth_kind: String,
    pub oauth_provider: Option<String>,
    pub credentials: Vec<u8>,
    pub provider: String,
    pub max_connections: u32,
}

#[derive(Debug, Clone, Default)]
pub struct AccountSettings {
    pub label: Option<String>,
    pub max_connections: Option<u32>,
    pub sync_interval_minutes: Option<u32>,
    pub smtp_host: Option<Option<String>>,
    pub smtp_port: Option<Option<u16>>,
}

const COLUMNS: &str = "id, label, email, imap_host, imap_port, imap_tls, smtp_host, smtp_port, username, \
    auth_kind, oauth_provider, credentials, provider, max_connections, capabilities, \
    sync_interval_minutes, created_at, last_synced_at, last_error";

fn from_row(row: &Row<'_>) -> rusqlite::Result<Account> {
    Ok(Account {
        id: row.get(0)?,
        label: row.get(1)?,
        email: row.get(2)?,
        imap_host: row.get(3)?,
        imap_port: row.get(4)?,
        imap_tls: row.get(5)?,
        smtp_host: row.get(6)?,
        smtp_port: row.get(7)?,
        username: row.get(8)?,
        auth_kind: row.get(9)?,
        oauth_provider: row.get(10)?,
        credentials: row.get(11)?,
        provider: row.get(12)?,
        max_connections: row.get(13)?,
        capabilities: row.get(14)?,
        sync_interval_minutes: row.get(15)?,
        created_at: row.get(16)?,
        last_synced_at: row.get(17)?,
        last_error: row.get(18)?,
    })
}

pub fn insert(conn: &Connection, account: &NewAccount) -> Result<i64> {
    conn.execute(
        "INSERT INTO accounts (label, email, imap_host, imap_port, imap_tls, smtp_host, smtp_port, \
         username, auth_kind, oauth_provider, credentials, provider, max_connections, created_at) \
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            account.label,
            account.email,
            account.imap_host,
            account.imap_port,
            account.imap_tls,
            account.smtp_host,
            account.smtp_port,
            account.username,
            account.auth_kind,
            account.oauth_provider,
            account.credentials,
            account.provider,
            account.max_connections,
            now(),
        ],
    )?;
    Ok(conn.last_insert_rowid())
}

pub fn get(conn: &Connection, id: i64) -> Result<Account> {
    conn.query_row(
        &format!("SELECT {COLUMNS} FROM accounts WHERE id = ?1"),
        [id],
        from_row,
    )
    .optional()?
    .ok_or(Error::NotFound)
}

pub fn list(conn: &Connection) -> Result<Vec<Account>> {
    let mut stmt = conn.prepare_cached(&format!("SELECT {COLUMNS} FROM accounts ORDER BY id"))?;
    let rows = stmt
        .query_map([], from_row)?
        .collect::<rusqlite::Result<_>>()?;
    Ok(rows)
}

pub fn delete(conn: &Connection, id: i64) -> Result<()> {
    match conn.execute("DELETE FROM accounts WHERE id = ?1", [id])? {
        0 => Err(Error::NotFound),
        _ => Ok(()),
    }
}

pub fn update_settings(conn: &Connection, id: i64, settings: &AccountSettings) -> Result<()> {
    let current = get(conn, id)?;
    conn.execute(
        "UPDATE accounts SET label = ?2, max_connections = ?3, sync_interval_minutes = ?4, \
         smtp_host = ?5, smtp_port = ?6 WHERE id = ?1",
        params![
            id,
            settings.label.as_ref().unwrap_or(&current.label),
            settings.max_connections.unwrap_or(current.max_connections),
            settings
                .sync_interval_minutes
                .unwrap_or(current.sync_interval_minutes),
            settings.smtp_host.clone().unwrap_or(current.smtp_host),
            settings.smtp_port.unwrap_or(current.smtp_port),
        ],
    )?;
    Ok(())
}

pub fn update_credentials(conn: &Connection, id: i64, credentials: &[u8]) -> Result<()> {
    conn.execute(
        "UPDATE accounts SET credentials = ?2 WHERE id = ?1",
        params![id, credentials],
    )?;
    Ok(())
}

pub fn set_server_info(
    conn: &Connection,
    id: i64,
    provider: &str,
    capabilities: &str,
) -> Result<()> {
    conn.execute(
        "UPDATE accounts SET provider = ?2, capabilities = ?3 WHERE id = ?1",
        params![id, provider, capabilities],
    )?;
    Ok(())
}

pub fn set_sync_result(conn: &Connection, id: i64, error: Option<&str>) -> Result<()> {
    match error {
        None => conn.execute(
            "UPDATE accounts SET last_synced_at = ?2, last_error = NULL WHERE id = ?1",
            params![id, now()],
        )?,
        Some(error) => conn.execute(
            "UPDATE accounts SET last_error = ?2 WHERE id = ?1",
            params![id, error],
        )?,
    };
    Ok(())
}
