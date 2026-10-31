use std::fmt;
use std::str::FromStr;

use async_imap::types::NameAttribute;
use futures::TryStreamExt;

use crate::{Capabilities, Connection, Result};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Provider {
    Generic,
    Gmail,
}

impl Provider {
    pub fn detect(capabilities: &Capabilities) -> Provider {
        if capabilities.gmail() {
            Provider::Gmail
        } else {
            Provider::Generic
        }
    }

    pub fn default_connections(self) -> u32 {
        match self {
            Provider::Gmail => 8,
            Provider::Generic => 4,
        }
    }

    pub fn default_sync(self, folder: &Folder) -> bool {
        match (self, folder.special_use.as_deref()) {
            (Provider::Gmail, Some("\\All")) => true,
            (Provider::Gmail, _) => false,
            (
                Provider::Generic,
                Some("\\All" | "\\Trash" | "\\Junk" | "\\Drafts" | "\\Sent" | "\\Flagged"),
            ) => false,
            (Provider::Generic, _) => true,
        }
    }
}

impl fmt::Display for Provider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Provider::Generic => "generic",
            Provider::Gmail => "gmail",
        })
    }
}

impl FromStr for Provider {
    type Err = std::convert::Infallible;

    fn from_str(value: &str) -> std::result::Result<Self, Self::Err> {
        Ok(if value == "gmail" {
            Provider::Gmail
        } else {
            Provider::Generic
        })
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Folder {
    pub name: String,
    pub delimiter: Option<String>,
    pub special_use: Option<String>,
}

pub async fn list_folders(conn: &mut Connection) -> Result<Vec<Folder>> {
    let names: Vec<_> = conn
        .session
        .list(Some(""), Some("*"))
        .await?
        .try_collect()
        .await?;
    let mut folders = Vec::with_capacity(names.len());
    for name in &names {
        let mut special_use = None;
        let mut selectable = true;
        for attribute in name.attributes() {
            match attribute {
                NameAttribute::NoSelect => selectable = false,
                NameAttribute::All => special_use = Some("\\All".to_string()),
                NameAttribute::Archive => special_use = Some("\\Archive".to_string()),
                NameAttribute::Drafts => special_use = Some("\\Drafts".to_string()),
                NameAttribute::Flagged => special_use = Some("\\Flagged".to_string()),
                NameAttribute::Junk => special_use = Some("\\Junk".to_string()),
                NameAttribute::Sent => special_use = Some("\\Sent".to_string()),
                NameAttribute::Trash => special_use = Some("\\Trash".to_string()),
                NameAttribute::Extension(extension)
                    if extension.eq_ignore_ascii_case("\\NonExistent") =>
                {
                    selectable = false
                }
                NameAttribute::Extension(extension)
                    if extension.eq_ignore_ascii_case("\\Important") =>
                {
                    special_use = Some("\\Important".to_string())
                }
                _ => {}
            }
        }
        if !selectable {
            continue;
        }
        let special_use = special_use
            .or_else(|| special_use_by_name(name.name(), name.delimiter()).map(str::to_string));
        folders.push(Folder {
            name: name.name().to_string(),
            delimiter: name.delimiter().map(str::to_string),
            special_use,
        });
    }
    Ok(folders)
}

fn special_use_by_name(name: &str, delimiter: Option<&str>) -> Option<&'static str> {
    let leaf = match delimiter {
        Some(delimiter) if !delimiter.is_empty() => name.rsplit(delimiter).next().unwrap_or(name),
        _ => name,
    };
    match leaf.to_lowercase().as_str() {
        "trash" | "deleted items" | "deleted messages" | "deleted" | "bin" => Some("\\Trash"),
        "junk" | "spam" | "junk e-mail" | "junk email" | "bulk mail" => Some("\\Junk"),
        "archive" | "archives" => Some("\\Archive"),
        "sent" | "sent items" | "sent messages" | "sent mail" => Some("\\Sent"),
        "drafts" | "draft" => Some("\\Drafts"),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn folder(name: &str, special_use: Option<&str>) -> Folder {
        Folder {
            name: name.into(),
            delimiter: Some("/".into()),
            special_use: special_use.map(Into::into),
        }
    }

    #[test]
    fn guesses_special_use_from_leaf_name() {
        assert_eq!(
            special_use_by_name("INBOX.Trash", Some(".")),
            Some("\\Trash")
        );
        assert_eq!(
            special_use_by_name("Deleted Items", Some("/")),
            Some("\\Trash")
        );
        assert_eq!(special_use_by_name("Receipts", Some("/")), None);
    }

    #[test]
    fn default_sync_avoids_duplicates() {
        assert!(Provider::Gmail.default_sync(&folder("[Gmail]/All Mail", Some("\\All"))));
        assert!(!Provider::Gmail.default_sync(&folder("INBOX", None)));
        assert!(Provider::Generic.default_sync(&folder("INBOX", None)));
        assert!(Provider::Generic.default_sync(&folder("Receipts", None)));
        assert!(!Provider::Generic.default_sync(&folder("Trash", Some("\\Trash"))));
        assert_eq!("gmail".parse::<Provider>().unwrap(), Provider::Gmail);
    }
}
