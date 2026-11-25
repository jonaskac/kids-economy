//! The v1 stored format: every Event round-trips, and the JSON shape is pinned.

use super::*;
use serde_json::json;
use time::macros::{date, datetime};

fn sek() -> Currency {
    Currency::new("SEK").expect("currency")
}

fn kr(whole: i64) -> Amount {
    Amount::new(whole * 100, sek()).expect("amount")
}

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn leg(n: u128, direction: Direction, category: Category) -> Transaction {
    Transaction::restore(TransactionParts {
        id: TransactionId::from_uuid(id(n)),
        account_id: AccountId::from_uuid(id(2)),
        direction,
        amount: kr(20),
        category,
        memo: Memo::new("Pocket money week 48").expect("memo"),
        occurred_on: date!(2026 - 11 - 25),
        recorded_at: datetime!(2026-11-25 13:00 UTC),
        actor: MemberId::from_uuid(id(1_001)),
        transfer_id: (category == Category::Transfer).then(|| TransferId::from_uuid(id(50))),
        corrects: (category == Category::Correction).then(|| TransactionId::from_uuid(id(10))),
    })
}

fn goal() -> SavingsGoal {
    SavingsGoal {
        name: AccountName::new("Lego castle").expect("name"),
        target: kr(400),
        target_date: Some(date!(2027 - 06 - 01)),
    }
}

fn every_event() -> Vec<WalletEvent> {
    vec![
        WalletEvent::WalletOpened {
            wallet_id: WalletId::from_uuid(id(900)),
            family_id: FamilyId::from_uuid(id(901)),
            kid_id: KidId::from_uuid(id(902)),
            currency: sek(),
            main_account_id: AccountId::from_uuid(id(1)),
            main_account_name: AccountName::new("Main").expect("name"),
            opened_by: MemberId::from_uuid(id(1_001)),
            opened_at: datetime!(2026-11-25 13:00 UTC),
        },
        WalletEvent::AccountOpened {
            account_id: AccountId::from_uuid(id(2)),
            name: AccountName::new("Lego set").expect("name"),
            savings_goal: Some(goal()),
        },
        WalletEvent::AccountOpened {
            account_id: AccountId::from_uuid(id(3)),
            name: AccountName::new("Fun stuff").expect("name"),
            savings_goal: None,
        },
        WalletEvent::SavingsGoalSet {
            account_id: AccountId::from_uuid(id(2)),
            savings_goal: None,
        },
        WalletEvent::SavingsGoalReached {
            account_id: AccountId::from_uuid(id(2)),
        },
        WalletEvent::AccountClosed {
            account_id: AccountId::from_uuid(id(3)),
            closed_at: datetime!(2026-11-26 08:30 +01:00),
        },
        WalletEvent::TransactionRecorded {
            transaction: leg(10, Direction::In, Category::Allowance),
        },
        WalletEvent::TransferRecorded {
            out_leg: leg(11, Direction::Out, Category::Transfer),
            in_leg: leg(12, Direction::In, Category::Transfer),
        },
        WalletEvent::CorrectionRecorded {
            correction: leg(13, Direction::Out, Category::Correction),
            paired: None,
        },
        WalletEvent::CorrectionRecorded {
            correction: leg(14, Direction::Out, Category::Correction),
            paired: Some(leg(15, Direction::In, Category::Correction)),
        },
    ]
}

#[test]
fn every_event_reads_back_as_it_was_written() {
    for event in every_event() {
        let stored = encode(&event).expect("encode");
        let back = decode(&stored.kind, stored.body.clone()).expect("decode");
        assert_eq!(back, event, "kind {}", stored.kind);
    }
}

#[test]
fn every_kind_has_its_own_name() {
    let kinds: Vec<String> = every_event()
        .iter()
        .map(|e| encode(e).expect("encode").kind)
        .collect();
    assert_eq!(
        kinds,
        [
            "wallet_opened",
            "account_opened",
            "account_opened",
            "savings_goal_set",
            "savings_goal_reached",
            "account_closed",
            "transaction_recorded",
            "transfer_recorded",
            "correction_recorded",
            "correction_recorded",
        ]
    );
}

#[test]
fn the_v1_format_of_a_deposit_is_pinned() {
    let event = WalletEvent::TransactionRecorded {
        transaction: leg(10, Direction::In, Category::Allowance),
    };
    let stored = encode(&event).expect("encode");
    assert_eq!(stored.kind, "transaction_recorded");
    assert_eq!(
        stored.body,
        json!({
            "v": 1,
            "transaction": {
                "id": "00000000-0000-0000-0000-00000000000a",
                "account_id": "00000000-0000-0000-0000-000000000002",
                "direction": "in",
                "amount": { "minor": 2000, "currency": "SEK" },
                "category": "allowance",
                "memo": "Pocket money week 48",
                "occurred_on": "2026-11-25",
                "recorded_at": "2026-11-25T13:00:00Z",
                "actor": "00000000-0000-0000-0000-0000000003e9",
                "transfer_id": null,
                "corrects": null
            }
        })
    );
}

#[test]
fn the_v1_format_of_a_savings_goal_is_pinned() {
    let event = WalletEvent::SavingsGoalSet {
        account_id: AccountId::from_uuid(id(2)),
        savings_goal: Some(goal()),
    };
    let stored = encode(&event).expect("encode");
    assert_eq!(
        stored.body,
        json!({
            "v": 1,
            "account_id": "00000000-0000-0000-0000-000000000002",
            "savings_goal": {
                "name": "Lego castle",
                "target": { "minor": 40000, "currency": "SEK" },
                "target_date": "2027-06-01"
            }
        })
    );
}

#[test]
fn a_body_without_a_known_version_is_refused() {
    let reached = json!({ "account_id": "00000000-0000-0000-0000-000000000002" });
    assert!(matches!(
        decode("savings_goal_reached", reached),
        Err(BodyError::UnknownVersion(None))
    ));
    let future = json!({ "v": 2, "account_id": "00000000-0000-0000-0000-000000000002" });
    assert!(matches!(
        decode("savings_goal_reached", future),
        Err(BodyError::UnknownVersion(Some(2)))
    ));
}

#[test]
fn an_unknown_kind_or_field_is_refused() {
    let body = json!({ "v": 1, "account_id": "00000000-0000-0000-0000-000000000002" });
    assert!(matches!(
        decode("account_deleted", body),
        Err(BodyError::Json(_))
    ));
    let extra = json!({
        "v": 1,
        "account_id": "00000000-0000-0000-0000-000000000002",
        "balance": 100
    });
    assert!(matches!(
        decode("savings_goal_reached", extra),
        Err(BodyError::Json(_))
    ));
}

#[test]
fn stored_money_and_text_are_checked_by_the_domain_rules() {
    let deposit = |minor: i64, memo: &str| {
        let mut body = encode(&WalletEvent::TransactionRecorded {
            transaction: leg(10, Direction::In, Category::Allowance),
        })
        .expect("encode")
        .body;
        body["transaction"]["amount"]["minor"] = json!(minor);
        body["transaction"]["memo"] = json!(memo);
        decode("transaction_recorded", body)
    };
    assert!(matches!(deposit(0, "ok"), Err(BodyError::Money(_))));
    assert!(matches!(deposit(100, "   "), Err(BodyError::Text(_))));
    assert!(deposit(100, "ok").is_ok());
}
