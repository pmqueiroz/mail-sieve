mod actions;
mod connection;
mod error;
mod fetch;
mod folders;
mod mailbox_name;
mod parallel;
mod parse;

pub use actions::{Operation, apply};
pub use async_imap::extensions::idle::IdleResponse;
pub use connection::{
    Capabilities, Connection, Credentials, ImapConfig, MailboxStatus, Security, Session, connect,
};
pub use error::{Error, Result};
pub use fetch::{FlagState, fetch_flags, fetch_headers, fetch_raw, search_all_uids};
pub use folders::{Folder, Provider, list_folders};
pub use mailbox_name::decode_mailbox_name;
pub use parallel::{HeaderFetchPlan, fetch_headers_parallel};
pub use parse::apply_headers;
