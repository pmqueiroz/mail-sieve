use axum::Json;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use serde_json::json;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Store(#[from] mail_sieve_store::Error),
    #[error(transparent)]
    Imap(#[from] mail_sieve_imap::Error),
    #[error(transparent)]
    Unsubscribe(#[from] mail_sieve_unsub::Error),
    #[error(transparent)]
    Query(#[from] mail_sieve_core::QueryError),
    #[error("{0}")]
    BadRequest(String),
    #[error("not authenticated")]
    Unauthorized,
    #[error("{0}")]
    Conflict(String),
    #[error("oauth: {0}")]
    OAuth(String),
    #[error("{0}")]
    Internal(String),
}

pub type Result<T> = std::result::Result<T, Error>;

impl Error {
    fn status(&self) -> StatusCode {
        match self {
            Error::Store(mail_sieve_store::Error::NotFound) => StatusCode::NOT_FOUND,
            Error::Store(mail_sieve_store::Error::Query(_))
            | Error::Query(_)
            | Error::BadRequest(_) => StatusCode::BAD_REQUEST,
            Error::Unauthorized => StatusCode::UNAUTHORIZED,
            Error::Conflict(_) => StatusCode::CONFLICT,
            Error::Imap(_) | Error::Unsubscribe(_) | Error::OAuth(_) => StatusCode::BAD_GATEWAY,
            Error::Store(_) | Error::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = self.status();
        if status.is_server_error() {
            tracing::error!(error = %self, "request failed");
        }
        (status, Json(json!({ "error": self.to_string() }))).into_response()
    }
}

impl From<tokio::task::JoinError> for Error {
    fn from(error: tokio::task::JoinError) -> Self {
        Error::Internal(error.to_string())
    }
}
