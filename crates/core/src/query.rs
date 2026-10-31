use std::fmt;

use crate::Category;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Query {
    pub terms: Vec<Term>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Term {
    pub negated: bool,
    pub kind: TermKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TermKind {
    From(String),
    Domain(String),
    Subject(String),
    List(String),
    Mailbox(String),
    Unread,
    Flagged,
    Answered,
    Pinned,
    HasUnsubscribe,
    HasList,
    OlderThan(i64),
    NewerThan(i64),
    Before(i64),
    After(i64),
    Larger(u64),
    Smaller(u64),
    Category(Category),
    Text(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum QueryError {
    UnknownField(String),
    InvalidValue { field: String, value: String },
    EmptyValue(String),
    UnterminatedQuote,
}

impl fmt::Display for QueryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            QueryError::UnknownField(field) => write!(f, "unknown field `{field}`"),
            QueryError::InvalidValue { field, value } => {
                write!(f, "invalid value `{value}` for `{field}`")
            }
            QueryError::EmptyValue(field) => write!(f, "`{field}` needs a value"),
            QueryError::UnterminatedQuote => f.write_str("unterminated quote"),
        }
    }
}

impl std::error::Error for QueryError {}

impl Query {
    pub fn parse(input: &str) -> Result<Query, QueryError> {
        let terms = tokenize(input)?
            .into_iter()
            .map(parse_token)
            .collect::<Result<Vec<_>, _>>()?;
        Ok(Query { terms })
    }

    pub fn is_empty(&self) -> bool {
        self.terms.is_empty()
    }
}

struct Token {
    negated: bool,
    field: Option<String>,
    value: String,
}

fn tokenize(input: &str) -> Result<Vec<Token>, QueryError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();
    loop {
        while chars.next_if(|c| c.is_whitespace()).is_some() {}
        let Some(&first) = chars.peek() else { break };
        let negated = first == '-';
        if negated {
            chars.next();
        }
        let mut field = None;
        let mut value = String::new();
        loop {
            match chars.peek().copied() {
                None => break,
                Some(c) if c.is_whitespace() => break,
                Some('"') => {
                    chars.next();
                    let mut closed = false;
                    for c in chars.by_ref() {
                        if c == '"' {
                            closed = true;
                            break;
                        }
                        value.push(c);
                    }
                    if !closed {
                        return Err(QueryError::UnterminatedQuote);
                    }
                }
                Some(':') if field.is_none() && !value.is_empty() => {
                    chars.next();
                    field = Some(std::mem::take(&mut value).to_ascii_lowercase());
                }
                Some(c) => {
                    chars.next();
                    value.push(c);
                }
            }
        }
        if field.is_none() && value.is_empty() {
            continue;
        }
        tokens.push(Token {
            negated,
            field,
            value,
        });
    }
    Ok(tokens)
}

fn parse_token(token: Token) -> Result<Term, QueryError> {
    let Token {
        negated,
        field,
        value,
    } = token;
    let Some(field) = field else {
        return Ok(Term {
            negated,
            kind: TermKind::Text(value),
        });
    };
    if value.is_empty() {
        return Err(QueryError::EmptyValue(field));
    }
    let invalid = || QueryError::InvalidValue {
        field: field.clone(),
        value: value.clone(),
    };
    let kind = match field.as_str() {
        "from" => TermKind::From(value.to_lowercase()),
        "domain" => TermKind::Domain(value.to_lowercase()),
        "subject" => TermKind::Subject(value),
        "list" => TermKind::List(value.to_lowercase()),
        "in" | "mailbox" => TermKind::Mailbox(value),
        "is" => match value.to_ascii_lowercase().as_str() {
            "unread" => TermKind::Unread,
            "read" => {
                return Ok(Term {
                    negated: !negated,
                    kind: TermKind::Unread,
                });
            }
            "flagged" | "starred" => TermKind::Flagged,
            "answered" | "replied" => TermKind::Answered,
            "pinned" | "protected" => TermKind::Pinned,
            _ => return Err(invalid()),
        },
        "has" => match value.to_ascii_lowercase().as_str() {
            "unsubscribe" | "unsub" => TermKind::HasUnsubscribe,
            "list" => TermKind::HasList,
            _ => return Err(invalid()),
        },
        "older_than" | "older" => TermKind::OlderThan(parse_duration(&value).ok_or_else(invalid)?),
        "newer_than" | "newer" => TermKind::NewerThan(parse_duration(&value).ok_or_else(invalid)?),
        "before" => TermKind::Before(parse_date(&value).ok_or_else(invalid)?),
        "after" => TermKind::After(parse_date(&value).ok_or_else(invalid)?),
        "larger" | "size" => TermKind::Larger(parse_size(&value).ok_or_else(invalid)?),
        "smaller" => TermKind::Smaller(parse_size(&value).ok_or_else(invalid)?),
        "category" | "cat" => TermKind::Category(value.parse().map_err(|()| invalid())?),
        _ => return Err(QueryError::UnknownField(field)),
    };
    Ok(Term { negated, kind })
}

const DAY: i64 = 86_400;

pub fn parse_duration(value: &str) -> Option<i64> {
    let value = value.trim().to_ascii_lowercase();
    let split = value.find(|c: char| !c.is_ascii_digit())?;
    let (amount, unit) = value.split_at(split);
    let amount: i64 = amount.parse().ok()?;
    let unit_seconds = match unit {
        "h" => 3_600,
        "d" => DAY,
        "w" => 7 * DAY,
        "m" => 30 * DAY,
        "y" => 365 * DAY,
        _ => return None,
    };
    amount.checked_mul(unit_seconds)
}

pub fn parse_size(value: &str) -> Option<u64> {
    let value = value.trim().to_ascii_lowercase();
    let split = value
        .find(|c: char| !c.is_ascii_digit() && c != '.')
        .unwrap_or(value.len());
    let (amount, unit) = value.split_at(split);
    let amount: f64 = amount.parse().ok()?;
    let multiplier = match unit {
        "" | "b" => 1.0,
        "k" | "kb" => 1024.0,
        "m" | "mb" => 1024.0 * 1024.0,
        "g" | "gb" => 1024.0 * 1024.0 * 1024.0,
        _ => return None,
    };
    (amount >= 0.0).then_some((amount * multiplier) as u64)
}

pub fn parse_date(value: &str) -> Option<i64> {
    let mut parts = value.trim().splitn(3, ['-', '/']);
    let year: i64 = parts.next()?.parse().ok()?;
    let month: u32 = parts.next()?.parse().ok()?;
    let day: u32 = parts.next()?.parse().ok()?;
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    Some(days_from_civil(year, month, day) * DAY)
}

fn days_from_civil(year: i64, month: u32, day: u32) -> i64 {
    let year = if month <= 2 { year - 1 } else { year };
    let era = year.div_euclid(400);
    let year_of_era = year - era * 400;
    let month = i64::from(month);
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    era * 146_097 + day_of_era - 719_468
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<(bool, TermKind)> {
        Query::parse(input)
            .unwrap()
            .terms
            .into_iter()
            .map(|t| (t.negated, t.kind))
            .collect()
    }

    #[test]
    fn parses_fields_negation_and_quotes() {
        assert_eq!(
            kinds(r#"from:*@LinkedIn.com -is:read subject:"weekly digest" hello"#),
            vec![
                (false, TermKind::From("*@linkedin.com".into())),
                (false, TermKind::Unread),
                (false, TermKind::Subject("weekly digest".into())),
                (false, TermKind::Text("hello".into())),
            ]
        );
    }

    #[test]
    fn parses_units() {
        assert_eq!(
            kinds("older_than:1y"),
            vec![(false, TermKind::OlderThan(365 * DAY))]
        );
        assert_eq!(
            kinds("larger:5mb"),
            vec![(false, TermKind::Larger(5 * 1024 * 1024))]
        );
        assert_eq!(
            kinds("category:Receipts -cat:social"),
            vec![
                (false, TermKind::Category(Category::Receipts)),
                (true, TermKind::Category(Category::Social)),
            ]
        );
        assert_eq!(
            kinds("-has:unsubscribe"),
            vec![(true, TermKind::HasUnsubscribe)]
        );
        assert_eq!(kinds("-is:pinned"), vec![(true, TermKind::Pinned)]);
    }

    #[test]
    fn parses_dates_as_utc_midnight() {
        assert_eq!(parse_date("1970-01-01"), Some(0));
        assert_eq!(parse_date("2024-03-01"), Some(1_709_251_200));
        assert_eq!(parse_date("2024-13-01"), None);
    }

    #[test]
    fn reports_errors() {
        assert_eq!(
            Query::parse("foo:bar"),
            Err(QueryError::UnknownField("foo".into()))
        );
        assert_eq!(
            Query::parse("from:"),
            Err(QueryError::EmptyValue("from".into()))
        );
        assert_eq!(
            Query::parse("subject:\"open"),
            Err(QueryError::UnterminatedQuote)
        );
        assert!(matches!(
            Query::parse("older_than:abc"),
            Err(QueryError::InvalidValue { .. })
        ));
    }

    #[test]
    fn empty_query_matches_all() {
        assert!(Query::parse("   ").unwrap().is_empty());
    }
}
