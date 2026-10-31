use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use mail_sieve_store::jobs::{self, Job};
use serde_json::json;

use crate::error::Result;
use crate::state::AppState;
use crate::{actions, rules, sync, undo};

const POLL_INTERVAL: Duration = Duration::from_secs(5);
const GLOBAL_LANE: i64 = 0;

pub async fn enqueue_sync(state: &AppState, account_id: i64) -> Result<Option<i64>> {
    let job_id = state
        .write(move |tx| {
            if jobs::has_active(tx, account_id, "sync")? {
                return Ok(None);
            }
            jobs::create(tx, Some(account_id), "sync", &json!({})).map(Some)
        })
        .await?;
    if let Some(job_id) = job_id {
        state.emit_job(job_id).await;
        state.jobs_wakeup.notify_one();
    }
    Ok(job_id)
}

pub fn spawn_runner(state: AppState) {
    tokio::spawn(async move {
        if let Err(error) = state.write(|tx| jobs::requeue_interrupted(tx)).await {
            tracing::error!(%error, "cannot requeue interrupted jobs");
        }
        let lanes: Arc<Mutex<HashSet<i64>>> = Arc::default();
        loop {
            match state.read(jobs::queued).await {
                Ok(queued) => {
                    for job in queued {
                        let lane = job.account_id.unwrap_or(GLOBAL_LANE);
                        if !lanes.lock().unwrap_or_else(|p| p.into_inner()).insert(lane) {
                            continue;
                        }
                        let state = state.clone();
                        let lanes = lanes.clone();
                        tokio::spawn(async move {
                            execute(&state, job).await;
                            lanes
                                .lock()
                                .unwrap_or_else(|p| p.into_inner())
                                .remove(&lane);
                            state.jobs_wakeup.notify_one();
                        });
                    }
                }
                Err(error) => tracing::error!(%error, "cannot read job queue"),
            }
            tokio::select! {
                _ = state.jobs_wakeup.notified() => {}
                _ = tokio::time::sleep(POLL_INTERVAL) => {}
            }
        }
    });
}

async fn execute(state: &AppState, job: Job) {
    let job_id = job.id;
    if state
        .write(move |tx| jobs::mark_running(tx, job_id))
        .await
        .is_err()
    {
        return;
    }
    state.emit_job(job_id).await;
    tracing::info!(job_id, kind = %job.kind, account_id = ?job.account_id, "job started");
    let result = match job.kind.as_str() {
        "sync" => match job.account_id {
            Some(account_id) => sync::sync_account(state, account_id, Some(job_id))
                .await
                .map(|report| {
                    tracing::info!(
                        job_id,
                        fetched = report.fetched,
                        removed = report.removed,
                        flags = report.flags_updated,
                        seconds = report.elapsed.as_secs_f64(),
                        "sync finished"
                    );
                }),
            None => Ok(()),
        },
        "action" => actions::run(state, &job).await,
        "undo" => undo::run(state, &job).await,
        other => Err(crate::error::Error::Internal(format!(
            "unknown job kind {other}"
        ))),
    };
    let (final_state, error) = match &result {
        Ok(()) => (jobs::DONE, None),
        Err(error) => (jobs::FAILED, Some(error.to_string())),
    };
    if let Some(error) = &error {
        tracing::warn!(job_id, %error, "job failed");
    }
    let _ = state
        .write(move |tx| jobs::finish(tx, job_id, final_state, error.as_deref()))
        .await;
    state.emit_job(job_id).await;
    if result.is_ok()
        && job.kind == "sync"
        && let Some(account_id) = job.account_id
        && let Err(error) = rules::run_after_sync(state, account_id).await
    {
        tracing::warn!(%error, account_id, "after-sync rules failed");
    }
}
