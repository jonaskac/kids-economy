//! Password reset links (ADR-0012): found by the hash of their secret, used once.

use crate::error::StoreError;
use domain::ids::UserId;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// A stored Password reset link.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PasswordReset {
    pub user_id: UserId,
    pub expires_at: OffsetDateTime,
    pub used_at: Option<OffsetDateTime>,
}

#[derive(Debug, Clone)]
pub struct PasswordResetStore {
    pool: PgPool,
}

impl PasswordResetStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Stores a new link for `user_id`. Earlier unused links for that User stop working.
    pub async fn create(
        &self,
        token_hash: &str,
        user_id: UserId,
        expires_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("delete from password_resets where user_id = $1 and used_at is null")
            .bind(user_id.as_uuid())
            .execute(&mut *tx)
            .await?;
        sqlx::query(
            "insert into password_resets (token_hash, user_id, expires_at) values ($1, $2, $3)",
        )
        .bind(token_hash)
        .bind(user_id.as_uuid())
        .bind(expires_at)
        .execute(&mut *tx)
        .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn find(&self, token_hash: &str) -> Result<Option<PasswordReset>, StoreError> {
        let row: Option<(Uuid, OffsetDateTime, Option<OffsetDateTime>)> = sqlx::query_as(
            "select user_id, expires_at, used_at from password_resets where token_hash = $1",
        )
        .bind(token_hash)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(|(user_id, expires_at, used_at)| PasswordReset {
            user_id: UserId::from_uuid(user_id),
            expires_at,
            used_at,
        }))
    }

    /// In one step: marks the link used, stores the new password hash on the User and
    /// ends all of that User's Sessions. [`StoreError::PasswordResetNotPending`] if the
    /// link is unknown, already used or expired at `now`.
    pub async fn use_link(
        &self,
        token_hash: &str,
        password_hash: &str,
        now: OffsetDateTime,
    ) -> Result<UserId, StoreError> {
        let mut tx = self.pool.begin().await?;
        let used: Option<(Uuid,)> = sqlx::query_as(
            "update password_resets set used_at = $2
             where token_hash = $1 and used_at is null and expires_at > $2
             returning user_id",
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(&mut *tx)
        .await?;
        let Some((user_id,)) = used else {
            return Err(StoreError::PasswordResetNotPending);
        };
        sqlx::query(
            "update documents
             set body = jsonb_set(body, '{password_hash}', to_jsonb($2::text)),
                 version = version + 1, updated_at = now()
             where id = $1 and kind = 'user'",
        )
        .bind(user_id)
        .bind(password_hash)
        .execute(&mut *tx)
        .await?;
        sqlx::query("delete from sessions where user_id = $1")
            .bind(user_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(UserId::from_uuid(user_id))
    }
}
