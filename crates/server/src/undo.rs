use std::collections::BTreeMap;

use mail_sieve_core::{Action, UidSet};
use mail_sieve_imap::{Operation, Provider, apply, connect, search_message_ids};
use mail_sieve_store::jobs::{self, Job, UndoTarget};
use mail_sieve_store::search::Target;
use serde_json::json;

use crate::accounts;
use crate::actions::plan;
use crate::error::{Error, Result};
use crate::jobs::enqueue_sync;
use crate::state::AppState;

const UNDO_KIND: &str = "undo";

fn undoable_action(job: &Job) -> Result<Action> {
    if job.kind != "action" {
        return Err(Error::BadRequest("only actions can be undone".into()));
    }
    if job.state != jobs::DONE && job.state != jobs::CANCELLED {
        return Err(Error::Conflict("the action is still running".into()));
    }
    if !job.payload["undone_by"].is_null() {
        return Err(Error::Conflict("this action was already undone".into()));
    }
    let action: Action = job.payload["action"]
        .as_str()
        .unwrap_or_default()
        .parse()
        .map_err(|e: mail_sieve_core::UnknownAction| Error::BadRequest(e.to_string()))?;
    if !action.is_undoable() {
        return Err(Error::BadRequest(
            "permanent deletion cannot be undone".into(),
        ));
    }
    Ok(action)
}

pub async fn start(state: &AppState, job_id: i64) -> Result<i64> {
    let original = state.read(move |c| jobs::get(c, job_id)).await?;
    let action = undoable_action(&original)?;
    let account_id = original
        .account_id
        .ok_or_else(|| Error::Internal("action job without account".into()))?;
    let inverse = action.inverse_flag_action();
    let undo_id = state
        .write(move |tx| {
            let current = jobs::get(tx, job_id)?;
            if !current.payload["undone_by"].is_null() {
                return Ok(None);
            }
            let targets = jobs::undo_targets(tx, job_id)?;
            if targets.is_empty() {
                return Ok(None);
            }
            let id = match &inverse {
                Some(inverse) => {
                    let payload = json!({ "action": inverse.to_string(), "origin": "undo", "undo_of": job_id });
                    let id = jobs::create(tx, Some(account_id), "action", &payload)?;
                    let targets: Vec<Target> = targets
                        .iter()
                        .map(|target| Target {
                            mailbox_id: target.mailbox_id,
                            uid: target.uid,
                        })
                        .collect();
                    jobs::add_targets(tx, id, &targets)?;
                    id
                }
                None => {
                    let payload = json!({ "action": action.to_string(), "origin": "undo", "undo_of": job_id });
                    let id = jobs::create(tx, Some(account_id), UNDO_KIND, &payload)?;
                    jobs::set_progress(tx, id, 0, targets.len() as u64)?;
                    id
                }
            };
            jobs::mark_undone(tx, job_id, id)?;
            Ok(Some(id))
        })
        .await?
        .ok_or_else(|| Error::Conflict("nothing left to undo".into()))?;
    state.emit_job(job_id).await;
    state.emit_job(undo_id).await;
    state.jobs_wakeup.notify_one();
    Ok(undo_id)
}

pub async fn run(state: &AppState, job: &Job) -> Result<()> {
    let account_id = job
        .account_id
        .ok_or_else(|| Error::Internal("undo job without account".into()))?;
    let original_id = job.payload["undo_of"]
        .as_i64()
        .ok_or_else(|| Error::Internal("undo job without original".into()))?;
    let action: Action = job.payload["action"]
        .as_str()
        .unwrap_or_default()
        .parse()
        .map_err(|e: mail_sieve_core::UnknownAction| Error::BadRequest(e.to_string()))?;
    let account = accounts::load(state, account_id).await?;
    let provider: Provider = account.provider.parse().unwrap_or(Provider::Generic);
    let targets = state
        .read(move |c| jobs::undo_targets(c, original_id))
        .await?;
    let total = targets.len() as u64;
    let mut by_mailbox: BTreeMap<i64, Vec<UndoTarget>> = BTreeMap::new();
    for target in targets {
        by_mailbox
            .entry(target.mailbox_id)
            .or_default()
            .push(target);
    }
    let mailboxes = state
        .read(move |c| mail_sieve_store::mailboxes::list(c, account_id))
        .await?;
    let config = accounts::imap_config(state, &account).await?;
    let mut conn = connect(&config).await?;
    let job_id = job.id;
    let mut done = 0u64;
    for (mailbox_id, group) in by_mailbox {
        if state.read(move |c| jobs::is_cancelled(c, job_id)).await? {
            break;
        }
        let Some(source) = mailboxes.iter().find(|mailbox| mailbox.id == mailbox_id) else {
            continue;
        };
        match plan(provider, &action, source, &mailboxes)? {
            Some((Operation::RemoveGmailLabel(label), _)) => {
                conn.select(&source.name).await?;
                let uids = UidSet::from_uids(group.iter().map(|target| target.uid));
                apply(&mut conn, &uids, &Operation::AddGmailLabel(label)).await?;
            }
            Some((Operation::Move(destination), _)) => {
                let ids: Vec<String> = group
                    .iter()
                    .filter_map(|target| target.message_id.clone())
                    .collect();
                if !ids.is_empty() {
                    conn.select(&destination).await?;
                    let found = search_message_ids(&mut conn, &ids).await?;
                    apply(
                        &mut conn,
                        &UidSet::from_uids(found),
                        &Operation::Move(source.name.clone()),
                    )
                    .await?;
                }
            }
            _ => {}
        }
        done += group.len() as u64;
        state
            .write(move |tx| jobs::set_progress(tx, job_id, done, total))
            .await?;
        state.emit_job(job_id).await;
    }
    conn.logout().await;
    enqueue_sync(state, account_id).await?;
    Ok(())
}
