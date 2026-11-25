//! Short, Kid-readable texts: Account names, Savings goal names and Memos.

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

/// Name of an Account or a Savings goal ("Fun stuff", "Lego set").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AccountName(String);

impl AccountName {
    pub const MAX_CHARS: usize = 40;

    pub fn new(text: impl AsRef<str>) -> Result<Self, TextError> {
        checked(text.as_ref(), Self::MAX_CHARS).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// A short, Kid-readable text on a Transaction ("Pocket money week 48").
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Memo(String);

impl Memo {
    pub const MAX_CHARS: usize = 80;

    pub fn new(text: impl AsRef<str>) -> Result<Self, TextError> {
        checked(text.as_ref(), Self::MAX_CHARS).map(Self)
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
#[path = "text_tests.rs"]
mod tests;
