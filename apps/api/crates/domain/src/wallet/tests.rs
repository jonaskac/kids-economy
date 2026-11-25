//! Behaviour of the Wallet aggregate, in glossary terms.

use super::*;
use crate::ids::{FamilyId, KidId, MemberId, NewIds, WalletId};
use crate::money::{Amount, Currency};
use crate::role::{Actor, Role};
use crate::text::{AccountName, Memo};
use time::macros::{date, datetime};
use time::OffsetDateTime;
use uuid::Uuid;

// ---------- fixture ----------

struct SequentialIds(u128);

impl NewIds for SequentialIds {
    fn next_id(&mut self) -> Uuid {
        self.0 += 1;
        Uuid::from_u128(self.0)
    }
}

fn sek() -> Currency {
    Currency::new("SEK").expect("currency")
}

fn kr(whole: i64) -> Amount {
    Amount::new(whole * 100, sek()).expect("amount")
}

fn name(text: &str) -> AccountName {
    AccountName::new(text).expect("name")
}

fn memo(text: &str) -> Memo {
    Memo::new(text).expect("memo")
}

const NOW: OffsetDateTime = datetime!(2026-11-25 13:00 UTC);

fn member(n: u128) -> MemberId {
    MemberId::from_uuid(Uuid::from_u128(1_000 + n))
}

fn actor(role: Role) -> Actor {
    Actor {
        member_id: member(1),
        role,
    }
}

struct Fixture {
    wallet: Wallet,
    ids: SequentialIds,
    history: Vec<WalletEvent>,
}

impl Fixture {
    fn new() -> Self {
        let mut ids = SequentialIds(0);
        let opened = Wallet::open(
            &OpenWallet {
                wallet_id: WalletId::from_uuid(Uuid::from_u128(900)),
                family_id: FamilyId::from_uuid(Uuid::from_u128(901)),
                kid_id: KidId::from_uuid(Uuid::from_u128(902)),
                currency: sek(),
                main_account_name: name("Main"),
            },
            &Context {
                actor: actor(Role::Parent),
                now: NOW,
            },
            &mut ids,
        )
        .expect("open wallet");
        let history = vec![opened];
        let wallet = Wallet::replay(&history).expect("replay");
        Fixture {
            wallet,
            ids,
            history,
        }
    }

    fn main(&self) -> AccountId {
        self.wallet.main_account_id()
    }

    fn run_as(
        &mut self,
        role: Role,
        command: WalletCommand,
    ) -> Result<Vec<WalletEvent>, WalletError> {
        let context = Context {
            actor: actor(role),
            now: NOW,
        };
        let events = self.wallet.decide(&command, &context, &mut self.ids)?;
        for event in &events {
            self.wallet.apply(event).expect("decided events apply");
        }
        self.history.extend(events.iter().cloned());
        Ok(events)
    }

    fn run(&mut self, command: WalletCommand) -> Result<Vec<WalletEvent>, WalletError> {
        self.run_as(Role::Parent, command)
    }

    fn open_account(&mut self, account_name: &str) -> AccountId {
        let events = self
            .run(WalletCommand::OpenAccount {
                name: name(account_name),
                savings_goal: None,
            })
            .expect("open account");
        match events.as_slice() {
            [WalletEvent::AccountOpened { account_id, .. }] => *account_id,
            other => panic!("unexpected events {other:?}"),
        }
    }

    fn deposit(
        &mut self,
        account: Option<AccountId>,
        amount: Amount,
    ) -> Result<Vec<WalletEvent>, WalletError> {
        self.run(deposit(account, amount))
    }

    fn balance(&self, account: AccountId) -> i64 {
        self.wallet
            .account(account)
            .expect("account")
            .balance()
            .minor()
    }

    fn total(&self) -> i64 {
        self.wallet.total().minor()
    }

    fn only_transaction(events: &[WalletEvent]) -> Transaction {
        match events {
            [WalletEvent::TransactionRecorded { transaction }] => transaction.clone(),
            other => panic!("unexpected events {other:?}"),
        }
    }
}

fn deposit(account: Option<AccountId>, amount: Amount) -> WalletCommand {
    WalletCommand::Deposit {
        account_id: account,
        amount,
        category: DepositCategory::Allowance,
        memo: memo("Pocket money week 48"),
        occurred_on: date!(2026 - 11 - 24),
    }
}

fn withdraw(account: AccountId, amount: Amount) -> WalletCommand {
    WalletCommand::Withdraw {
        account_id: account,
        amount,
        category: WithdrawalCategory::Purchase,
        memo: memo("Comic"),
        occurred_on: date!(2026 - 11 - 25),
    }
}

fn transfer(from: AccountId, to: AccountId, amount: Amount) -> WalletCommand {
    WalletCommand::Transfer {
        from,
        to,
        amount,
        memo: memo("Saving for Lego"),
        occurred_on: date!(2026 - 11 - 25),
    }
}

fn correct(transaction_id: TransactionId) -> WalletCommand {
    WalletCommand::Correct {
        transaction_id,
        reason: memo("Typed the wrong amount"),
        occurred_on: date!(2026 - 11 - 25),
    }
}

/// Closes `account`, moving what is left to `move_to` (`None`: the Main account).
fn close(account: AccountId, move_to: Option<AccountId>) -> WalletCommand {
    WalletCommand::CloseAccount {
        account_id: account,
        move_to,
        memo: memo("Jar closed"),
        occurred_on: date!(2026 - 11 - 25),
    }
}

fn goal(target: Amount) -> SavingsGoal {
    SavingsGoal {
        name: name("Lego set"),
        target,
        target_date: Some(date!(2027 - 06 - 21)),
    }
}

// ---------- opening a wallet ----------

#[test]
fn a_new_wallet_has_exactly_one_main_account_with_zero_balance() {
    let fx = Fixture::new();
    let accounts: Vec<_> = fx.wallet.accounts().collect();
    assert_eq!(accounts.len(), 1);
    assert_eq!(accounts[0].id(), fx.main());
    assert_eq!(accounts[0].name().as_str(), "Main");
    assert!(accounts[0].is_main());
    assert_eq!(fx.balance(fx.main()), 0);
    assert_eq!(fx.total(), 0);
    assert_eq!(fx.wallet.currency(), sek());
}

#[test]
fn a_viewer_cannot_open_a_wallet() {
    let result = Wallet::open(
        &OpenWallet {
            wallet_id: WalletId::from_uuid(Uuid::from_u128(900)),
            family_id: FamilyId::from_uuid(Uuid::from_u128(901)),
            kid_id: KidId::from_uuid(Uuid::from_u128(902)),
            currency: sek(),
            main_account_name: name("Main"),
        },
        &Context {
            actor: actor(Role::Viewer),
            now: NOW,
        },
        &mut SequentialIds(0),
    );
    assert_eq!(result, Err(WalletError::NotAllowed));
}

// ---------- Deposit ----------

#[test]
fn a_deposit_without_an_account_lands_in_the_main_account() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(20)).expect("deposit");
    assert_eq!(fx.balance(fx.main()), 2000);
    assert_eq!(fx.total(), 2000);
}

#[test]
fn a_deposit_can_land_in_a_chosen_account() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.deposit(Some(lego), kr(50)).expect("deposit");
    assert_eq!(fx.balance(lego), 5000);
    assert_eq!(fx.balance(fx.main()), 0);
}

#[test]
fn a_deposit_records_what_happened_who_entered_it_and_when() {
    let mut fx = Fixture::new();
    let events = fx.deposit(None, kr(20)).expect("deposit");
    let transaction = Fixture::only_transaction(&events);
    assert_eq!(transaction.account_id(), fx.main());
    assert_eq!(transaction.direction(), Direction::In);
    assert_eq!(transaction.amount(), kr(20));
    assert_eq!(transaction.category(), Category::Allowance);
    assert_eq!(transaction.memo().as_str(), "Pocket money week 48");
    assert_eq!(transaction.occurred_on(), date!(2026 - 11 - 24));
    assert_eq!(transaction.recorded_at(), NOW);
    assert_eq!(transaction.actor(), member(1));
    assert_eq!(transaction.transfer_id(), None);
    assert_eq!(transaction.corrects(), None);
    assert_eq!(
        fx.wallet.account(fx.main()).expect("main").ledger(),
        &[transaction]
    );
}

#[test]
fn a_deposit_in_another_currency_is_rejected() {
    let mut fx = Fixture::new();
    let euros = Amount::new(100, Currency::new("EUR").expect("eur")).expect("amount");
    assert_eq!(fx.deposit(None, euros), Err(WalletError::CurrencyMismatch));
    assert_eq!(fx.total(), 0);
}

#[test]
fn a_deposit_to_an_unknown_account_is_rejected() {
    let mut fx = Fixture::new();
    let stranger = AccountId::from_uuid(Uuid::from_u128(77));
    assert_eq!(
        fx.deposit(Some(stranger), kr(1)),
        Err(WalletError::UnknownAccount)
    );
}

#[test]
fn a_deposit_to_a_closed_account_is_rejected() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Old jar");
    fx.run(close(jar, None)).expect("close");
    assert_eq!(
        fx.deposit(Some(jar), kr(1)),
        Err(WalletError::AccountClosed)
    );
}

#[test]
fn a_deposit_that_would_overflow_the_wallet_total_is_rejected() {
    let mut fx = Fixture::new();
    let other = fx.open_account("Other");
    fx.deposit(None, Amount::new(i64::MAX - 10, sek()).expect("amount"))
        .expect("deposit");
    assert_eq!(
        fx.deposit(Some(other), Amount::new(11, sek()).expect("amount")),
        Err(WalletError::TotalOverflow)
    );
    assert_eq!(fx.balance(other), 0);
}

// ---------- Withdrawal ----------

#[test]
fn a_withdrawal_takes_money_out_of_the_wallet() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(50)).expect("deposit");
    let events = fx.run(withdraw(fx.main(), kr(35))).expect("withdraw");
    let transaction = Fixture::only_transaction(&events);
    assert_eq!(transaction.direction(), Direction::Out);
    assert_eq!(transaction.category(), Category::Purchase);
    assert_eq!(fx.balance(fx.main()), 1500);
    assert_eq!(fx.total(), 1500);
}

#[test]
fn a_withdrawal_may_empty_an_account_exactly() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(20)).expect("deposit");
    fx.run(withdraw(fx.main(), kr(20))).expect("withdraw");
    assert_eq!(fx.balance(fx.main()), 0);
}

#[test]
fn a_withdrawal_that_would_overdraw_the_account_is_rejected() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(20)).expect("deposit");
    let one_ore_too_much = Amount::new(2001, sek()).expect("amount");
    assert_eq!(
        fx.run(withdraw(fx.main(), one_ore_too_much)),
        Err(WalletError::InsufficientFunds)
    );
    assert_eq!(fx.balance(fx.main()), 2000);
}

#[test]
fn money_in_another_account_does_not_cover_a_withdrawal() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.deposit(Some(lego), kr(100)).expect("deposit");
    assert_eq!(
        fx.run(withdraw(fx.main(), kr(1))),
        Err(WalletError::InsufficientFunds)
    );
}

// ---------- Transfer ----------

#[test]
fn a_transfer_records_two_linked_legs_and_keeps_the_wallet_total() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.deposit(None, kr(50)).expect("deposit");

    let events = fx.run(transfer(fx.main(), lego, kr(30))).expect("transfer");

    let (out_leg, in_leg) = match events.as_slice() {
        [WalletEvent::TransferRecorded { out_leg, in_leg }] => (out_leg.clone(), in_leg.clone()),
        other => panic!("unexpected events {other:?}"),
    };
    assert_eq!(out_leg.account_id(), fx.main());
    assert_eq!(out_leg.direction(), Direction::Out);
    assert_eq!(in_leg.account_id(), lego);
    assert_eq!(in_leg.direction(), Direction::In);
    assert_eq!(out_leg.amount(), in_leg.amount());
    assert_eq!(out_leg.category(), Category::Transfer);
    assert_eq!(in_leg.category(), Category::Transfer);
    assert!(out_leg.transfer_id().is_some());
    assert_eq!(out_leg.transfer_id(), in_leg.transfer_id());
    assert_ne!(out_leg.id(), in_leg.id());

    assert_eq!(fx.balance(fx.main()), 2000);
    assert_eq!(fx.balance(lego), 3000);
    assert_eq!(fx.total(), 5000);
}

#[test]
fn a_transfer_to_the_same_account_is_rejected() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(10)).expect("deposit");
    assert_eq!(
        fx.run(transfer(fx.main(), fx.main(), kr(1))),
        Err(WalletError::SameAccount)
    );
}

#[test]
fn a_transfer_that_would_overdraw_the_source_records_neither_leg() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.deposit(None, kr(10)).expect("deposit");
    assert_eq!(
        fx.run(transfer(fx.main(), lego, kr(11))),
        Err(WalletError::InsufficientFunds)
    );
    assert_eq!(fx.balance(fx.main()), 1000);
    assert!(fx.wallet.account(lego).expect("lego").ledger().is_empty());
}

#[test]
fn a_transfer_needs_two_open_accounts_of_this_wallet() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Old jar");
    fx.run(close(jar, None)).expect("close");
    fx.deposit(None, kr(10)).expect("deposit");
    assert_eq!(
        fx.run(transfer(fx.main(), jar, kr(1))),
        Err(WalletError::AccountClosed)
    );
    let stranger = AccountId::from_uuid(Uuid::from_u128(77));
    assert_eq!(
        fx.run(transfer(fx.main(), stranger, kr(1))),
        Err(WalletError::UnknownAccount)
    );
}

// ---------- Accounts and Savings goals ----------

#[test]
fn an_account_with_a_savings_goal_is_a_savings_account() {
    let mut fx = Fixture::new();
    let events = fx
        .run(WalletCommand::OpenAccount {
            name: name("Lego set"),
            savings_goal: Some(goal(kr(400))),
        })
        .expect("open");
    let lego = match events.as_slice() {
        [WalletEvent::AccountOpened { account_id, .. }] => *account_id,
        other => panic!("unexpected events {other:?}"),
    };
    let account = fx.wallet.account(lego).expect("lego");
    assert!(account.is_savings());
    assert!(!account.is_main());
    assert_eq!(account.savings_goal(), Some(&goal(kr(400))));
    assert!(!fx.wallet.account(fx.main()).expect("main").is_savings());
}

#[test]
fn a_savings_goal_must_be_in_the_wallet_currency() {
    let mut fx = Fixture::new();
    let euros = Amount::new(100, Currency::new("EUR").expect("eur")).expect("amount");
    assert_eq!(
        fx.run(WalletCommand::OpenAccount {
            name: name("Trip"),
            savings_goal: Some(goal(euros)),
        }),
        Err(WalletError::CurrencyMismatch)
    );
}

#[test]
fn reaching_a_savings_goal_is_celebrated_once() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.run(WalletCommand::SetSavingsGoal {
        account_id: lego,
        savings_goal: Some(goal(kr(400))),
    })
    .expect("set goal");

    let events = fx.deposit(Some(lego), kr(300)).expect("deposit");
    assert_eq!(events.len(), 1, "not reached yet");

    let events = fx.deposit(Some(lego), kr(100)).expect("deposit");
    assert!(matches!(
        events.as_slice(),
        [WalletEvent::TransactionRecorded { .. }, WalletEvent::SavingsGoalReached { account_id }]
            if *account_id == lego
    ));

    let events = fx.deposit(Some(lego), kr(1)).expect("deposit");
    assert_eq!(events.len(), 1, "celebrated only the first time");
}

#[test]
fn reaching_a_goal_does_not_lock_the_money() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.run(WalletCommand::SetSavingsGoal {
        account_id: lego,
        savings_goal: Some(goal(kr(10))),
    })
    .expect("set goal");
    fx.deposit(Some(lego), kr(10)).expect("deposit");
    fx.run(withdraw(lego, kr(10)))
        .expect("spending is the kid's choice");
    assert_eq!(fx.balance(lego), 0);
}

#[test]
fn a_transfer_into_a_savings_account_can_reach_its_goal() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.run(WalletCommand::SetSavingsGoal {
        account_id: lego,
        savings_goal: Some(goal(kr(10))),
    })
    .expect("set goal");
    fx.deposit(None, kr(10)).expect("deposit");
    let events = fx.run(transfer(fx.main(), lego, kr(10))).expect("transfer");
    assert!(matches!(
        events.as_slice(),
        [
            WalletEvent::TransferRecorded { .. },
            WalletEvent::SavingsGoalReached { .. }
        ]
    ));
}

#[test]
fn setting_a_goal_already_covered_by_the_balance_is_reached_at_once() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(100)).expect("deposit");
    let events = fx
        .run(WalletCommand::SetSavingsGoal {
            account_id: fx.main(),
            savings_goal: Some(goal(kr(50))),
        })
        .expect("set goal");
    assert!(matches!(
        events.as_slice(),
        [
            WalletEvent::SavingsGoalSet { .. },
            WalletEvent::SavingsGoalReached { .. }
        ]
    ));
}

#[test]
fn removing_a_savings_goal_makes_it_a_plain_account() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.run(WalletCommand::SetSavingsGoal {
        account_id: lego,
        savings_goal: Some(goal(kr(10))),
    })
    .expect("set goal");
    fx.run(WalletCommand::SetSavingsGoal {
        account_id: lego,
        savings_goal: None,
    })
    .expect("remove goal");
    assert!(!fx.wallet.account(lego).expect("lego").is_savings());
}

#[test]
fn closing_an_empty_account_records_only_the_closing() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Jar");
    let events = fx.run(close(jar, None)).expect("close");
    assert_eq!(
        events,
        vec![WalletEvent::AccountClosed {
            account_id: jar,
            closed_at: NOW
        }]
    );
    let account = fx.wallet.account(jar).expect("closed accounts are kept");
    assert_eq!(account.closed_at(), Some(NOW));
    assert_eq!(
        fx.run(close(jar, None)),
        Err(WalletError::AccountClosed),
        "an Account closes once"
    );
}

#[test]
fn closing_an_account_moves_its_balance_to_the_main_account_with_a_transfer() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Jar");
    fx.deposit(Some(jar), kr(30)).expect("deposit");
    let events = fx.run(close(jar, None)).expect("close");

    let [WalletEvent::TransferRecorded { out_leg, in_leg }, WalletEvent::AccountClosed { account_id, .. }] =
        events.as_slice()
    else {
        panic!("unexpected events {events:?}");
    };
    assert_eq!(*account_id, jar);
    assert_eq!(
        (out_leg.account_id(), out_leg.direction(), out_leg.amount()),
        (jar, Direction::Out, kr(30))
    );
    assert_eq!(
        (in_leg.account_id(), in_leg.direction(), in_leg.amount()),
        (fx.main(), Direction::In, kr(30))
    );
    assert_eq!(out_leg.category(), Category::Transfer);
    assert_eq!(out_leg.memo(), &memo("Jar closed"));
    assert_eq!(out_leg.transfer_id(), in_leg.transfer_id());
    assert!(out_leg.transfer_id().is_some());

    assert_eq!(fx.balance(jar), 0);
    assert_eq!(fx.balance(fx.main()), 3000);
    assert_eq!(fx.total(), 3000, "closing never changes the Wallet total");
    assert_eq!(
        fx.wallet.account(jar).expect("kept").ledger().len(),
        2,
        "history stays"
    );
}

#[test]
fn closing_an_account_can_move_its_balance_to_another_open_account() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Jar");
    let bike = fx.open_account("Bike");
    fx.deposit(Some(jar), kr(12)).expect("deposit");
    fx.run(close(jar, Some(bike))).expect("close");
    assert_eq!(fx.balance(bike), 1200);
    assert_eq!(fx.balance(fx.main()), 0);
    assert!(fx.wallet.account(jar).expect("kept").is_closed());
}

#[test]
fn the_balance_must_go_to_another_open_account_of_this_wallet() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Jar");
    let old = fx.open_account("Old");
    fx.run(close(old, None)).expect("close old");
    fx.deposit(Some(jar), kr(5)).expect("deposit");
    let stranger = AccountId::from_uuid(Uuid::from_u128(77_777));
    assert_eq!(fx.run(close(jar, Some(jar))), Err(WalletError::SameAccount));
    assert_eq!(
        fx.run(close(jar, Some(old))),
        Err(WalletError::AccountClosed)
    );
    assert_eq!(
        fx.run(close(jar, Some(stranger))),
        Err(WalletError::UnknownAccount)
    );
    assert!(
        !fx.wallet.account(jar).expect("jar").is_closed(),
        "nothing changed"
    );
    assert_eq!(fx.balance(jar), 500);
}

#[test]
fn moving_the_balance_into_a_savings_account_can_reach_its_goal() {
    let mut fx = Fixture::new();
    let jar = fx.open_account("Jar");
    let lego = fx.open_account("Lego");
    fx.run(WalletCommand::SetSavingsGoal {
        account_id: lego,
        savings_goal: Some(goal(kr(40))),
    })
    .expect("goal");
    fx.deposit(Some(jar), kr(40)).expect("deposit");
    let events = fx.run(close(jar, Some(lego))).expect("close");
    assert!(events.contains(&WalletEvent::SavingsGoalReached { account_id: lego }));
    assert!(matches!(
        events.last(),
        Some(WalletEvent::AccountClosed { .. })
    ));
}

#[test]
fn the_main_account_cannot_be_closed() {
    let mut fx = Fixture::new();
    assert_eq!(
        fx.run(close(fx.main(), None)),
        Err(WalletError::MainAccount)
    );
}

// ---------- Correction ----------

#[test]
fn a_correction_reverses_a_transaction_and_keeps_both_in_the_ledger() {
    let mut fx = Fixture::new();
    let original = Fixture::only_transaction(&fx.deposit(None, kr(200)).expect("deposit"));

    let correction = Fixture::only_correction(&fx.run(correct(original.id())).expect("correct"));

    assert_eq!(correction.direction(), Direction::Out);
    assert_eq!(correction.amount(), original.amount());
    assert_eq!(correction.category(), Category::Correction);
    assert_eq!(correction.corrects(), Some(original.id()));
    assert_eq!(correction.memo().as_str(), "Typed the wrong amount");
    assert_eq!(fx.balance(fx.main()), 0);
    assert_eq!(
        fx.wallet.account(fx.main()).expect("main").ledger(),
        &[original, correction]
    );
}

#[test]
fn a_transaction_can_be_corrected_only_once() {
    let mut fx = Fixture::new();
    let original = Fixture::only_transaction(&fx.deposit(None, kr(20)).expect("deposit"));
    fx.run(correct(original.id())).expect("first correction");
    assert_eq!(
        fx.run(correct(original.id())),
        Err(WalletError::AlreadyCorrected)
    );
}

#[test]
fn a_correction_can_itself_be_corrected() {
    let mut fx = Fixture::new();
    let original = Fixture::only_transaction(&fx.deposit(None, kr(20)).expect("deposit"));
    let correction = Fixture::only_correction(&fx.run(correct(original.id())).expect("correct"));
    fx.run(correct(correction.id()))
        .expect("correct the correction");
    assert_eq!(fx.balance(fx.main()), 2000);
}

#[test]
fn a_correction_that_would_overdraw_the_account_is_rejected() {
    let mut fx = Fixture::new();
    let original = Fixture::only_transaction(&fx.deposit(None, kr(20)).expect("deposit"));
    fx.run(withdraw(fx.main(), kr(15))).expect("already spent");
    assert_eq!(
        fx.run(correct(original.id())),
        Err(WalletError::InsufficientFunds)
    );
}

#[test]
fn correcting_a_withdrawal_gives_the_money_back() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(20)).expect("deposit");
    let spent = Fixture::only_transaction(&fx.run(withdraw(fx.main(), kr(15))).expect("withdraw"));
    let correction = Fixture::only_correction(&fx.run(correct(spent.id())).expect("correct"));
    assert_eq!(correction.direction(), Direction::In);
    assert_eq!(fx.balance(fx.main()), 2000);
}

#[test]
fn correcting_one_side_of_a_transfer_corrects_both() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.deposit(None, kr(50)).expect("deposit");
    let in_leg = match fx
        .run(transfer(fx.main(), lego, kr(30)))
        .expect("transfer")
        .as_slice()
    {
        [WalletEvent::TransferRecorded { in_leg, .. }] => in_leg.clone(),
        other => panic!("unexpected events {other:?}"),
    };

    let events = fx.run(correct(in_leg.id())).expect("correct");

    let (correction, paired) = match events.as_slice() {
        [WalletEvent::CorrectionRecorded {
            correction,
            paired: Some(paired),
        }] => (correction.clone(), paired.clone()),
        other => panic!("unexpected events {other:?}"),
    };
    assert_eq!(correction.corrects(), Some(in_leg.id()));
    assert_eq!(correction.account_id(), lego);
    assert_eq!(paired.account_id(), fx.main());
    assert!(correction.transfer_id().is_some());
    assert_eq!(correction.transfer_id(), paired.transfer_id());
    assert_ne!(correction.transfer_id(), in_leg.transfer_id());
    assert_eq!(fx.balance(fx.main()), 5000);
    assert_eq!(fx.balance(lego), 0);
    assert_eq!(fx.total(), 5000);

    let out_leg_id = paired
        .corrects()
        .expect("paired correction points at the out leg");
    assert_eq!(
        fx.run(correct(out_leg_id)),
        Err(WalletError::AlreadyCorrected)
    );
}

#[test]
fn correcting_an_unknown_transaction_is_rejected() {
    let mut fx = Fixture::new();
    let stranger = TransactionId::from_uuid(Uuid::from_u128(77));
    assert_eq!(
        fx.run(correct(stranger)),
        Err(WalletError::UnknownTransaction)
    );
}

// ---------- Roles ----------

#[test]
fn a_viewer_can_change_nothing() {
    let mut fx = Fixture::new();
    fx.deposit(None, kr(10)).expect("deposit");
    let lego = fx.open_account("Lego set");
    let commands = [
        deposit(None, kr(1)),
        withdraw(fx.main(), kr(1)),
        transfer(fx.main(), lego, kr(1)),
        WalletCommand::OpenAccount {
            name: name("Sneaky"),
            savings_goal: None,
        },
        WalletCommand::SetSavingsGoal {
            account_id: lego,
            savings_goal: None,
        },
        close(lego, None),
    ];
    for command in commands {
        assert_eq!(
            fx.run_as(Role::Viewer, command),
            Err(WalletError::NotAllowed)
        );
    }
    assert_eq!(fx.total(), 1000);
}

#[test]
fn owners_and_parents_record_money() {
    let mut fx = Fixture::new();
    fx.run_as(Role::Owner, deposit(None, kr(1))).expect("owner");
    fx.run_as(Role::Parent, deposit(None, kr(1)))
        .expect("parent");
    assert_eq!(fx.total(), 200);
}

// ---------- Replay ----------

#[test]
fn replaying_the_history_rebuilds_the_same_wallet() {
    let mut fx = Fixture::new();
    let lego = fx.open_account("Lego set");
    fx.deposit(None, kr(50)).expect("deposit");
    fx.run(transfer(fx.main(), lego, kr(20))).expect("transfer");
    fx.run(withdraw(fx.main(), kr(5))).expect("withdraw");

    let replayed = Wallet::replay(&fx.history).expect("replay");
    assert_eq!(replayed, fx.wallet);
}

#[test]
fn a_history_must_start_with_the_wallet_being_opened() {
    let mut fx = Fixture::new();
    let events = fx.deposit(None, kr(1)).expect("deposit");
    assert_eq!(Wallet::replay(&events), Err(ReplayError::NotOpened));
    assert_eq!(Wallet::replay(&[]), Err(ReplayError::NotOpened));
}

#[test]
fn a_wallet_is_opened_only_once() {
    let fx = Fixture::new();
    let mut twice = fx.history.clone();
    twice.push(fx.history[0].clone());
    assert_eq!(Wallet::replay(&twice), Err(ReplayError::AlreadyOpened));
}

impl Fixture {
    fn only_correction(events: &[WalletEvent]) -> Transaction {
        match events {
            [WalletEvent::CorrectionRecorded {
                correction,
                paired: None,
            }] => correction.clone(),
            other => panic!("unexpected events {other:?}"),
        }
    }
}

// ---------- Restoring stored Transactions ----------

#[test]
fn a_stored_transaction_is_restored_exactly() {
    let mut fx = Fixture::new();
    let events = fx.deposit(None, kr(20)).expect("deposit");
    let WalletEvent::TransactionRecorded { transaction } = &events[0] else {
        panic!("unexpected events {events:?}");
    };

    let restored = Transaction::restore(TransactionParts {
        id: transaction.id(),
        account_id: transaction.account_id(),
        direction: transaction.direction(),
        amount: transaction.amount(),
        category: transaction.category(),
        memo: transaction.memo().clone(),
        occurred_on: transaction.occurred_on(),
        recorded_at: transaction.recorded_at(),
        actor: transaction.actor(),
        transfer_id: transaction.transfer_id(),
        corrects: transaction.corrects(),
    });

    assert_eq!(&restored, transaction);
    assert_eq!(restored.parts(), transaction.parts());
}
