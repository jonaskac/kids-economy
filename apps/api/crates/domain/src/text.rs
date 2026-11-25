//! Short, Kid-readable texts: names of Families, Kids, people and Accounts, and Memos.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum TextError {
    #[error("the text is empty")]
    Empty,
    #[error("the text is longer than {max} characters")]
    TooLong { max: usize },
}

fn checked(text: &str, max: usize) -> Result<String, TextError> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Err(TextError::Empty);
    }
    if trimmed.chars().count() > max {
        return Err(TextError::TooLong { max });
    }
    Ok(trimmed.to_owned())
}

macro_rules! short_text {
    ($(#[$doc:meta])* $name:ident, $max:expr) => {
        $(#[$doc])*
        #[derive(Debug, Clone, PartialEq, Eq, Hash)]
        pub struct $name(String);

        impl $name {
            pub const MAX_CHARS: usize = $max;

            /// Trims the text; it must not be blank or longer than `MAX_CHARS`.
            pub fn new(text: impl AsRef<str>) -> Result<Self, TextError> {
                checked(text.as_ref(), Self::MAX_CHARS).map(Self)
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

short_text!(
    /// Name of an Account or a Savings goal ("Fun stuff", "Lego set").
    AccountName,
    40
);
short_text!(
    /// A short, Kid-readable text on a Transaction ("Pocket money week 48").
    Memo,
    80
);
short_text!(
    /// Name of a Family ("The Kacs").
    FamilyName,
    60
);
short_text!(
    /// A Kid's name as the Family uses it ("Elsa").
    KidName,
    40
);
short_text!(
    /// An adult's name as other Members see it ("Mormor", "Jonas").
    PersonName,
    60
);

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
