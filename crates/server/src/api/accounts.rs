use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use mail_sieve_core::Query;
use mail_sieve_imap::decode_mailbox_name;
use mail_sieve_store::accounts::{Account, AccountSettings};
use mail_sieve_store::mailboxes::Mailbox;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::accounts::{self, AccountInput};
use crate::error::{Error, Result};
use crate::jobs::enqueue_sync;
use crate::state::{AppState, Event};
use crate::vault::StoredCredentials;

#[derive(Serialize)]
pub struct AccountView {
    #[serde(flatten)]
    account: Account,
    message_count: u64,
    total_size: u64,
}

async fn view(state: &AppState, account: Account) -> Result<AccountView> {
    let account_id = account.id;
    let now = mail_sieve_store::now();
    let totals = state
        .read(move |c| mail_sieve_store::search::totals(c, account_id, &Query::default(), now))
        .await?;
    Ok(AccountView {
        account,
        message_count: totals.count,
        total_size: totals.bytes,
    })
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<AccountView>>> {
    let accounts = state.read(mail_sieve_store::accounts::list).await?;
    let mut views = Vec::with_capacity(accounts.len());
    for account in accounts {
        views.push(view(&state, account).await?);
    }
    Ok(Json(views))
}

#[derive(Deserialize)]
pub struct CreateAccount {
    label: Option<String>,
    email: String,
    imap_host: String,
    imap_port: Option<u16>,
    #[serde(default = "default_security")]
    security: String,
    username: Option<String>,
    password: String,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
}

fn default_security() -> String {
    "tls".into()
}

fn blank_to_none(value: Option<String>) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<CreateAccount>,
) -> Result<(StatusCode, Json<AccountView>)> {
    let email = body.email.trim().to_string();
    let imap_host = body.imap_host.trim().to_string();
    if email.is_empty() || imap_host.is_empty() || body.password.is_empty() {
        return Err(Error::BadRequest(
            "email, IMAP host and password are required".into(),
        ));
    }
    let imap_tls = match body.security.as_str() {
        "tls" => true,
        "plain" => false,
        other => return Err(Error::BadRequest(format!("unknown security `{other}`"))),
    };
    let input = AccountInput {
        label: blank_to_none(body.label),
        username: blank_to_none(body.username).unwrap_or_else(|| email.clone()),
        email,
        imap_port: body.imap_port.unwrap_or(if imap_tls { 993 } else { 143 }),
        imap_host,
        imap_tls,
        smtp_host: blank_to_none(body.smtp_host),
        smtp_port: body.smtp_port,
        auth_kind: "password",
        oauth_provider: None,
        credentials: StoredCredentials::Password {
            password: body.password,
        },
    };
    let account = accounts::create(&state, input).await?;
    enqueue_sync(&state, account.id).await?;
    state.emit(Event::Account {
        account_id: account.id,
    });
    Ok((StatusCode::CREATED, Json(view(&state, account).await?)))
}

#[derive(Deserialize)]
pub struct UpdateAccount {
    label: Option<String>,
    max_connections: Option<u32>,
    sync_interval_minutes: Option<u32>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    password: Option<String>,
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateAccount>,
) -> Result<Json<AccountView>> {
    if body.max_connections.is_some_and(|n| !(1..=15).contains(&n)) {
        return Err(Error::BadRequest(
            "connections must be between 1 and 15".into(),
        ));
    }
    let account = accounts::load(&state, id).await?;
    if let Some(password) = body.password.filter(|p| !p.is_empty()) {
        if account.auth_kind != "password" {
            return Err(Error::BadRequest("this account signs in with OAuth".into()));
        }
        let sealed = state
            .vault
            .seal(&StoredCredentials::Password { password })?;
        state
            .write(move |tx| mail_sieve_store::accounts::update_credentials(tx, id, &sealed))
            .await?;
    }
    let settings = AccountSettings {
        label: blank_to_none(body.label),
        max_connections: body.max_connections,
        sync_interval_minutes: body.sync_interval_minutes,
        smtp_host: body.smtp_host.map(|host| blank_to_none(Some(host))),
        smtp_port: body.smtp_port.map(Some),
    };
    state
        .write(move |tx| mail_sieve_store::accounts::update_settings(tx, id, &settings))
        .await?;
    state.emit(Event::Account { account_id: id });
    Ok(Json(view(&state, accounts::load(&state, id).await?).await?))
}

pub async fn remove(State(state): State<AppState>, Path(id): Path<i64>) -> Result<StatusCode> {
    state
        .write(move |tx| mail_sieve_store::accounts::delete(tx, id))
        .await?;
    state.emit(Event::Account { account_id: id });
    Ok(StatusCode::NO_CONTENT)
}

pub async fn sync(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>> {
    accounts::load(&state, id).await?;
    let job_id = enqueue_sync(&state, id).await?;
    Ok(Json(json!({ "job_id": job_id })))
}

#[derive(Serialize)]
pub struct MailboxView {
    #[serde(flatten)]
    mailbox: Mailbox,
    display_name: String,
}

pub async fn mailboxes(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<MailboxView>>> {
    let mailboxes = state
        .read(move |c| mail_sieve_store::mailboxes::list(c, id))
        .await?;
    Ok(Json(
        mailboxes
            .into_iter()
            .map(|mailbox| MailboxView {
                display_name: decode_mailbox_name(&mailbox.name),
                mailbox,
            })
            .collect(),
    ))
}

#[derive(Deserialize)]
pub struct UpdateMailbox {
    sync_enabled: bool,
}

pub async fn update_mailbox(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<UpdateMailbox>,
) -> Result<Json<Value>> {
    let enabled = body.sync_enabled;
    let mailbox = state
        .write(move |tx| {
            mail_sieve_store::mailboxes::set_sync_enabled(tx, id, enabled)?;
            if !enabled {
                mail_sieve_store::mailboxes::reset(tx, id)?;
            }
            mail_sieve_store::mailboxes::get(tx, id)
        })
        .await?;
    if enabled {
        enqueue_sync(&state, mailbox.account_id).await?;
    }
    state.emit(Event::Account {
        account_id: mailbox.account_id,
    });
    Ok(Json(json!({ "sync_enabled": enabled })))
}
