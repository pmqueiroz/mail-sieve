use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::Redirect;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::accounts::{self, AccountInput};
use crate::error::{Error, Result};
use crate::jobs::enqueue_sync;
use crate::oauth::{self, OAuthProvider};
use crate::state::AppState;
use crate::vault::StoredCredentials;

fn provider(raw: &str) -> Result<OAuthProvider> {
    OAuthProvider::parse(raw).ok_or_else(|| Error::BadRequest(format!("unknown provider `{raw}`")))
}

pub async fn start(State(state): State<AppState>, Path(raw): Path<String>) -> Result<Json<Value>> {
    Ok(Json(
        json!({ "url": oauth::start(&state, provider(&raw)?)? }),
    ))
}

#[derive(Deserialize)]
pub struct CallbackParams {
    code: Option<String>,
    state: Option<String>,
    error: Option<String>,
}

fn back_to_accounts(query: &str) -> Redirect {
    Redirect::to(&format!("/#/accounts?{query}"))
}

pub async fn callback(
    State(state): State<AppState>,
    Path(raw): Path<String>,
    Query(params): Query<CallbackParams>,
) -> Redirect {
    match finish(&state, &raw, params).await {
        Ok(account_id) => back_to_accounts(&format!("added={account_id}")),
        Err(error) => {
            tracing::warn!(%error, "oauth sign-in failed");
            let message: String = url_escape(&error.to_string());
            back_to_accounts(&format!("error={message}"))
        }
    }
}

fn url_escape(value: &str) -> String {
    value
        .bytes()
        .map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                (byte as char).to_string()
            }
            _ => format!("%{byte:02X}"),
        })
        .collect()
}

async fn finish(state: &AppState, raw: &str, params: CallbackParams) -> Result<i64> {
    let provider = provider(raw)?;
    if let Some(error) = params.error {
        return Err(Error::OAuth(error));
    }
    let code = params
        .code
        .ok_or_else(|| Error::OAuth("missing code".into()))?;
    let csrf = params
        .state
        .ok_or_else(|| Error::OAuth("missing state".into()))?;
    let completed = oauth::complete(state, provider, &code, &csrf).await?;
    let refresh_token = completed
        .tokens
        .refresh_token
        .clone()
        .ok_or_else(|| Error::OAuth("provider did not grant offline access".into()))?;
    let endpoints = provider.endpoints();
    let input = AccountInput {
        label: None,
        email: completed.email.clone(),
        imap_host: endpoints.imap_host.into(),
        imap_port: 993,
        imap_tls: true,
        smtp_host: Some(endpoints.smtp_host.into()),
        smtp_port: Some(587),
        username: completed.email,
        auth_kind: "oauth2",
        oauth_provider: Some(provider.as_str().into()),
        credentials: StoredCredentials::OAuth2 {
            provider: provider.as_str().into(),
            refresh_token,
            access_token: Some(completed.tokens.access_token),
            expires_at: Some(mail_sieve_store::now() + completed.tokens.expires_in.unwrap_or(3600)),
        },
    };
    let account = accounts::create(state, input).await?;
    enqueue_sync(state, account.id).await?;
    Ok(account.id)
}
