use async_imap::types::{Fetch, Flag};
use mail_parser::{HeaderName, MessageParser};
use mail_sieve_core::headers::{parse_list_id, parse_list_unsubscribe};
use mail_sieve_core::{Address, Flags, MessageMeta};

const MAX_CLOCK_SKEW: i64 = 86_400;

pub(crate) fn flags_of(fetch: &Fetch) -> Flags {
    fetch.flags().fold(Flags::default(), |acc, flag| {
        acc | match flag {
            Flag::Seen => Flags::SEEN,
            Flag::Answered => Flags::ANSWERED,
            Flag::Flagged => Flags::FLAGGED,
            Flag::Deleted => Flags::DELETED,
            Flag::Draft => Flags::DRAFT,
            _ => Flags::default(),
        }
    })
}

pub(crate) fn labels_of(fetch: &Fetch) -> Option<Vec<String>> {
    fetch
        .gmail_labels()
        .map(|labels| labels.iter().map(|label| label.to_string()).collect())
}

pub(crate) fn message_meta(fetch: &Fetch) -> Option<MessageMeta> {
    let mut meta = MessageMeta {
        uid: fetch.uid?,
        size: fetch.size.unwrap_or(0),
        flags: flags_of(fetch),
        internal_date: fetch.internal_date().map(|date| date.timestamp()),
        gm_msgid: fetch.gmail_msg_id().copied(),
        labels: labels_of(fetch).unwrap_or_default(),
        ..Default::default()
    };
    if let Some(header) = fetch.header() {
        apply_headers(header, &mut meta);
    }
    Some(meta)
}

pub fn apply_headers(raw: &[u8], meta: &mut MessageMeta) {
    let Some(message) = MessageParser::default().parse_headers(raw) else {
        return;
    };
    meta.from = message
        .from()
        .and_then(|from| from.first())
        .and_then(|addr| {
            let email = addr.address()?.trim();
            (!email.is_empty()).then(|| Address {
                name: addr
                    .name()
                    .map(str::trim)
                    .filter(|name| !name.is_empty())
                    .map(str::to_string),
                email: email.to_string(),
            })
        });
    meta.subject = message.subject().map(|subject| subject.trim().to_string());
    meta.message_id = message.message_id().map(str::to_string);
    meta.date = message
        .date()
        .map(|date| date.to_timestamp())
        .filter(|&date| match meta.internal_date {
            Some(internal) => date > 0 && date <= internal + MAX_CLOCK_SKEW,
            None => date > 0,
        });
    meta.list_id = message
        .header_raw(HeaderName::ListId)
        .and_then(parse_list_id);
    if let Some(value) = message.header_raw(HeaderName::ListUnsubscribe) {
        meta.unsubscribe =
            parse_list_unsubscribe(value, message.header_raw(HeaderName::ListUnsubscribePost));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const HEADERS: &[u8] = b"From: =?UTF-8?Q?Caf=C3=A9_News?= <News@Cafe.example>\r\n\
Subject: =?UTF-8?B?V2Vla2x5IGRpZ2VzdA==?=\r\n\
Date: Tue, 01 Oct 2024 10:00:00 +0000\r\n\
Message-ID: <abc@cafe.example>\r\n\
List-Id: Cafe News <news.cafe.example>\r\n\
List-Unsubscribe: <mailto:u@cafe.example>,\r\n <https://cafe.example/u/1>\r\n\
List-Unsubscribe-Post: List-Unsubscribe=One-Click\r\n\r\n";

    #[test]
    fn parses_header_fields() {
        let mut meta = MessageMeta {
            internal_date: Some(1_727_776_800),
            ..Default::default()
        };
        apply_headers(HEADERS, &mut meta);
        let from = meta.from.unwrap();
        assert_eq!(from.email, "News@Cafe.example");
        assert_eq!(from.name.as_deref(), Some("Café News"));
        assert_eq!(meta.subject.as_deref(), Some("Weekly digest"));
        assert_eq!(meta.date, Some(1_727_776_800));
        assert_eq!(meta.message_id.as_deref(), Some("abc@cafe.example"));
        assert_eq!(meta.list_id.as_deref(), Some("news.cafe.example"));
        assert_eq!(
            meta.unsubscribe.http.as_deref(),
            Some("https://cafe.example/u/1")
        );
        assert_eq!(
            meta.unsubscribe.mailto.as_deref(),
            Some("mailto:u@cafe.example")
        );
        assert!(meta.unsubscribe.one_click);
    }

    #[test]
    fn rejects_dates_from_the_future() {
        let mut meta = MessageMeta {
            internal_date: Some(1_000_000_000),
            ..Default::default()
        };
        apply_headers(HEADERS, &mut meta);
        assert_eq!(meta.date, None);
    }
}
