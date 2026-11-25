//! Why the store could not load or save a Wallet.

use crate::body::BodyError;
use domain::wallet::ReplayError;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum StoreError {
    /// The Wallet moved past the Version the decision was made on. Nothing was saved;
    /// load the Wallet again and decide again.
    #[error("the Wallet changed since it was loaded")]
    Conflict,
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
