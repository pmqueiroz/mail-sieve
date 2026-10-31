use std::collections::BTreeMap;

use mail_sieve_core::{Action, Flags, Query, UidSet};
use mail_sieve_imap::{Connection, Operation, Provider, apply, connect};
use mail_sieve_store::jobs::{self, ActionLogEntry, Job};
use mail_sieve_store::mailboxes::Mailbox;
use mail_sieve_store::messages;
use mail_sieve_store::search::{self, Target};
use serde_json::json;

use crate::accounts;
use crate::error::{Error, Result};
use crate::state::AppState;

const ACTION_CHUNK: usize = 1_000;
const MAX_UID_SET_BYTES: usize = 4_000;
const GMAIL_INBOX_LABEL: &str = "\\Inbox";

pub enum Selection {
    Query(Query),
    AnyOf(Vec<Query>),
    Ids(Vec<i64>),
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum LocalEffect {
    Remove,
    Flags { add: Flags, remove: Flags },
    RemoveLabel(String),
}

pub async fn enqueue(
    state: &AppState,
    account_id: i64,
    selection: Selection,
    action: &Action,
    origin: &str,
) -> Result<Option<i64>> {
    let now = mail_sieve_store::now();
    let payload = json!({ "action": action.to_string(), "origin": origin });
    let undoable = action.is_undoable();
    let job_id = state
        .write(move |tx| {
            let targets = match &selection {
                Selection::Query(query) => search::targets(tx, account_id, query, now)?,
                Selection::AnyOf(queries) => {
                    let mut union = Vec::new();
                    for query in queries {
                        union.extend(search::targets(tx, account_id, query, now)?);
                    }
                    union.sort_by_key(|target| (target.mailbox_id, target.uid));
                    union.dedup();
                    union
                }
                Selection::Ids(ids) => search::targets_for_ids(tx, account_id, ids)?,
            };
            if targets.is_empty() {
                return Ok(None);
            }
            let job_id = jobs::create(tx, Some(account_id), "action", &payload)?;
            jobs::add_targets(tx, job_id, &targets)?;
            if undoable {
                jobs::record_undo(tx, job_id)?;
            }
            Ok(Some(job_id))
        })
        .await?;
    if let Some(job_id) = job_id {
        state.emit_job(job_id).await;
        state.jobs_wakeup.notify_one();
    }
    Ok(job_id)
}

fn special<'a>(mailboxes: &'a [Mailbox], special_use: &str) -> Option<&'a Mailbox> {
    mailboxes
        .iter()
        .find(|mailbox| mailbox.special_use.as_deref() == Some(special_use))
}

fn plan(
    provider: Provider,
    action: &Action,
    source: &Mailbox,
    mailboxes: &[Mailbox],
) -> Result<Option<(Operation, LocalEffect)>> {
    let move_to = |target: &Mailbox| {
        (target.id != source.id)
            .then(|| (Operation::Move(target.name.clone()), LocalEffect::Remove))
    };
    let trash = || {
        special(mailboxes, "\\Trash")
            .ok_or_else(|| Error::BadRequest("this account has no Trash folder".into()))
    };
    Ok(match action {
        Action::MarkRead => Some((
            Operation::AddFlags(Flags::SEEN),
            LocalEffect::Flags {
                add: Flags::SEEN,
                remove: Flags::default(),
            },
        )),
        Action::MarkUnread => Some((
            Operation::RemoveFlags(Flags::SEEN),
            LocalEffect::Flags {
                add: Flags::default(),
                remove: Flags::SEEN,
            },
        )),
        Action::Flag => Some((
            Operation::AddFlags(Flags::FLAGGED),
            LocalEffect::Flags {
                add: Flags::FLAGGED,
                remove: Flags::default(),
            },
        )),
        Action::Unflag => Some((
            Operation::RemoveFlags(Flags::FLAGGED),
            LocalEffect::Flags {
                add: Flags::default(),
                remove: Flags::FLAGGED,
            },
        )),
        Action::Archive => match provider {
            Provider::Gmail if source.special_use.as_deref() == Some("\\All") => Some((
                Operation::RemoveGmailLabel(GMAIL_INBOX_LABEL.into()),
                LocalEffect::RemoveLabel(GMAIL_INBOX_LABEL.into()),
            )),
            Provider::Gmail => special(mailboxes, "\\All").and_then(move_to),
            Provider::Generic => {
                let archive = special(mailboxes, "\\Archive").ok_or_else(|| {
                    Error::BadRequest("this account has no Archive folder".into())
                })?;
                move_to(archive)
            }
        },
        Action::Trash => move_to(trash()?),
        Action::Delete => match provider {
            Provider::Gmail => move_to(trash()?),
            Provider::Generic => Some((Operation::Expunge, LocalEffect::Remove)),
        },
        Action::Move(name) => {
            let target = mailboxes
                .iter()
                .find(|mailbox| &mailbox.name == name)
                .ok_or_else(|| Error::BadRequest(format!("folder `{name}` does not exist")))?;
            move_to(target)
        }
    })
}

async fn ensure_archive(
    state: &AppState,
    account_id: i64,
    conn: &mut Connection,
    mailboxes: &mut Vec<Mailbox>,
) -> Result<()> {
    if special(mailboxes, "\\Archive").is_some() {
        return Ok(());
    }
    conn.session
        .create("Archive")
        .await
        .map_err(mail_sieve_imap::Error::from)?;
    accounts::refresh_folders(state, account_id, conn).await?;
    let archive_name = "Archive".to_string();
    state
        .write(move |tx| {
            if let Some(mailbox) =
                mail_sieve_store::mailboxes::find_by_name(tx, account_id, &archive_name)?
            {
                mail_sieve_store::mailboxes::set_sync_enabled(tx, mailbox.id, false)?;
            }
            Ok(())
        })
        .await?;
    *mailboxes = state
        .read(move |c| mail_sieve_store::mailboxes::list(c, account_id))
        .await?;
    Ok(())
}

pub async fn run(state: &AppState, job: &Job) -> Result<()> {
    let account_id = job
        .account_id
        .ok_or_else(|| Error::Internal("action job without account".into()))?;
    let action: Action = job.payload["action"]
        .as_str()
        .unwrap_or_default()
        .parse()
        .map_err(|e: mail_sieve_core::UnknownAction| Error::BadRequest(e.to_string()))?;
    let account = accounts::load(state, account_id).await?;
    let provider: Provider = account.provider.parse().unwrap_or(Provider::Generic);
    let job_id = job.id;
    let targets = state
        .read(move |c| jobs::remaining_targets(c, job_id))
        .await?;
    let mut by_mailbox: BTreeMap<i64, Vec<u32>> = BTreeMap::new();
    for Target { mailbox_id, uid } in targets {
        by_mailbox.entry(mailbox_id).or_default().push(uid);
    }
    let mut mailboxes = state
        .read(move |c| mail_sieve_store::mailboxes::list(c, account_id))
        .await?;
    let config = accounts::imap_config(state, &account).await?;
    let mut conn = connect(&config).await?;
    if action == Action::Archive && provider == Provider::Generic {
        ensure_archive(state, account_id, &mut conn, &mut mailboxes).await?;
    }
    for (mailbox_id, uids) in by_mailbox {
        let Some(source) = mailboxes
            .iter()
            .find(|mailbox| mailbox.id == mailbox_id)
            .cloned()
        else {
            state
                .write(move |tx| jobs::complete_targets(tx, job_id, mailbox_id, &uids))
                .await?;
            continue;
        };
        let Some((operation, effect)) = plan(provider, &action, &source, &mailboxes)? else {
            state
                .write(move |tx| jobs::complete_targets(tx, job_id, mailbox_id, &uids))
                .await?;
            continue;
        };
        let status = conn.select(&source.name).await?;
        if source
            .uidvalidity
            .is_some_and(|known| known != status.uidvalidity)
        {
            return Err(Error::Conflict(format!(
                "folder `{}` changed on the server; sync and retry",
                source.name
            )));
        }
        let chunks: Vec<UidSet> = UidSet::from_uids(uids)
            .chunks_by_count(ACTION_CHUNK)
            .iter()
            .flat_map(|chunk| chunk.chunks_by_encoded_len(MAX_UID_SET_BYTES))
            .collect();
        for chunk in chunks {
            if state.read(move |c| jobs::is_cancelled(c, job_id)).await? {
                conn.logout().await;
                return Ok(());
            }
            apply(&mut conn, &chunk, &operation).await?;
            let chunk_uids: Vec<u32> = chunk.iter().collect();
            let effect = effect.clone();
            let action_name = action.to_string();
            let mailbox_name = source.name.clone();
            let encoded = chunk.to_string();
            state
                .write(move |tx| {
                    let bytes = match &effect {
                        LocalEffect::Remove => messages::total_size(tx, mailbox_id, &chunk_uids)?,
                        _ => 0,
                    };
                    match &effect {
                        LocalEffect::Remove => {
                            messages::delete_uids(tx, mailbox_id, &chunk_uids)?;
                        }
                        LocalEffect::Flags { add, remove } => {
                            messages::apply_flag_change(
                                tx,
                                mailbox_id,
                                &chunk_uids,
                                *add,
                                *remove,
                            )?;
                        }
                        LocalEffect::RemoveLabel(label) => {
                            messages::remove_label(tx, mailbox_id, &chunk_uids, label)?;
                        }
                    }
                    jobs::complete_targets(tx, job_id, mailbox_id, &chunk_uids)?;
                    jobs::log_action(
                        tx,
                        &ActionLogEntry {
                            job_id: Some(job_id),
                            account_id,
                            action: &action_name,
                            mailbox: &mailbox_name,
                            uids: &encoded,
                            count: chunk_uids.len(),
                            bytes,
                        },
                    )
                })
                .await?;
            state.emit_job(job_id).await;
        }
    }
    conn.logout().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn mailbox(id: i64, name: &str, special_use: Option<&str>) -> Mailbox {
        Mailbox {
            id,
            account_id: 1,
            name: name.into(),
            delimiter: Some("/".into()),
            special_use: special_use.map(Into::into),
            sync_enabled: true,
            uidvalidity: Some(1),
            uidnext: Some(10),
            highestmodseq: None,
            last_synced_at: None,
            message_count: 0,
        }
    }

    #[test]
    fn plans_generic_operations() {
        let boxes = vec![
            mailbox(1, "INBOX", None),
            mailbox(2, "Archive", Some("\\Archive")),
            mailbox(3, "Trash", Some("\\Trash")),
        ];
        let inbox = &boxes[0];
        let planned = |action: Action, source: &Mailbox| {
            plan(Provider::Generic, &action, source, &boxes).unwrap()
        };
        assert_eq!(
            planned(Action::Archive, inbox),
            Some((Operation::Move("Archive".into()), LocalEffect::Remove))
        );
        assert_eq!(planned(Action::Archive, &boxes[1]), None);
        assert_eq!(
            planned(Action::Trash, inbox),
            Some((Operation::Move("Trash".into()), LocalEffect::Remove))
        );
        assert_eq!(
            planned(Action::Delete, inbox),
            Some((Operation::Expunge, LocalEffect::Remove))
        );
        assert!(
            matches!(planned(Action::MarkRead, inbox), Some((Operation::AddFlags(f), _)) if f == Flags::SEEN)
        );
        assert!(
            plan(
                Provider::Generic,
                &Action::Move("Nope".into()),
                inbox,
                &boxes
            )
            .is_err()
        );
    }

    #[test]
    fn plans_gmail_operations() {
        let boxes = vec![
            mailbox(1, "[Gmail]/All Mail", Some("\\All")),
            mailbox(2, "[Gmail]/Trash", Some("\\Trash")),
            mailbox(3, "INBOX", None),
        ];
        let all = &boxes[0];
        let planned = |action: Action, source: &Mailbox| {
            plan(Provider::Gmail, &action, source, &boxes).unwrap()
        };
        assert_eq!(
            planned(Action::Archive, all),
            Some((
                Operation::RemoveGmailLabel("\\Inbox".into()),
                LocalEffect::RemoveLabel("\\Inbox".into())
            ))
        );
        assert_eq!(
            planned(Action::Archive, &boxes[2]),
            Some((
                Operation::Move("[Gmail]/All Mail".into()),
                LocalEffect::Remove
            ))
        );
        assert_eq!(
            planned(Action::Delete, all),
            Some((Operation::Move("[Gmail]/Trash".into()), LocalEffect::Remove))
        );
        let no_trash = vec![mailbox(1, "INBOX", None)];
        assert!(plan(Provider::Generic, &Action::Trash, &no_trash[0], &no_trash).is_err());
    }
}
