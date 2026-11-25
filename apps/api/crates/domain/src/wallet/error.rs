//! Why a Wallet rejects a command, or refuses a history.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum WalletError {
    #[error("this Member's Role may not change the Wallet")]
    NotAllowed,
    #[error("the Amount is not in the Wallet's Currency")]
    CurrencyMismatch,
    #[error("no such Account in this Wallet")]
    UnknownAccount,
    #[error("the Account is closed")]
    AccountClosed,
    #[error("a Transfer needs two different Accounts")]
    SameAccount,
    #[error("the Account's Balance is too low (no Overdraft)")]
    InsufficientFunds,
    #[error("the Wallet total would be too large")]
    TotalOverflow,
    #[error("an Account closes only at a zero Balance")]
    NonZeroBalance,
    #[error("the Main account cannot be closed")]
    MainAccount,
    #[error("no such Transaction in this Wallet")]
    UnknownTransaction,
    #[error("the Transaction is already corrected")]
    AlreadyCorrected,
}

/// A stored history that does not describe a valid Wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum ReplayError {
    #[error("the history does not start with WalletOpened")]
    NotOpened,
    #[error("WalletOpened appears twice")]
    AlreadyOpened,
    #[error("an event names an Account the Wallet does not have")]
    UnknownAccount,
    #[error("an event opens an Account that already exists")]
    DuplicateAccount,
}
