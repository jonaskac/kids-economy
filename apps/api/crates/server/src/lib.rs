//! Wiring for the Kids Economy server (ADR-0007, ADR-0014): configuration from the
//! environment, the web app served next to the API, and waiting for Postgres at start.

pub mod config;
pub mod startup;
pub mod web;

pub use config::{Config, ConfigError};
