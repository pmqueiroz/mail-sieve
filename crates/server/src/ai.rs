use std::time::Duration;

use reqwest::Url;
use serde::Deserialize;
use serde_json::{Value, json};

use crate::error::{Error, Result};

const ANTHROPIC_VERSION: &str = "2026-10-21";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
const MAX_ERROR_LEN: usize = 300;
const COMPLETION_TIMEOUT: Duration = Duration::from_secs(90);
const MAX_PROMPT_BYTES: usize = 100 * 1024;
pub const MAX_TOKENS: u32 = 4096;

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

fn endpoint(base_url: &str, path: &str) -> Result<Url> {
    let base = base_url.trim().trim_end_matches('/');
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

pub fn models_url(kind: ProviderKind, base_url: &str) -> Result<Url> {
    let path = match kind {
        ProviderKind::Anthropic => "/v1/models?limit=1000",
        ProviderKind::Openai => "/models",
    };
    endpoint(base_url, path)
}

pub fn completion_url(kind: ProviderKind, base_url: &str) -> Result<Url> {
    let path = match kind {
        ProviderKind::Anthropic => "/v1/messages",
        ProviderKind::Openai => "/chat/completions",
    };
    endpoint(base_url, path)
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

fn authorize(
    request: reqwest::RequestBuilder,
    connection: &ProviderConnection,
) -> reqwest::RequestBuilder {
    let key = connection.api_key.as_deref().filter(|key| !key.is_empty());
    match (connection.kind, key) {
        (ProviderKind::Anthropic, key) => request
            .header("anthropic-version", ANTHROPIC_VERSION)
            .header("x-api-key", key.unwrap_or_default()),
        (ProviderKind::Openai, Some(key)) => request.bearer_auth(key),
        (ProviderKind::Openai, None) => request,
    }
}

async fn send(request: reqwest::RequestBuilder) -> Result<String> {
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
    Ok(body)
}

pub async fn list_models(
    http: &reqwest::Client,
    connection: &ProviderConnection,
) -> Result<Vec<String>> {
    let url = models_url(connection.kind, &connection.base_url)?;
    let request = authorize(http.get(url).timeout(REQUEST_TIMEOUT), connection);
    parse_models(&send(request).await?)
}

pub struct Completion {
    pub model: String,
    pub system: String,
    pub prompt: String,
    pub max_tokens: u32,
}

impl Completion {
    pub fn validate(&self) -> Result<()> {
        if self.model.trim().is_empty() {
            return Err(Error::BadRequest("choose a model".into()));
        }
        if self.prompt.trim().is_empty() {
            return Err(Error::BadRequest("the prompt is empty".into()));
        }
        if self.prompt.len() + self.system.len() > MAX_PROMPT_BYTES {
            return Err(Error::BadRequest(format!(
                "the prompt is larger than {} KB",
                MAX_PROMPT_BYTES / 1024
            )));
        }
        Ok(())
    }
}

pub fn completion_body(kind: ProviderKind, completion: &Completion) -> Value {
    let model = completion.model.trim();
    let max_tokens = completion.max_tokens.clamp(1, MAX_TOKENS);
    match kind {
        ProviderKind::Anthropic => {
            let mut body = json!({
                "model": model,
                "max_tokens": max_tokens,
                "messages": [{ "role": "user", "content": completion.prompt }],
            });
            if !completion.system.trim().is_empty() {
                body["system"] = json!(completion.system);
            }
            body
        }
        ProviderKind::Openai => {
            let mut messages = Vec::new();
            if !completion.system.trim().is_empty() {
                messages.push(json!({ "role": "system", "content": completion.system }));
            }
            messages.push(json!({ "role": "user", "content": completion.prompt }));
            json!({ "model": model, "max_tokens": max_tokens, "messages": messages })
        }
    }
}

pub fn parse_completion(kind: ProviderKind, body: &str) -> Result<String> {
    let invalid = || Error::Upstream("the provider did not return a completion".into());
    let value: Value = serde_json::from_str(body).map_err(|_| invalid())?;
    let text = match kind {
        ProviderKind::Anthropic => value
            .get("content")
            .and_then(Value::as_array)
            .map(|blocks| {
                blocks
                    .iter()
                    .filter(|block| block.get("type").and_then(Value::as_str) == Some("text"))
                    .filter_map(|block| block.get("text").and_then(Value::as_str))
                    .collect::<String>()
            }),
        ProviderKind::Openai => value
            .pointer("/choices/0/message/content")
            .and_then(Value::as_str)
            .map(str::to_string),
    };
    text.ok_or_else(invalid)
}

pub async fn complete(
    http: &reqwest::Client,
    connection: &ProviderConnection,
    completion: &Completion,
) -> Result<String> {
    completion.validate()?;
    let url = completion_url(connection.kind, &connection.base_url)?;
    let request = authorize(
        http.post(url)
            .timeout(COMPLETION_TIMEOUT)
            .json(&completion_body(connection.kind, completion)),
        connection,
    );
    parse_completion(connection.kind, &send(request).await?)
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

    #[test]
    fn builds_completion_requests_per_provider() {
        assert_eq!(
            completion_url(ProviderKind::Anthropic, "https://api.anthropic.com")
                .unwrap()
                .as_str(),
            "https://api.anthropic.com/v1/messages"
        );
        assert_eq!(
            completion_url(ProviderKind::Openai, "http://ollama:11434/v1/")
                .unwrap()
                .as_str(),
            "http://ollama:11434/v1/chat/completions"
        );
        let completion = Completion {
            model: " claude-opus-5-5 ".into(),
            system: "Be brief.".into(),
            prompt: "Hi".into(),
            max_tokens: 100_000,
        };
        let anthropic = completion_body(ProviderKind::Anthropic, &completion);
        assert_eq!(anthropic["model"], "claude-opus-5-5");
        assert_eq!(anthropic["system"], "Be brief.");
        assert_eq!(anthropic["max_tokens"], MAX_TOKENS);
        assert_eq!(anthropic["messages"][0]["content"], "Hi");
        let openai = completion_body(ProviderKind::Openai, &completion);
        assert_eq!(openai["messages"][0]["role"], "system");
        assert_eq!(openai["messages"][1]["content"], "Hi");
    }

    #[test]
    fn parses_completions_per_provider() {
        let anthropic = r#"{"content":[{"type":"thinking","thinking":"x"},{"type":"text","text":"[1,"},{"type":"text","text":"2]"}]}"#;
        assert_eq!(
            parse_completion(ProviderKind::Anthropic, anthropic).unwrap(),
            "[1,2]"
        );
        let openai = r#"{"choices":[{"message":{"role":"assistant","content":"hello"}}]}"#;
        assert_eq!(
            parse_completion(ProviderKind::Openai, openai).unwrap(),
            "hello"
        );
        assert!(parse_completion(ProviderKind::Openai, r#"{"choices":[]}"#).is_err());
    }

    #[test]
    fn rejects_empty_or_oversized_prompts() {
        let completion = |prompt: String| Completion {
            model: "m".into(),
            system: String::new(),
            prompt,
            max_tokens: 10,
        };
        assert!(completion("  ".into()).validate().is_err());
        assert!(
            completion("x".repeat(MAX_PROMPT_BYTES + 1))
                .validate()
                .is_err()
        );
        assert!(completion("ok".into()).validate().is_ok());
    }
}
