use axum::Json;
use axum::body::Body;
use axum::extract::{Path, State};
use axum::http::{StatusCode, header};
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::ai::{self, ProviderConnection, ProviderKind};
use crate::error::{Error, Result};
use crate::plugins::{self, CommunityPlugin, PluginConfig, SecretChanges, Secrets, States};

const API_KEY_SECRET: &str = "api_key";
use crate::state::AppState;

#[derive(Serialize)]
pub struct PluginList {
    states: States,
    community: Vec<CommunityPlugin>,
}

#[derive(Deserialize)]
pub struct StateBody {
    enabled: bool,
}

pub async fn list(State(state): State<AppState>) -> Result<Json<PluginList>> {
    let states = state.read(plugins::states).await?;
    let root = state.config.plugins_dir();
    let community = tokio::task::spawn_blocking(move || plugins::discover(&root))
        .await
        .map_err(|e| Error::Internal(e.to_string()))?;
    Ok(Json(PluginList { states, community }))
}

pub async fn set_state(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<StateBody>,
) -> Result<Json<States>> {
    plugins::check_id(&id)?;
    let enabled = body.enabled;
    Ok(Json(
        state
            .write(move |tx| plugins::set_state(tx, &id, enabled))
            .await?,
    ))
}

pub async fn file(
    State(state): State<AppState>,
    Path((id, path)): Path<(String, String)>,
) -> Result<Response> {
    let dir = plugins::plugin_dir(&state.config.plugins_dir(), &id)?;
    let resolved = tokio::task::spawn_blocking(move || {
        plugins::resolve_file(&dir, &path).map(|file| {
            let mime = mime_guess::from_path(&file).first_or_octet_stream();
            std::fs::read(&file).map(|data| (mime, data))
        })
    })
    .await
    .map_err(|e| Error::Internal(e.to_string()))?;
    let Some(read) = resolved else {
        return Ok(StatusCode::NOT_FOUND.into_response());
    };
    let (mime, data) = read.map_err(|e| Error::Internal(e.to_string()))?;
    Ok(Response::builder()
        .header(header::CONTENT_TYPE, mime.as_ref())
        .header(header::CACHE_CONTROL, "no-cache")
        .body(Body::from(data))
        .unwrap_or_else(|_| StatusCode::INTERNAL_SERVER_ERROR.into_response()))
}

#[derive(Deserialize)]
pub struct ConfigBody {
    #[serde(default)]
    settings: Value,
    #[serde(default)]
    secrets: SecretChanges,
}

async fn stored_secrets(state: &AppState, id: &str) -> Result<(Value, Secrets)> {
    let owned = id.to_string();
    let stored = state.read(move |c| plugins::load_config(c, &owned)).await?;
    let secrets = plugins::open_secrets(&state.vault, stored.sealed_secrets.as_deref())?;
    Ok((stored.settings, secrets))
}

pub async fn get_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> Result<Json<PluginConfig>> {
    plugins::check_id(&id)?;
    let (settings, secrets) = stored_secrets(&state, &id).await?;
    Ok(Json(PluginConfig {
        settings,
        secrets: secrets.into_keys().collect(),
    }))
}

pub async fn put_config(
    State(state): State<AppState>,
    Path(id): Path<String>,
    Json(body): Json<ConfigBody>,
) -> Result<Json<PluginConfig>> {
    plugins::check_id(&id)?;
    plugins::validate_settings(&body.settings)?;
    let (_, current) = stored_secrets(&state, &id).await?;
    let secrets = plugins::apply_secret_changes(current, body.secrets)?;
    let sealed = plugins::seal_secrets(&state.vault, &secrets)?;
    let settings = body.settings;
    let saved = settings.clone();
    state
        .write(move |tx| plugins::save_config(tx, &id, &saved, &sealed))
        .await?;
    Ok(Json(PluginConfig {
        settings,
        secrets: secrets.into_keys().collect(),
    }))
}

#[derive(Deserialize)]
pub struct ModelsBody {
    kind: ProviderKind,
    base_url: String,
    api_key: Option<String>,
    plugin: Option<String>,
}

#[derive(Serialize)]
pub struct ModelsResponse {
    models: Vec<String>,
}

pub async fn ai_models(
    State(state): State<AppState>,
    Json(body): Json<ModelsBody>,
) -> Result<Json<ModelsResponse>> {
    let typed = body
        .api_key
        .map(|key| key.trim().to_string())
        .filter(|key| !key.is_empty());
    let api_key = match (typed, body.plugin) {
        (Some(key), _) => Some(key),
        (None, Some(plugin)) => {
            plugins::check_id(&plugin)?;
            stored_secrets(&state, &plugin)
                .await?
                .1
                .remove(API_KEY_SECRET)
        }
        (None, None) => None,
    };
    let connection = ProviderConnection {
        kind: body.kind,
        base_url: body.base_url,
        api_key,
    };
    let models = ai::list_models(&state.http, &connection).await?;
    Ok(Json(ModelsResponse { models }))
}
