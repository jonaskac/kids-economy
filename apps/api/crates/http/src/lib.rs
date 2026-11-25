//! HTTP layer: axum routes, DTOs and auth. Everything is mounted under `/api`.
//!
//! Sign-in uses email + password and a Session cookie; Invitations travel as links
//! (ADR-0011), and so do Password reset links (ADR-0012). Every Family route answers 404 to anyone who is not an active Member,
//! so Families stay invisible to each other (glossary: Family).

mod auth;
mod error;
mod families;
mod invitations;
mod password_resets;
mod users;
mod wallets;

pub use auth::{Passwords, SESSION_COOKIE};
pub use error::ApiError;

use axum::routing::{delete, get, patch, post};
use axum::{Json, Router};
use domain::ids::NewIds;
use serde::Serialize;
use sqlx::PgPool;
use store::{FamilyStore, PasswordResetStore, SessionStore, UserStore, WalletStore};
use uuid::Uuid;

/// What every handler needs: the stores and the password hasher.
#[derive(Clone)]
pub struct AppState {
    pub(crate) users: UserStore,
    pub(crate) sessions: SessionStore,
    pub(crate) password_resets: PasswordResetStore,
    pub(crate) families: FamilyStore,
    pub(crate) wallets: WalletStore,
    pub(crate) passwords: Passwords,
}

impl AppState {
    pub fn new(pool: PgPool, passwords: Passwords) -> Self {
        Self {
            users: UserStore::new(pool.clone()),
            sessions: SessionStore::new(pool.clone()),
            password_resets: PasswordResetStore::new(pool.clone()),
            families: FamilyStore::new(pool.clone()),
            wallets: WalletStore::new(pool),
            passwords,
        }
    }
}

/// Body of `GET /api/health`, used by Sliplane's health check (ADR-0007).
#[derive(Debug, Serialize)]
pub struct Health {
    pub status: &'static str,
}

/// Builds the API router. The server mounts it at the root; routes carry the `/api` prefix.
pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/api/health", get(health))
        .route("/api/users", post(users::sign_up))
        .route("/api/sessions", post(users::sign_in))
        .route("/api/sessions/current", delete(users::sign_out))
        .route("/api/me", get(users::me))
        .route("/api/families", post(families::create))
        .route("/api/families/{family_id}", get(families::view))
        .route(
            "/api/families/{family_id}/invitations",
            post(families::invite),
        )
        .route(
            "/api/families/{family_id}/invitations/{invitation_id}",
            delete(families::revoke),
        )
        .route(
            "/api/families/{family_id}/members/{member_id}",
            patch(families::change_role).delete(families::remove_member),
        )
        .route(
            "/api/families/{family_id}/members/{member_id}/password-reset",
            post(password_resets::create),
        )
        .route("/api/families/{family_id}/kids", post(families::add_kid))
        .route(
            "/api/families/{family_id}/kids/{kid_id}/archive",
            post(families::archive_kid),
        )
        .route(
            "/api/families/{family_id}/wallets/{wallet_id}",
            get(wallets::view),
        )
        .route(
            "/api/families/{family_id}/wallets/{wallet_id}/deposits",
            post(wallets::deposit),
        )
        .route(
            "/api/families/{family_id}/wallets/{wallet_id}/withdrawals",
            post(wallets::withdraw),
        )
        .route(
            "/api/families/{family_id}/wallets/{wallet_id}/transfers",
            post(wallets::transfer),
        )
        .route(
            "/api/families/{family_id}/wallets/{wallet_id}/corrections",
            post(wallets::correct),
        )
        .route(
            "/api/families/{family_id}/wallets/{wallet_id}/accounts",
            post(wallets::open_account),
        )
        .route("/api/invitations/{secret}", get(invitations::preview))
        .route(
            "/api/invitations/{secret}/accept",
            post(invitations::accept),
        )
        .route(
            "/api/password-resets/{secret}",
            get(password_resets::preview).post(password_resets::reset),
        )
        .with_state(state)
}

async fn health() -> Json<Health> {
    Json(Health { status: "ok" })
}

/// Fresh random ids (UUID v4) for the domain's decisions.
pub(crate) struct RandomIds;

impl NewIds for RandomIds {
    fn next_id(&mut self) -> Uuid {
        Uuid::new_v4()
    }
}
