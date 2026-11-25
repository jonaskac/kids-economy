//! Loads and saves Families as current-state documents (ADR-0010).

use crate::error::StoreError;
use crate::family_body::{decode_family, encode_family, invitation_pattern, member_pattern};
use crate::version::Version;
use crate::wallet_store::open_in;
use domain::family::{Family, InvitationLinkHash};
use domain::ids::{FamilyId, UserId};
use domain::wallet::WalletEvent;
use serde_json::Value;
use sqlx::PgPool;
use uuid::Uuid;

/// A Family loaded from storage, with the Version to decide against.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoadedFamily {
    pub family: Family,
    pub version: Version,
}

#[derive(Debug, Clone)]
pub struct FamilyStore {
    pool: PgPool,
}

impl FamilyStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Saves a new Family at Version 1. Creating the same Family twice is a Conflict.
    pub async fn create(&self, family: &Family) -> Result<Version, StoreError> {
        let id = family.id().as_uuid();
        let body = encode(family)?;
        sqlx::query(
            "insert into documents (id, kind, family_id, version, body)
             values ($1, 'family', $1, 1, $2)",
        )
        .bind(id)
        .bind(body)
        .execute(&self.pool)
        .await?;
        Ok(Version::from_sequence(1))
    }

    pub async fn load(&self, family_id: FamilyId) -> Result<Option<LoadedFamily>, StoreError> {
        let row: Option<(Uuid, i64, Value)> = sqlx::query_as(
            "select id, version, body from documents where kind = 'family' and id = $1",
        )
        .bind(family_id.as_uuid())
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode).transpose()
    }

    /// The Families where the User is an active Member, oldest first.
    pub async fn of_user(&self, user_id: UserId) -> Result<Vec<Family>, StoreError> {
        let rows: Vec<(Uuid, i64, Value)> = sqlx::query_as(
            "select id, version, body from documents
             where kind = 'family' and body @> $1
             order by created_at, id",
        )
        .bind(member_pattern(user_id))
        .fetch_all(&self.pool)
        .await?;
        let mut families = Vec::with_capacity(rows.len());
        for row in rows {
            let loaded = decode(row)?;
            if loaded.family.actor_for(user_id).is_some() {
                families.push(loaded.family);
            }
        }
        Ok(families)
    }

    /// The Family holding the Invitation behind a link, whatever the Invitation's state.
    pub async fn by_invitation_link(
        &self,
        link_hash: &InvitationLinkHash,
    ) -> Result<Option<LoadedFamily>, StoreError> {
        let row: Option<(Uuid, i64, Value)> = sqlx::query_as(
            "select id, version, body from documents
             where kind = 'family' and body @> $1
             limit 1",
        )
        .bind(invitation_pattern(link_hash))
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode).transpose()
    }

    /// Saves a decision on a Family loaded at `expected`. Otherwise a Conflict, nothing saved.
    pub async fn save(&self, family: &Family, expected: Version) -> Result<Version, StoreError> {
        let mut tx = self.pool.begin().await?;
        let version = update_in(&mut tx, family, expected).await?;
        tx.commit().await?;
        Ok(version)
    }

    /// Saves a Family with a newly added Kid and opens the Kid's Wallet: both or neither.
    pub async fn save_with_new_wallet(
        &self,
        family: &Family,
        expected: Version,
        opened: &WalletEvent,
    ) -> Result<Version, StoreError> {
        match opened {
            WalletEvent::WalletOpened { family_id, .. } if *family_id == family.id() => {}
            _ => return Err(StoreError::WrongFamily),
        }
        let mut tx = self.pool.begin().await?;
        let version = update_in(&mut tx, family, expected).await?;
        open_in(&mut tx, opened).await?;
        tx.commit().await?;
        Ok(version)
    }
}

async fn update_in(
    tx: &mut sqlx::PgConnection,
    family: &Family,
    expected: Version,
) -> Result<Version, StoreError> {
    let body = encode(family)?;
    let updated = sqlx::query(
        "update documents set body = $3, version = version + 1, updated_at = now()
         where kind = 'family' and id = $1 and version = $2",
    )
    .bind(family.id().as_uuid())
    .bind(expected.sequence())
    .bind(body)
    .execute(&mut *tx)
    .await?;
    if updated.rows_affected() == 1 {
        Ok(Version::from_sequence(expected.sequence() + 1))
    } else {
        Err(StoreError::Conflict)
    }
}

fn encode(family: &Family) -> Result<Value, StoreError> {
    encode_family(family).map_err(|source| StoreError::Document {
        id: family.id().as_uuid(),
        source,
    })
}

fn decode((id, version, body): (Uuid, i64, Value)) -> Result<LoadedFamily, StoreError> {
    let family = decode_family(id, body).map_err(|source| StoreError::Document { id, source })?;
    Ok(LoadedFamily {
        family,
        version: Version::from_sequence(version),
    })
}
