use std::collections::VecDeque;
use std::sync::Mutex;

use mail_sieve_core::{MessageMeta, UidSet};
use tokio::sync::mpsc;

use crate::{Error, ImapConfig, Result, connect, fetch_headers};

const MAX_UID_SET_BYTES: usize = 4_000;
const MAX_ATTEMPTS: u32 = 3;

pub struct HeaderFetchPlan<'a> {
    pub config: &'a ImapConfig,
    pub mailbox: &'a str,
    pub uidvalidity: u32,
    pub uids: &'a UidSet,
    pub connections: usize,
    pub chunk_size: usize,
}

pub async fn fetch_headers_parallel(
    plan: &HeaderFetchPlan<'_>,
    sink: mpsc::Sender<Vec<MessageMeta>>,
) -> Result<()> {
    let chunks: VecDeque<UidSet> = plan
        .uids
        .chunks_by_count(plan.chunk_size)
        .iter()
        .flat_map(|chunk| chunk.chunks_by_encoded_len(MAX_UID_SET_BYTES))
        .rev()
        .collect();
    if chunks.is_empty() {
        return Ok(());
    }
    let workers = plan.connections.clamp(1, chunks.len());
    let queue = Mutex::new(chunks);
    let tasks = (0..workers).map(|_| worker(plan, &queue, &sink));
    futures::future::try_join_all(tasks).await?;
    Ok(())
}

async fn open(plan: &HeaderFetchPlan<'_>) -> Result<crate::Connection> {
    let mut conn = connect(plan.config).await?;
    let status = conn.examine(plan.mailbox).await?;
    if status.uidvalidity != plan.uidvalidity {
        return Err(Error::Unsupported(format!(
            "UIDVALIDITY of `{}` changed during sync",
            plan.mailbox
        )));
    }
    Ok(conn)
}

fn next_chunk(queue: &Mutex<VecDeque<UidSet>>) -> Option<UidSet> {
    queue
        .lock()
        .unwrap_or_else(|poisoned| poisoned.into_inner())
        .pop_front()
}

async fn worker(
    plan: &HeaderFetchPlan<'_>,
    queue: &Mutex<VecDeque<UidSet>>,
    sink: &mpsc::Sender<Vec<MessageMeta>>,
) -> Result<()> {
    let mut conn = open(plan).await?;
    while let Some(chunk) = next_chunk(queue) {
        let mut attempt = 1;
        let batch = loop {
            match fetch_headers(&mut conn, &chunk).await {
                Ok(batch) => break batch,
                Err(error) if attempt < MAX_ATTEMPTS => {
                    tracing::warn!(%error, attempt, mailbox = plan.mailbox, "header fetch failed, reconnecting");
                    attempt += 1;
                    conn = open(plan).await?;
                }
                Err(error) => return Err(error),
            }
        };
        if sink.send(batch).await.is_err() {
            break;
        }
    }
    conn.logout().await;
    Ok(())
}
