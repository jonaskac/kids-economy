//! Transaction: one immutable movement of money on one Account.

use crate::ids::{AccountId, MemberId, TransactionId, TransferId};
use crate::money::Amount;
use crate::text::Memo;
use time::{Date, OffsetDateTime};

/// Whether money comes into or goes out of an Account. Amounts are always positive;
/// the Direction carries the sign.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Direction {
    In,
    Out,
}

impl Direction {
    pub const fn reversed(self) -> Self {
        match self {
            Direction::In => Direction::Out,
            Direction::Out => Direction::In,
        }
    }
}

/// Why money moved, as a teaching label (not accounting).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Category {
    Allowance,
    Chore,
    Gift,
    Interest,
    Purchase,
    Donation,
    Transfer,
    Correction,
}

/// The Categories a Deposit may carry: money from outside the Wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum DepositCategory {
    Allowance,
    Chore,
    Gift,
    Interest,
}

impl From<DepositCategory> for Category {
    fn from(category: DepositCategory) -> Self {
        match category {
            DepositCategory::Allowance => Category::Allowance,
            DepositCategory::Chore => Category::Chore,
            DepositCategory::Gift => Category::Gift,
            DepositCategory::Interest => Category::Interest,
        }
    }
}

/// The Categories a Withdrawal may carry: money leaving the Wallet.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WithdrawalCategory {
    Purchase,
    Donation,
}

impl From<WithdrawalCategory> for Category {
    fn from(category: WithdrawalCategory) -> Self {
        match category {
            WithdrawalCategory::Purchase => Category::Purchase,
            WithdrawalCategory::Donation => Category::Donation,
        }
    }
}

/// One recorded movement of money on one Account. Immutable once recorded:
/// fields are private and there are no setters.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Transaction {
    pub(super) id: TransactionId,
    pub(super) account_id: AccountId,
    pub(super) direction: Direction,
    pub(super) amount: Amount,
    pub(super) category: Category,
    pub(super) memo: Memo,
    pub(super) occurred_on: Date,
    pub(super) recorded_at: OffsetDateTime,
    pub(super) actor: MemberId,
    pub(super) transfer_id: Option<TransferId>,
    pub(super) corrects: Option<TransactionId>,
}

/// Every field of a Transaction, in the open. For adapters that store and read back a
/// History: a restored Transaction is trusted only after [`super::Wallet::replay`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TransactionParts {
    pub id: TransactionId,
    pub account_id: AccountId,
    pub direction: Direction,
    pub amount: Amount,
    pub category: Category,
    pub memo: Memo,
    pub occurred_on: Date,
    pub recorded_at: OffsetDateTime,
    pub actor: MemberId,
    pub transfer_id: Option<TransferId>,
    pub corrects: Option<TransactionId>,
}

impl Transaction {
    /// Rebuilds a Transaction read back from storage. New Transactions are only ever
    /// made by the Wallet's decisions.
    pub fn restore(parts: TransactionParts) -> Self {
        let TransactionParts {
            id,
            account_id,
            direction,
            amount,
            category,
            memo,
            occurred_on,
            recorded_at,
            actor,
            transfer_id,
            corrects,
        } = parts;
        Self {
            id,
            account_id,
            direction,
            amount,
            category,
            memo,
            occurred_on,
            recorded_at,
            actor,
            transfer_id,
            corrects,
        }
    }

    /// All fields, for storing.
    pub fn parts(&self) -> TransactionParts {
        TransactionParts {
            id: self.id,
            account_id: self.account_id,
            direction: self.direction,
            amount: self.amount,
            category: self.category,
            memo: self.memo.clone(),
            occurred_on: self.occurred_on,
            recorded_at: self.recorded_at,
            actor: self.actor,
            transfer_id: self.transfer_id,
            corrects: self.corrects,
        }
    }

    pub const fn id(&self) -> TransactionId {
        self.id
    }
    pub const fn account_id(&self) -> AccountId {
        self.account_id
    }
    pub const fn direction(&self) -> Direction {
        self.direction
    }
    pub const fn amount(&self) -> Amount {
        self.amount
    }
    pub const fn category(&self) -> Category {
        self.category
    }
    pub const fn memo(&self) -> &Memo {
        &self.memo
    }
    /// The calendar day it happened for the Kid. History is sorted by it.
    pub const fn occurred_on(&self) -> Date {
        self.occurred_on
    }
    /// When the system stored it (server time). Audit order.
    pub const fn recorded_at(&self) -> OffsetDateTime {
        self.recorded_at
    }
    /// The Member who entered it.
    pub const fn actor(&self) -> MemberId {
        self.actor
    }
    /// Set on both legs of a Transfer, and on both legs of a corrected Transfer.
    pub const fn transfer_id(&self) -> Option<TransferId> {
        self.transfer_id
    }
    /// Set on a Correction: the Transaction it reverses.
    pub const fn corrects(&self) -> Option<TransactionId> {
        self.corrects
    }
}
