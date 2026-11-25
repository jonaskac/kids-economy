//! How the API says no: a status and `{"error": "<code>", "message": "…"}`.

use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::Json;
use domain::family::FamilyError;
use domain::identity::{EmailError, PasswordError};
use domain::money::MoneyError;
use domain::text::TextError;
use domain::wallet::WalletError;
use serde_json::json;
use store::StoreError;

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("sign in first")]
    NotSignedIn,
    #[error("wrong email or password")]
    WrongEmailOrPassword,
    #[error("the email address is already registered")]
    EmailTaken,
    #[error("no such Family")]
    FamilyNotFound,
    #[error("someone else changed this at the same time; try again")]
    Conflict,
    #[error(transparent)]
    Email(#[from] EmailError),
    #[error(transparent)]
    Password(#[from] PasswordError),
    #[error(transparent)]
    Text(#[from] TextError),
    #[error(transparent)]
    Money(#[from] MoneyError),
    #[error(transparent)]
    Family(#[from] FamilyError),
    #[error(transparent)]
    Wallet(#[from] WalletError),
    #[error("something went wrong on our side")]
    Internal,
}

impl From<StoreError> for ApiError {
    fn from(error: StoreError) -> Self {
        match error {
            StoreError::Conflict => ApiError::Conflict,
            StoreError::EmailTaken => ApiError::EmailTaken,
            other => {
                tracing::error!(error = %other, "store failed");
                ApiError::Internal
            }
        }
    }
}

impl ApiError {
    fn status_and_code(&self) -> (StatusCode, &'static str) {
        use StatusCode as S;
        match self {
            ApiError::NotSignedIn => (S::UNAUTHORIZED, "not_signed_in"),
            ApiError::WrongEmailOrPassword => (S::UNAUTHORIZED, "wrong_email_or_password"),
            ApiError::EmailTaken => (S::CONFLICT, "email_taken"),
            ApiError::FamilyNotFound => (S::NOT_FOUND, "family_not_found"),
            ApiError::Conflict => (S::CONFLICT, "conflict"),
            ApiError::Email(_) => (S::UNPROCESSABLE_ENTITY, "invalid_email"),
            ApiError::Password(PasswordError::TooShort { .. }) => {
                (S::UNPROCESSABLE_ENTITY, "password_too_short")
            }
            ApiError::Password(PasswordError::TooLong { .. }) => {
                (S::UNPROCESSABLE_ENTITY, "password_too_long")
            }
            ApiError::Text(_) => (S::UNPROCESSABLE_ENTITY, "invalid_text"),
            ApiError::Money(MoneyError::InvalidCurrency) => {
                (S::UNPROCESSABLE_ENTITY, "invalid_currency")
            }
            ApiError::Money(MoneyError::NotPositive) => (S::UNPROCESSABLE_ENTITY, "invalid_amount"),
            ApiError::Family(error) => family_status(*error),
            ApiError::Wallet(WalletError::NotAllowed) => (S::FORBIDDEN, "not_allowed"),
            ApiError::Wallet(_) => (S::UNPROCESSABLE_ENTITY, "wallet_rejected"),
            ApiError::Internal => (S::INTERNAL_SERVER_ERROR, "internal"),
        }
    }
}

fn family_status(error: FamilyError) -> (StatusCode, &'static str) {
    use StatusCode as S;
    match error {
        FamilyError::NotAllowed => (S::FORBIDDEN, "not_allowed"),
        FamilyError::NotAMember => (S::NOT_FOUND, "family_not_found"),
        FamilyError::AlreadyMember => (S::CONFLICT, "already_member"),
        FamilyError::InvitationPending => (S::CONFLICT, "invitation_pending"),
        FamilyError::UnknownInvitation => (S::NOT_FOUND, "unknown_invitation"),
        FamilyError::InvitationNotPending => (S::CONFLICT, "invitation_not_pending"),
        FamilyError::InvitationExpired => (S::GONE, "invitation_expired"),
        FamilyError::WrongEmail => (S::FORBIDDEN, "wrong_email"),
        FamilyError::UnknownMember => (S::NOT_FOUND, "unknown_member"),
        FamilyError::LastOwner => (S::CONFLICT, "last_owner"),
        FamilyError::UnknownKid => (S::NOT_FOUND, "unknown_kid"),
        FamilyError::KidArchived => (S::CONFLICT, "kid_archived"),
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, code) = self.status_and_code();
        let body = Json(json!({ "error": code, "message": self.to_string() }));
        (status, body).into_response()
    }
}
