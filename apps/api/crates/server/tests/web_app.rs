//! The server serves the built web app next to the API, in-process (`oneshot`).

use std::fs;

use axum::body::Body;
use axum::http::{header, Request, StatusCode};
use axum::routing::get;
use axum::Router;
use http_body_util::BodyExt;
use server::web::with_web_app;
use tempfile::TempDir;
use tower::ServiceExt;

/// A built web app: `index.html` plus one hashed asset, as `vite build` writes them.
fn built_web_app() -> TempDir {
    let dir = tempfile::tempdir().expect("temp dir");
    fs::write(
        dir.path().join("index.html"),
        "<!doctype html><title>Kids</title>",
    )
    .expect("write");
    fs::create_dir(dir.path().join("assets")).expect("mkdir");
    fs::write(dir.path().join("assets/index-abc123.js"), "console.log(1)").expect("write");
    dir
}

fn app(dir: &TempDir) -> Router {
    let api = Router::new().route("/api/health", get(|| async { "ok" }));
    with_web_app(api, dir.path())
}

struct Reply {
    status: StatusCode,
    cache_control: Option<String>,
    content_type: Option<String>,
    body: String,
}

async fn get_path(app: Router, path: &str) -> Reply {
    let response = app
        .oneshot(Request::get(path).body(Body::empty()).expect("request"))
        .await
        .expect("response");
    let header_value = |name| {
        response
            .headers()
            .get(name)
            .and_then(|v| v.to_str().ok())
            .map(str::to_owned)
    };
    let cache_control = header_value(header::CACHE_CONTROL);
    let content_type = header_value(header::CONTENT_TYPE);
    let status = response.status();
    let bytes = response
        .into_body()
        .collect()
        .await
        .expect("body")
        .to_bytes();
    Reply {
        status,
        cache_control,
        content_type,
        body: String::from_utf8_lossy(&bytes).into_owned(),
    }
}

#[tokio::test]
async fn the_api_still_answers() {
    let dir = built_web_app();
    let reply = get_path(app(&dir), "/api/health").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, "ok");
}

#[tokio::test]
async fn the_root_serves_the_web_app_and_is_always_revalidated() {
    let dir = built_web_app();
    let reply = get_path(app(&dir), "/").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert!(reply.body.contains("<title>Kids</title>"));
    assert_eq!(reply.cache_control.as_deref(), Some("no-cache"));
}

#[tokio::test]
async fn app_paths_like_join_links_serve_the_web_app() {
    let dir = built_web_app();
    for path in ["/join/s3cret", "/reset/s3cret", "/families/f1/wallets/w1"] {
        let reply = get_path(app(&dir), path).await;
        assert_eq!(reply.status, StatusCode::OK, "{path}");
        assert!(reply.body.contains("<title>Kids</title>"), "{path}");
        assert!(
            reply
                .content_type
                .as_deref()
                .is_some_and(|t| t.starts_with("text/html")),
            "{path}: {:?}",
            reply.content_type
        );
        assert_eq!(reply.cache_control.as_deref(), Some("no-cache"), "{path}");
    }
}

#[tokio::test]
async fn hashed_assets_are_cached_for_a_year() {
    let dir = built_web_app();
    let reply = get_path(app(&dir), "/assets/index-abc123.js").await;
    assert_eq!(reply.status, StatusCode::OK);
    assert_eq!(reply.body, "console.log(1)");
    assert_eq!(
        reply.cache_control.as_deref(),
        Some("public, max-age=31536000, immutable")
    );
}

#[tokio::test]
async fn a_missing_asset_is_not_found_rather_than_the_web_app() {
    let dir = built_web_app();
    let reply = get_path(app(&dir), "/assets/index-old.js").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
}

#[tokio::test]
async fn an_unknown_api_path_is_not_found_rather_than_the_web_app() {
    let dir = built_web_app();
    let reply = get_path(app(&dir), "/api/nope").await;
    assert_eq!(reply.status, StatusCode::NOT_FOUND);
    assert!(!reply.body.contains("<title>"));
}
