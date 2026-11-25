//! Users (as documents) and Sessions (ADR-0010, ADR-0011).

use crate::error::StoreError;
use crate::family_body::{decode_user, encode_user};
use domain::identity::{Email, User};
use domain::ids::UserId;
use serde_json::Value;
use sqlx::PgPool;
use time::OffsetDateTime;
use uuid::Uuid;

/// A User with the hash of their password, for signing in.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserRecord {
    pub user: User,
    pub password_hash: String,
}

#[derive(Debug, Clone)]
pub struct UserStore {
    pool: PgPool,
}

impl UserStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    /// Saves a new User. [`StoreError::EmailTaken`] if the email is already registered.
    pub async fn register(&self, user: &User, password_hash: &str) -> Result<(), StoreError> {
        let id = user.id.as_uuid();
        let body = encode_user(user, password_hash)
            .map_err(|source| StoreError::Document { id, source })?;
        let result = sqlx::query(
            "insert into documents (id, kind, family_id, version, body)
             values ($1, 'user', null, 1, $2)",
        )
        .bind(id)
        .bind(body)
        .execute(&self.pool)
        .await;
        match result {
            Ok(_) => Ok(()),
            Err(error)
                if error
                    .as_database_error()
                    .and_then(|db| db.constraint())
                    .is_some_and(|c| c == "documents_user_email") =>
            {
                Err(StoreError::EmailTaken)
            }
            Err(error) => Err(error.into()),
        }
    }

    pub async fn find_by_email(&self, email: &Email) -> Result<Option<UserRecord>, StoreError> {
        let row: Option<(Uuid, Value)> = sqlx::query_as(
            "select id, body from documents where kind = 'user' and body ->> 'email' = $1",
        )
        .bind(email.as_str())
        .fetch_optional(&self.pool)
        .await?;
        row.map(decode).transpose()
    }

    pub async fn get(&self, id: UserId) -> Result<Option<User>, StoreError> {
        let row: Option<(Uuid, Value)> =
            sqlx::query_as("select id, body from documents where kind = 'user' and id = $1")
                .bind(id.as_uuid())
                .fetch_optional(&self.pool)
                .await?;
        Ok(row.map(decode).transpose()?.map(|record| record.user))
    }
}

fn decode((id, body): (Uuid, Value)) -> Result<UserRecord, StoreError> {
    let (user, password_hash) =
        decode_user(id, body).map_err(|source| StoreError::Document { id, source })?;
    Ok(UserRecord {
        user,
        password_hash,
    })
}

/// Signed-in devices, found by the hash of their cookie secret.
#[derive(Debug, Clone)]
pub struct SessionStore {
    pool: PgPool,
}

impl SessionStore {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn start(
        &self,
        token_hash: &str,
        user_id: UserId,
        expires_at: OffsetDateTime,
    ) -> Result<(), StoreError> {
        sqlx::query("insert into sessions (token_hash, user_id, expires_at) values ($1, $2, $3)")
            .bind(token_hash)
            .bind(user_id.as_uuid())
            .bind(expires_at)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    /// The signed-in User, if the Session exists and has not expired at `now`.
    pub async fn user_for(
        &self,
        token_hash: &str,
        now: OffsetDateTime,
    ) -> Result<Option<User>, StoreError> {
        let row: Option<(Uuid, Value)> = sqlx::query_as(
            "select d.id, d.body from sessions s join documents d on d.id = s.user_id
             where s.token_hash = $1 and s.expires_at > $2 and d.kind = 'user'",
        )
        .bind(token_hash)
        .bind(now)
        .fetch_optional(&self.pool)
        .await?;
        Ok(row.map(decode).transpose()?.map(|record| record.user))
    }

    /// Signs out: the Session is gone. Ending an unknown Session does nothing.
    pub async fn end(&self, token_hash: &str) -> Result<(), StoreError> {
        sqlx::query("delete from sessions where token_hash = $1")
            .bind(token_hash)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
