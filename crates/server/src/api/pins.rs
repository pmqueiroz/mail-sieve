use axum::Json;
use axum::extract::{Path, Query as QueryParams, State};
use axum::http::StatusCode;
use mail_sieve_store::pins::{self, Pin, PinKind};
use serde::Deserialize;

use crate::accounts;
use crate::error::{Error, Result};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct PinBody {
    kind: PinKind,
    key: String,
}

fn validate(body: &PinBody) -> Result<String> {
    let key = pins::normalize_key(&body.key);
    let valid = match body.kind {
        PinKind::Sender => key.contains('@'),
        PinKind::Domain => !key.contains('@'),
    };
    if key.is_empty() || key.contains(char::is_whitespace) || !valid {
        return Err(Error::BadRequest(format!(
            "`{}` is not a valid {} to pin",
            body.key.trim(),
            body.kind
        )));
    }
    Ok(key)
}

pub async fn list(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Vec<Pin>>> {
    accounts::load(&state, id).await?;
    Ok(Json(state.read(move |c| pins::list(c, id)).await?))
}

pub async fn add(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<PinBody>,
) -> Result<(StatusCode, Json<Pin>)> {
    let key = validate(&body)?;
    accounts::load(&state, id).await?;
    let kind = body.kind;
    let pin = state.write(move |tx| pins::add(tx, id, kind, &key)).await?;
    Ok((StatusCode::CREATED, Json(pin)))
}

pub async fn remove(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    QueryParams(params): QueryParams<PinBody>,
) -> Result<StatusCode> {
    let key = validate(&params)?;
    let kind = params.kind;
    state
        .write(move |tx| pins::remove(tx, id, kind, &key))
        .await?;
    Ok(StatusCode::NO_CONTENT)
}
