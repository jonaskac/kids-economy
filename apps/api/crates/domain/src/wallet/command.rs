//! What a Member asks a Wallet to do.

use super::account::SavingsGoal;
use super::transaction::{DepositCategory, WithdrawalCategory};
use crate::ids::{AccountId, FamilyId, KidId, TransactionId, WalletId};
use crate::money::{Amount, Currency};
pub use crate::role::Context;
use crate::text::{AccountName, Memo};
use time::Date;

/// Opens a Kid's Wallet together with its Main account (done when the Kid is added).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OpenWallet {
    pub wallet_id: WalletId,
    pub family_id: FamilyId,
    pub kid_id: KidId,
    /// The Family's Currency.
    pub currency: Currency,
    pub main_account_name: AccountName,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletCommand {
    OpenAccount {
        name: AccountName,
        savings_goal: Option<SavingsGoal>,
    },
    /// `None` removes the goal.
    SetSavingsGoal {
        account_id: AccountId,
        savings_goal: Option<SavingsGoal>,
    },
    CloseAccount {
        account_id: AccountId,
    },
    Deposit {
        /// `None` means the Main account.
        account_id: Option<AccountId>,
        amount: Amount,
        category: DepositCategory,
        memo: Memo,
        occurred_on: Date,
    },
    Withdraw {
        account_id: AccountId,
        amount: Amount,
        category: WithdrawalCategory,
        memo: Memo,
        occurred_on: Date,
    },
    Transfer {
        from: AccountId,
        to: AccountId,
        amount: Amount,
        memo: Memo,
        occurred_on: Date,
    },
    Correct {
        transaction_id: TransactionId,
        /// Shown to the Kid as the Correction's Memo.
        reason: Memo,
        occurred_on: Date,
    },
}
