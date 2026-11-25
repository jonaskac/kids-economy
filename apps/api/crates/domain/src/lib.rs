//! Pure domain model for Kids Economy: Family, Wallet, Account, Transaction.
//!
//! No IO and no async in this crate. Every rule is driven by a unit test.
//! Terms follow `docs/glossary.md`.

pub mod ids;
pub mod money;
pub mod role;
pub mod text;
pub mod wallet;
