//! Passwords, Session secrets and the Session cookie (ADR-0011).

use crate::error::ApiError;
use crate::AppState;
use argon2::password_hash::rand_core::OsRng;
use argon2::password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString};
use argon2::{Algorithm, Argon2, Params, Version};
use axum::extract::FromRequestParts;
use axum::http::header::{HeaderValue, COOKIE};
use axum::http::request::Parts;
use axum::http::HeaderMap;
use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;
use domain::identity::{Password, User};
use rand::RngCore;
use sha2::{Digest, Sha256};
use std::sync::Arc;
use time::{Duration, OffsetDateTime};

/// Name of the cookie that carries the Session secret.
pub const SESSION_COOKIE: &str = "session";

/// How long a Session lasts after sign-in.
pub(crate) const SESSION_LIFETIME: Duration = Duration::days(30);

/// Hashes and verifies passwords with Argon2id.
#[derive(Clone)]
pub struct Passwords {
    params: Params,
    /// A hash of nothing in particular, verified when the email is unknown so that a
    /// failed sign-in takes as long whether or not the account exists.
    decoy: Arc<String>,
}

impl Default for Passwords {
    /// OWASP's recommended Argon2id cost: 19 MiB, 2 passes, 1 lane.
    fn default() -> Self {
        Self::from_params(Params::default())
    }
}

impl Passwords {
    /// A custom cost (memory in KiB, passes, lanes). Low costs are only for tests.
    pub fn with_cost(memory_kib: u32, passes: u32, lanes: u32) -> Result<Self, argon2::Error> {
        Ok(Self::from_params(Params::new(
            memory_kib, passes, lanes, None,
        )?))
    }

    fn from_params(params: Params) -> Self {
        let mut passwords = Self {
            params,
            decoy: Arc::new(String::new()),
        };
        let decoy = passwords
            .hash_now("decoy password, never valid")
            .unwrap_or_default();
        passwords.decoy = Arc::new(decoy);
        passwords
    }

    fn argon2(&self) -> Argon2<'static> {
        Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone())
    }

    fn hash_now(&self, password: &str) -> Result<String, argon2::password_hash::Error> {
        let salt = SaltString::generate(&mut OsRng);
        Ok(self
            .argon2()
            .hash_password(password.as_bytes(), &salt)?
            .to_string())
    }

    /// The PHC string to store. Runs off the async threads: hashing is slow on purpose.
    pub(crate) async fn hash(&self, password: &Password) -> Result<String, ApiError> {
        let this = self.clone();
        let password = password.expose().to_owned();
        tokio::task::spawn_blocking(move || this.hash_now(&password))
            .await
            .map_err(|_| ApiError::Internal)?
            .map_err(|_| ApiError::Internal)
    }

    /// Whether `password` matches the stored hash; `None` checks against the decoy.
    pub(crate) async fn verify(&self, password: &str, stored: Option<String>) -> bool {
        let this = self.clone();
        let password = password.to_owned();
        let known = stored.is_some();
        let stored = stored.unwrap_or_else(|| this.decoy.as_str().to_owned());
        let matches = tokio::task::spawn_blocking(move || {
            PasswordHash::new(&stored)
                .map(|hash| {
                    this.argon2()
                        .verify_password(password.as_bytes(), &hash)
                        .is_ok()
                })
                .unwrap_or(false)
        })
        .await
        .unwrap_or(false);
        known && matches
    }
}

/// A fresh Session secret (32 random bytes, URL-safe base64) and the hash to store.
pub(crate) fn new_secret() -> (String, String) {
    let mut bytes = [0_u8; 32];
    rand::rngs::OsRng.fill_bytes(&mut bytes);
    let secret = URL_SAFE_NO_PAD.encode(bytes);
    let hash = hash_secret(&secret);
    (secret, hash)
}

/// SHA-256 of a Session or Invitation link secret, as lower-case hex.
pub(crate) fn hash_secret(secret: &str) -> String {
    Sha256::digest(secret.as_bytes())
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

/// `Set-Cookie` value that stores the Session secret on the device.
pub(crate) fn session_cookie(secret: &str) -> HeaderValue {
    let value = format!(
        "{SESSION_COOKIE}={secret}; HttpOnly; Secure; SameSite=Lax; Path=/api; Max-Age={}",
        SESSION_LIFETIME.whole_seconds()
    );
    // A base64url secret is always a valid header value.
    HeaderValue::from_str(&value).unwrap_or_else(|_| HeaderValue::from_static(""))
}

/// `Set-Cookie` value that removes the Session cookie.
pub(crate) fn cleared_cookie() -> HeaderValue {
    HeaderValue::from_static("session=; HttpOnly; Secure; SameSite=Lax; Path=/api; Max-Age=0")
}

/// The Session secret from a request's `Cookie` headers, if any.
pub(crate) fn session_secret(headers: &HeaderMap) -> Option<String> {
    headers
        .get_all(COOKIE)
        .iter()
        .filter_map(|value| value.to_str().ok())
        .flat_map(|header| header.split(';'))
        .filter_map(|pair| pair.trim().split_once('='))
        .find(|(name, value)| *name == SESSION_COOKIE && !value.is_empty())
        .map(|(_, value)| value.to_owned())
}

/// A request from a signed-in User. Rejects with 401 otherwise.
pub(crate) struct SignedIn {
    pub user: User,
}

impl FromRequestParts<AppState> for SignedIn {
    type Rejection = ApiError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> Result<Self, ApiError> {
        let secret = session_secret(&parts.headers).ok_or(ApiError::NotSignedIn)?;
        let token_hash = hash_secret(&secret);
        let user = state
            .sessions
            .user_for(&token_hash, OffsetDateTime::now_utc())
            .await?
            .ok_or(ApiError::NotSignedIn)?;
        Ok(SignedIn { user })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use axum::http::Request;

    fn parts(cookies: &[&str]) -> HeaderMap {
        let mut request = Request::builder();
        for cookie in cookies {
            request = request.header(COOKIE, *cookie);
        }
        request.body(()).expect("request").into_parts().0.headers
    }

    #[test]
    fn the_session_secret_is_read_from_any_cookie_header() {
        assert_eq!(session_secret(&parts(&[])), None);
        assert_eq!(session_secret(&parts(&["theme=dark"])), None);
        assert_eq!(session_secret(&parts(&["session="])), None);
        assert_eq!(
            session_secret(&parts(&["theme=dark; session=abc"])),
            Some("abc".to_owned())
        );
        assert_eq!(
            session_secret(&parts(&["theme=dark", "session=xyz"])),
            Some("xyz".to_owned())
        );
        assert_eq!(session_secret(&parts(&["mysession=abc"])), None);
    }

    #[test]
    fn secrets_are_random_and_stored_only_as_their_hash() {
        let (a, a_hash) = new_secret();
        let (b, _) = new_secret();
        assert_ne!(a, b);
        assert_eq!(a.len(), 43);
        assert_eq!(a_hash, hash_secret(&a));
        assert_eq!(a_hash.len(), 64);
        assert!(!a_hash.contains(&a));
    }

    #[tokio::test]
    async fn a_password_verifies_only_against_its_own_hash() {
        let passwords = Passwords::with_cost(8, 1, 1).expect("params");
        let password = Password::new("correct horse battery").expect("password");
        let hash = passwords.hash(&password).await.expect("hash");
        assert!(hash.starts_with("$argon2id$"));
        assert!(
            passwords
                .verify("correct horse battery", Some(hash.clone()))
                .await
        );
        assert!(!passwords.verify("wrong horse battery", Some(hash)).await);
        assert!(!passwords.verify("decoy password, never valid", None).await);
    }
}
