//! Kids Economy server: reads configuration from the environment and serves the API.
//!
//! Environment (see `.env.example`):
//! - `PORT`: TCP port to listen on (default 8080)

use std::net::SocketAddr;

use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()))
        .init();

    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr).await?;
    tracing::info!(%addr, "listening");
    axum::serve(listener, kids_http::router())
        .with_graceful_shutdown(shutdown_signal())
        .await
}

async fn shutdown_signal() {
    // If the signal handler cannot be installed, run until the process is killed.
    if tokio::signal::ctrl_c().await.is_err() {
        std::future::pending::<()>().await;
    }
}
