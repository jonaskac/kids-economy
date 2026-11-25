//! Postgres adapter: each Wallet's History in the append-only `ledger_events` table,
//! numbered per Wallet (ADR-0005, ADR-0009); Families and Users as `documents`, and
//! Sessions (ADR-0010, ADR-0011).
//!
//! Tested against real Postgres via `#[sqlx::test]` in `tests/`.

pub mod body;
mod error;
pub mod family_body;
mod family_store;
mod identity_store;
mod version;
mod wallet_store;

pub use error::StoreError;
pub use family_store::{FamilyStore, LoadedFamily};
pub use identity_store::{SessionStore, UserRecord, UserStore};
pub use version::Version;
pub use wallet_store::{LoadedWallet, WalletStore};

/// The schema migrations in `migrations/`, embedded in the binary.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
