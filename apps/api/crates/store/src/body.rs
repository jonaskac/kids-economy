//! The stored format of Wallet Events: `kind` column plus a JSONB body with `"v": 1`.
//!
//! Owned by the store, not the domain (ADR-0009): renaming a Rust field never changes
//! stored data. Text and Amounts are validated again on read with the domain's own
//! constructors. When the format changes, bump `v` and upcast old bodies in [`decode`].

use domain::ids::{AccountId, FamilyId, KidId, MemberId, TransactionId, TransferId, WalletId};
use domain::money::{Amount, Currency, MoneyError};
use domain::text::{AccountName, Memo, TextError};
use domain::wallet::{
    Category, Direction, SavingsGoal, Transaction, TransactionParts, WalletEvent,
};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use thiserror::Error;
use time::format_description::well_known::{Iso8601, Rfc3339};
use time::{Date, OffsetDateTime};
use uuid::Uuid;

/// The body format this build writes.
pub const CURRENT_VERSION: u64 = 1;

/// Why an Event cannot be turned into a stored body, or back.
#[derive(Debug, Error)]
pub enum BodyError {
    #[error("the body is not a JSON object")]
    NotAnObject,
    #[error("unknown body format version {0:?}")]
    UnknownVersion(Option<u64>),
    #[error("the body does not match its kind: {0}")]
    Json(#[from] serde_json::Error),
    #[error("invalid money: {0}")]
    Money(#[from] MoneyError),
    #[error("invalid text: {0}")]
    Text(#[from] TextError),
    #[error("invalid date or time: {0}")]
    TimeParse(#[from] time::error::Parse),
    #[error("a date or time cannot be written: {0}")]
    TimeFormat(#[from] time::error::Format),
}

/// One Event ready for the `kind` and `body` columns.
#[derive(Debug, Clone, PartialEq)]
pub struct StoredBody {
    pub kind: String,
    pub body: Value,
}

pub fn encode(event: &WalletEvent) -> Result<StoredBody, BodyError> {
    let value = serde_json::to_value(EventV1::try_from(event)?)?;
    let Value::Object(mut body) = value else {
        return Err(BodyError::NotAnObject);
    };
    let kind = match body.remove("kind") {
        Some(Value::String(kind)) => kind,
        _ => return Err(BodyError::NotAnObject),
    };
    body.insert("v".to_owned(), Value::from(CURRENT_VERSION));
    Ok(StoredBody {
        kind,
        body: Value::Object(body),
    })
}

pub fn decode(kind: &str, body: Value) -> Result<WalletEvent, BodyError> {
    let Value::Object(mut body) = body else {
        return Err(BodyError::NotAnObject);
    };
    match body.remove("v").as_ref().and_then(Value::as_u64) {
        Some(CURRENT_VERSION) => {}
        other => return Err(BodyError::UnknownVersion(other)),
    }
    let mut tagged = Map::with_capacity(body.len() + 1);
    tagged.insert("kind".to_owned(), Value::from(kind));
    tagged.extend(body);
    let event: EventV1 = serde_json::from_value(Value::Object(tagged))?;
    event.try_into()
}

// ---------- v1 shapes ----------

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum EventV1 {
    WalletOpened {
        wallet_id: Uuid,
        family_id: Uuid,
        kid_id: Uuid,
        currency: String,
        main_account_id: Uuid,
        main_account_name: String,
        opened_by: Uuid,
        opened_at: String,
    },
    AccountOpened {
        account_id: Uuid,
        name: String,
        savings_goal: Option<SavingsGoalV1>,
    },
    SavingsGoalSet {
        account_id: Uuid,
        savings_goal: Option<SavingsGoalV1>,
    },
    SavingsGoalReached {
        account_id: Uuid,
    },
    AccountClosed {
        account_id: Uuid,
        closed_at: String,
    },
    TransactionRecorded {
        transaction: TransactionV1,
    },
    TransferRecorded {
        out_leg: TransactionV1,
        in_leg: TransactionV1,
    },
    CorrectionRecorded {
        correction: TransactionV1,
        paired: Option<TransactionV1>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct AmountV1 {
    minor: i64,
    currency: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct SavingsGoalV1 {
    name: String,
    target: AmountV1,
    target_date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum DirectionV1 {
    In,
    Out,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum CategoryV1 {
    Allowance,
    Chore,
    Gift,
    Interest,
    Purchase,
    Donation,
    Transfer,
    Correction,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct TransactionV1 {
    id: Uuid,
    account_id: Uuid,
    direction: DirectionV1,
    amount: AmountV1,
    category: CategoryV1,
    memo: String,
    occurred_on: String,
    recorded_at: String,
    actor: Uuid,
    transfer_id: Option<Uuid>,
    corrects: Option<Uuid>,
}

// ---------- domain -> v1 ----------

fn date_out(date: Date) -> Result<String, BodyError> {
    Ok(date.format(&Iso8601::DATE)?)
}

pub(crate) fn time_out(at: OffsetDateTime) -> Result<String, BodyError> {
    Ok(at.format(&Rfc3339)?)
}

fn amount_out(amount: Amount) -> AmountV1 {
    AmountV1 {
        minor: amount.minor(),
        currency: amount.currency().code().to_owned(),
    }
}

fn goal_out(goal: Option<&SavingsGoal>) -> Result<Option<SavingsGoalV1>, BodyError> {
    goal.map(|goal| {
        Ok(SavingsGoalV1 {
            name: goal.name.as_str().to_owned(),
            target: amount_out(goal.target),
            target_date: goal.target_date.map(date_out).transpose()?,
        })
    })
    .transpose()
}

impl TryFrom<&Transaction> for TransactionV1 {
    type Error = BodyError;

    fn try_from(transaction: &Transaction) -> Result<Self, BodyError> {
        let parts = transaction.parts();
        Ok(Self {
            id: parts.id.as_uuid(),
            account_id: parts.account_id.as_uuid(),
            direction: match parts.direction {
                Direction::In => DirectionV1::In,
                Direction::Out => DirectionV1::Out,
            },
            amount: amount_out(parts.amount),
            category: match parts.category {
                Category::Allowance => CategoryV1::Allowance,
                Category::Chore => CategoryV1::Chore,
                Category::Gift => CategoryV1::Gift,
                Category::Interest => CategoryV1::Interest,
                Category::Purchase => CategoryV1::Purchase,
                Category::Donation => CategoryV1::Donation,
                Category::Transfer => CategoryV1::Transfer,
                Category::Correction => CategoryV1::Correction,
            },
            memo: parts.memo.as_str().to_owned(),
            occurred_on: date_out(parts.occurred_on)?,
            recorded_at: time_out(parts.recorded_at)?,
            actor: parts.actor.as_uuid(),
            transfer_id: parts.transfer_id.map(|id| id.as_uuid()),
            corrects: parts.corrects.map(|id| id.as_uuid()),
        })
    }
}

impl TryFrom<&WalletEvent> for EventV1 {
    type Error = BodyError;

    fn try_from(event: &WalletEvent) -> Result<Self, BodyError> {
        Ok(match event {
            WalletEvent::WalletOpened {
                wallet_id,
                family_id,
                kid_id,
                currency,
                main_account_id,
                main_account_name,
                opened_by,
                opened_at,
            } => EventV1::WalletOpened {
                wallet_id: wallet_id.as_uuid(),
                family_id: family_id.as_uuid(),
                kid_id: kid_id.as_uuid(),
                currency: currency.code().to_owned(),
                main_account_id: main_account_id.as_uuid(),
                main_account_name: main_account_name.as_str().to_owned(),
                opened_by: opened_by.as_uuid(),
                opened_at: time_out(*opened_at)?,
            },
            WalletEvent::AccountOpened {
                account_id,
                name,
                savings_goal,
            } => EventV1::AccountOpened {
                account_id: account_id.as_uuid(),
                name: name.as_str().to_owned(),
                savings_goal: goal_out(savings_goal.as_ref())?,
            },
            WalletEvent::SavingsGoalSet {
                account_id,
                savings_goal,
            } => EventV1::SavingsGoalSet {
                account_id: account_id.as_uuid(),
                savings_goal: goal_out(savings_goal.as_ref())?,
            },
            WalletEvent::SavingsGoalReached { account_id } => EventV1::SavingsGoalReached {
                account_id: account_id.as_uuid(),
            },
            WalletEvent::AccountClosed {
                account_id,
                closed_at,
            } => EventV1::AccountClosed {
                account_id: account_id.as_uuid(),
                closed_at: time_out(*closed_at)?,
            },
            WalletEvent::TransactionRecorded { transaction } => EventV1::TransactionRecorded {
                transaction: transaction.try_into()?,
            },
            WalletEvent::TransferRecorded { out_leg, in_leg } => EventV1::TransferRecorded {
                out_leg: out_leg.try_into()?,
                in_leg: in_leg.try_into()?,
            },
            WalletEvent::CorrectionRecorded { correction, paired } => EventV1::CorrectionRecorded {
                correction: correction.try_into()?,
                paired: paired.as_ref().map(TryInto::try_into).transpose()?,
            },
        })
    }
}

// ---------- v1 -> domain ----------

fn date_in(text: &str) -> Result<Date, BodyError> {
    Ok(Date::parse(text, &Iso8601::DATE)?)
}

pub(crate) fn time_in(text: &str) -> Result<OffsetDateTime, BodyError> {
    Ok(OffsetDateTime::parse(text, &Rfc3339)?)
}

fn amount_in(amount: AmountV1) -> Result<Amount, BodyError> {
    Ok(Amount::new(amount.minor, Currency::new(&amount.currency)?)?)
}

fn goal_in(goal: Option<SavingsGoalV1>) -> Result<Option<SavingsGoal>, BodyError> {
    goal.map(|goal| {
        Ok(SavingsGoal {
            name: AccountName::new(goal.name)?,
            target: amount_in(goal.target)?,
            target_date: goal.target_date.as_deref().map(date_in).transpose()?,
        })
    })
    .transpose()
}

impl TryFrom<TransactionV1> for Transaction {
    type Error = BodyError;

    fn try_from(stored: TransactionV1) -> Result<Self, BodyError> {
        Ok(Transaction::restore(TransactionParts {
            id: TransactionId::from_uuid(stored.id),
            account_id: AccountId::from_uuid(stored.account_id),
            direction: match stored.direction {
                DirectionV1::In => Direction::In,
                DirectionV1::Out => Direction::Out,
            },
            amount: amount_in(stored.amount)?,
            category: match stored.category {
                CategoryV1::Allowance => Category::Allowance,
                CategoryV1::Chore => Category::Chore,
                CategoryV1::Gift => Category::Gift,
                CategoryV1::Interest => Category::Interest,
                CategoryV1::Purchase => Category::Purchase,
                CategoryV1::Donation => Category::Donation,
                CategoryV1::Transfer => Category::Transfer,
                CategoryV1::Correction => Category::Correction,
            },
            memo: Memo::new(stored.memo)?,
            occurred_on: date_in(&stored.occurred_on)?,
            recorded_at: time_in(&stored.recorded_at)?,
            actor: MemberId::from_uuid(stored.actor),
            transfer_id: stored.transfer_id.map(TransferId::from_uuid),
            corrects: stored.corrects.map(TransactionId::from_uuid),
        }))
    }
}

impl TryFrom<EventV1> for WalletEvent {
    type Error = BodyError;

    fn try_from(stored: EventV1) -> Result<Self, BodyError> {
        Ok(match stored {
            EventV1::WalletOpened {
                wallet_id,
                family_id,
                kid_id,
                currency,
                main_account_id,
                main_account_name,
                opened_by,
                opened_at,
            } => WalletEvent::WalletOpened {
                wallet_id: WalletId::from_uuid(wallet_id),
                family_id: FamilyId::from_uuid(family_id),
                kid_id: KidId::from_uuid(kid_id),
                currency: Currency::new(&currency)?,
                main_account_id: AccountId::from_uuid(main_account_id),
                main_account_name: AccountName::new(main_account_name)?,
                opened_by: MemberId::from_uuid(opened_by),
                opened_at: time_in(&opened_at)?,
            },
            EventV1::AccountOpened {
                account_id,
                name,
                savings_goal,
            } => WalletEvent::AccountOpened {
                account_id: AccountId::from_uuid(account_id),
                name: AccountName::new(name)?,
                savings_goal: goal_in(savings_goal)?,
            },
            EventV1::SavingsGoalSet {
                account_id,
                savings_goal,
            } => WalletEvent::SavingsGoalSet {
                account_id: AccountId::from_uuid(account_id),
                savings_goal: goal_in(savings_goal)?,
            },
            EventV1::SavingsGoalReached { account_id } => WalletEvent::SavingsGoalReached {
                account_id: AccountId::from_uuid(account_id),
            },
            EventV1::AccountClosed {
                account_id,
                closed_at,
            } => WalletEvent::AccountClosed {
                account_id: AccountId::from_uuid(account_id),
                closed_at: time_in(&closed_at)?,
            },
            EventV1::TransactionRecorded { transaction } => WalletEvent::TransactionRecorded {
                transaction: transaction.try_into()?,
            },
            EventV1::TransferRecorded { out_leg, in_leg } => WalletEvent::TransferRecorded {
                out_leg: out_leg.try_into()?,
                in_leg: in_leg.try_into()?,
            },
            EventV1::CorrectionRecorded { correction, paired } => WalletEvent::CorrectionRecorded {
                correction: correction.try_into()?,
                paired: paired.map(TryInto::try_into).transpose()?,
            },
        })
    }
}

#[cfg(test)]
#[path = "body_tests.rs"]
mod tests;
