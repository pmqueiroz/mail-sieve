use futures::TryStreamExt;
use mail_sieve_core::{Flags, UidSet};

use crate::{Connection, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Operation {
    Move(String),
    AddFlags(Flags),
    RemoveFlags(Flags),
    Expunge,
    RemoveGmailLabel(String),
    AddGmailLabel(String),
}

fn flag_list(flags: Flags) -> String {
    let names = [
        (Flags::SEEN, "\\Seen"),
        (Flags::ANSWERED, "\\Answered"),
        (Flags::FLAGGED, "\\Flagged"),
        (Flags::DELETED, "\\Deleted"),
        (Flags::DRAFT, "\\Draft"),
    ];
    let selected: Vec<&str> = names
        .iter()
        .filter(|(flag, _)| flags.contains(*flag))
        .map(|(_, name)| *name)
        .collect();
    format!("({})", selected.join(" "))
}

fn gmail_label(label: &str) -> String {
    if label.starts_with('\\') {
        label.to_string()
    } else {
        format!("\"{}\"", label.replace('\\', "\\\\").replace('"', "\\\""))
    }
}

async fn store(conn: &mut Connection, uids: &UidSet, command: String) -> Result<()> {
    let _: Vec<_> = conn
        .session
        .uid_store(uids.to_string(), command)
        .await?
        .try_collect()
        .await?;
    Ok(())
}

async fn expunge(conn: &mut Connection, uids: &UidSet) -> Result<()> {
    if conn.capabilities.uidplus() {
        let _: Vec<_> = conn
            .session
            .uid_expunge(uids.to_string())
            .await?
            .try_collect()
            .await?;
    } else {
        let _: Vec<_> = conn.session.expunge().await?.try_collect().await?;
    }
    Ok(())
}

pub async fn apply(conn: &mut Connection, uids: &UidSet, operation: &Operation) -> Result<()> {
    if uids.is_empty() {
        return Ok(());
    }
    match operation {
        Operation::Move(destination) if conn.capabilities.supports_move() => {
            conn.session.uid_mv(uids.to_string(), destination).await?;
        }
        Operation::Move(destination) => {
            conn.session.uid_copy(uids.to_string(), destination).await?;
            store(
                conn,
                uids,
                format!("+FLAGS.SILENT {}", flag_list(Flags::DELETED)),
            )
            .await?;
            expunge(conn, uids).await?;
        }
        Operation::AddFlags(flags) => {
            store(conn, uids, format!("+FLAGS.SILENT {}", flag_list(*flags))).await?
        }
        Operation::RemoveFlags(flags) => {
            store(conn, uids, format!("-FLAGS.SILENT {}", flag_list(*flags))).await?
        }
        Operation::Expunge => {
            store(
                conn,
                uids,
                format!("+FLAGS.SILENT {}", flag_list(Flags::DELETED)),
            )
            .await?;
            expunge(conn, uids).await?;
        }
        Operation::RemoveGmailLabel(label) => {
            store(conn, uids, format!("-X-GM-LABELS ({})", gmail_label(label))).await?
        }
        Operation::AddGmailLabel(label) => {
            store(conn, uids, format!("+X-GM-LABELS ({})", gmail_label(label))).await?
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_flag_lists_and_labels() {
        assert_eq!(
            flag_list(Flags::SEEN | Flags::FLAGGED),
            "(\\Seen \\Flagged)"
        );
        assert_eq!(gmail_label("\\Inbox"), "\\Inbox");
        assert_eq!(gmail_label("My \"Label\""), "\"My \\\"Label\\\"\"");
    }
}
