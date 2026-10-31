use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Action {
    Archive,
    Trash,
    Delete,
    MarkRead,
    MarkUnread,
    Flag,
    Unflag,
    Move(String),
}

impl Action {
    pub fn is_destructive(&self) -> bool {
        matches!(self, Action::Delete)
    }

    pub fn removes_from_source(&self) -> bool {
        matches!(self, Action::Archive | Action::Trash | Action::Delete | Action::Move(_))
    }
}

impl fmt::Display for Action {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Action::Archive => f.write_str("archive"),
            Action::Trash => f.write_str("trash"),
            Action::Delete => f.write_str("delete"),
            Action::MarkRead => f.write_str("mark_read"),
            Action::MarkUnread => f.write_str("mark_unread"),
            Action::Flag => f.write_str("flag"),
            Action::Unflag => f.write_str("unflag"),
            Action::Move(mailbox) => write!(f, "move:{mailbox}"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownAction(pub String);

impl fmt::Display for UnknownAction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "unknown action `{}`", self.0)
    }
}

impl std::error::Error for UnknownAction {}

impl FromStr for Action {
    type Err = UnknownAction;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(match value {
            "archive" => Action::Archive,
            "trash" => Action::Trash,
            "delete" => Action::Delete,
            "mark_read" => Action::MarkRead,
            "mark_unread" => Action::MarkUnread,
            "flag" => Action::Flag,
            "unflag" => Action::Unflag,
            other => match other.strip_prefix("move:") {
                Some(mailbox) if !mailbox.is_empty() => Action::Move(mailbox.to_string()),
                _ => return Err(UnknownAction(other.to_string())),
            },
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trips_through_strings() {
        for action in [
            Action::Archive,
            Action::Trash,
            Action::Delete,
            Action::MarkRead,
            Action::MarkUnread,
            Action::Flag,
            Action::Unflag,
            Action::Move("Receipts/2024".into()),
        ] {
            assert_eq!(action.to_string().parse::<Action>(), Ok(action));
        }
        assert!("move:".parse::<Action>().is_err());
    }
}
