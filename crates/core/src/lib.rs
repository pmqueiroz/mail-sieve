mod flags;
mod message;
pub mod query;
mod uid_set;

pub use flags::Flags;
pub use message::{Address, MessageMeta, Unsubscribe};
pub use query::{Query, QueryError, Term, TermKind};
pub use uid_set::UidSet;
