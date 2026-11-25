//! Loads and saves Wallets as their History (ADR-0009).

use crate::body::{self, StoredBody};
use crate::error::StoreError;
use crate::version::Version;
use domain::ids::{FamilyId, WalletId};
use domain::wallet::{ReplayError, Wallet, WalletEvent};
use serde_json::Value;
use sqlx::PgPool;

/// A Wallet rebuilt from storage, with the Version to decide against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedWallet {
    pub wallet: Wallet,
    pub version: Version,
}

#[derive(Debug, Clone)]
pub struct WalletStore {
    pool: PgPool,
}

impl WalletStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Saves a new Wallet's first Event. Opening the same Wallet twice is a Conflict.
    pub async fn open(&self, opened: &WalletEvent) -> Result<Version, StoreError> {
        let wallet = Wallet::replay([opened])?;
        let rows = encode_from(Version::NONE, std::slice::from_ref(opened))?;
        self.insert(&wallet, Version::NONE, &rows).await
    }

    /// The Wallet's History in Sequence order, only if it belongs to `family_id`.
    /// Empty when there is no such Wallet in that Family.
    pub async fn history(
        &self,
        family_id: FamilyId,
        wallet_id: WalletId,
    ) -> Result<Vec<WalletEvent>, StoreError> {
        let rows: Vec<(i64, String, Value)> = sqlx::query_as(
            "select sequence, kind, body from ledger_events
             where family_id = $1 and wallet_id = $2
             order by sequence",
        )
        .bind(family_id.as_uuid())
        .bind(wallet_id.as_uuid())
        .fetch_all(&self.pool)
        .await?;
        rows.into_iter()
            .map(|(sequence, kind, body)| {
                body::decode(&kind, body).map_err(|source| StoreError::Body { sequence, source })
            })
            .collect()
    }

    /// Rebuilds the Wallet from its History. `None` if the Family has no such Wallet.
    pub async fn load(
        &self,
        family_id: FamilyId,
        wallet_id: WalletId,
    ) -> Result<Option<LoadedWallet>, StoreError> {
        let history = self.history(family_id, wallet_id).await?;
        if history.is_empty() {
            return Ok(None);
        }
        Ok(Some(LoadedWallet {
            wallet: Wallet::replay(&history)?,
            version: Version::from_count(history.len()),
        }))
    }

    /// Saves the Events a decision on `wallet` returned, if the Wallet is still at
    /// `expected`. All or nothing: otherwise [`StoreError::Conflict`] and nothing is saved.
    pub async fn append(
        &self,
        wallet: &Wallet,
        expected: Version,
        events: &[WalletEvent],
    ) -> Result<Version, StoreError> {
        if expected == Version::NONE {
            // A Wallet is saved first with `open`.
            return Err(ReplayError::NotOpened.into());
        }
        // An invalid History never reaches the database.
        let mut check = wallet.clone();
        for event in events {
            check.apply(event)?;
        }
        let rows = encode_from(expected, events)?;
        self.insert(wallet, expected, &rows).await
    }

    async fn insert(
        &self,
        wallet: &Wallet,
        expected: Version,
        rows: &[(i64, StoredBody)],
    ) -> Result<Version, StoreError> {
        let family_id = wallet.family_id().as_uuid();
        let wallet_id = wallet.id().as_uuid();
        let mut tx = self.pool.begin().await?;

        let (current,): (Option<i64>,) =
            sqlx::query_as("select max(sequence) from ledger_events where wallet_id = $1")
                .bind(wallet_id)
                .fetch_one(&mut *tx)
                .await?;
        if current.unwrap_or(0) != expected.sequence() {
            return Err(StoreError::Conflict);
        }
        // Two saves at the same Version both pass the check above; the primary key
        // (wallet_id, sequence) lets only the first one commit.
        for (sequence, stored) in rows {
            sqlx::query(
                "insert into ledger_events (family_id, wallet_id, sequence, kind, body)
                 values ($1, $2, $3, $4, $5)",
            )
            .bind(family_id)
            .bind(wallet_id)
            .bind(sequence)
            .bind(&stored.kind)
            .bind(&stored.body)
            .execute(&mut *tx)
            .await?;
        }
        tx.commit().await?;

        Ok(rows
            .last()
            .map_or(expected, |(sequence, _)| Version::from_sequence(*sequence)))
    }
}

fn encode_from(
    expected: Version,
    events: &[WalletEvent],
) -> Result<Vec<(i64, StoredBody)>, StoreError> {
    events
        .iter()
        .enumerate()
        .map(|(n, event)| {
            let sequence = expected.next(n);
            body::encode(event)
                .map(|stored| (sequence, stored))
                .map_err(|source| StoreError::Body { sequence, source })
        })
        .collect()
}
