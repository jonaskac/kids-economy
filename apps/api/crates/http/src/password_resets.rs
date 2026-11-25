//! Password reset links (ADR-0012): an Owner makes one for a Member of their Family and
//! shares it; whoever opens it within 24 hours chooses a new password, once.

use crate::auth::{hash_secret, new_secret, SignedIn};
use crate::error::ApiError;
use crate::families::load_as_member;
use crate::users::signed_in;
use crate::AppState;
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::response::Response;
use axum::Json;
use domain::identity::Password;
use domain::ids::MemberId;
use serde::{Deserialize, Serialize};
use store::PasswordReset;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

/// How long a Password reset link works after it was made.
const PASSWORD_RESET_LIFETIME: Duration = Duration::hours(24);

#[derive(Debug, Serialize)]
pub struct NewLink {
    /// `/reset/<secret>`, shown once.
    link: String,
    #[serde(with = "time::serde::rfc3339")]
    expires_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct Preview {
    email: String,
    name: String,
    state: &'static str,
    #[serde(with = "time::serde::rfc3339")]
    expires_at: OffsetDateTime,
}

#[derive(Debug, Deserialize)]
pub struct NewPassword {
    password: String,
}

/// `POST /api/families/{id}/members/{member_id}/password-reset`: Owners only.
pub async fn create(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path((family_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<(StatusCode, Json<NewLink>), ApiError> {
    let (loaded, actor) = load_as_member(&state, family_id, &signed_in.user).await?;
    let member = loaded
        .family
        .member_for_password_reset(actor, MemberId::from_uuid(member_id))?;
    let (secret, hash) = new_secret();
    let expires_at = OffsetDateTime::now_utc() + PASSWORD_RESET_LIFETIME;
    state
        .password_resets
        .create(&hash, member.user_id, expires_at)
        .await?;
    Ok((
        StatusCode::CREATED,
        Json(NewLink {
            link: format!("/reset/{secret}"),
            expires_at,
        }),
    ))
}

/// `GET /api/password-resets/{secret}`: who the link is for, and whether it still works.
pub async fn preview(
    State(state): State<AppState>,
    Path(secret): Path<String>,
) -> Result<Json<Preview>, ApiError> {
    let reset = find(&state, &secret).await?;
    let user = state
        .users
        .get(reset.user_id)
        .await?
        .ok_or(ApiError::UnknownPasswordReset)?;
    Ok(Json(Preview {
        email: user.email.as_str().to_owned(),
        name: user.name.as_str().to_owned(),
        state: ResetState::of(&reset, OffsetDateTime::now_utc()).name(),
        expires_at: reset.expires_at,
    }))
}

/// `POST /api/password-resets/{secret}`: sets the new password, ends every other Session
/// of that User and signs them in here.
pub async fn reset(
    State(state): State<AppState>,
    Path(secret): Path<String>,
    Json(body): Json<NewPassword>,
) -> Result<Response, ApiError> {
    let reset = find(&state, &secret).await?;
    let now = OffsetDateTime::now_utc();
    match ResetState::of(&reset, now) {
        ResetState::Used => return Err(ApiError::PasswordResetUsed),
        ResetState::Expired => return Err(ApiError::PasswordResetExpired),
        ResetState::Pending => {}
    }
    let password = Password::new(body.password)?;
    let hash = state.passwords.hash(&password).await?;
    let user_id = state
        .password_resets
        .use_link(&hash_secret(&secret), &hash, now)
        .await?;
    let user = state.users.get(user_id).await?.ok_or(ApiError::Internal)?;
    signed_in(&state, StatusCode::OK, &user).await
}

async fn find(state: &AppState, secret: &str) -> Result<PasswordReset, ApiError> {
    state
        .password_resets
        .find(&hash_secret(secret))
        .await?
        .ok_or(ApiError::UnknownPasswordReset)
}

/// Whether a Password reset link still works.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ResetState {
    Pending,
    Used,
    Expired,
}

impl ResetState {
    fn of(reset: &PasswordReset, now: OffsetDateTime) -> Self {
        if reset.used_at.is_some() {
            Self::Used
        } else if reset.expires_at <= now {
            Self::Expired
        } else {
            Self::Pending
        }
    }

    const fn name(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Used => "used",
            Self::Expired => "expired",
        }
    }
}
