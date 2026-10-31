use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use mail_sieve_store::{Connection, Store, Transaction};
use serde::Serialize;
use tokio::sync::{Notify, broadcast};

use crate::config::Config;
use crate::error::Result;
use crate::oauth::PendingAuthorization;
use crate::vault::Vault;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum Event {
    Job { job: mail_sieve_store::jobs::Job },
    Account { account_id: i64 },
}

#[derive(Clone)]
pub struct AppState {
    pub inner: Arc<Inner>,
}

pub struct Inner {
    pub config: Config,
    pub store: Store,
    pub vault: Vault,
    pub events: broadcast::Sender<Event>,
    pub jobs_wakeup: Notify,
    pub http: reqwest::Client,
    pub pending_oauth: Mutex<HashMap<String, PendingAuthorization>>,
}

impl std::ops::Deref for AppState {
    type Target = Inner;

    fn deref(&self) -> &Inner {
        &self.inner
    }
}

impl AppState {
    pub fn new(config: Config, store: Store, vault: Vault) -> AppState {
        let (events, _) = broadcast::channel(256);
        let http = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(30))
            .user_agent(concat!("mail-sieve/", env!("CARGO_PKG_VERSION")))
            .build()
            .expect("http client");
        AppState {
            inner: Arc::new(Inner {
                config,
                store,
                vault,
                events,
                jobs_wakeup: Notify::new(),
                http,
                pending_oauth: Mutex::new(HashMap::new()),
            }),
        }
    }

    pub async fn read<T, F>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Connection) -> mail_sieve_store::Result<T> + Send + 'static,
    {
        let store = self.store.clone();
        Ok(tokio::task::spawn_blocking(move || store.read(f)).await??)
    }

    pub async fn write<T, F>(&self, f: F) -> Result<T>
    where
        T: Send + 'static,
        F: FnOnce(&Transaction) -> mail_sieve_store::Result<T> + Send + 'static,
    {
        let store = self.store.clone();
        Ok(tokio::task::spawn_blocking(move || store.write(f)).await??)
    }

    pub fn emit(&self, event: Event) {
        let _ = self.events.send(event);
    }

    pub async fn emit_job(&self, job_id: i64) {
        if let Ok(job) = self
            .read(move |conn| mail_sieve_store::jobs::get(conn, job_id))
            .await
        {
            self.emit(Event::Job { job });
        }
    }
}
