#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash)]
pub struct Flags(u8);

impl Flags {
    pub const SEEN: Flags = Flags(1);
    pub const ANSWERED: Flags = Flags(1 << 1);
    pub const FLAGGED: Flags = Flags(1 << 2);
    pub const DELETED: Flags = Flags(1 << 3);
    pub const DRAFT: Flags = Flags(1 << 4);

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn from_bits(bits: u8) -> Self {
        Flags(bits & 0b1_1111)
    }

    pub const fn contains(self, other: Flags) -> bool {
        self.0 & other.0 == other.0
    }

    pub fn insert(&mut self, other: Flags) {
        self.0 |= other.0;
    }

    pub fn from_imap_flag(name: &str) -> Option<Flags> {
        match name.to_ascii_lowercase().as_str() {
            "\\seen" => Some(Self::SEEN),
            "\\answered" => Some(Self::ANSWERED),
            "\\flagged" => Some(Self::FLAGGED),
            "\\deleted" => Some(Self::DELETED),
            "\\draft" => Some(Self::DRAFT),
            _ => None,
        }
    }

    pub fn from_imap_flags<'a>(names: impl IntoIterator<Item = &'a str>) -> Flags {
        names.into_iter().filter_map(Self::from_imap_flag).fold(
            Flags::default(),
            |mut acc, flag| {
                acc.insert(flag);
                acc
            },
        )
    }
}

impl std::ops::BitOr for Flags {
    type Output = Flags;

    fn bitor(self, rhs: Flags) -> Flags {
        Flags(self.0 | rhs.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_system_flags_case_insensitively() {
        let flags = Flags::from_imap_flags(["\\Seen", "\\FLAGGED", "$Junk"]);
        assert!(flags.contains(Flags::SEEN | Flags::FLAGGED));
        assert!(!flags.contains(Flags::DELETED));
        assert_eq!(Flags::from_bits(flags.bits()), flags);
    }
}
