mod action;
mod flags;
pub mod headers;
mod message;
pub mod query;
mod uid_set;

pub use action::{Action, UnknownAction};
pub use flags::Flags;
pub use message::{
    Address, LABEL_SEPARATOR, MessageMeta, Unsubscribe, join_labels, normalize_label,
};
pub use query::{Query, QueryError, Term, TermKind};
pub use uid_set::UidSet;
