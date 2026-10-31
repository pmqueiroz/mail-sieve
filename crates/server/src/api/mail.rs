use axum::Json;
use axum::extract::{Path, Query as QueryParams, State};
use mail_parser::MimeHeaders;
use mail_sieve_core::{Action, Query};
use mail_sieve_imap::{connect, fetch_raw};
use mail_sieve_store::search::{self, Cursor, GroupBy, GroupRequest, SortBy};
use mail_sieve_unsub::{Options, Outcome};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::accounts;
use crate::actions::{self, Selection};
use crate::error::{Error, Result};
use crate::state::AppState;

const MAX_PREVIEW_CHARS: usize = 50_000;

fn parse_query(raw: Option<&str>) -> Result<Query> {
    Ok(Query::parse(raw.unwrap_or_default())?)
}

#[derive(Deserialize)]
pub struct GroupParams {
    q: Option<String>,
    #[serde(default)]
    group: GroupBy,
    #[serde(default)]
    sort: SortBy,
    limit: Option<u32>,
    offset: Option<u32>,
}

pub async fn groups(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    QueryParams(params): QueryParams<GroupParams>,
) -> Result<Json<Vec<search::Group>>> {
    let query = parse_query(params.q.as_deref())?;
    let now = mail_sieve_store::now();
    let groups = state
        .read(move |c| {
            search::groups(
                c,
                &GroupRequest {
                    account_id: id,
                    query: &query,
                    now,
                    group_by: params.group,
                    sort_by: params.sort,
                    limit: params.limit.unwrap_or(200).min(2_000),
                    offset: params.offset.unwrap_or(0),
                },
            )
        })
        .await?;
    Ok(Json(groups))
}

#[derive(Deserialize)]
pub struct MessageParams {
    q: Option<String>,
    before_date: Option<i64>,
    before_id: Option<i64>,
    limit: Option<u32>,
}

pub async fn messages(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    QueryParams(params): QueryParams<MessageParams>,
) -> Result<Json<Value>> {
    let query = parse_query(params.q.as_deref())?;
    let cursor = match (params.before_date, params.before_id) {
        (Some(date), Some(id)) => Some(Cursor { date, id }),
        _ => None,
    };
    let limit = params.limit.unwrap_or(200).min(1_000);
    let now = mail_sieve_store::now();
    let rows = state
        .read(move |c| search::messages(c, id, &query, now, cursor, limit))
        .await?;
    let next = (rows.len() as u32 == limit)
        .then(|| {
            rows.last()
                .map(|last| json!({ "before_date": last.date, "before_id": last.id }))
        })
        .flatten();
    Ok(Json(json!({ "messages": rows, "next": next })))
}

#[derive(Deserialize)]
pub struct CountParams {
    q: Option<String>,
}

pub async fn count(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    QueryParams(params): QueryParams<CountParams>,
) -> Result<Json<search::Totals>> {
    let query = parse_query(params.q.as_deref())?;
    let now = mail_sieve_store::now();
    Ok(Json(
        state
            .read(move |c| search::totals(c, id, &query, now))
            .await?,
    ))
}

#[derive(Deserialize)]
pub struct ActionBody {
    query: Option<String>,
    queries: Option<Vec<String>>,
    ids: Option<Vec<i64>>,
    action: String,
    #[serde(default)]
    confirm: bool,
}

fn parse_action(raw: &str) -> Result<Action> {
    raw.parse()
        .map_err(|e: mail_sieve_core::UnknownAction| Error::BadRequest(e.to_string()))
}

pub async fn action(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<ActionBody>,
) -> Result<Json<Value>> {
    let action = parse_action(&body.action)?;
    let selection = match (body.ids, body.queries, body.query) {
        (Some(ids), _, _) if !ids.is_empty() => Selection::Ids(ids),
        (_, Some(queries), _) if !queries.is_empty() => {
            let parsed = queries
                .iter()
                .map(|raw| parse_query(Some(raw)))
                .collect::<Result<Vec<_>>>()?;
            if parsed.iter().any(Query::is_empty) {
                return Err(Error::BadRequest(
                    "every query in a batch needs at least one term".into(),
                ));
            }
            Selection::AnyOf(parsed)
        }
        (_, _, query) => {
            let query = parse_query(query.as_deref())?;
            if query.is_empty() && !body.confirm {
                return Err(Error::BadRequest(
                    "an empty query selects every message; confirm to continue".into(),
                ));
            }
            Selection::Query(query)
        }
    };
    if action.is_destructive() && !body.confirm {
        return Err(Error::BadRequest(
            "permanent deletion needs confirmation".into(),
        ));
    }
    accounts::load(&state, id).await?;
    let job_id = actions::enqueue(&state, id, selection, &action, "manual").await?;
    Ok(Json(json!({ "job_id": job_id })))
}

#[derive(Deserialize)]
pub struct UnsubscribeBody {
    from_addr: String,
    then: Option<String>,
}

pub async fn unsubscribe(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UnsubscribeBody>,
) -> Result<Json<Value>> {
    let from_addr = body.from_addr.trim().to_lowercase();
    let follow_up = body.then.as_deref().map(parse_action).transpose()?;
    if follow_up.as_ref().is_some_and(Action::is_destructive) {
        return Err(Error::BadRequest(
            "use trash after unsubscribing; permanent deletion is not chained".into(),
        ));
    }
    let account = accounts::load(&state, id).await?;
    let lookup = from_addr.clone();
    let sender = state
        .read(move |c| mail_sieve_store::unsubscribes::sender_options(c, id, &lookup))
        .await?
        .ok_or_else(|| Error::BadRequest(format!("no messages from {from_addr}")))?;
    let options = Options {
        http: sender.http,
        mailto: sender.mailto,
        one_click: sender.one_click,
    };
    let smtp = accounts::smtp_settings(&state, &account).await?;
    let outcome = mail_sieve_unsub::unsubscribe(&options, smtp.as_ref()).await;
    let (method, status, detail, response) = match &outcome {
        Ok(Outcome::Completed { method, detail }) => (
            method.as_str(),
            "completed",
            Some(detail.clone()),
            json!({ "status": "completed", "method": method.as_str(), "detail": detail }),
        ),
        Ok(Outcome::OpenLink(url)) => (
            "link",
            "link_opened",
            None,
            json!({ "status": "open_link", "url": url }),
        ),
        Err(error) => (
            "none",
            "failed",
            Some(error.to_string()),
            json!({ "status": "failed", "error": error.to_string() }),
        ),
    };
    let record_addr = from_addr.clone();
    let detail_record = detail.clone();
    state
        .write(move |tx| {
            mail_sieve_store::unsubscribes::record(
                tx,
                id,
                &record_addr,
                method,
                status,
                detail_record.as_deref(),
            )
        })
        .await?;
    let job_id = match (&outcome, follow_up) {
        (Ok(_), Some(action)) => {
            let query = Query::parse(&format!("from:\"{from_addr}\""))?;
            actions::enqueue(&state, id, Selection::Query(query), &action, "unsubscribe").await?
        }
        _ => None,
    };
    Ok(Json(json!({ "outcome": response, "job_id": job_id })))
}

pub async fn preview(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>> {
    let location = state.read(move |c| search::locate(c, id)).await?;
    let account = accounts::load(&state, location.account_id).await?;
    let config = accounts::imap_config(&state, &account).await?;
    let mut conn = connect(&config).await?;
    let status = conn.examine(&location.mailbox).await?;
    if location
        .uidvalidity
        .is_some_and(|known| known != status.uidvalidity)
    {
        conn.logout().await;
        return Err(Error::Conflict(
            "folder changed on the server; sync first".into(),
        ));
    }
    let raw = fetch_raw(&mut conn, location.uid).await?;
    conn.logout().await;
    let raw =
        raw.ok_or_else(|| Error::Conflict("message no longer exists on the server".into()))?;
    let message = mail_parser::MessageParser::default()
        .parse(&raw)
        .ok_or_else(|| Error::Internal("cannot parse message".into()))?;
    let mut text = message
        .body_text(0)
        .map(|body| body.into_owned())
        .unwrap_or_default();
    if text.chars().count() > MAX_PREVIEW_CHARS {
        text = text.chars().take(MAX_PREVIEW_CHARS).collect();
        text.push_str("\n…");
    }
    let addresses = |address: Option<&mail_parser::Address>| -> Vec<String> {
        address
            .map(|list| {
                list.iter()
                    .filter_map(|addr| {
                        addr.address().map(|email| match addr.name() {
                            Some(name) => format!("{name} <{email}>"),
                            None => email.to_string(),
                        })
                    })
                    .collect()
            })
            .unwrap_or_default()
    };
    let attachments: Vec<String> = message
        .attachments()
        .map(|part| part.attachment_name().unwrap_or("unnamed").to_string())
        .collect();
    Ok(Json(json!({
        "subject": message.subject(),
        "from": addresses(message.from()),
        "to": addresses(message.to()),
        "date": message.date().map(|date| date.to_timestamp()),
        "text": text,
        "attachments": attachments,
        "size": raw.len(),
    })))
}
