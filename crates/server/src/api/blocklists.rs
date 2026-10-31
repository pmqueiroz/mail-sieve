use std::collections::{BTreeMap, BTreeSet};

use axum::Json;
use axum::extract::{Path, State};
use mail_sieve_core::blocklist;
use serde::{Deserialize, Serialize};

use crate::accounts;
use crate::error::{Error, Result};
use crate::state::AppState;

const MAX_LIST_BYTES: usize = 8 * 1024 * 1024;
const MAX_MATCHES: usize = 500;

#[derive(Deserialize)]
pub struct PreviewBody {
    url: String,
}

#[derive(Debug, Serialize, PartialEq, Eq)]
pub struct Match {
    domain: String,
    messages: i64,
    sender_domains: Vec<String>,
}

#[derive(Serialize)]
pub struct Preview {
    listed: usize,
    matches: Vec<Match>,
}

fn fetch_error(error: mail_sieve_unsub::Error) -> Error {
    match error {
        mail_sieve_unsub::Error::UnsafeUrl(reason) => {
            Error::BadRequest(format!("this blocklist URL is not allowed: {reason}"))
        }
        mail_sieve_unsub::Error::TooLarge(limit) => Error::BadRequest(format!(
            "the blocklist is larger than {} MB",
            limit / (1024 * 1024)
        )),
        mail_sieve_unsub::Error::Status(status) => {
            Error::Upstream(format!("the blocklist server answered {status}"))
        }
        other => Error::Upstream(format!("cannot fetch the blocklist: {other}")),
    }
}

fn match_domains(listed: &BTreeSet<String>, counts: Vec<(String, i64)>) -> Vec<Match> {
    let mut grouped: BTreeMap<&str, Match> = BTreeMap::new();
    for (domain, messages) in counts {
        let Some(entry) = blocklist::listed_entry(listed, &domain) else {
            continue;
        };
        let found = grouped.entry(entry).or_insert_with(|| Match {
            domain: entry.to_string(),
            messages: 0,
            sender_domains: Vec::new(),
        });
        found.messages += messages;
        found.sender_domains.push(domain);
    }
    let mut matches: Vec<Match> = grouped.into_values().collect();
    for found in &mut matches {
        found.sender_domains.sort();
    }
    matches.sort_by(|a, b| b.messages.cmp(&a.messages).then(a.domain.cmp(&b.domain)));
    matches.truncate(MAX_MATCHES);
    matches
}

pub async fn preview(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PreviewBody>,
) -> Result<Json<Preview>> {
    accounts::load(&state, id).await?;
    let text = mail_sieve_unsub::fetch_public_text(body.url.trim(), MAX_LIST_BYTES)
        .await
        .map_err(fetch_error)?;
    let listed = blocklist::parse_domains(&text);
    if listed.is_empty() {
        return Err(Error::BadRequest(
            "the file at this URL lists no domains".into(),
        ));
    }
    let counts = state
        .read(move |c| mail_sieve_store::messages::domain_counts(c, id))
        .await?;
    Ok(Json(Preview {
        listed: listed.len(),
        matches: match_domains(&listed, counts),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn groups_sender_domains_under_their_listed_entry() {
        let listed = blocklist::parse_domains("spam.com\nads.net\n");
        let counts = vec![
            ("mail.spam.com".to_string(), 3),
            ("spam.com".to_string(), 2),
            ("ads.net".to_string(), 7),
            ("friend.org".to_string(), 40),
        ];
        assert_eq!(
            match_domains(&listed, counts),
            [
                Match {
                    domain: "ads.net".into(),
                    messages: 7,
                    sender_domains: vec!["ads.net".into()],
                },
                Match {
                    domain: "spam.com".into(),
                    messages: 5,
                    sender_domains: vec!["mail.spam.com".into(), "spam.com".into()],
                },
            ]
        );
    }
}
