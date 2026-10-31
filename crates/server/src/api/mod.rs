mod accounts;
mod auth;
mod jobs;
mod mail;
mod oauth;
mod rules;

use axum::Router;
use axum::extract::{Request, State};
use axum::http::{HeaderValue, header};
use axum::middleware::{Next, from_fn_with_state};
use axum::response::Response;
use axum::routing::{get, patch, post, put};
use tower_http::compression::CompressionLayer;
use tower_http::set_header::SetResponseHeaderLayer;
use tower_http::trace::TraceLayer;

use crate::assets;
use crate::error::{Error, Result};
use crate::state::AppState;

const CONTENT_SECURITY_POLICY: &str = "default-src 'self'; img-src 'self' data:; style-src 'self' 'unsafe-inline'; \
     connect-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'self'";

pub fn router(state: AppState) -> Router {
    let protected = Router::new()
        .route("/accounts", get(accounts::list).post(accounts::create))
        .route(
            "/accounts/{id}",
            patch(accounts::update).delete(accounts::remove),
        )
        .route("/accounts/{id}/sync", post(accounts::sync))
        .route("/accounts/{id}/mailboxes", get(accounts::mailboxes))
        .route("/mailboxes/{id}", patch(accounts::update_mailbox))
        .route("/accounts/{id}/groups", get(mail::groups))
        .route("/accounts/{id}/messages", get(mail::messages))
        .route("/accounts/{id}/count", get(mail::count))
        .route("/accounts/{id}/overview", get(mail::overview))
        .route("/accounts/{id}/actions", post(mail::action))
        .route("/accounts/{id}/unsubscribe", post(mail::unsubscribe))
        .route(
            "/accounts/{id}/unsubscribes/pending",
            get(mail::pending_unsubscribes),
        )
        .route(
            "/accounts/{id}/unsubscribes/done",
            post(mail::finish_unsubscribe),
        )
        .route("/messages/{id}/preview", get(mail::preview))
        .route("/jobs", get(jobs::list))
        .route("/jobs/{id}", get(jobs::get))
        .route("/jobs/{id}/cancel", post(jobs::cancel))
        .route("/events", get(jobs::events))
        .route("/rules", get(rules::list).post(rules::create))
        .route("/rules/{id}", put(rules::update).delete(rules::remove))
        .route("/rules/{id}/run", post(rules::run))
        .route("/oauth/{provider}/start", get(oauth::start))
        .route("/oauth/{provider}/callback", get(oauth::callback))
        .route("/auth/password", post(auth::change_password))
        .layer(from_fn_with_state(state.clone(), require_session));
    let public = Router::new()
        .route("/health", get(|| async { "ok" }))
        .route("/auth/status", get(auth::status))
        .route("/auth/setup", post(auth::setup))
        .route("/auth/login", post(auth::login))
        .route("/auth/logout", post(auth::logout));
    Router::new()
        .nest("/api", public.merge(protected))
        .fallback(assets::serve)
        .layer(SetResponseHeaderLayer::if_not_present(
            header::CONTENT_SECURITY_POLICY,
            HeaderValue::from_static(CONTENT_SECURITY_POLICY),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_CONTENT_TYPE_OPTIONS,
            HeaderValue::from_static("nosniff"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::X_FRAME_OPTIONS,
            HeaderValue::from_static("DENY"),
        ))
        .layer(SetResponseHeaderLayer::if_not_present(
            header::REFERRER_POLICY,
            HeaderValue::from_static("no-referrer"),
        ))
        .layer(CompressionLayer::new())
        .layer(TraceLayer::new_for_http())
        .with_state(state)
}

pub async fn is_authenticated(state: &AppState, headers: &axum::http::HeaderMap) -> Result<bool> {
    if let Some(trusted) = &state.config.trusted_header
        && headers.contains_key(trusted.as_str())
    {
        return Ok(true);
    }
    let Some(token) = crate::auth::session_token(headers) else {
        return Ok(false);
    };
    let hash = crate::auth::hash_token(&token);
    state
        .read(move |c| mail_sieve_store::sessions::is_valid(c, &hash))
        .await
}

async fn require_session(
    State(state): State<AppState>,
    request: Request,
    next: Next,
) -> Result<Response> {
    if is_authenticated(&state, request.headers()).await? {
        Ok(next.run(request).await)
    } else {
        Err(Error::Unauthorized)
    }
}
