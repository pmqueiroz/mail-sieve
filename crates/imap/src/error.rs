#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("imap: {0}")]
    Imap(#[from] async_imap::error::Error),
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("tls: invalid server name `{0}`")]
    InvalidServerName(String),
    #[error("connection timed out")]
    Timeout,
    #[error("server closed the connection before greeting")]
    NoGreeting,
    #[error("server reported no UIDVALIDITY for `{0}`")]
    NoUidValidity(String),
    #[error("{0}")]
    Unsupported(String),
}

pub type Result<T> = std::result::Result<T, Error>;
