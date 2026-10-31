use futures::TryStreamExt;
use mail_sieve_core::{Flags, MessageMeta, UidSet};

use crate::parse::{flags_of, labels_of, message_meta};
use crate::{Connection, Result};

const HEADER_FIELDS: &str =
    "FROM SUBJECT DATE MESSAGE-ID LIST-ID LIST-UNSUBSCRIBE LIST-UNSUBSCRIBE-POST";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FlagState {
    pub uid: u32,
    pub flags: Flags,
    pub labels: Option<Vec<String>>,
}

pub async fn search_all_uids(conn: &mut Connection) -> Result<Vec<u32>> {
    let mut uids: Vec<u32> = conn.session.uid_search("ALL").await?.into_iter().collect();
    uids.sort_unstable();
    Ok(uids)
}

pub async fn fetch_headers(conn: &mut Connection, uids: &UidSet) -> Result<Vec<MessageMeta>> {
    if uids.is_empty() {
        return Ok(Vec::new());
    }
    let gmail = if conn.capabilities.gmail() {
        " X-GM-MSGID X-GM-LABELS"
    } else {
        ""
    };
    let query = format!(
        "(UID FLAGS RFC822.SIZE INTERNALDATE{gmail} BODY.PEEK[HEADER.FIELDS ({HEADER_FIELDS})])"
    );
    let mut stream = conn.session.uid_fetch(uids.to_string(), query).await?;
    let mut messages = Vec::with_capacity(uids.len());
    while let Some(fetch) = stream.try_next().await? {
        if let Some(meta) = message_meta(&fetch) {
            messages.push(meta);
        }
    }
    Ok(messages)
}

pub async fn fetch_flags(
    conn: &mut Connection,
    set: &str,
    changed_since: Option<u64>,
) -> Result<Vec<FlagState>> {
    let gmail = if conn.capabilities.gmail() {
        " X-GM-LABELS"
    } else {
        ""
    };
    let modifier = match changed_since {
        Some(modseq) if conn.capabilities.condstore() => format!(" (CHANGEDSINCE {modseq})"),
        _ => String::new(),
    };
    let mut stream = conn
        .session
        .uid_fetch(set, format!("(UID FLAGS{gmail}){modifier}"))
        .await?;
    let mut states = Vec::new();
    while let Some(fetch) = stream.try_next().await? {
        if let Some(uid) = fetch.uid {
            states.push(FlagState {
                uid,
                flags: flags_of(&fetch),
                labels: labels_of(&fetch),
            });
        }
    }
    Ok(states)
}

pub async fn fetch_raw(conn: &mut Connection, uid: u32) -> Result<Option<Vec<u8>>> {
    let mut stream = conn
        .session
        .uid_fetch(uid.to_string(), "(UID BODY.PEEK[])")
        .await?;
    let mut body = None;
    while let Some(fetch) = stream.try_next().await? {
        if fetch.uid == Some(uid) {
            body = fetch.body().map(<[u8]>::to_vec);
        }
    }
    Ok(body)
}
