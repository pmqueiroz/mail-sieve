use axum::Json;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use mail_sieve_store::rules::{self, Rule, RuleInput};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::Result;
use crate::rules::{run_rule, validate};
use crate::state::AppState;

#[derive(Deserialize)]
pub struct RuleBody {
    account_id: i64,
    name: String,
    query: String,
    action: String,
    #[serde(default = "enabled_by_default")]
    enabled: bool,
    interval_minutes: Option<u32>,
    #[serde(default)]
    run_after_sync: bool,
}

fn enabled_by_default() -> bool {
    true
}

impl RuleBody {
    fn into_input(self) -> Result<RuleInput> {
        let input = RuleInput {
            account_id: self.account_id,
            name: self.name.trim().to_string(),
            query: self.query.trim().to_string(),
            action: self.action,
            enabled: self.enabled,
            interval_minutes: self.interval_minutes,
            run_after_sync: self.run_after_sync,
        };
        validate(&input)?;
        Ok(input)
    }
}

pub async fn list(State(state): State<AppState>) -> Result<Json<Vec<Rule>>> {
    Ok(Json(state.read(rules::list).await?))
}

pub async fn create(
    State(state): State<AppState>,
    Json(body): Json<RuleBody>,
) -> Result<(StatusCode, Json<Rule>)> {
    let input = body.into_input()?;
    let rule = state
        .write(move |tx| {
            let id = rules::insert(tx, &input)?;
            rules::get(tx, id)
        })
        .await?;
    Ok((StatusCode::CREATED, Json(rule)))
}

pub async fn update(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(body): Json<RuleBody>,
) -> Result<Json<Rule>> {
    let input = body.into_input()?;
    let rule = state
        .write(move |tx| {
            rules::update(tx, id, &input)?;
            rules::get(tx, id)
        })
        .await?;
    Ok(Json(rule))
}

pub async fn remove(State(state): State<AppState>, Path(id): Path<i64>) -> Result<StatusCode> {
    state.write(move |tx| rules::delete(tx, id)).await?;
    Ok(StatusCode::NO_CONTENT)
}

pub async fn run(State(state): State<AppState>, Path(id): Path<i64>) -> Result<Json<Value>> {
    let rule = state.read(move |c| rules::get(c, id)).await?;
    let job_id = run_rule(&state, &rule).await?;
    Ok(Json(json!({ "job_id": job_id })))
}
