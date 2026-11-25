//! The Wallet aggregate: all of one Kid's Accounts and their Ledgers.
//!
//! The Wallet is the unit of consistency. It is a pure decider:
//! [`Wallet::decide`] checks a command against the current state and returns the
//! events to record (or why not), and [`Wallet::apply`] folds a recorded event into the
//! state. [`Wallet::replay`] rebuilds a Wallet from its whole history. See ADR-0008.

mod account;
mod command;
mod error;
mod event;
mod transaction;

pub use account::{Account, SavingsGoal};
pub use command::{Context, OpenWallet, WalletCommand};
pub use error::{ReplayError, WalletError};
pub use event::WalletEvent;
pub use transaction::{Category, DepositCategory, Direction, Transaction, WithdrawalCategory};

use crate::ids::{AccountId, FamilyId, KidId, NewIds, TransactionId, TransferId, WalletId};
use crate::money::{Amount, Balance, Currency};
use crate::text::Memo;
use time::Date;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wallet {
    id: WalletId,
    family_id: FamilyId,
    kid_id: KidId,
    currency: Currency,
    main_account_id: AccountId,
    /// In opening order; the Main account first. Closed Accounts are kept.
    accounts: Vec<Account>,
}

/// What a new Transaction is, before the Wallet stamps ids, time and Actor on it.
struct Entry<'a> {
    account_id: AccountId,
    direction: Direction,
    amount: Amount,
    category: Category,
    memo: &'a Memo,
    occurred_on: Date,
    transfer_id: Option<TransferId>,
    corrects: Option<TransactionId>,
}

impl Wallet {
    /// Decides opening a Wallet with its Main account.
    pub fn open(
        command: &OpenWallet,
        context: &Context,
        ids: &mut impl NewIds,
    ) -> Result<WalletEvent, WalletError> {
        ensure_may_change(context)?;
        Ok(WalletEvent::WalletOpened {
            wallet_id: command.wallet_id,
            family_id: command.family_id,
            kid_id: command.kid_id,
            currency: command.currency,
            main_account_id: AccountId::from_uuid(ids.next_id()),
            main_account_name: command.main_account_name.clone(),
            opened_by: context.actor.member_id,
            opened_at: context.now,
        })
    }

    /// Rebuilds a Wallet from its full, ordered history.
    pub fn replay<'a>(
        history: impl IntoIterator<Item = &'a WalletEvent>,
    ) -> Result<Self, ReplayError> {
        let mut events = history.into_iter();
        let mut wallet = match events.next() {
            Some(WalletEvent::WalletOpened {
                wallet_id,
                family_id,
                kid_id,
                currency,
                main_account_id,
                main_account_name,
                ..
            }) => Wallet {
                id: *wallet_id,
                family_id: *family_id,
                kid_id: *kid_id,
                currency: *currency,
                main_account_id: *main_account_id,
                accounts: vec![Account::new(
                    *main_account_id,
                    main_account_name.clone(),
                    *currency,
                    true,
                    None,
                )],
            },
            _ => return Err(ReplayError::NotOpened),
        };
        for event in events {
            wallet.apply(event)?;
        }
        Ok(wallet)
    }

    pub const fn id(&self) -> WalletId {
        self.id
    }
    pub const fn family_id(&self) -> FamilyId {
        self.family_id
    }
    pub const fn kid_id(&self) -> KidId {
        self.kid_id
    }
    pub const fn currency(&self) -> Currency {
        self.currency
    }
    pub const fn main_account_id(&self) -> AccountId {
        self.main_account_id
    }
    pub fn accounts(&self) -> impl Iterator<Item = &Account> {
        self.accounts.iter()
    }
    pub fn account(&self, id: AccountId) -> Option<&Account> {
        self.accounts.iter().find(|a| a.id == id)
    }

    /// The sum of all Account Balances: how much the Kid has. Transfers never change it.
    pub fn total(&self) -> Balance {
        let minor = self
            .accounts
            .iter()
            .fold(0_i64, |sum, a| sum.saturating_add(a.balance().minor()));
        Balance::from_minor(minor, self.currency)
    }

    /// Checks `command` against the Wallet's rules and returns the events to record.
    /// Nothing changes until the caller applies (and stores) them: both or neither.
    pub fn decide(
        &self,
        command: &WalletCommand,
        context: &Context,
        ids: &mut impl NewIds,
    ) -> Result<Vec<WalletEvent>, WalletError> {
        ensure_may_change(context)?;
        match command {
            WalletCommand::OpenAccount { name, savings_goal } => {
                self.ensure_goal_currency(savings_goal.as_ref())?;
                Ok(vec![WalletEvent::AccountOpened {
                    account_id: AccountId::from_uuid(ids.next_id()),
                    name: name.clone(),
                    savings_goal: savings_goal.clone(),
                }])
            }
            WalletCommand::SetSavingsGoal {
                account_id,
                savings_goal,
            } => {
                let account = self.open_account(*account_id)?;
                self.ensure_goal_currency(savings_goal.as_ref())?;
                let mut events = vec![WalletEvent::SavingsGoalSet {
                    account_id: *account_id,
                    savings_goal: savings_goal.clone(),
                }];
                if savings_goal
                    .as_ref()
                    .is_some_and(|goal| account.balance().reaches(goal.target))
                {
                    events.push(WalletEvent::SavingsGoalReached {
                        account_id: *account_id,
                    });
                }
                Ok(events)
            }
            WalletCommand::CloseAccount { account_id } => {
                let account = self.open_account(*account_id)?;
                if account.is_main {
                    return Err(WalletError::MainAccount);
                }
                if !account.balance().is_zero() {
                    return Err(WalletError::NonZeroBalance);
                }
                Ok(vec![WalletEvent::AccountClosed {
                    account_id: *account_id,
                    closed_at: context.now,
                }])
            }
            WalletCommand::Deposit {
                account_id,
                amount,
                category,
                memo,
                occurred_on,
            } => {
                let account_id = account_id.unwrap_or(self.main_account_id);
                let entry = Entry {
                    account_id,
                    direction: Direction::In,
                    amount: *amount,
                    category: (*category).into(),
                    memo,
                    occurred_on: *occurred_on,
                    transfer_id: None,
                    corrects: None,
                };
                let transaction = self.check(entry, context, ids)?;
                let mut events = vec![WalletEvent::TransactionRecorded { transaction }];
                self.celebrate(account_id, *amount, &mut events)?;
                Ok(events)
            }
            WalletCommand::Withdraw {
                account_id,
                amount,
                category,
                memo,
                occurred_on,
            } => {
                let entry = Entry {
                    account_id: *account_id,
                    direction: Direction::Out,
                    amount: *amount,
                    category: (*category).into(),
                    memo,
                    occurred_on: *occurred_on,
                    transfer_id: None,
                    corrects: None,
                };
                let transaction = self.check(entry, context, ids)?;
                Ok(vec![WalletEvent::TransactionRecorded { transaction }])
            }
            WalletCommand::Transfer {
                from,
                to,
                amount,
                memo,
                occurred_on,
            } => {
                if from == to {
                    return Err(WalletError::SameAccount);
                }
                let transfer_id = Some(TransferId::from_uuid(ids.next_id()));
                let out_leg = self.check(
                    Entry {
                        account_id: *from,
                        direction: Direction::Out,
                        amount: *amount,
                        category: Category::Transfer,
                        memo,
                        occurred_on: *occurred_on,
                        transfer_id,
                        corrects: None,
                    },
                    context,
                    ids,
                )?;
                let in_leg = self.check(
                    Entry {
                        account_id: *to,
                        direction: Direction::In,
                        amount: *amount,
                        category: Category::Transfer,
                        memo,
                        occurred_on: *occurred_on,
                        transfer_id,
                        corrects: None,
                    },
                    context,
                    ids,
                )?;
                let mut events = vec![WalletEvent::TransferRecorded { out_leg, in_leg }];
                self.celebrate(*to, *amount, &mut events)?;
                Ok(events)
            }
            WalletCommand::Correct {
                transaction_id,
                reason,
                occurred_on,
            } => self.decide_correction(*transaction_id, reason, *occurred_on, context, ids),
        }
    }

    /// Folds one recorded event into the Wallet.
    pub fn apply(&mut self, event: &WalletEvent) -> Result<(), ReplayError> {
        match event {
            WalletEvent::WalletOpened { .. } => return Err(ReplayError::AlreadyOpened),
            WalletEvent::AccountOpened {
                account_id,
                name,
                savings_goal,
            } => {
                if self.account(*account_id).is_some() {
                    return Err(ReplayError::DuplicateAccount);
                }
                self.accounts.push(Account::new(
                    *account_id,
                    name.clone(),
                    self.currency,
                    false,
                    savings_goal.clone(),
                ));
            }
            WalletEvent::SavingsGoalSet {
                account_id,
                savings_goal,
            } => {
                let account = self.account_mut(*account_id)?;
                account.savings_goal = savings_goal.clone();
                account.goal_reached = false;
            }
            WalletEvent::SavingsGoalReached { account_id } => {
                self.account_mut(*account_id)?.goal_reached = true;
            }
            WalletEvent::AccountClosed {
                account_id,
                closed_at,
            } => {
                self.account_mut(*account_id)?.closed_at = Some(*closed_at);
            }
            WalletEvent::TransactionRecorded { transaction } => self.append(transaction)?,
            WalletEvent::TransferRecorded { out_leg, in_leg } => {
                // Check both Accounts first so a bad event changes nothing.
                self.account_mut(in_leg.account_id)?;
                self.append(out_leg)?;
                self.append(in_leg)?;
            }
            WalletEvent::CorrectionRecorded { correction, paired } => {
                if let Some(paired) = paired {
                    self.account_mut(paired.account_id)?;
                }
                self.append(correction)?;
                if let Some(paired) = paired {
                    self.append(paired)?;
                }
            }
        }
        Ok(())
    }

    // ---------- rules ----------

    fn decide_correction(
        &self,
        transaction_id: TransactionId,
        reason: &Memo,
        occurred_on: Date,
        context: &Context,
        ids: &mut impl NewIds,
    ) -> Result<Vec<WalletEvent>, WalletError> {
        let original = self
            .transaction(transaction_id)
            .ok_or(WalletError::UnknownTransaction)?;
        if self.is_corrected(original.id) {
            return Err(WalletError::AlreadyCorrected);
        }
        // Correcting one side of a Transfer corrects both, as a new linked pair.
        let other_leg = original
            .transfer_id
            .and_then(|transfer| self.other_leg(transfer, original.id));
        let transfer_id = other_leg.map(|_| TransferId::from_uuid(ids.next_id()));
        let reversal = |t: &Transaction| Entry {
            account_id: t.account_id,
            direction: t.direction.reversed(),
            amount: t.amount,
            category: Category::Correction,
            memo: reason,
            occurred_on,
            transfer_id,
            corrects: Some(t.id),
        };
        // Each reversal is checked against its own Account (no Overdraft per Account).
        let correction = self.check(reversal(original), context, ids)?;
        let paired = match other_leg {
            Some(leg) => Some(self.check(reversal(leg), context, ids)?),
            None => None,
        };
        Ok(vec![WalletEvent::CorrectionRecorded { correction, paired }])
    }

    /// Validates one new Transaction against its Account and stamps it.
    fn check(
        &self,
        entry: Entry<'_>,
        context: &Context,
        ids: &mut impl NewIds,
    ) -> Result<Transaction, WalletError> {
        if entry.amount.currency() != self.currency {
            return Err(WalletError::CurrencyMismatch);
        }
        let account = self.open_account(entry.account_id)?;
        match entry.direction {
            Direction::Out if !account.balance().covers(entry.amount) => {
                return Err(WalletError::InsufficientFunds)
            }
            Direction::In if entry.transfer_id.is_none() => {
                // Money from outside raises the Wallet total; it must still fit.
                self.total()
                    .plus(entry.amount)
                    .ok_or(WalletError::TotalOverflow)?;
            }
            _ => {}
        }
        Ok(Transaction {
            id: TransactionId::from_uuid(ids.next_id()),
            account_id: entry.account_id,
            direction: entry.direction,
            amount: entry.amount,
            category: entry.category,
            memo: entry.memo.clone(),
            occurred_on: entry.occurred_on,
            recorded_at: context.now,
            actor: context.actor.member_id,
            transfer_id: entry.transfer_id,
            corrects: entry.corrects,
        })
    }

    /// Adds `SavingsGoalReached` if money coming in reaches the Account's goal.
    fn celebrate(
        &self,
        account_id: AccountId,
        incoming: Amount,
        events: &mut Vec<WalletEvent>,
    ) -> Result<(), WalletError> {
        let account = self.open_account(account_id)?;
        let after = account
            .balance()
            .plus(incoming)
            .ok_or(WalletError::TotalOverflow)?;
        if account.newly_reaches_goal(after) {
            events.push(WalletEvent::SavingsGoalReached { account_id });
        }
        Ok(())
    }

    fn ensure_goal_currency(&self, goal: Option<&SavingsGoal>) -> Result<(), WalletError> {
        match goal {
            Some(goal) if goal.target.currency() != self.currency => {
                Err(WalletError::CurrencyMismatch)
            }
            _ => Ok(()),
        }
    }

    fn open_account(&self, id: AccountId) -> Result<&Account, WalletError> {
        let account = self.account(id).ok_or(WalletError::UnknownAccount)?;
        if account.is_closed() {
            return Err(WalletError::AccountClosed);
        }
        Ok(account)
    }

    fn transactions(&self) -> impl Iterator<Item = &Transaction> {
        self.accounts.iter().flat_map(|a| a.ledger.iter())
    }

    fn transaction(&self, id: TransactionId) -> Option<&Transaction> {
        self.transactions().find(|t| t.id == id)
    }

    fn is_corrected(&self, id: TransactionId) -> bool {
        self.transactions().any(|t| t.corrects == Some(id))
    }

    fn other_leg(&self, transfer: TransferId, leg: TransactionId) -> Option<&Transaction> {
        self.transactions()
            .find(|t| t.transfer_id == Some(transfer) && t.id != leg)
    }

    // ---------- state changes ----------

    fn account_mut(&mut self, id: AccountId) -> Result<&mut Account, ReplayError> {
        self.accounts
            .iter_mut()
            .find(|a| a.id == id)
            .ok_or(ReplayError::UnknownAccount)
    }

    fn append(&mut self, transaction: &Transaction) -> Result<(), ReplayError> {
        self.account_mut(transaction.account_id)?
            .ledger
            .push(transaction.clone());
        Ok(())
    }
}

fn ensure_may_change(context: &Context) -> Result<(), WalletError> {
    if context.actor.role.may_change_wallets() {
        Ok(())
    } else {
        Err(WalletError::NotAllowed)
    }
}

#[cfg(test)]
mod tests;
