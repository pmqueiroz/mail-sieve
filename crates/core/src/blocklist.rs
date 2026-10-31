use std::collections::BTreeSet;

pub const MAX_DOMAINS: usize = 200_000;

const SINK_ADDRESSES: [&str; 4] = ["0.0.0.0", "127.0.0.1", "::", "::1"];
const LOCAL_NAMES: [&str; 4] = [
    "localhost",
    "localhost.localdomain",
    "local",
    "broadcasthost",
];

fn clean_entry(line: &str) -> Option<String> {
    let line = line.split('#').next()?.trim();
    if line.is_empty() || line.starts_with('!') || line.starts_with('[') {
        return None;
    }
    let mut tokens = line.split_whitespace();
    let first = tokens.next()?;
    let entry = if SINK_ADDRESSES.contains(&first) {
        tokens.next()?
    } else {
        first
    };
    let entry = entry
        .trim_start_matches("||")
        .trim_end_matches('^')
        .trim_start_matches("*.")
        .trim_start_matches('.');
    let domain = entry
        .rsplit_once('@')
        .map_or(entry, |(_, domain)| domain)
        .trim_end_matches('.')
        .to_ascii_lowercase();
    valid_domain(&domain).then_some(domain)
}

fn valid_domain(domain: &str) -> bool {
    domain.contains('.')
        && !LOCAL_NAMES.contains(&domain)
        && domain.parse::<std::net::IpAddr>().is_err()
        && domain.split('.').all(|label| {
            !label.is_empty()
                && label.len() <= 63
                && !label.starts_with('-')
                && !label.ends_with('-')
                && label
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '-' || c == '_')
        })
}

pub fn parse_domains(text: &str) -> BTreeSet<String> {
    text.lines()
        .filter_map(clean_entry)
        .take(MAX_DOMAINS)
        .collect()
}

pub fn listed_entry<'a>(listed: &'a BTreeSet<String>, domain: &str) -> Option<&'a str> {
    let mut candidate = domain;
    loop {
        if let Some(entry) = listed.get(candidate) {
            return Some(entry);
        }
        candidate = candidate.split_once('.')?.1;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_plain_hosts_and_adblock_lines() {
        let text = "# a comment\n\
                    ads.example.com\n\
                    0.0.0.0 tracker.example.net # inline comment\n\
                    127.0.0.1 localhost\n\
                    ||spam.example.org^\n\
                    *.wild.example.io\n\
                    @promo.example.co\n\
                    news@mailer.example.biz\n\
                    ! adblock comment\n\
                    [Adblock Plus 2.0]\n\
                    10.0.0.1\n\
                    nodot\n\
                    Bad_-.example\n";
        let domains: Vec<_> = parse_domains(text).into_iter().collect();
        assert_eq!(
            domains,
            [
                "ads.example.com",
                "mailer.example.biz",
                "promo.example.co",
                "spam.example.org",
                "tracker.example.net",
                "wild.example.io",
            ]
        );
    }

    #[test]
    fn matches_listed_domains_and_their_subdomains() {
        let listed = parse_domains("example.com\nads.other.net\n");
        assert_eq!(listed_entry(&listed, "example.com"), Some("example.com"));
        assert_eq!(
            listed_entry(&listed, "mail.news.example.com"),
            Some("example.com")
        );
        assert_eq!(listed_entry(&listed, "other.net"), None);
        assert_eq!(listed_entry(&listed, "notexample.com"), None);
    }
}
