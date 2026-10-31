use std::fmt;
use std::sync::{Arc, OnceLock};
use std::time::Duration;

use async_imap::types::Capability;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use tokio_rustls::TlsConnector;
use tokio_rustls::rustls::pki_types::ServerName;
use tokio_rustls::rustls::{ClientConfig, RootCertStore};

use crate::{Error, Result};

pub trait ImapIo: AsyncRead + AsyncWrite + Unpin + Send + fmt::Debug {}

impl<T: AsyncRead + AsyncWrite + Unpin + Send + fmt::Debug> ImapIo for T {}

pub type Session = async_imap::Session<Box<dyn ImapIo>>;

const CONNECT_TIMEOUT: Duration = Duration::from_secs(30);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Security {
    Tls,
    Plain,
}

#[derive(Clone)]
pub enum Credentials {
    Password(String),
    OAuth2 { access_token: String },
}

#[derive(Clone)]
pub struct ImapConfig {
    pub host: String,
    pub port: u16,
    pub security: Security,
    pub username: String,
    pub credentials: Credentials,
    pub compress: bool,
}

impl fmt::Debug for ImapConfig {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ImapConfig")
            .field("host", &self.host)
            .field("port", &self.port)
            .field("security", &self.security)
            .field("username", &self.username)
            .finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, Default)]
pub struct Capabilities {
    names: Vec<String>,
}

impl Capabilities {
    pub fn from_names(names: impl IntoIterator<Item = String>) -> Self {
        let mut names: Vec<String> = names
            .into_iter()
            .map(|name| name.to_ascii_uppercase())
            .collect();
        names.sort();
        names.dedup();
        Capabilities { names }
    }

    pub fn has(&self, name: &str) -> bool {
        self.names.binary_search(&name.to_ascii_uppercase()).is_ok()
    }

    pub fn names(&self) -> &[String] {
        &self.names
    }

    pub fn supports_move(&self) -> bool {
        self.has("MOVE")
    }

    pub fn uidplus(&self) -> bool {
        self.has("UIDPLUS")
    }

    pub fn condstore(&self) -> bool {
        self.has("CONDSTORE") || self.has("QRESYNC")
    }

    pub fn gmail(&self) -> bool {
        self.has("X-GM-EXT-1")
    }

    pub fn idle(&self) -> bool {
        self.has("IDLE")
    }
}

pub struct Connection {
    pub session: Session,
    pub capabilities: Capabilities,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MailboxStatus {
    pub exists: u32,
    pub uidvalidity: u32,
    pub uidnext: u32,
    pub highestmodseq: Option<u64>,
}

impl Connection {
    pub async fn examine(&mut self, mailbox: &str) -> Result<MailboxStatus> {
        let selected = if self.capabilities.condstore() {
            self.session.select_condstore(mailbox).await?
        } else {
            self.session.examine(mailbox).await?
        };
        status(mailbox, selected)
    }

    pub async fn select(&mut self, mailbox: &str) -> Result<MailboxStatus> {
        let selected = if self.capabilities.condstore() {
            self.session.select_condstore(mailbox).await?
        } else {
            self.session.select(mailbox).await?
        };
        status(mailbox, selected)
    }

    pub async fn logout(mut self) {
        let _ = self.session.logout().await;
    }
}

fn status(mailbox: &str, selected: async_imap::types::Mailbox) -> Result<MailboxStatus> {
    let uidvalidity = selected
        .uid_validity
        .ok_or_else(|| Error::NoUidValidity(mailbox.to_string()))?;
    Ok(MailboxStatus {
        exists: selected.exists,
        uidvalidity,
        uidnext: selected.uid_next.unwrap_or(1),
        highestmodseq: selected.highest_modseq,
    })
}

struct XOAuth2<'a> {
    user: &'a str,
    access_token: &'a str,
}

impl async_imap::Authenticator for XOAuth2<'_> {
    type Response = String;

    fn process(&mut self, _challenge: &[u8]) -> String {
        format!(
            "user={}\x01auth=Bearer {}\x01\x01",
            self.user, self.access_token
        )
    }
}

fn tls_connector() -> TlsConnector {
    static CONFIG: OnceLock<Arc<ClientConfig>> = OnceLock::new();
    let config = CONFIG.get_or_init(|| {
        let mut roots = RootCertStore::empty();
        roots.extend(webpki_roots::TLS_SERVER_ROOTS.iter().cloned());
        let provider = Arc::new(tokio_rustls::rustls::crypto::ring::default_provider());
        let config = ClientConfig::builder_with_provider(provider)
            .with_safe_default_protocol_versions()
            .expect("ring supports the default protocol versions")
            .with_root_certificates(roots)
            .with_no_client_auth();
        Arc::new(config)
    });
    TlsConnector::from(config.clone())
}

fn capability_name(capability: &Capability) -> String {
    match capability {
        Capability::Imap4rev1 => "IMAP4REV1".to_string(),
        Capability::Auth(mechanism) => format!("AUTH={mechanism}"),
        Capability::Atom(atom) => atom.to_string(),
    }
}

pub async fn connect(config: &ImapConfig) -> Result<Connection> {
    let tcp = tokio::time::timeout(
        CONNECT_TIMEOUT,
        TcpStream::connect((config.host.as_str(), config.port)),
    )
    .await
    .map_err(|_| Error::Timeout)??;
    tcp.set_nodelay(true)?;
    let stream: Box<dyn ImapIo> = match config.security {
        Security::Tls => {
            let name = ServerName::try_from(config.host.clone())
                .map_err(|_| Error::InvalidServerName(config.host.clone()))?;
            let tls = tokio::time::timeout(CONNECT_TIMEOUT, tls_connector().connect(name, tcp))
                .await
                .map_err(|_| Error::Timeout)??;
            Box::new(tls)
        }
        Security::Plain => Box::new(tcp),
    };
    let mut client = async_imap::Client::new(stream);
    client.read_response().await?.ok_or(Error::NoGreeting)?;
    let mut session = match &config.credentials {
        Credentials::Password(password) => client
            .login(&config.username, password)
            .await
            .map_err(|(error, _)| error)?,
        Credentials::OAuth2 { access_token } => client
            .authenticate(
                "XOAUTH2",
                XOAuth2 {
                    user: &config.username,
                    access_token,
                },
            )
            .await
            .map_err(|(error, _)| error)?,
    };
    let capabilities =
        Capabilities::from_names(session.capabilities().await?.iter().map(capability_name));
    if config.compress && capabilities.has("COMPRESS=DEFLATE") {
        session = session
            .compress(|stream| Box::new(stream) as Box<dyn ImapIo>)
            .await?;
    }
    Ok(Connection {
        session,
        capabilities,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capability_lookup_is_case_insensitive() {
        let capabilities = Capabilities::from_names(["move".to_string(), "X-GM-EXT-1".to_string()]);
        assert!(capabilities.supports_move());
        assert!(capabilities.gmail());
        assert!(!capabilities.uidplus());
    }

    #[test]
    fn debug_hides_credentials() {
        let config = ImapConfig {
            host: "imap.example.com".into(),
            port: 993,
            security: Security::Tls,
            username: "me".into(),
            credentials: Credentials::Password("hunter2".into()),
            compress: true,
        };
        assert!(!format!("{config:?}").contains("hunter2"));
    }
}
