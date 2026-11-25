//! Server configuration, read only from environment variables (12-factor, ADR-0007).
//! Every variable is documented in `.env.example` and `docs/deploy.md`.

use std::path::PathBuf;

use thiserror::Error;

/// Port used when `PORT` is not set.
pub const DEFAULT_PORT: u16 = 8080;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Config {
    /// TCP port to listen on, on all interfaces (`PORT`).
    pub port: u16,
    /// Postgres to connect to; migrations run on it at start (`DATABASE_URL`).
    pub database_url: String,
    /// Folder with the built web app (`WEB_DIR`). Unset: only the API is served.
    pub web_dir: Option<PathBuf>,
}

#[derive(Debug, Error, PartialEq, Eq)]
pub enum ConfigError {
    #[error("DATABASE_URL is not set")]
    MissingDatabaseUrl,
    #[error("PORT must be a number from 1 to 65535, got {0:?}")]
    InvalidPort(String),
}

impl Config {
    /// Reads the configuration from the process environment.
    pub fn from_env() -> Result<Self, ConfigError> {
        Self::from_lookup(|name| std::env::var(name).ok())
    }

    /// Reads the configuration through `lookup`, which returns a variable's value if set.
    /// Empty values count as unset, as hosting dashboards often save blank fields.
    pub fn from_lookup(lookup: impl Fn(&str) -> Option<String>) -> Result<Self, ConfigError> {
        let get = |name: &str| lookup(name).filter(|value| !value.trim().is_empty());

        let port = match get("PORT") {
            None => DEFAULT_PORT,
            Some(raw) => match raw.trim().parse::<u16>() {
                Ok(port) if port > 0 => port,
                _ => return Err(ConfigError::InvalidPort(raw)),
            },
        };
        let database_url = get("DATABASE_URL").ok_or(ConfigError::MissingDatabaseUrl)?;
        let web_dir = get("WEB_DIR").map(PathBuf::from);

        Ok(Config {
            port,
            database_url,
            web_dir,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(vars: &[(&str, &str)]) -> Result<Config, ConfigError> {
        let vars: Vec<(String, String)> = vars
            .iter()
            .map(|(k, v)| ((*k).to_owned(), (*v).to_owned()))
            .collect();
        Config::from_lookup(|name| vars.iter().find(|(k, _)| k == name).map(|(_, v)| v.clone()))
    }

    const DB: (&str, &str) = ("DATABASE_URL", "postgres://u:p@db:5432/kids");

    #[test]
    fn only_database_url_is_required() {
        assert_eq!(
            config(&[DB]),
            Ok(Config {
                port: 8080,
                database_url: "postgres://u:p@db:5432/kids".into(),
                web_dir: None,
            })
        );
    }

    #[test]
    fn a_missing_or_blank_database_url_is_an_error() {
        assert_eq!(config(&[]), Err(ConfigError::MissingDatabaseUrl));
        assert_eq!(
            config(&[("DATABASE_URL", "  ")]),
            Err(ConfigError::MissingDatabaseUrl)
        );
    }

    #[test]
    fn port_and_web_dir_are_read() {
        let config = config(&[DB, ("PORT", "3000"), ("WEB_DIR", "/app/web")]).expect("config");
        assert_eq!(config.port, 3000);
        assert_eq!(config.web_dir, Some(PathBuf::from("/app/web")));
    }

    #[test]
    fn a_blank_port_or_web_dir_counts_as_unset() {
        let config = config(&[DB, ("PORT", ""), ("WEB_DIR", "")]).expect("config");
        assert_eq!(config.port, DEFAULT_PORT);
        assert_eq!(config.web_dir, None);
    }

    #[test]
    fn a_port_that_is_not_a_valid_number_is_an_error_not_a_silent_default() {
        for bad in ["http", "0", "70000", "-1"] {
            assert_eq!(
                config(&[DB, ("PORT", bad)]),
                Err(ConfigError::InvalidPort(bad.into())),
                "PORT={bad}"
            );
        }
    }
}
