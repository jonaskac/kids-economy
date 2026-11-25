//! Kids Economy server: reads configuration from the environment, waits for Postgres,
//! runs migrations, then serves the API and (with `WEB_DIR`) the web app.
//!
//! Environment (see `.env.example` and `docs/deploy.md`):
//! - `PORT`: TCP port to listen on (default 8080)
//! - `DATABASE_URL`: Postgres to connect to; migrations run on start
//! - `WEB_DIR`: folder with the built web app; unset serves only the API

use std::net::SocketAddr;
use std::process::ExitCode;
use std::time::Duration;

use kids_http::{AppState, Passwords};
use server::startup::{connect_options, retry_delays};
use server::web::with_web_app;
use server::Config;
use sqlx::postgres::PgPoolOptions;
use sqlx::PgPool;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> ExitCode {
    match run().await {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            tracing::error!("{error}");
            ExitCode::FAILURE
        }
    }
}

async fn run() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let config = Config::from_env()?;
    let pool = connect(&config.database_url).await?;
    store::MIGRATOR.run(&pool).await?;
    tracing::info!("migrations are up to date");

    let api = kids_http::router(AppState::new(pool, Passwords::default()));
    let app = match &config.web_dir {
        Some(dir) => {
            tracing::info!(dir = %dir.display(), "serving the web app");
            with_web_app(api, dir)
        }
        None => api,
    };

    let addr = SocketAddr::from(([0, 0, 0, 0], config.port));
    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, app)
        .with_graceful_shutdown(shutdown_signal())
        .await?;
    Ok(())
}

/// Connects to Postgres, retrying while it starts up (a fresh install starts both at once).
async fn connect(database_url: &str) -> Result<PgPool, sqlx::Error> {
    let options = connect_options(database_url)?;
    let mut delays = retry_delays();
    loop {
        match PgPoolOptions::new()
            .max_connections(10)
            .acquire_timeout(Duration::from_secs(3))
            .connect_with(options.clone())
            .await
        {
            Ok(pool) => return Ok(pool),
            Err(error) => match delays.next() {
                Some(delay) => {
                    tracing::warn!(%error, ?delay, "Postgres is not reachable yet, retrying");
                    tokio::time::sleep(delay).await;
                }
                None => return Err(error),
            },
        }
    }
}

/// Ends on Ctrl-C or SIGTERM (what container hosts send on redeploy).
async fn shutdown_signal() {
    let ctrl_c = async {
        if tokio::signal::ctrl_c().await.is_err() {
            std::future::pending::<()>().await;
        }
    };
    #[cfg(unix)]
    let terminate = async {
        match tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate()) {
            Ok(mut signal) => {
                signal.recv().await;
            }
            Err(_) => std::future::pending::<()>().await,
        }
    };
    #[cfg(not(unix))]
    let terminate = std::future::pending::<()>();
    tokio::select! {
        () = ctrl_c => {},
        () = terminate => {},
    }
}
