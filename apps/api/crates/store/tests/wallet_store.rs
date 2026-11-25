//! The Wallet store against real Postgres (ADR-0009). Each test gets a fresh database
//! with the migrations applied; set `DATABASE_URL` to a server you may create databases on.

use domain::ids::{AccountId, FamilyId, KidId, MemberId, NewIds, WalletId};
use domain::money::{Amount, Currency};
use domain::role::{Actor, Role};
use domain::text::{AccountName, Memo};
use domain::wallet::{
    Context, DepositCategory, OpenWallet, SavingsGoal, Wallet, WalletCommand, WalletEvent,
    WithdrawalCategory,
};
use sqlx::PgPool;
use store::{StoreError, Version, WalletStore};
use time::macros::{date, datetime};
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

fn memo(text: &str) -> Memo {
    Memo::new(text).expect("memo")
}

fn context() -> Context {
    Context {
        actor: Actor {
            member_id: MemberId::from_uuid(Uuid::from_u128(1_001)),
            role: Role::Parent,
        },
        now: datetime!(2026-11-25 13:00 UTC),
    }
}

fn family() -> FamilyId {
    FamilyId::from_uuid(Uuid::from_u128(901))
}

fn opened(wallet: u128, ids: &mut SequentialIds) -> WalletEvent {
    Wallet::open(
        &OpenWallet {
            wallet_id: WalletId::from_uuid(Uuid::from_u128(wallet)),
            family_id: family(),
            kid_id: KidId::from_uuid(Uuid::from_u128(wallet + 1)),
            currency: sek(),
            main_account_name: AccountName::new("Main").expect("name"),
        },
        &context(),
        ids,
    )
    .expect("open wallet")
}

fn deposit(amount: Amount) -> WalletCommand {
    WalletCommand::Deposit {
        account_id: None,
        amount,
        category: DepositCategory::Allowance,
        memo: memo("Pocket money week 48"),
        occurred_on: date!(2026 - 11 - 25),
    }
}

fn withdraw(account_id: AccountId, amount: Amount) -> WalletCommand {
    WalletCommand::Withdraw {
        account_id,
        amount,
        category: WithdrawalCategory::Purchase,
        memo: memo("LEGO"),
        occurred_on: date!(2026 - 11 - 25),
    }
}

/// Loads, decides and saves one Command, like a request handler will.
async fn record(
    store: &WalletStore,
    wallet_id: WalletId,
    command: WalletCommand,
    ids: &mut SequentialIds,
) -> Result<Version, StoreError> {
    let loaded = store
        .load(family(), wallet_id)
        .await?
        .expect("wallet exists");
    let events = loaded
        .wallet
        .decide(&command, &context(), ids)
        .expect("decide");
    store.append(&loaded.wallet, loaded.version, &events).await
}

fn wallet_id(n: u128) -> WalletId {
    WalletId::from_uuid(Uuid::from_u128(n))
}

// ---------- behaviour ----------

#[sqlx::test]
async fn an_opened_wallet_is_found_again_with_version_one(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);

    let version = store.open(&opened(500, &mut ids)).await.expect("open");

    assert_eq!(version, Version::from_count(1));
    let loaded = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    assert_eq!(loaded.version, version);
    assert_eq!(loaded.wallet.total().minor(), 0);
}

#[sqlx::test]
async fn an_unknown_wallet_is_not_found(pool: PgPool) {
    let store = WalletStore::new(pool);
    let loaded = store.load(family(), wallet_id(404)).await.expect("load");
    assert!(loaded.is_none());
}

#[sqlx::test]
async fn a_wallet_is_never_visible_from_another_family(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");

    let other = FamilyId::from_uuid(Uuid::from_u128(77));
    assert!(store
        .load(other, wallet_id(500))
        .await
        .expect("load")
        .is_none());
}

#[sqlx::test]
async fn a_wallet_is_opened_only_once(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    let event = opened(500, &mut ids);
    store.open(&event).await.expect("first open");

    assert!(matches!(
        store.open(&event).await,
        Err(StoreError::Conflict)
    ));
}

#[sqlx::test]
async fn only_a_wallet_opened_event_opens_a_wallet(pool: PgPool) {
    let store = WalletStore::new(pool);
    let not_opening = WalletEvent::SavingsGoalReached {
        account_id: AccountId::from_uuid(Uuid::from_u128(1)),
    };
    assert!(matches!(
        store.open(&not_opening).await,
        Err(StoreError::InvalidHistory(_))
    ));
}

#[sqlx::test]
async fn every_kind_of_event_survives_the_round_trip(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");
    let main = {
        let loaded = store.load(family(), wallet_id(500)).await.expect("load");
        loaded.expect("found").wallet.main_account_id()
    };

    record(&store, wallet_id(500), deposit(kr(100)), &mut ids)
        .await
        .expect("deposit");
    record(
        &store,
        wallet_id(500),
        WalletCommand::OpenAccount {
            name: AccountName::new("Lego set").expect("name"),
            savings_goal: Some(SavingsGoal {
                name: AccountName::new("Lego castle").expect("name"),
                target: kr(30),
                target_date: Some(date!(2027 - 06 - 01)),
            }),
        },
        &mut ids,
    )
    .await
    .expect("open account");
    let lego = {
        let loaded = store.load(family(), wallet_id(500)).await.expect("load");
        let wallet = loaded.expect("found").wallet;
        let id = wallet.accounts().nth(1).expect("lego").id();
        id
    };
    // Reaches the goal: TransferRecorded + SavingsGoalReached.
    record(
        &store,
        wallet_id(500),
        WalletCommand::Transfer {
            from: main,
            to: lego,
            amount: kr(30),
            memo: memo("Saving up"),
            occurred_on: date!(2026 - 11 - 25),
        },
        &mut ids,
    )
    .await
    .expect("transfer");
    let transfer_leg = {
        let loaded = store.load(family(), wallet_id(500)).await.expect("load");
        let wallet = loaded.expect("found").wallet;
        let id = wallet.account(lego).expect("lego").ledger()[0].id();
        id
    };
    record(
        &store,
        wallet_id(500),
        WalletCommand::Correct {
            transaction_id: transfer_leg,
            reason: memo("Wrong account"),
            occurred_on: date!(2026 - 11 - 26),
        },
        &mut ids,
    )
    .await
    .expect("correct");
    record(
        &store,
        wallet_id(500),
        WalletCommand::SetSavingsGoal {
            account_id: lego,
            savings_goal: None,
        },
        &mut ids,
    )
    .await
    .expect("remove goal");
    record(
        &store,
        wallet_id(500),
        WalletCommand::CloseAccount {
            account_id: lego,
            move_to: None,
            memo: Memo::new("Lego closed").expect("memo"),
            occurred_on: date!(2026 - 11 - 25),
        },
        &mut ids,
    )
    .await
    .expect("close");
    let version = record(&store, wallet_id(500), withdraw(main, kr(40)), &mut ids)
        .await
        .expect("withdraw");

    let history = store
        .history(family(), wallet_id(500))
        .await
        .expect("history");
    assert_eq!(Version::from_count(history.len()), version);
    let replayed = Wallet::replay(&history).expect("replay");
    let loaded = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    assert_eq!(loaded.wallet, replayed);
    assert_eq!(loaded.wallet.total().minor(), 60 * 100);
    assert!(loaded.wallet.account(lego).expect("lego").is_closed());
}

#[sqlx::test]
async fn events_are_numbered_per_wallet_without_gaps(pool: PgPool) {
    let store = WalletStore::new(pool.clone());
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open a");
    store.open(&opened(600, &mut ids)).await.expect("open b");
    record(&store, wallet_id(500), deposit(kr(1)), &mut ids)
        .await
        .expect("a");
    record(&store, wallet_id(600), deposit(kr(1)), &mut ids)
        .await
        .expect("b");
    record(&store, wallet_id(500), deposit(kr(1)), &mut ids)
        .await
        .expect("a again");

    let rows: Vec<(Uuid, i64)> = sqlx::query_as(
        "select wallet_id, sequence from ledger_events order by wallet_id, sequence",
    )
    .fetch_all(&pool)
    .await
    .expect("rows");
    let a = Uuid::from_u128(500);
    let b = Uuid::from_u128(600);
    assert_eq!(rows, vec![(a, 1), (a, 2), (a, 3), (b, 1), (b, 2)]);
}

#[sqlx::test]
async fn saving_against_an_old_version_is_a_conflict(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");
    let stale = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    record(&store, wallet_id(500), deposit(kr(5)), &mut ids)
        .await
        .expect("first");

    let events = stale
        .wallet
        .decide(&deposit(kr(5)), &context(), &mut ids)
        .expect("decide");
    let result = store.append(&stale.wallet, stale.version, &events).await;

    assert!(matches!(result, Err(StoreError::Conflict)));
    let now = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    assert_eq!(now.version, Version::from_count(2));
}

#[sqlx::test]
async fn saving_against_a_version_ahead_of_the_wallet_is_a_conflict(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");
    let loaded = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    let events = loaded
        .wallet
        .decide(&deposit(kr(5)), &context(), &mut ids)
        .expect("decide");

    let result = store
        .append(&loaded.wallet, Version::from_count(3), &events)
        .await;
    assert!(matches!(result, Err(StoreError::Conflict)));
}

#[sqlx::test]
async fn two_members_withdrawing_at_once_cannot_overdraw(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");
    record(&store, wallet_id(500), deposit(kr(100)), &mut ids)
        .await
        .expect("deposit");
    let seen = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    let main = seen.wallet.main_account_id();
    // Both Members see 100 kr and each decides to take out 80 kr.
    let first = seen
        .wallet
        .decide(&withdraw(main, kr(80)), &context(), &mut ids)
        .expect("first decides");
    let second = seen
        .wallet
        .decide(&withdraw(main, kr(80)), &context(), &mut ids)
        .expect("second decides");

    let (a, b) = tokio::join!(
        store.append(&seen.wallet, seen.version, &first),
        store.append(&seen.wallet, seen.version, &second),
    );

    let saved = [&a, &b].iter().filter(|r| r.is_ok()).count();
    let conflicts = [&a, &b]
        .iter()
        .filter(|r| matches!(r, Err(StoreError::Conflict)))
        .count();
    assert_eq!((saved, conflicts), (1, 1));
    let after = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");
    assert_eq!(after.wallet.total().minor(), 20 * 100);
}

#[sqlx::test]
async fn events_that_do_not_fit_the_wallet_are_never_saved(pool: PgPool) {
    let store = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    let event = opened(500, &mut ids);
    store.open(&event).await.expect("open");
    let loaded = store
        .load(family(), wallet_id(500))
        .await
        .expect("load")
        .expect("found");

    let result = store.append(&loaded.wallet, loaded.version, &[event]).await;

    assert!(matches!(result, Err(StoreError::InvalidHistory(_))));
    let history = store
        .history(family(), wallet_id(500))
        .await
        .expect("history");
    assert_eq!(history.len(), 1);
}

#[sqlx::test]
async fn a_stored_body_in_an_unknown_format_is_reported_not_guessed(pool: PgPool) {
    let store = WalletStore::new(pool.clone());
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");
    sqlx::query(
        "insert into ledger_events (family_id, wallet_id, sequence, kind, body)
         values ($1, $2, 2, 'savings_goal_reached', '{\"v\": 99}')",
    )
    .bind(family().as_uuid())
    .bind(wallet_id(500).as_uuid())
    .execute(&pool)
    .await
    .expect("insert");

    let result = store.load(family(), wallet_id(500)).await;
    assert!(matches!(result, Err(StoreError::Body { sequence: 2, .. })));
}

#[sqlx::test]
async fn the_database_refuses_to_edit_or_delete_history(pool: PgPool) {
    let store = WalletStore::new(pool.clone());
    let mut ids = SequentialIds(0);
    store.open(&opened(500, &mut ids)).await.expect("open");

    let update = sqlx::query("update ledger_events set kind = 'x'")
        .execute(&pool)
        .await;
    let delete = sqlx::query("delete from ledger_events")
        .execute(&pool)
        .await;

    assert!(update.is_err(), "update must be refused");
    assert!(delete.is_err(), "delete must be refused");
    let history = store
        .history(family(), wallet_id(500))
        .await
        .expect("history");
    assert_eq!(history.len(), 1);
}
