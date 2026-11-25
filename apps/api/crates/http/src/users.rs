//! Signing up, in and out, and who is signed in (ADR-0011).

use crate::auth::{
    cleared_cookie, hash_secret, new_secret, session_cookie, session_secret, SignedIn,
    SESSION_LIFETIME,
};
use crate::error::ApiError;
use crate::families::role_name;
use crate::AppState;
use axum::extract::State;
use axum::http::header::SET_COOKIE;
use axum::http::{HeaderMap, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::Json;
use domain::identity::{Email, Password, User};
use domain::ids::UserId;
use domain::text::PersonName;
use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Deserialize)]
pub struct SignUp {
    email: String,
    name: String,
    password: String,
}

#[derive(Debug, Deserialize)]
pub struct SignIn {
    email: String,
    password: String,
}

#[derive(Debug, Serialize)]
pub struct UserDto {
    id: Uuid,
    email: String,
    name: String,
}

impl From<&User> for UserDto {
    fn from(user: &User) -> Self {
        Self {
            id: user.id.as_uuid(),
            email: user.email.as_str().to_owned(),
            name: user.name.as_str().to_owned(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct Membership {
    id: Uuid,
    name: String,
    role: &'static str,
}

#[derive(Debug, Serialize)]
pub struct Me {
    user: UserDto,
    families: Vec<Membership>,
}

/// `POST /api/users`: registers a User and signs them in.
pub async fn sign_up(
    State(state): State<AppState>,
    Json(body): Json<SignUp>,
) -> Result<Response, ApiError> {
    let email = Email::new(&body.email)?;
    let name = PersonName::new(&body.name)?;
    let password = Password::new(body.password)?;
    let user = User {
        id: UserId::from_uuid(Uuid::new_v4()),
        email,
        name,
        registered_at: OffsetDateTime::now_utc(),
    };
    let hash = state.passwords.hash(&password).await?;
    state.users.register(&user, &hash).await?;
    signed_in(&state, StatusCode::CREATED, &user).await
}

/// `POST /api/sessions`: signs in with email and password.
pub async fn sign_in(
    State(state): State<AppState>,
    Json(body): Json<SignIn>,
) -> Result<Response, ApiError> {
    let record = match Email::new(&body.email) {
        Ok(email) => state.users.find_by_email(&email).await?,
        Err(_) => None,
    };
    let stored = record.as_ref().map(|r| r.password_hash.clone());
    if !state.passwords.verify(&body.password, stored).await {
        return Err(ApiError::WrongEmailOrPassword);
    }
    let user = record
        .map(|r| r.user)
        .ok_or(ApiError::WrongEmailOrPassword)?;
    signed_in(&state, StatusCode::OK, &user).await
}

/// `DELETE /api/sessions/current`: signs out on this device.
pub async fn sign_out(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> Result<Response, ApiError> {
    if let Some(secret) = session_secret(&headers) {
        state.sessions.end(&hash_secret(&secret)).await?;
    }
    Ok((StatusCode::NO_CONTENT, [(SET_COOKIE, cleared_cookie())]).into_response())
}

/// `GET /api/me`: the signed-in User and the Families they are a Member of.
pub async fn me(State(state): State<AppState>, signed_in: SignedIn) -> Result<Json<Me>, ApiError> {
    let families = state.families.of_user(signed_in.user.id).await?;
    let families = families
        .iter()
        .filter_map(|family| {
            family.actor_for(signed_in.user.id).map(|actor| Membership {
                id: family.id().as_uuid(),
                name: family.name().as_str().to_owned(),
                role: role_name(actor.role),
            })
        })
        .collect();
    Ok(Json(Me {
        user: UserDto::from(&signed_in.user),
        families,
    }))
}

pub(crate) async fn signed_in(
    state: &AppState,
    status: StatusCode,
    user: &User,
) -> Result<Response, ApiError> {
    let (secret, hash) = new_secret();
    let expires_at = OffsetDateTime::now_utc() + SESSION_LIFETIME;
    state.sessions.start(&hash, user.id, expires_at).await?;
    let body = serde_json::json!({ "user": UserDto::from(user) });
    Ok((status, [(SET_COOKIE, session_cookie(&secret))], Json(body)).into_response())
}
