//! The built web app (`apps/web/dist`), served from the same origin as the API (ADR-0007).
//!
//! - Files that exist are served as they are. Vite names everything under `/assets/`
//!   after its content, so those are cached for a year.
//! - Any other path that is not under `/api` is a page of the web app (`/join/…`,
//!   `/families/…`), so it gets `index.html`, which browsers must revalidate: after a
//!   new release they load the new assets.
//! - Unknown `/api/…` paths and missing assets stay 404.

use std::path::Path;

use axum::http::{header, HeaderValue, StatusCode};
use axum::routing::any;
use axum::Router;
use tower_http::services::{ServeDir, ServeFile};
use tower_http::set_header::SetResponseHeaderLayer;

const IMMUTABLE: &str = "public, max-age=31536000, immutable";
const REVALIDATE: &str = "no-cache";

/// Adds the web app in `web_dir` to the API router.
pub fn with_web_app(api: Router, web_dir: &Path) -> Router {
    let index = ServeFile::new(web_dir.join("index.html"));

    let assets = Router::new()
        .fallback_service(ServeDir::new(web_dir.join("assets")))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static(IMMUTABLE),
        ));

    let pages = Router::new()
        .fallback_service(ServeDir::new(web_dir).fallback(index))
        .layer(SetResponseHeaderLayer::overriding(
            header::CACHE_CONTROL,
            HeaderValue::from_static(REVALIDATE),
        ));

    api.route("/api", any(not_found))
        .route("/api/{*rest}", any(not_found))
        .nest("/assets", assets)
        .merge(pages)
}

async fn not_found() -> StatusCode {
    StatusCode::NOT_FOUND
}
