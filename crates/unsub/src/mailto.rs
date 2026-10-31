use lettre::message::Mailbox;
use lettre::transport::smtp::authentication::{Credentials, Mechanism};
use lettre::{AsyncSmtpTransport, AsyncTransport, Message, Tokio1Executor};
use url::Url;

use crate::{Error, REQUEST_TIMEOUT, Result};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Mailto {
    pub to: String,
    pub subject: String,
    pub body: String,
}

pub fn parse_mailto(raw: &str) -> Option<Mailto> {
    let url = Url::parse(raw.trim()).ok()?;
    if url.scheme() != "mailto" {
        return None;
    }
    let to = percent_decode(url.path());
    let to = to.split(',').next()?.trim().to_string();
    if !to.contains('@') {
        return None;
    }
    let mut mailto = Mailto {
        to,
        subject: "unsubscribe".into(),
        body: "unsubscribe".into(),
    };
    for (key, value) in url.query_pairs() {
        match key.to_ascii_lowercase().as_str() {
            "subject" if !value.is_empty() => mailto.subject = value.into_owned(),
            "body" if !value.is_empty() => mailto.body = value.into_owned(),
            _ => {}
        }
    }
    Some(mailto)
}

fn percent_decode(value: &str) -> String {
    url::form_urlencoded::parse(format!("v={}", value.replace('+', "%2B")).as_bytes())
        .next()
        .map(|(_, decoded)| decoded.into_owned())
        .unwrap_or_default()
}

#[derive(Clone)]
pub enum SmtpAuth {
    Password(String),
    OAuth2(String),
}

#[derive(Clone)]
pub struct SmtpSettings {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub auth: SmtpAuth,
    pub from: String,
}

pub async fn send_mailto(settings: &SmtpSettings, mailto: &Mailto) -> Result<()> {
    let smtp_error = |error: &dyn std::fmt::Display| Error::Smtp(error.to_string());
    let from: Mailbox = settings.from.parse().map_err(|e| smtp_error(&e))?;
    let to: Mailbox = mailto.to.parse().map_err(|e| smtp_error(&e))?;
    let message = Message::builder()
        .from(from)
        .to(to)
        .subject(&mailto.subject)
        .body(mailto.body.clone())
        .map_err(|e| smtp_error(&e))?;
    let builder = if settings.port == 465 {
        AsyncSmtpTransport::<Tokio1Executor>::relay(&settings.host)
    } else {
        AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&settings.host)
    }
    .map_err(|e| smtp_error(&e))?;
    let (secret, mechanism) = match &settings.auth {
        SmtpAuth::Password(password) => (password.clone(), Mechanism::Plain),
        SmtpAuth::OAuth2(token) => (token.clone(), Mechanism::Xoauth2),
    };
    let transport = builder
        .port(settings.port)
        .timeout(Some(REQUEST_TIMEOUT))
        .credentials(Credentials::new(settings.username.clone(), secret))
        .authentication(vec![mechanism])
        .build();
    transport.send(message).await.map_err(|e| smtp_error(&e))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_mailto_with_query() {
        let parsed =
            parse_mailto("mailto:unsub%2B123@news.example?subject=Remove%20me&body=bye").unwrap();
        assert_eq!(
            parsed,
            Mailto {
                to: "unsub+123@news.example".into(),
                subject: "Remove me".into(),
                body: "bye".into()
            }
        );
        let defaults = parse_mailto("mailto:u@news.example").unwrap();
        assert_eq!(defaults.subject, "unsubscribe");
        assert!(parse_mailto("https://x.example").is_none());
        assert!(parse_mailto("mailto:nobody").is_none());
    }
}
