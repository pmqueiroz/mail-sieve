use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Newsletters,
    Notifications,
    Promotions,
    Receipts,
    Social,
    People,
}

impl Category {
    pub const ALL: [Category; 6] = [
        Category::Newsletters,
        Category::Notifications,
        Category::Promotions,
        Category::Receipts,
        Category::Social,
        Category::People,
    ];

    pub fn as_str(self) -> &'static str {
        match self {
            Category::Newsletters => "newsletters",
            Category::Notifications => "notifications",
            Category::Promotions => "promotions",
            Category::Receipts => "receipts",
            Category::Social => "social",
            Category::People => "people",
        }
    }
}

impl fmt::Display for Category {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for Category {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let value = value.to_ascii_lowercase();
        Category::ALL
            .into_iter()
            .find(|category| {
                category.as_str() == value || category.as_str().trim_end_matches('s') == value
            })
            .ok_or(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_names_and_singulars() {
        for category in Category::ALL {
            assert_eq!(category.as_str().parse(), Ok(category));
        }
        assert_eq!("Receipt".parse(), Ok(Category::Receipts));
        assert_eq!("newsletter".parse(), Ok(Category::Newsletters));
        assert!("spam".parse::<Category>().is_err());
    }
}
