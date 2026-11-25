//! A Kid's Wallet: its Accounts, Balances and Transactions, and the Deposits,
//! Withdrawals and Transfers Members record on it (ADR-0008, ADR-0009).
//!
//! Every route sits under `/api/families/{family_id}/wallets/{wallet_id}`, so the Family's
//! tenant boundary (404 to non-Members) applies first. Each change answers with the whole
//! Wallet as the Member now sees it.

use crate::error::ApiError;
use crate::families::{load_as_member, MoneyDto};
use crate::{AppState, RandomIds};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use domain::family::Family;
use domain::ids::{AccountId, WalletId};
use domain::money::{Amount, Balance, Currency};
use domain::role::Context;
use domain::text::{AccountName, Memo};
use domain::wallet::{
    Category, DepositCategory, Direction, SavingsGoal, Transaction, Wallet, WalletCommand,
    WithdrawalCategory,
};
use serde::{Deserialize, Serialize};
use store::{LoadedWallet, StoreError};
use time::{Date, Month, OffsetDateTime};
use uuid::Uuid;

/// How often a decision is retried when another Member saved first.
const ATTEMPTS: usize = 3;

// ---------- DTOs ----------

/// An Amount as sent by the app: `{"minor": 1250, "currency": "SEK"}`.
#[derive(Debug, Clone, Deserialize)]
pub struct AmountIn {
    minor: i64,
    currency: String,
}

impl AmountIn {
    fn to_amount(&self) -> Result<Amount, ApiError> {
        Ok(Amount::new(
            self.minor,
            Currency::new(self.currency.trim())?,
        )?)
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DepositCategoryDto {
    Allowance,
    Chore,
    Gift,
    Interest,
}

impl From<DepositCategoryDto> for DepositCategory {
    fn from(category: DepositCategoryDto) -> Self {
        match category {
            DepositCategoryDto::Allowance => DepositCategory::Allowance,
            DepositCategoryDto::Chore => DepositCategory::Chore,
            DepositCategoryDto::Gift => DepositCategory::Gift,
            DepositCategoryDto::Interest => DepositCategory::Interest,
        }
    }
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WithdrawalCategoryDto {
    Purchase,
    Donation,
}

impl From<WithdrawalCategoryDto> for WithdrawalCategory {
    fn from(category: WithdrawalCategoryDto) -> Self {
        match category {
            WithdrawalCategoryDto::Purchase => WithdrawalCategory::Purchase,
            WithdrawalCategoryDto::Donation => WithdrawalCategory::Donation,
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct NewDeposit {
    /// Left out: the Main account.
    account_id: Option<Uuid>,
    amount: AmountIn,
    category: DepositCategoryDto,
    memo: String,
    /// `YYYY-MM-DD`; left out: today.
    occurred_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewWithdrawal {
    account_id: Uuid,
    amount: AmountIn,
    category: WithdrawalCategoryDto,
    memo: String,
    occurred_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewTransfer {
    from: Uuid,
    to: Uuid,
    amount: AmountIn,
    memo: String,
    occurred_on: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct SavingsGoalIn {
    name: String,
    target: AmountIn,
    target_date: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct NewAccount {
    name: String,
    savings_goal: Option<SavingsGoalIn>,
}

#[derive(Debug, Serialize)]
pub struct KidRef {
    id: Uuid,
    name: String,
}

#[derive(Debug, Serialize)]
pub struct SavingsGoalDto {
    name: String,
    target: MoneyDto,
    target_date: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AccountDto {
    id: Uuid,
    name: String,
    /// The Main account: never closes, Deposits land here by default.
    main: bool,
    balance: MoneyDto,
    savings_goal: Option<SavingsGoalDto>,
}

#[derive(Debug, Serialize)]
pub struct TransactionDto {
    id: Uuid,
    account_id: Uuid,
    direction: &'static str,
    amount: MoneyDto,
    category: &'static str,
    memo: String,
    occurred_on: String,
    #[serde(with = "time::serde::rfc3339")]
    recorded_at: OffsetDateTime,
    transfer_id: Option<Uuid>,
    corrects: Option<Uuid>,
}

#[derive(Debug, Serialize)]
pub struct WalletView {
    id: Uuid,
    kid: KidRef,
    currency: String,
    /// How much the Kid has: the sum of open Accounts' Balances.
    total: MoneyDto,
    /// Open Accounts, Main first.
    accounts: Vec<AccountDto>,
    /// Every Transaction on every Account, newest first.
    transactions: Vec<TransactionDto>,
}

// ---------- handlers ----------

/// `GET /api/families/{id}/wallets/{wallet_id}`: the Wallet as any Member sees it.
pub async fn view(
    State(state): State<AppState>,
    signed_in: crate::auth::SignedIn,
    Path((family_id, wallet_id)): Path<(Uuid, Uuid)>,
) -> Result<Json<WalletView>, ApiError> {
    let (loaded, _) = load_as_member(&state, family_id, &signed_in.user).await?;
    let wallet = load_wallet(&state, &loaded.family, wallet_id).await?;
    Ok(Json(view_of(&loaded.family, &wallet.wallet)?))
}

/// `POST …/deposits`: money comes into the Wallet.
pub async fn deposit(
    State(state): State<AppState>,
    signed_in: crate::auth::SignedIn,
    Path((family_id, wallet_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<NewDeposit>,
) -> Result<(StatusCode, Json<WalletView>), ApiError> {
    let command = WalletCommand::Deposit {
        account_id: body.account_id.map(AccountId::from_uuid),
        amount: body.amount.to_amount()?,
        category: body.category.into(),
        memo: Memo::new(&body.memo)?,
        occurred_on: occurred_on(body.occurred_on.as_deref())?,
    };
    change(&state, &signed_in, family_id, wallet_id, &command).await
}

/// `POST …/withdrawals`: money leaves the Wallet. Never below zero (no Overdraft).
pub async fn withdraw(
    State(state): State<AppState>,
    signed_in: crate::auth::SignedIn,
    Path((family_id, wallet_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<NewWithdrawal>,
) -> Result<(StatusCode, Json<WalletView>), ApiError> {
    let command = WalletCommand::Withdraw {
        account_id: AccountId::from_uuid(body.account_id),
        amount: body.amount.to_amount()?,
        category: body.category.into(),
        memo: Memo::new(&body.memo)?,
        occurred_on: occurred_on(body.occurred_on.as_deref())?,
    };
    change(&state, &signed_in, family_id, wallet_id, &command).await
}

/// `POST …/transfers`: money moves between two of the Kid's Accounts; the total stays.
pub async fn transfer(
    State(state): State<AppState>,
    signed_in: crate::auth::SignedIn,
    Path((family_id, wallet_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<NewTransfer>,
) -> Result<(StatusCode, Json<WalletView>), ApiError> {
    let command = WalletCommand::Transfer {
        from: AccountId::from_uuid(body.from),
        to: AccountId::from_uuid(body.to),
        amount: body.amount.to_amount()?,
        memo: Memo::new(&body.memo)?,
        occurred_on: occurred_on(body.occurred_on.as_deref())?,
    };
    change(&state, &signed_in, family_id, wallet_id, &command).await
}

/// `POST …/accounts`: opens another Account, optionally with a Savings goal.
pub async fn open_account(
    State(state): State<AppState>,
    signed_in: crate::auth::SignedIn,
    Path((family_id, wallet_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<NewAccount>,
) -> Result<(StatusCode, Json<WalletView>), ApiError> {
    let savings_goal = match body.savings_goal {
        Some(goal) => Some(SavingsGoal {
            name: AccountName::new(&goal.name)?,
            target: goal.target.to_amount()?,
            target_date: goal.target_date.as_deref().map(parse_date).transpose()?,
        }),
        None => None,
    };
    let command = WalletCommand::OpenAccount {
        name: AccountName::new(&body.name)?,
        savings_goal,
    };
    change(&state, &signed_in, family_id, wallet_id, &command).await
}

// ---------- helpers ----------

/// Loads the Wallet of one of the Family's active Kids; 404 otherwise.
async fn load_wallet(
    state: &AppState,
    family: &Family,
    wallet_id: Uuid,
) -> Result<LoadedWallet, ApiError> {
    let wallet_id = WalletId::from_uuid(wallet_id);
    if !family.active_kids().any(|kid| kid.wallet_id == wallet_id) {
        return Err(ApiError::WalletNotFound);
    }
    state
        .wallets
        .load(family.id(), wallet_id)
        .await?
        .ok_or(ApiError::WalletNotFound)
}

/// Loads, decides and appends, deciding again if another Member saved first.
async fn change(
    state: &AppState,
    signed_in: &crate::auth::SignedIn,
    family_id: Uuid,
    wallet_id: Uuid,
    command: &WalletCommand,
) -> Result<(StatusCode, Json<WalletView>), ApiError> {
    for _ in 0..ATTEMPTS {
        let (loaded, actor) = load_as_member(state, family_id, &signed_in.user).await?;
        let current = load_wallet(state, &loaded.family, wallet_id).await?;
        let context = Context {
            actor,
            now: OffsetDateTime::now_utc(),
        };
        let events = current.wallet.decide(command, &context, &mut RandomIds)?;
        match state
            .wallets
            .append(&current.wallet, current.version, &events)
            .await
        {
            Ok(_) => {
                let mut next = current.wallet.clone();
                for event in &events {
                    next.apply(event).map_err(|_| ApiError::Internal)?;
                }
                return Ok((StatusCode::CREATED, Json(view_of(&loaded.family, &next)?)));
            }
            Err(StoreError::Conflict) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ApiError::Conflict)
}

fn occurred_on(text: Option<&str>) -> Result<Date, ApiError> {
    match text {
        Some(text) => parse_date(text),
        None => Ok(OffsetDateTime::now_utc().date()),
    }
}

/// Parses `YYYY-MM-DD`.
fn parse_date(text: &str) -> Result<Date, ApiError> {
    let mut parts = text.trim().splitn(3, '-');
    let mut next = || parts.next().and_then(|p| p.parse::<i32>().ok());
    let (Some(year), Some(month), Some(day)) = (next(), next(), next()) else {
        return Err(ApiError::InvalidDate);
    };
    let month = u8::try_from(month)
        .ok()
        .and_then(|m| Month::try_from(m).ok())
        .ok_or(ApiError::InvalidDate)?;
    let day = u8::try_from(day).map_err(|_| ApiError::InvalidDate)?;
    Date::from_calendar_date(year, month, day).map_err(|_| ApiError::InvalidDate)
}

fn money(minor: i64, currency: Currency) -> MoneyDto {
    MoneyDto {
        minor,
        currency: currency.code().to_owned(),
    }
}

fn balance(balance: Balance) -> MoneyDto {
    money(balance.minor(), balance.currency())
}

fn amount(amount: Amount) -> MoneyDto {
    money(amount.minor(), amount.currency())
}

const fn category_name(category: Category) -> &'static str {
    match category {
        Category::Allowance => "allowance",
        Category::Chore => "chore",
        Category::Gift => "gift",
        Category::Interest => "interest",
        Category::Purchase => "purchase",
        Category::Donation => "donation",
        Category::Transfer => "transfer",
        Category::Correction => "correction",
    }
}

fn transaction(t: &Transaction) -> TransactionDto {
    TransactionDto {
        id: t.id().as_uuid(),
        account_id: t.account_id().as_uuid(),
        direction: match t.direction() {
            Direction::In => "in",
            Direction::Out => "out",
        },
        amount: amount(t.amount()),
        category: category_name(t.category()),
        memo: t.memo().as_str().to_owned(),
        occurred_on: t.occurred_on().to_string(),
        recorded_at: t.recorded_at(),
        transfer_id: t.transfer_id().map(|id| id.as_uuid()),
        corrects: t.corrects().map(|id| id.as_uuid()),
    }
}

fn view_of(family: &Family, wallet: &Wallet) -> Result<WalletView, ApiError> {
    let kid = family
        .kid(wallet.kid_id())
        .ok_or(ApiError::WalletNotFound)?;
    let accounts = wallet
        .accounts()
        .filter(|a| !a.is_closed())
        .map(|a| AccountDto {
            id: a.id().as_uuid(),
            name: a.name().as_str().to_owned(),
            main: a.is_main(),
            balance: balance(a.balance()),
            savings_goal: a.savings_goal().map(|goal| SavingsGoalDto {
                name: goal.name.as_str().to_owned(),
                target: amount(goal.target),
                target_date: goal.target_date.map(|d| d.to_string()),
            }),
        })
        .collect();
    let mut rows: Vec<&Transaction> = wallet.accounts().flat_map(|a| a.ledger()).collect();
    // Newest first; the two sides of a Transfer stay together.
    rows.sort_by(|a, b| {
        (b.occurred_on(), b.recorded_at()).cmp(&(a.occurred_on(), a.recorded_at()))
    });
    Ok(WalletView {
        id: wallet.id().as_uuid(),
        kid: KidRef {
            id: kid.id.as_uuid(),
            name: kid.name.as_str().to_owned(),
        },
        currency: wallet.currency().code().to_owned(),
        total: balance(wallet.total()),
        accounts,
        transactions: rows.into_iter().map(transaction).collect(),
    })
}
