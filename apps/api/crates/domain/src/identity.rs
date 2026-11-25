//! Identity: what a User signs in with. Hashing and Sessions live outside the domain
//! (they need randomness and crypto); here are only the rules (ADR-0011).

use crate::ids::UserId;
use crate::text::PersonName;
use thiserror::Error;
use time::OffsetDateTime;

/// A technical identity that can sign in, and joins Families as a Member.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: UserId,
    pub email: Email,
    /// The name new Memberships start with.
    pub name: PersonName,
    pub registered_at: OffsetDateTime,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum EmailError {
    #[error("not a valid email address")]
    Invalid,
}

/// An email address, trimmed and lower-cased so the same person always matches.
#[derive(Debug, Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct Email(String);

impl Email {
    pub const MAX_CHARS: usize = 254;

    /// A deliberately simple check: `local@domain.tld`, no spaces, one `@`.
    pub fn new(text: impl AsRef<str>) -> Result<Self, EmailError> {
        let email = text.as_ref().trim().to_lowercase();
        let valid = email.chars().count() <= Self::MAX_CHARS
            && !email.chars().any(char::is_whitespace)
            && match email.split_once('@') {
                Some((local, domain)) => {
                    !local.is_empty()
                        && !domain.contains('@')
                        && domain
                            .split_once('.')
                            .is_some_and(|(name, _)| !name.is_empty())
                        && !domain.ends_with('.')
                }
                None => false,
            };
        if valid {
            Ok(Self(email))
        } else {
            Err(EmailError::Invalid)
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum PasswordError {
    #[error("the password needs at least {min} characters")]
    TooShort { min: usize },
    #[error("the password may have at most {max} characters")]
    TooLong { max: usize },
}

/// A password as the User typed it (not trimmed). Never printed, never stored: only
/// its hash is kept, outside the domain.
#[derive(Clone, PartialEq, Eq)]
pub struct Password(String);

impl Password {
    pub const MIN_CHARS: usize = 10;
    pub const MAX_CHARS: usize = 128;

    pub fn new(text: impl Into<String>) -> Result<Self, PasswordError> {
        let text = text.into();
        let chars = text.chars().count();
        if chars < Self::MIN_CHARS {
            return Err(PasswordError::TooShort {
                min: Self::MIN_CHARS,
            });
        }
        if chars > Self::MAX_CHARS {
            return Err(PasswordError::TooLong {
                max: Self::MAX_CHARS,
            });
        }
        Ok(Self(text))
    }

    /// The typed text, for hashing or verifying only.
    pub fn expose(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Debug for Password {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Password(…)")
    }
}

#[cfg(test)]
#[path = "identity_tests.rs"]
mod tests;
