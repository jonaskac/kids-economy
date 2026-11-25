//! Money: Currency, Amount and Balance. Integer minor units, never a float.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum MoneyError {
    #[error("a currency is a three-letter ISO 4217 code")]
    InvalidCurrency,
    #[error("an amount must be more than zero")]
    NotPositive,
}

/// ISO 4217 code, stored upper case (e.g. `SEK`).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Currency([u8; 3]);

impl Currency {
    pub fn new(code: &str) -> Result<Self, MoneyError> {
        match code.as_bytes() {
            [a, b, c] if [a, b, c].iter().all(|x| x.is_ascii_alphabetic()) => Ok(Self([
                a.to_ascii_uppercase(),
                b.to_ascii_uppercase(),
                c.to_ascii_uppercase(),
            ])),
            _ => Err(MoneyError::InvalidCurrency),
        }
    }

    pub fn code(&self) -> &str {
        // Only ASCII letters are ever stored, so this is always valid UTF-8.
        std::str::from_utf8(&self.0).unwrap_or("???")
    }
}

/// A positive quantity of money in minor units (öre, cents) plus a Currency.
/// A zero Amount is never recorded; direction (in or out) lives on the Transaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Amount {
    minor: i64,
    currency: Currency,
}

impl Amount {
    pub fn new(minor: i64, currency: Currency) -> Result<Self, MoneyError> {
        if minor <= 0 {
            return Err(MoneyError::NotPositive);
        }
        Ok(Self { minor, currency })
    }

    pub const fn minor(&self) -> i64 {
        self.minor
    }

    pub const fn currency(&self) -> Currency {
        self.currency
    }
}

/// The sum of Transactions on an Account (or of Balances in a Wallet total).
/// Derived, never edited. Never below zero in v1 (no Overdraft).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Balance {
    minor: i64,
    currency: Currency,
}

impl Balance {
    pub const fn zero(currency: Currency) -> Self {
        Self { minor: 0, currency }
    }

    /// Only the domain derives Balances, from a Ledger.
    pub(crate) const fn from_minor(minor: i64, currency: Currency) -> Self {
        Self { minor, currency }
    }

    pub const fn minor(&self) -> i64 {
        self.minor
    }

    pub const fn currency(&self) -> Currency {
        self.currency
    }

    /// `None` if the result would not fit.
    pub fn plus(self, amount: Amount) -> Option<Self> {
        Some(Self {
            minor: self.minor.checked_add(amount.minor)?,
            currency: self.currency,
        })
    }

    /// `None` if the result would not fit.
    pub fn minus(self, amount: Amount) -> Option<Self> {
        Some(Self {
            minor: self.minor.checked_sub(amount.minor)?,
            currency: self.currency,
        })
    }

    /// Whether taking `amount` out leaves the Balance at zero or above.
    pub fn covers(&self, amount: Amount) -> bool {
        self.minor >= amount.minor
    }

    /// Whether the Balance has reached `target`.
    pub fn reaches(&self, target: Amount) -> bool {
        self.covers(target)
    }

    pub const fn is_zero(&self) -> bool {
        self.minor == 0
    }
}

#[cfg(test)]
#[path = "money_tests.rs"]
mod tests;
