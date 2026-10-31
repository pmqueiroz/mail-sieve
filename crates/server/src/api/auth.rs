use std::time::Duration;

use axum::Json;
use axum::extract::State;
use axum::http::{HeaderMap, header};
use axum::response::{AppendHeaders, IntoResponse};
use mail_sieve_store::{sessions, settings};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::auth::{
    MIN_PASSWORD_LENGTH, SESSION_TTL_SECONDS, hash_password, hash_token, new_session_token,
    session_cookie, session_token, verify_password,
};
use crate::error::{Error, Result};
use crate::oauth::OAuthProvider;
use crate::state::AppState;

const FAILED_LOGIN_DELAY: Duration = Duration::from_millis(750);

#[derive(Deserialize)]
pub struct PasswordBody {
    password: String,
}

#[derive(Deserialize)]
pub struct ChangePasswordBody {
    current: String,
    new: String,
}

async fn stored_hash(state: &AppState) -> Result<Option<String>> {
    state
        .read(|c| settings::get(c, settings::ADMIN_PASSWORD_HASH))
        .await
}

async fn start_session(state: &AppState) -> Result<impl IntoResponse + use<>> {
    let token = new_session_token()?;
    let hash = hash_token(&token);
    state
        .write(move |tx| sessions::insert(tx, &hash, SESSION_TTL_SECONDS))
        .await?;
    let cookie = session_cookie(&token, state.config.secure_cookies(), SESSION_TTL_SECONDS);
    Ok((
        AppendHeaders([(header::SET_COOKIE, cookie)]),
        Json(json!({ "authenticated": true })),
    ))
}

fn validate_new_password(password: &str) -> Result<()> {
    if password.chars().count() < MIN_PASSWORD_LENGTH {
        return Err(Error::BadRequest(format!(
            "password must have at least {MIN_PASSWORD_LENGTH} characters"
        )));
    }
    Ok(())
}

async fn verify(hash: String, password: String) -> Result<bool> {
    Ok(tokio::task::spawn_blocking(move || verify_password(&password, &hash)).await?)
}

pub async fn status(State(state): State<AppState>, headers: HeaderMap) -> Result<Json<Value>> {
    let configured = stored_hash(&state).await?.is_some();
    let authenticated = super::is_authenticated(&state, &headers).await?;
    Ok(Json(json!({
        "configured": configured,
        "authenticated": authenticated,
        "version": env!("CARGO_PKG_VERSION"),
        "oauth": {
            "google": OAuthProvider::Google.client(&state).is_some(),
            "microsoft": OAuthProvider::Microsoft.client(&state).is_some(),
        },
    })))
}

pub async fn setup(
    State(state): State<AppState>,
    Json(body): Json<PasswordBody>,
) -> Result<impl IntoResponse> {
    if stored_hash(&state).await?.is_some() {
        return Err(Error::Conflict(
            "an admin password is already configured".into(),
        ));
    }
    validate_new_password(&body.password)?;
    let hash = tokio::task::spawn_blocking(move || hash_password(&body.password)).await??;
    state
        .write(move |tx| settings::set(tx, settings::ADMIN_PASSWORD_HASH, &hash))
        .await?;
    start_session(&state).await
}

pub async fn login(
    State(state): State<AppState>,
    Json(body): Json<PasswordBody>,
) -> Result<impl IntoResponse> {
    let hash = stored_hash(&state)
        .await?
        .ok_or_else(|| Error::Conflict("run the first-time setup".into()))?;
    if !verify(hash, body.password).await? {
        tokio::time::sleep(FAILED_LOGIN_DELAY).await;
        return Err(Error::Unauthorized);
    }
    start_session(&state).await
}

pub async fn logout(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<impl IntoResponse> {
    if let Some(token) = session_token(&headers) {
        let hash = hash_token(&token);
        state.write(move |tx| sessions::delete(tx, &hash)).await?;
    }
    let cookie = session_cookie("", state.config.secure_cookies(), 0);
    Ok((
        AppendHeaders([(header::SET_COOKIE, cookie)]),
        Json(json!({ "authenticated": false })),
    ))
}

pub async fn change_password(
    State(state): State<AppState>,
    Json(body): Json<ChangePasswordBody>,
) -> Result<impl IntoResponse> {
    let hash = stored_hash(&state).await?.ok_or(Error::Unauthorized)?;
    if !verify(hash, body.current).await? {
        tokio::time::sleep(FAILED_LOGIN_DELAY).await;
        return Err(Error::BadRequest("current password is wrong".into()));
    }
    validate_new_password(&body.new)?;
    let new_hash = tokio::task::spawn_blocking(move || hash_password(&body.new)).await??;
    state
        .write(move |tx| {
            settings::set(tx, settings::ADMIN_PASSWORD_HASH, &new_hash)?;
            sessions::delete_all(tx)
        })
        .await?;
    start_session(&state).await
}
