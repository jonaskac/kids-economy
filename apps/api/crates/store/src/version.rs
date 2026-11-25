//! Version: how many Events a Wallet's History holds (glossary).

/// The Sequence of a Wallet's last Event; 0 before the Wallet is opened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Version(i64);

impl Version {
    /// No History yet.
    pub const NONE: Version = Version(0);

    pub fn from_count(count: usize) -> Self {
        // A History of more than i64::MAX Events cannot exist in Postgres anyway.
        Self(i64::try_from(count).unwrap_or(i64::MAX))
    }

    pub(crate) const fn from_sequence(sequence: i64) -> Self {
        Self(sequence)
    }

    /// The Sequence of the last Event (0 for none).
    pub const fn sequence(self) -> i64 {
        self.0
    }

    /// The Sequence the `n`th new Event (counted from 0) gets.
    pub(crate) fn next(self, n: usize) -> i64 {
        self.0
            .saturating_add(i64::try_from(n).unwrap_or(i64::MAX))
            .saturating_add(1)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_events_continue_right_after_the_last_one() {
        let version = Version::from_count(3);
        assert_eq!(version.sequence(), 3);
        assert_eq!((version.next(0), version.next(1)), (4, 5));
        assert_eq!(Version::NONE.next(0), 1);
    }
}
