//! Why the store could not load or save a Wallet, Family, User or Session.

use crate::body::BodyError;
use crate::family_body::DocumentError;
use domain::wallet::ReplayError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    /// The Wallet or Family moved past the Version the decision was made on. Nothing was
    /// saved; load it again and decide again.
    #[error("it changed since it was loaded")]
    Conflict,
    /// Another User already signs in with this email address.
    #[error("the email address is already registered")]
    EmailTaken,
    /// The Password reset link is unknown, already used or expired.
    #[error("the password reset link is unknown, used or expired")]
    PasswordResetNotPending,
    /// A Wallet opened together with a Kid must belong to that Kid's Family.
    #[error("the Wallet belongs to another Family")]
    WrongFamily,
    /// A stored Family or User, or one about to be stored, does not fit the body format.
    #[error("document {id} has an invalid body: {source}")]
    Document {
        id: uuid::Uuid,
        #[source]
        source: DocumentError,
    },
    /// The Events do not continue the Wallet's History (or a stored History is broken).
    #[error("not a valid Wallet History: {0}")]
    InvalidHistory(#[from] ReplayError),
    /// A stored Event, or one about to be stored, does not fit the body format.
    #[error("event {sequence} has an invalid body: {source}")]
    Body {
        sequence: i64,
        #[source]
        source: BodyError,
    },
    #[error("database error: {0}")]
    Database(#[source] sqlx::Error),
}

impl From<sqlx::Error> for StoreError {
    /// A duplicate `(wallet_id, sequence)` means another save got there first.
    fn from(error: sqlx::Error) -> Self {
        let duplicate = error
            .as_database_error()
            .is_some_and(|db| db.is_unique_violation());
        if duplicate {
            StoreError::Conflict
        } else {
            StoreError::Database(error)
        }
    }
}
