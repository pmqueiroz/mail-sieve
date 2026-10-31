pub mod accounts;
mod error;
pub mod mailboxes;
pub mod messages;
pub mod search;

use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex, MutexGuard};
use std::time::{SystemTime, UNIX_EPOCH};

pub use error::{Error, Result};
pub use rusqlite::{Connection, Transaction};
use rusqlite::{OpenFlags, TransactionBehavior};

const MIGRATIONS: &[&str] = &[include_str!("schema.sql")];
const MAX_IDLE_READERS: usize = 8;

#[derive(Clone)]
pub struct Store {
    inner: Arc<Inner>,
}

struct Inner {
    path: PathBuf,
    writer: Mutex<Connection>,
    readers: Mutex<Vec<Connection>>,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Store> {
        let path = path.as_ref().to_path_buf();
        let mut writer = open_connection(&path, false)?;
        migrate(&mut writer)?;
        Ok(Store {
            inner: Arc::new(Inner {
                path,
                writer: Mutex::new(writer),
                readers: Mutex::new(Vec::new()),
            }),
        })
    }

    pub fn read<T>(&self, f: impl FnOnce(&Connection) -> Result<T>) -> Result<T> {
        let idle = lock(&self.inner.readers).pop();
        let conn = match idle {
            Some(conn) => conn,
            None => open_connection(&self.inner.path, true)?,
        };
        let result = f(&conn);
        let mut readers = lock(&self.inner.readers);
        if readers.len() < MAX_IDLE_READERS {
            readers.push(conn);
        }
        result
    }

    pub fn write<T>(&self, f: impl FnOnce(&Transaction) -> Result<T>) -> Result<T> {
        let mut conn = lock(&self.inner.writer);
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let value = f(&tx)?;
        tx.commit()?;
        Ok(value)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> MutexGuard<'_, T> {
    mutex
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
}

fn open_connection(path: &Path, read_only: bool) -> Result<Connection> {
    let flags = if read_only {
        OpenFlags::SQLITE_OPEN_READ_ONLY
            | OpenFlags::SQLITE_OPEN_NO_MUTEX
            | OpenFlags::SQLITE_OPEN_URI
    } else {
        OpenFlags::default()
    };
    let conn = Connection::open_with_flags(path, flags)?;
    conn.busy_timeout(std::time::Duration::from_secs(10))?;
    conn.execute_batch(
        "PRAGMA foreign_keys = ON;
         PRAGMA synchronous = NORMAL;
         PRAGMA temp_store = MEMORY;
         PRAGMA cache_size = -65536;
         PRAGMA mmap_size = 268435456;",
    )?;
    if !read_only {
        conn.pragma_update(None, "journal_mode", "WAL")?;
    }
    conn.set_prepared_statement_cache_capacity(64);
    Ok(conn)
}

fn migrate(conn: &mut Connection) -> Result<()> {
    let current: i64 = conn.pragma_query_value(None, "user_version", |row| row.get(0))?;
    for (index, migration) in MIGRATIONS.iter().enumerate().skip(current as usize) {
        let tx = conn.transaction()?;
        tx.execute_batch(migration)?;
        tx.pragma_update(None, "user_version", index as i64 + 1)?;
        tx.commit()?;
    }
    Ok(())
}

pub fn now() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |d| d.as_secs() as i64)
}

#[cfg(test)]
pub(crate) mod test_support {
    use super::*;

    pub struct TestStore {
        pub store: Store,
        _dir: tempfile::TempDir,
    }

    impl std::ops::Deref for TestStore {
        type Target = Store;

        fn deref(&self) -> &Store {
            &self.store
        }
    }

    pub fn store() -> TestStore {
        let dir = tempfile::tempdir().unwrap();
        let store = Store::open(dir.path().join("test.db")).unwrap();
        TestStore { store, _dir: dir }
    }

    pub fn account(store: &Store) -> i64 {
        store
            .write(|tx| {
                accounts::insert(
                    tx,
                    &accounts::NewAccount {
                        label: "Test".into(),
                        email: "me@example.com".into(),
                        imap_host: "imap.example.com".into(),
                        imap_port: 993,
                        imap_tls: true,
                        smtp_host: None,
                        smtp_port: None,
                        username: "me@example.com".into(),
                        auth_kind: "password".into(),
                        oauth_provider: None,
                        credentials: vec![1, 2, 3],
                        provider: "generic".into(),
                        max_connections: 4,
                    },
                )
            })
            .unwrap()
    }

    pub fn mailbox(store: &Store, account_id: i64, name: &str) -> i64 {
        store
            .write(|tx| {
                tx.execute(
                    "INSERT INTO mailboxes (account_id, name, delimiter) VALUES (?1, ?2, '/')",
                    rusqlite::params![account_id, name],
                )?;
                Ok(tx.last_insert_rowid())
            })
            .unwrap()
    }
}
