use std::collections::HashSet;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use mail_sieve_core::{MessageMeta, UidSet};
use mail_sieve_imap::{
    Connection, HeaderFetchPlan, ImapConfig, connect, fetch_flags, fetch_headers_parallel,
    search_all_uids,
};
use mail_sieve_store::mailboxes::{Mailbox, SyncState};
use mail_sieve_store::messages::{self, FlagUpdate};
use tokio::sync::mpsc;

use crate::accounts;
use crate::error::Result;
use crate::state::{AppState, Event};

const HEADER_CHUNK: usize = 500;
const FLAG_CHUNK_BYTES: usize = 4_000;
const PROGRESS_INTERVAL: Duration = Duration::from_millis(300);

#[derive(Debug, Default, Clone, Copy)]
pub struct SyncReport {
    pub mailboxes: usize,
    pub fetched: u64,
    pub removed: u64,
    pub flags_updated: u64,
    pub elapsed: Duration,
}

struct Progress {
    state: AppState,
    job_id: Option<i64>,
    done: AtomicU64,
    total: AtomicU64,
    last_emit: std::sync::Mutex<Instant>,
}

impl Progress {
    async fn report(&self, force: bool) {
        let Some(job_id) = self.job_id else { return };
        {
            let mut last = self.last_emit.lock().unwrap_or_else(|p| p.into_inner());
            if !force && last.elapsed() < PROGRESS_INTERVAL {
                return;
            }
            *last = Instant::now();
        }
        let done = self.done.load(Ordering::Relaxed);
        let total = self.total.load(Ordering::Relaxed);
        if self
            .state
            .write(move |tx| mail_sieve_store::jobs::set_progress(tx, job_id, done, total))
            .await
            .is_ok()
        {
            self.state.emit_job(job_id).await;
        }
    }
}

pub async fn sync_account(
    state: &AppState,
    account_id: i64,
    job_id: Option<i64>,
) -> Result<SyncReport> {
    let started = Instant::now();
    let result = run(state, account_id, job_id).await;
    let error = result.as_ref().err().map(ToString::to_string);
    state
        .write(move |tx| {
            mail_sieve_store::accounts::set_sync_result(tx, account_id, error.as_deref())
        })
        .await?;
    state.emit(Event::Account { account_id });
    result.map(|report| SyncReport {
        elapsed: started.elapsed(),
        ..report
    })
}

async fn run(state: &AppState, account_id: i64, job_id: Option<i64>) -> Result<SyncReport> {
    let account = accounts::load(state, account_id).await?;
    let config = accounts::imap_config(state, &account).await?;
    let mut conn = connect(&config).await?;
    accounts::refresh_folders(state, account_id, &mut conn).await?;
    let mailboxes: Vec<Mailbox> = state
        .read(move |c| mail_sieve_store::mailboxes::list(c, account_id))
        .await?
        .into_iter()
        .filter(|mailbox| mailbox.sync_enabled)
        .collect();
    let progress = Arc::new(Progress {
        state: state.clone(),
        job_id,
        done: AtomicU64::new(0),
        total: AtomicU64::new(0),
        last_emit: std::sync::Mutex::new(Instant::now()),
    });
    let mut report = SyncReport {
        mailboxes: mailboxes.len(),
        ..Default::default()
    };
    for mailbox in &mailboxes {
        if let Some(job_id) = job_id
            && state
                .read(move |c| mail_sieve_store::jobs::is_cancelled(c, job_id))
                .await?
        {
            break;
        }
        let connections = account.max_connections.max(1) as usize;
        sync_mailbox(
            state,
            &config,
            &mut conn,
            account_id,
            mailbox,
            connections,
            &progress,
            &mut report,
        )
        .await?;
    }
    progress.report(true).await;
    conn.logout().await;
    Ok(report)
}

#[allow(clippy::too_many_arguments)]
async fn sync_mailbox(
    state: &AppState,
    config: &ImapConfig,
    conn: &mut Connection,
    account_id: i64,
    mailbox: &Mailbox,
    connections: usize,
    progress: &Arc<Progress>,
    report: &mut SyncReport,
) -> Result<()> {
    let mailbox_id = mailbox.id;
    let status = conn.examine(&mailbox.name).await?;
    let mut known_modseq = mailbox.highestmodseq;
    if mailbox
        .uidvalidity
        .is_some_and(|uidvalidity| uidvalidity != status.uidvalidity)
    {
        tracing::warn!(mailbox = %mailbox.name, "UIDVALIDITY changed, rebuilding local index");
        state
            .write(move |tx| mail_sieve_store::mailboxes::reset(tx, mailbox_id))
            .await?;
        known_modseq = None;
    }
    let unchanged = known_modseq.is_some()
        && known_modseq == status.highestmodseq
        && mailbox.uidnext == Some(status.uidnext)
        && mailbox.uidvalidity == Some(status.uidvalidity);
    if unchanged {
        let local_count = mailbox.message_count;
        if local_count == u64::from(status.exists) {
            save_state(
                state,
                mailbox_id,
                status.uidvalidity,
                status.uidnext,
                status.highestmodseq,
            )
            .await?;
            return Ok(());
        }
    }

    let remote = search_all_uids(conn).await?;
    let local = state
        .read(move |c| messages::local_uids(c, mailbox_id))
        .await?;
    let remote_set: HashSet<u32> = remote.iter().copied().collect();
    let local_set: HashSet<u32> = local.iter().copied().collect();
    let vanished: Vec<u32> = local
        .iter()
        .copied()
        .filter(|uid| !remote_set.contains(uid))
        .collect();
    let fresh: Vec<u32> = remote
        .iter()
        .copied()
        .filter(|uid| !local_set.contains(uid))
        .collect();
    let existing: Vec<u32> = local
        .iter()
        .copied()
        .filter(|uid| remote_set.contains(uid))
        .collect();

    if !vanished.is_empty() {
        report.removed += vanished.len() as u64;
        state
            .write(move |tx| messages::delete_uids(tx, mailbox_id, &vanished))
            .await?;
    }

    if !existing.is_empty() {
        let changes = match known_modseq {
            Some(modseq) if conn.capabilities.condstore() => {
                fetch_flags(conn, "1:*", Some(modseq)).await?
            }
            _ => {
                let mut all = Vec::new();
                for chunk in UidSet::from_uids(existing.iter().copied())
                    .chunks_by_encoded_len(FLAG_CHUNK_BYTES)
                {
                    all.extend(fetch_flags(conn, &chunk.to_string(), None).await?);
                }
                all
            }
        };
        let updates: Vec<FlagUpdate> = changes
            .into_iter()
            .filter(|change| local_set.contains(&change.uid))
            .map(|change| FlagUpdate {
                uid: change.uid,
                flags: change.flags,
                labels: change.labels,
            })
            .collect();
        if !updates.is_empty() {
            report.flags_updated += state
                .write(move |tx| messages::update_flags(tx, mailbox_id, &updates))
                .await? as u64;
        }
    }

    if !fresh.is_empty() {
        progress
            .total
            .fetch_add(fresh.len() as u64, Ordering::Relaxed);
        progress.report(true).await;
        report.fetched += fetch_new(
            state,
            config,
            account_id,
            mailbox,
            status.uidvalidity,
            &fresh,
            connections,
            progress,
        )
        .await?;
    }

    save_state(
        state,
        mailbox_id,
        status.uidvalidity,
        status.uidnext,
        status.highestmodseq,
    )
    .await
}

async fn save_state(
    state: &AppState,
    mailbox_id: i64,
    uidvalidity: u32,
    uidnext: u32,
    highestmodseq: Option<u64>,
) -> Result<()> {
    let sync_state = SyncState {
        uidvalidity,
        uidnext,
        highestmodseq,
    };
    state
        .write(move |tx| mail_sieve_store::mailboxes::save_sync_state(tx, mailbox_id, sync_state))
        .await
}

#[allow(clippy::too_many_arguments)]
async fn fetch_new(
    state: &AppState,
    config: &ImapConfig,
    account_id: i64,
    mailbox: &Mailbox,
    uidvalidity: u32,
    fresh: &[u32],
    connections: usize,
    progress: &Arc<Progress>,
) -> Result<u64> {
    let uids = UidSet::from_uids(fresh.iter().copied());
    let (sender, mut receiver) = mpsc::channel::<Vec<MessageMeta>>(connections * 2);
    let mailbox_id = mailbox.id;
    let writer_state = state.clone();
    let writer_progress = progress.clone();
    let writer = tokio::spawn(async move {
        let mut written = 0u64;
        while let Some(batch) = receiver.recv().await {
            let count = batch.len() as u64;
            writer_state
                .write(move |tx| messages::upsert_batch(tx, account_id, mailbox_id, &batch))
                .await?;
            written += count;
            writer_progress.done.fetch_add(count, Ordering::Relaxed);
            writer_progress.report(false).await;
        }
        Ok::<u64, crate::error::Error>(written)
    });
    let plan = HeaderFetchPlan {
        config,
        mailbox: &mailbox.name,
        uidvalidity,
        uids: &uids,
        connections,
        chunk_size: HEADER_CHUNK,
    };
    let fetched = fetch_headers_parallel(&plan, sender).await;
    let written = writer.await??;
    fetched?;
    Ok(written)
}
