//! Facts a Wallet has recorded. Its history is the ordered list of these events.

use super::account::SavingsGoal;
use super::transaction::Transaction;
use crate::ids::{AccountId, FamilyId, KidId, MemberId, WalletId};
use crate::money::Currency;
use crate::text::AccountName;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WalletEvent {
    /// Always the first event of a Wallet. Opens the Main account with it.
    WalletOpened {
        wallet_id: WalletId,
        family_id: FamilyId,
        kid_id: KidId,
        currency: Currency,
        main_account_id: AccountId,
        main_account_name: AccountName,
        opened_by: MemberId,
        opened_at: OffsetDateTime,
    },
    AccountOpened {
        account_id: AccountId,
        name: AccountName,
        savings_goal: Option<SavingsGoal>,
    },
    SavingsGoalSet {
        account_id: AccountId,
        savings_goal: Option<SavingsGoal>,
    },
    /// The Account's Balance reached its Savings goal for the first time. Celebrate.
    SavingsGoalReached { account_id: AccountId },
    AccountClosed {
        account_id: AccountId,
        closed_at: OffsetDateTime,
    },
    /// A Deposit or a Withdrawal.
    TransactionRecorded { transaction: Transaction },
    /// Both legs of a Transfer, recorded together.
    TransferRecorded {
        out_leg: Transaction,
        in_leg: Transaction,
    },
    /// A Correction; `paired` is set when a Transfer is corrected (both legs at once).
    CorrectionRecorded {
        correction: Transaction,
        paired: Option<Transaction>,
    },
}
