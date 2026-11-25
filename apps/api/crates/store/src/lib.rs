//! Postgres adapter: each Wallet's History in the append-only `ledger_events` table,
//! numbered per Wallet (ADR-0005, ADR-0009).
//!
//! Tested against real Postgres via `#[sqlx::test]` in `tests/`.

pub mod body;
mod error;
mod version;
mod wallet_store;

pub use error::StoreError;
pub use version::Version;
pub use wallet_store::{LoadedWallet, WalletStore};

/// The schema migrations in `migrations/`, embedded in the binary.
pub static MIGRATOR: sqlx::migrate::Migrator = sqlx::migrate!();
