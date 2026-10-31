use base64::Engine;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::error::{Error, Result};
use crate::state::AppState;

const PENDING_TTL_SECONDS: i64 = 600;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OAuthProvider {
    Google,
    Microsoft,
}

pub struct ServerEndpoints {
    pub imap_host: &'static str,
    pub smtp_host: &'static str,
}

impl OAuthProvider {
    pub fn parse(value: &str) -> Option<OAuthProvider> {
        match value {
            "google" => Some(OAuthProvider::Google),
            "microsoft" => Some(OAuthProvider::Microsoft),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            OAuthProvider::Google => "google",
            OAuthProvider::Microsoft => "microsoft",
        }
    }

    fn authorize_url(self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://accounts.google.com/o/oauth2/v2/auth",
            OAuthProvider::Microsoft => {
                "https://login.microsoftonline.com/common/oauth2/v2.0/authorize"
            }
        }
    }

    fn token_url(self) -> &'static str {
        match self {
            OAuthProvider::Google => "https://oauth2.googleapis.com/token",
            OAuthProvider::Microsoft => {
                "https://login.microsoftonline.com/common/oauth2/v2.0/token"
            }
        }
    }

    fn scopes(self) -> &'static str {
        match self {
            OAuthProvider::Google => "openid email https://mail.google.com/",
            OAuthProvider::Microsoft => {
                "openid email offline_access https://outlook.office.com/IMAP.AccessAsUser.All \
                 https://outlook.office.com/SMTP.Send"
            }
        }
    }

    pub fn endpoints(self) -> ServerEndpoints {
        match self {
            OAuthProvider::Google => ServerEndpoints {
                imap_host: "imap.gmail.com",
                smtp_host: "smtp.gmail.com",
            },
            OAuthProvider::Microsoft => ServerEndpoints {
                imap_host: "outlook.office365.com",
                smtp_host: "smtp.office365.com",
            },
        }
    }

    pub fn client(self, state: &AppState) -> Option<(String, Option<String>)> {
        let config = &state.config;
        match self {
            OAuthProvider::Google => config
                .google_client_id
                .clone()
                .map(|id| (id, config.google_client_secret.clone())),
            OAuthProvider::Microsoft => config
                .microsoft_client_id
                .clone()
                .map(|id| (id, config.microsoft_client_secret.clone())),
        }
    }

    fn redirect_uri(self, state: &AppState) -> String {
        format!(
            "{}/api/oauth/{}/callback",
            state.config.public_base(),
            self.as_str()
        )
    }
}

pub struct PendingAuthorization {
    provider: OAuthProvider,
    verifier: String,
    created_at: i64,
}

#[derive(Debug, Deserialize)]
pub struct TokenResponse {
    pub access_token: String,
    pub refresh_token: Option<String>,
    pub expires_in: Option<i64>,
    pub id_token: Option<String>,
}

fn random_token() -> Result<String> {
    let mut bytes = [0u8; 32];
    getrandom::fill(&mut bytes).map_err(|e| Error::Internal(e.to_string()))?;
    Ok(URL_SAFE_NO_PAD.encode(bytes))
}

pub fn start(state: &AppState, provider: OAuthProvider) -> Result<String> {
    let (client_id, _) = provider.client(state).ok_or_else(|| {
        Error::BadRequest(format!("{} sign-in is not configured", provider.as_str()))
    })?;
    let verifier = random_token()?;
    let challenge = URL_SAFE_NO_PAD.encode(Sha256::digest(verifier.as_bytes()));
    let csrf = random_token()?;
    let now = mail_sieve_store::now();
    {
        let mut pending = state
            .pending_oauth
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        pending.retain(|_, entry| now - entry.created_at < PENDING_TTL_SECONDS);
        pending.insert(
            csrf.clone(),
            PendingAuthorization {
                provider,
                verifier,
                created_at: now,
            },
        );
    }
    let mut url = reqwest::Url::parse(provider.authorize_url()).expect("static url");
    url.query_pairs_mut()
        .append_pair("client_id", &client_id)
        .append_pair("redirect_uri", &provider.redirect_uri(state))
        .append_pair("response_type", "code")
        .append_pair("scope", provider.scopes())
        .append_pair("state", &csrf)
        .append_pair("code_challenge", &challenge)
        .append_pair("code_challenge_method", "S256")
        .append_pair("access_type", "offline")
        .append_pair("prompt", "consent");
    Ok(url.to_string())
}

async fn token_request(
    state: &AppState,
    provider: OAuthProvider,
    params: &[(&str, &str)],
) -> Result<TokenResponse> {
    let (client_id, client_secret) = provider
        .client(state)
        .ok_or_else(|| Error::OAuth(format!("{} is not configured", provider.as_str())))?;
    let mut form: Vec<(&str, &str)> = params.to_vec();
    form.push(("client_id", &client_id));
    if let Some(secret) = &client_secret {
        form.push(("client_secret", secret));
    }
    let response = state
        .http
        .post(provider.token_url())
        .form(&form)
        .send()
        .await
        .map_err(|e| Error::OAuth(e.to_string()))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(Error::OAuth(format!(
            "token endpoint answered {status}: {body}"
        )));
    }
    response
        .json()
        .await
        .map_err(|e| Error::OAuth(e.to_string()))
}

pub struct CompletedAuthorization {
    pub email: String,
    pub tokens: TokenResponse,
}

pub async fn complete(
    state: &AppState,
    provider: OAuthProvider,
    code: &str,
    csrf: &str,
) -> Result<CompletedAuthorization> {
    let pending = state
        .pending_oauth
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .remove(csrf)
        .ok_or_else(|| Error::OAuth("unknown or expired sign-in attempt".into()))?;
    if pending.provider != provider
        || mail_sieve_store::now() - pending.created_at >= PENDING_TTL_SECONDS
    {
        return Err(Error::OAuth("unknown or expired sign-in attempt".into()));
    }
    let redirect_uri = provider.redirect_uri(state);
    let tokens = token_request(
        state,
        provider,
        &[
            ("grant_type", "authorization_code"),
            ("code", code),
            ("redirect_uri", &redirect_uri),
            ("code_verifier", &pending.verifier),
        ],
    )
    .await?;
    let email = tokens
        .id_token
        .as_deref()
        .and_then(email_from_id_token)
        .ok_or_else(|| Error::OAuth("provider did not return the account email".into()))?;
    Ok(CompletedAuthorization { email, tokens })
}

pub async fn refresh(
    state: &AppState,
    provider: OAuthProvider,
    refresh_token: &str,
) -> Result<TokenResponse> {
    token_request(
        state,
        provider,
        &[
            ("grant_type", "refresh_token"),
            ("refresh_token", refresh_token),
        ],
    )
    .await
}

pub fn email_from_id_token(id_token: &str) -> Option<String> {
    let payload = id_token.split('.').nth(1)?;
    let decoded = URL_SAFE_NO_PAD.decode(payload.trim_end_matches('=')).ok()?;
    let claims: serde_json::Value = serde_json::from_slice(&decoded).ok()?;
    claims
        .get("email")
        .or_else(|| claims.get("preferred_username"))
        .and_then(|value| value.as_str())
        .map(str::to_string)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_email_claim() {
        let payload = URL_SAFE_NO_PAD.encode(br#"{"email":"me@gmail.com","sub":"1"}"#);
        assert_eq!(
            email_from_id_token(&format!("h.{payload}.s")).as_deref(),
            Some("me@gmail.com")
        );
        let microsoft = URL_SAFE_NO_PAD.encode(br#"{"preferred_username":"me@outlook.com"}"#);
        assert_eq!(
            email_from_id_token(&format!("h.{microsoft}.s")).as_deref(),
            Some("me@outlook.com")
        );
        assert_eq!(email_from_id_token("garbage"), None);
    }
}
