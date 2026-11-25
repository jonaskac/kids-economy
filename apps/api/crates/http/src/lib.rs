//! HTTP layer: axum routes and DTOs. Everything is mounted under `/api`.

use axum::{routing::get, Json, Router};
use serde::Serialize;

/// Body of `GET /api/health`, used by Sliplane's health check (ADR-0007).
#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
}

/// Builds the API router. The server mounts it at the root; routes carry the `/api` prefix.
pub fn router() -> Router {
    Router::new().route("/api/health", get(health))
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

#[cfg(test)]
mod tests {
    use super::router;
    use axum::body::Body;
    use axum::http::{Request, StatusCode};
    use http_body_util::BodyExt;
    use tower::ServiceExt;

    #[tokio::test]
    async fn health_reports_ok() {
        let response = router()
            .oneshot(
                Request::get("/api/health")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::OK);
        let bytes = response
            .into_body()
            .collect()
            .await
            .expect("body")
            .to_bytes();
        let json: serde_json::Value = serde_json::from_slice(&bytes).expect("json");
        assert_eq!(json, serde_json::json!({ "status": "ok" }));
    }

    #[tokio::test]
    async fn unknown_route_is_not_found() {
        let response = router()
            .oneshot(
                Request::get("/api/nope")
                    .body(Body::empty())
                    .expect("request"),
            )
            .await
            .expect("response");

        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }
}
