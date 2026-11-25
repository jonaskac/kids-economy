//! Waiting for Postgres at start. On a fresh install the app and its database start
//! together, and the database may take a few seconds before it accepts connections.

use std::time::Duration;

use sqlx::postgres::PgConnectOptions;

/// How long to wait before each new attempt to reach Postgres: doubling from one
/// second, capped at five, about two minutes in all before giving up.
pub fn retry_delays() -> impl Iterator<Item = Duration> {
    (0..26).map(|attempt| Duration::from_secs(1u64 << attempt.min(3)).min(Duration::from_secs(5)))
}

/// Connection options for `database_url`.
///
/// `sslrootcert=system` (libpq 16: "trust the system's certificate authorities", as
/// Sliplane's managed Postgres URLs say) is not a file, so it is dropped: without a root
/// certificate the client already trusts the well-known public authorities.
pub fn connect_options(database_url: &str) -> Result<PgConnectOptions, sqlx::Error> {
    let mut url =
        url::Url::parse(database_url).map_err(|e| sqlx::Error::Configuration(e.into()))?;
    if url
        .query_pairs()
        .any(|(key, value)| key == "sslrootcert" && value == "system")
    {
        let kept: Vec<(String, String)> = url
            .query_pairs()
            .filter(|(key, value)| !(key == "sslrootcert" && value == "system"))
            .map(|(k, v)| (k.into_owned(), v.into_owned()))
            .collect();
        url.query_pairs_mut().clear().extend_pairs(kept);
    }
    url.as_str().parse()
}

#[cfg(test)]
mod tests {
    use super::*;
    use sqlx::postgres::PgSslMode;

    #[test]
    fn a_plain_url_keeps_its_parts() {
        let options = connect_options("postgres://kid:pw@db.internal:5433/kids").expect("options");
        assert_eq!(options.get_host(), "db.internal");
        assert_eq!(options.get_port(), 5433);
        assert_eq!(options.get_username(), "kid");
        assert_eq!(options.get_database(), Some("kids"));
    }

    #[test]
    fn a_managed_postgres_url_with_system_root_certificates_verifies_the_server() {
        let options = connect_options(
            "postgres://app:pw@abc123.sliplane.app:31234/kids?sslmode=verify-full&sslrootcert=system",
        )
        .expect("options");
        assert!(matches!(options.get_ssl_mode(), PgSslMode::VerifyFull));
        assert_eq!(options.get_port(), 31234);
        assert!(
            !format!("{:?}", options).contains("\"system\""),
            "sslrootcert=system must not be read as a file: {options:?}"
        );
    }

    #[test]
    fn a_root_certificate_file_is_kept() {
        let options = connect_options(
            "postgres://app:pw@db/kids?sslmode=verify-ca&sslrootcert=/certs/ca.pem",
        )
        .expect("options");
        assert!(format!("{options:?}").contains("/certs/ca.pem"));
    }

    #[test]
    fn a_url_that_is_not_postgres_is_an_error() {
        assert!(connect_options("not a url").is_err());
    }

    #[test]
    fn delays_double_then_stay_at_five_seconds() {
        let secs: Vec<u64> = retry_delays().take(5).map(|d| d.as_secs()).collect();
        assert_eq!(secs, [1, 2, 4, 5, 5]);
    }

    #[test]
    fn retries_give_up_after_about_two_minutes() {
        let total: Duration = retry_delays().sum();
        assert!(total >= Duration::from_secs(110), "{total:?}");
        assert!(total <= Duration::from_secs(130), "{total:?}");
    }
}
