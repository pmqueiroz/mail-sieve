use crate::Flags;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Address {
    pub name: Option<String>,
    pub email: String,
}

impl Address {
    pub fn domain(&self) -> &str {
        self.email.rsplit_once('@').map_or("", |(_, domain)| domain)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Unsubscribe {
    pub http: Option<String>,
    pub mailto: Option<String>,
    pub one_click: bool,
}

impl Unsubscribe {
    pub fn is_available(&self) -> bool {
        self.http.is_some() || self.mailto.is_some()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct MessageMeta {
    pub uid: u32,
    pub message_id: Option<String>,
    pub gm_msgid: Option<u64>,
    pub from: Option<Address>,
    pub subject: Option<String>,
    pub date: Option<i64>,
    pub internal_date: Option<i64>,
    pub size: u32,
    pub flags: Flags,
    pub list_id: Option<String>,
    pub unsubscribe: Unsubscribe,
    pub labels: Vec<String>,
}

pub fn normalize_label(label: &str) -> String {
    label.trim().trim_start_matches('\\').to_lowercase()
}

pub const LABEL_SEPARATOR: char = '\u{1f}';

pub fn join_labels(labels: &[String]) -> Option<String> {
    if labels.is_empty() {
        return None;
    }
    let mut normalized: Vec<String> = labels.iter().map(|label| normalize_label(label)).collect();
    normalized.sort();
    normalized.dedup();
    Some(normalized.join(&LABEL_SEPARATOR.to_string()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn joins_normalized_labels() {
        let labels = vec![
            "\\Inbox".to_string(),
            "Receipts".to_string(),
            "\\inbox".to_string(),
        ];
        assert_eq!(join_labels(&labels).as_deref(), Some("inbox\u{1f}receipts"));
        assert_eq!(join_labels(&[]), None);
    }
}
