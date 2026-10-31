use std::convert::Infallible;
use std::time::Duration;

use axum::Json;
use axum::extract::{Path, Query, State};
use axum::response::sse::{Event as SseEvent, KeepAlive, Sse};
use futures::Stream;
use futures::StreamExt;
use mail_sieve_store::jobs::{self, Job};
use serde::Deserialize;
use serde_json::{Value, json};
use tokio_stream::wrappers::BroadcastStream;

use crate::error::{Error, Result};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct ListParams {
    limit: Option<u32>,
}

pub async fn list(
    State(state): State<AppState>,
    Query(params): Query<ListParams>,
) -> Result<Json<Vec<Job>>> {
    let limit = params.limit.unwrap_or(50).min(500);
    Ok(Json(state.read(move |c| jobs::list(c, limit)).await?))
}

pub async fn get(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Job>> {
    Ok(Json(state.read(move |c| jobs::get(c, id)).await?))
}

pub async fn cancel(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>> {
    if !state.write(move |tx| jobs::cancel(tx, id)).await? {
        return Err(Error::Conflict("job already finished".into()));
    }
    state.emit_job(id).await;
    Ok(Json(json!({ "cancelled": true })))
}

pub async fn undo(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>> {
    let job_id = crate::undo::start(&state, id).await?;
    Ok(Json(json!({ "job_id": job_id })))
}

pub async fn events(
    State(state): State<AppState>,
) -> Sse<impl Stream<Item = std::result::Result<SseEvent, Infallible>>> {
    let stream = BroadcastStream::new(state.events.subscribe()).filter_map(|event| async move {
        let event = event.ok()?;
        SseEvent::default().json_data(&event).ok().map(Ok)
    });
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(20)))
}
