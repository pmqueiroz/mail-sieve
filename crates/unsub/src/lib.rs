mod mailto;
mod one_click;

use std::time::Duration;

pub use mailto::{Mailto, SmtpAuth, SmtpSettings, parse_mailto, send_mailto};
pub use one_click::{is_public_address, post_one_click, validate_one_click_url};

pub const REQUEST_TIMEOUT: Duration = Duration::from_secs(20);

#[derive(Debug, Clone, Default)]
pub struct Options {
    pub http: Option<String>,
    pub mailto: Option<String>,
    pub one_click: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Method {
    OneClick,
    Mailto,
    Link,
}

impl Method {
    pub fn as_str(self) -> &'static str {
        match self {
            Method::OneClick => "one_click",
            Method::Mailto => "mailto",
            Method::Link => "link",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Outcome {
    Completed { method: Method, detail: String },
    OpenLink(String),
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("sender offers no unsubscribe method")]
    NoMethod,
    #[error("unsafe unsubscribe url: {0}")]
    UnsafeUrl(String),
    #[error("unsubscribe request failed: {0}")]
    Http(#[from] reqwest::Error),
    #[error("unsubscribe endpoint answered {0}")]
    Status(u16),
    #[error("invalid mailto uri")]
    InvalidMailto,
    #[error("smtp is not configured for this account")]
    NoSmtp,
    #[error("smtp: {0}")]
    Smtp(String),
}

pub type Result<T> = std::result::Result<T, Error>;

pub async fn unsubscribe(options: &Options, smtp: Option<&SmtpSettings>) -> Result<Outcome> {
    if options.one_click
        && let Some(url) = &options.http
    {
        match post_one_click(url).await {
            Ok(status) => {
                return Ok(Outcome::Completed {
                    method: Method::OneClick,
                    detail: format!("HTTP {status}"),
                });
            }
            Err(error) if options.mailto.is_none() => return Err(error),
            Err(_) => {}
        }
    }
    if let Some(uri) = &options.mailto {
        match smtp {
            Some(settings) => {
                let mailto = parse_mailto(uri).ok_or(Error::InvalidMailto)?;
                send_mailto(settings, &mailto).await?;
                return Ok(Outcome::Completed {
                    method: Method::Mailto,
                    detail: format!("sent to {}", mailto.to),
                });
            }
            None if options.http.is_none() => return Err(Error::NoSmtp),
            None => {}
        }
    }
    match &options.http {
        Some(url) => Ok(Outcome::OpenLink(url.clone())),
        None => Err(Error::NoMethod),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn falls_back_to_link_without_smtp() {
        let options = Options {
            http: Some("https://news.example/u".into()),
            mailto: Some("mailto:u@news.example".into()),
            one_click: false,
        };
        assert_eq!(
            unsubscribe(&options, None).await.unwrap(),
            Outcome::OpenLink("https://news.example/u".into())
        );
    }

    #[tokio::test]
    async fn reports_missing_methods() {
        assert!(matches!(
            unsubscribe(&Options::default(), None).await,
            Err(Error::NoMethod)
        ));
        let mailto_only = Options {
            mailto: Some("mailto:u@x.example".into()),
            ..Default::default()
        };
        assert!(matches!(
            unsubscribe(&mailto_only, None).await,
            Err(Error::NoSmtp)
        ));
    }
}
