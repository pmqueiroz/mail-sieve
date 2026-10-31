#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Sqlite(#[from] rusqlite::Error),
    #[error(transparent)]
    Query(#[from] mail_sieve_core::QueryError),
    #[error("not found")]
    NotFound,
}

pub type Result<T> = std::result::Result<T, Error>;
