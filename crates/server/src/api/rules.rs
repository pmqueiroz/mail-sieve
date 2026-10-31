use axum::Json;
use axum::extract::{Path, State};
use axum::http::{HeaderValue, StatusCode, header};
use axum::response::IntoResponse;
use mail_sieve_store::rules::{self, Rule, RuleInput};
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::Result;
use crate::rules::{run_rule, validate};
use crate::rules_file::{self, ImportReport, OnConflict};
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

pub async fn export(
    State(state): State<AppState>,
    Path(account_id): Path<i64>,
) -> Result<impl IntoResponse> {
    let (account, rules) = state
        .read(move |c| {
            let account = mail_sieve_store::accounts::get(c, account_id)?;
            Ok((account, rules::for_account(c, account_id)?))
        })
        .await?;
    let yaml = rules_file::export(&rules)?;
    let disposition = format!(
        "attachment; filename=\"{}\"",
        export_file_name(&account.email)
    );
    Ok((
        [
            (
                header::CONTENT_TYPE,
                HeaderValue::from_static("application/yaml; charset=utf-8"),
            ),
            (
                header::CONTENT_DISPOSITION,
                HeaderValue::try_from(disposition)
                    .map_err(|e| crate::error::Error::Internal(e.to_string()))?,
            ),
        ],
        yaml,
    ))
}

fn export_file_name(email: &str) -> String {
    let safe: String = email
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || matches!(c, '@' | '.' | '-' | '_') {
                c
            } else {
                '_'
            }
        })
        .collect();
    format!("mail-sieve-rules-{safe}.yaml")
}

#[derive(Deserialize)]
pub struct ImportBody {
    yaml: String,
    #[serde(default)]
    on_conflict: OnConflict,
    #[serde(default)]
    apply: bool,
}

pub async fn import(
    State(state): State<AppState>,
    Path(account_id): Path<i64>,
    Json(body): Json<ImportBody>,
) -> Result<Json<ImportReport>> {
    state
        .read(move |c| mail_sieve_store::accounts::get(c, account_id))
        .await?;
    let file = rules_file::parse(&body.yaml, account_id)?;
    let report = if body.apply {
        state
            .write(move |tx| rules_file::import(tx, file, body.on_conflict, true))
            .await?
    } else {
        state
            .read(move |c| rules_file::import(c, file, body.on_conflict, false))
            .await?
    };
    Ok(Json(report))
}
