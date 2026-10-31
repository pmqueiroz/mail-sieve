use std::collections::HashMap;
use std::time::Duration;

use tokio::task::JoinHandle;

use crate::state::AppState;
use crate::{idle, jobs, rules};

const TICK: Duration = Duration::from_secs(60);
const JOB_RETENTION_SECONDS: i64 = 30 * 86_400;

pub fn spawn(state: AppState) {
    tokio::spawn(async move {
        let mut watchers: HashMap<i64, JoinHandle<()>> = HashMap::new();
        loop {
            if let Err(error) = tick(&state, &mut watchers).await {
                tracing::warn!(%error, "scheduler tick failed");
            }
            tokio::time::sleep(TICK).await;
        }
    });
}

async fn tick(
    state: &AppState,
    watchers: &mut HashMap<i64, JoinHandle<()>>,
) -> crate::error::Result<()> {
    let now = mail_sieve_store::now();
    let accounts = state.read(mail_sieve_store::accounts::list).await?;
    for account in &accounts {
        let interval = i64::from(account.sync_interval_minutes.max(1)) * 60;
        let due = account
            .last_synced_at
            .is_none_or(|last| last + interval <= now);
        if due && account.sync_interval_minutes > 0 {
            jobs::enqueue_sync(state, account.id).await?;
        }
    }
    watchers.retain(|account_id, handle| {
        let keep =
            accounts.iter().any(|account| account.id == *account_id) && !handle.is_finished();
        if !keep {
            handle.abort();
        }
        keep
    });
    for account in &accounts {
        if account.last_synced_at.is_some() && !watchers.contains_key(&account.id) {
            watchers.insert(account.id, idle::spawn_watcher(state.clone(), account.id));
        }
    }
    rules::run_due(state).await?;
    state
        .write(|tx| mail_sieve_store::sessions::purge_expired(tx))
        .await?;
    state
        .write(move |tx| mail_sieve_store::jobs::prune_finished(tx, now - JOB_RETENTION_SECONDS))
        .await?;
    Ok(())
}
