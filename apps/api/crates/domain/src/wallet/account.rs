//! Account: one named pot of money in a Wallet, with its Ledger.

use super::transaction::{Direction, Transaction};
use crate::ids::AccountId;
use crate::money::{Amount, Balance, Currency};
use crate::text::AccountName;
use time::{Date, OffsetDateTime};

/// An optional target on an Account. An Account with one is a savings Account.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SavingsGoal {
    pub name: AccountName,
    pub target: Amount,
    pub target_date: Option<Date>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Account {
    pub(super) id: AccountId,
    pub(super) name: AccountName,
    pub(super) currency: Currency,
    pub(super) is_main: bool,
    pub(super) savings_goal: Option<SavingsGoal>,
    /// Whether the current Savings goal has been reached (celebrated) already.
    pub(super) goal_reached: bool,
    pub(super) closed_at: Option<OffsetDateTime>,
    /// Append-only, in recorded order.
    pub(super) ledger: Vec<Transaction>,
}

impl Account {
    pub(super) fn new(
        id: AccountId,
        name: AccountName,
        currency: Currency,
        is_main: bool,
        savings_goal: Option<SavingsGoal>,
    ) -> Self {
        Self {
            id,
            name,
            currency,
            is_main,
            savings_goal,
            goal_reached: false,
            closed_at: None,
            ledger: Vec::new(),
        }
    }

    pub const fn id(&self) -> AccountId {
        self.id
    }
    pub const fn name(&self) -> &AccountName {
        &self.name
    }
    pub const fn is_main(&self) -> bool {
        self.is_main
    }
    pub const fn savings_goal(&self) -> Option<&SavingsGoal> {
        self.savings_goal.as_ref()
    }
    pub const fn is_savings(&self) -> bool {
        self.savings_goal.is_some()
    }
    pub const fn closed_at(&self) -> Option<OffsetDateTime> {
        self.closed_at
    }
    pub const fn is_closed(&self) -> bool {
        self.closed_at.is_some()
    }
    pub fn ledger(&self) -> &[Transaction] {
        &self.ledger
    }

    /// The sum of the Ledger. Derived every time; never stored as truth.
    pub fn balance(&self) -> Balance {
        let minor = self.ledger.iter().fold(0_i64, |sum, t| match t.direction {
            // The Wallet never accepts a Transaction that would overflow or overdraw,
            // so saturating only guards against a corrupt history.
            Direction::In => sum.saturating_add(t.amount.minor()),
            Direction::Out => sum.saturating_sub(t.amount.minor()),
        });
        Balance::from_minor(minor, self.currency)
    }

    /// Whether the Balance would newly reach the Savings goal with `balance`.
    pub(super) fn newly_reaches_goal(&self, balance: Balance) -> bool {
        match &self.savings_goal {
            Some(goal) => !self.goal_reached && balance.reaches(goal.target),
            None => false,
        }
    }
}
