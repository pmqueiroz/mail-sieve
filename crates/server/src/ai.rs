use std::time::Duration;

use reqwest::Url;
use serde::Deserialize;

use crate::error::{Error, Result};

const ANTHROPIC_VERSION: &str = "2026-10-21";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ERROR_LEN: usize = 300;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    Anthropic,
    Openai,
}

pub struct ProviderConnection {
    pub kind: ProviderKind,
    pub base_url: String,
    pub api_key: Option<String>,
}

#[derive(Deserialize)]
struct ModelList {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
struct ModelEntry {
    id: String,
}

pub fn models_url(kind: ProviderKind, base_url: &str) -> Result<Url> {
    let base = base_url.trim().trim_end_matches('/');
    let path = match kind {
        ProviderKind::Anthropic => "/v1/models?limit=1000",
        ProviderKind::Openai => "/models",
    };
    let url = Url::parse(&format!("{base}{path}"))
        .map_err(|_| Error::BadRequest(format!("`{base_url}` is not a valid URL")))?;
    if !matches!(url.scheme(), "http" | "https")
        || !url.username().is_empty()
        || url.password().is_some()
    {
        return Err(Error::BadRequest(
            "the base URL must be http(s) without credentials".into(),
        ));
    }
    Ok(url)
}

pub fn parse_models(body: &str) -> Result<Vec<String>> {
    let list: ModelList = serde_json::from_str(body)
        .map_err(|_| Error::Upstream("the provider did not return a model list".into()))?;
    let mut models: Vec<String> = list.data.into_iter().map(|entry| entry.id).collect();
    models.sort();
    models.dedup();
    Ok(models)
}

fn upstream_error(status: reqwest::StatusCode, body: &str) -> Error {
    let message = serde_json::from_str::<serde_json::Value>(body)
        .ok()
        .and_then(|value| {
            value
                .pointer("/error/message")
                .or_else(|| value.pointer("/error"))
                .and_then(|message| message.as_str().map(str::to_string))
        })
        .unwrap_or_else(|| body.chars().take(MAX_ERROR_LEN).collect());
    Error::Upstream(format!(
        "the provider answered {status}: {}",
        message.trim()
    ))
}

fn describe(error: reqwest::Error) -> String {
    let error = error.without_url();
    let mut message = error.to_string();
    let mut source = std::error::Error::source(&error);
    while let Some(cause) = source {
        message = cause.to_string();
        source = cause.source();
    }
    message
}

pub async fn list_models(
    http: &reqwest::Client,
    connection: &ProviderConnection,
) -> Result<Vec<String>> {
    let url = models_url(connection.kind, &connection.base_url)?;
    let mut request = http.get(url).timeout(REQUEST_TIMEOUT);
    let key = connection.api_key.as_deref().filter(|key| !key.is_empty());
    request = match (connection.kind, key) {
        (ProviderKind::Anthropic, key) => request
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("x-api-key", key.unwrap_or_default()),
        (ProviderKind::Openai, Some(key)) => request.bearer_auth(key),
        (ProviderKind::Openai, None) => request,
    };
    let response = request
        .send()
        .await
        .map_err(|e| Error::Upstream(format!("cannot reach the provider: {}", describe(e))))?;
    let status = response.status();
    let body = response
        .text()
        .await
        .map_err(|e| Error::Upstream(describe(e)))?;
    if !status.is_success() {
        return Err(upstream_error(status, &body));
    }
    parse_models(&body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_model_urls_per_provider() {
        assert_eq!(
            models_url(ProviderKind::Anthropic, "https://api.anthropic.com/")
                .unwrap()
                .as_str(),
            "https://api.anthropic.com/v1/models?limit=1000"
        );
        assert_eq!(
            models_url(ProviderKind::Openai, "http://ollama:11434/v1")
                .unwrap()
                .as_str(),
            "http://ollama:11434/v1/models"
        );
        assert!(models_url(ProviderKind::Openai, "file:///etc/passwd").is_err());
        assert!(models_url(ProviderKind::Openai, "https://user:pw@host/v1").is_err());
        assert!(models_url(ProviderKind::Openai, "not a url").is_err());
    }

    #[test]
    fn parses_openai_style_and_anthropic_lists() {
        let models =
            parse_models(r#"{"object":"list","data":[{"id":"llama3.2"},{"id":"ai/smollm2"}]}"#)
                .unwrap();
        assert_eq!(models, ["ai/smollm2", "llama3.2"]);
        let models = parse_models(r#"{"data":[{"id":"claude-opus-5-5","display_name":"Claude Opus 5.5"}],"has_more":false}"#).unwrap();
        assert_eq!(models, ["claude-opus-5-5"]);
        assert!(parse_models("<html>").is_err());
    }

    #[test]
    fn upstream_errors_use_the_provider_message() {
        let error = upstream_error(
            reqwest::StatusCode::UNAUTHORIZED,
            r#"{"type":"error","error":{"type":"authentication_error","message":"invalid x-api-key"}}"#,
        );
        assert!(error.to_string().contains("invalid x-api-key"));
    }
}
