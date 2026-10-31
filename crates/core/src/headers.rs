use crate::Unsubscribe;

pub fn parse_list_unsubscribe(value: &str, post: Option<&str>) -> Unsubscribe {
    let mut unsubscribe = Unsubscribe::default();
    for uri in bracketed(value) {
        let lower = uri.to_ascii_lowercase();
        if lower.starts_with("mailto:") {
            unsubscribe.mailto.get_or_insert_with(|| uri.to_string());
        } else if lower.starts_with("https://") || lower.starts_with("http://") {
            unsubscribe.http.get_or_insert_with(|| uri.to_string());
        }
    }
    let one_click_requested = post.is_some_and(|post| {
        post.split_whitespace()
            .collect::<String>()
            .eq_ignore_ascii_case("list-unsubscribe=one-click")
    });
    unsubscribe.one_click = one_click_requested
        && unsubscribe
            .http
            .as_deref()
            .is_some_and(|http| http.to_ascii_lowercase().starts_with("https://"));
    unsubscribe
}

pub fn parse_list_id(value: &str) -> Option<String> {
    let id = bracketed(value).next().unwrap_or_else(|| value.trim());
    let id = id.trim().to_ascii_lowercase();
    (!id.is_empty()).then_some(id)
}

fn bracketed(value: &str) -> impl Iterator<Item = &str> {
    value
        .split('<')
        .skip(1)
        .filter_map(|part| part.split_once('>'))
        .map(|(inner, _)| inner.trim_matches(|c: char| c.is_whitespace()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_mailto_and_https_with_one_click() {
        let parsed = parse_list_unsubscribe(
            " <mailto:unsub@news.example.com?subject=stop>,\r\n <https://news.example.com/u/abc>",
            Some("List-Unsubscribe=One-Click"),
        );
        assert_eq!(
            parsed.mailto.as_deref(),
            Some("mailto:unsub@news.example.com?subject=stop")
        );
        assert_eq!(
            parsed.http.as_deref(),
            Some("https://news.example.com/u/abc")
        );
        assert!(parsed.one_click);
    }

    #[test]
    fn one_click_requires_https_and_post_header() {
        assert!(
            !parse_list_unsubscribe("<http://x.com/u>", Some("List-Unsubscribe=One-Click"))
                .one_click
        );
        assert!(!parse_list_unsubscribe("<https://x.com/u>", None).one_click);
        assert!(!parse_list_unsubscribe("garbage", None).is_available());
    }

    #[test]
    fn parses_list_id() {
        assert_eq!(
            parse_list_id("Weekly News <weekly.News.example.com>").as_deref(),
            Some("weekly.news.example.com")
        );
        assert_eq!(parse_list_id(" plain.id ").as_deref(), Some("plain.id"));
        assert_eq!(parse_list_id("  "), None);
    }
}
