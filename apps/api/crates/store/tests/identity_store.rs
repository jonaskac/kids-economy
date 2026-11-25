//! Users and Sessions against real Postgres (ADR-0010, ADR-0011).

use domain::identity::{Email, User};
use domain::ids::UserId;
use domain::text::PersonName;
use sqlx::PgPool;
use store::{SessionStore, StoreError, UserStore};
use time::macros::datetime;
use time::Duration;
use uuid::Uuid;

fn user(n: u128, email: &str) -> User {
    User {
        id: UserId::from_uuid(Uuid::from_u128(n)),
        email: Email::new(email).expect("email"),
        name: PersonName::new("Jonas").expect("name"),
        registered_at: datetime!(2026-11-25 13:00 UTC),
    }
}

#[sqlx::test]
async fn a_registered_user_is_found_by_email_with_their_password_hash(pool: PgPool) {
    let users = UserStore::new(pool);
    let jonas = user(1, "jonas@example.com");
    users
        .register(&jonas, "$argon2id$hash")
        .await
        .expect("register");

    let found = users
        .find_by_email(&Email::new("JONAS@example.com").expect("email"))
        .await
        .expect("find")
        .expect("found");
    assert_eq!(found.user, jonas);
    assert_eq!(found.password_hash, "$argon2id$hash");
    assert_eq!(users.get(jonas.id).await.expect("get"), Some(jonas));
}

#[sqlx::test]
async fn an_email_belongs_to_one_user(pool: PgPool) {
    let users = UserStore::new(pool);
    users
        .register(&user(1, "jonas@example.com"), "h1")
        .await
        .expect("register");
    let again = users.register(&user(2, "jonas@example.com"), "h2").await;
    assert!(matches!(again, Err(StoreError::EmailTaken)), "{again:?}");
}

#[sqlx::test]
async fn an_unknown_email_finds_no_user(pool: PgPool) {
    let users = UserStore::new(pool);
    let found = users
        .find_by_email(&Email::new("nobody@example.com").expect("email"))
        .await
        .expect("find");
    assert!(found.is_none());
}

#[sqlx::test]
async fn a_session_finds_its_user_until_it_expires_or_ends(pool: PgPool) {
    let users = UserStore::new(pool.clone());
    let sessions = SessionStore::new(pool);
    let jonas = user(1, "jonas@example.com");
    users.register(&jonas, "h").await.expect("register");
    let now = datetime!(2026-11-25 13:00 UTC);
    let expires = now + Duration::days(30);

    sessions
        .start("hash-a", jonas.id, expires)
        .await
        .expect("start");
    assert_eq!(
        sessions.user_for("hash-a", now).await.expect("lookup"),
        Some(jonas.clone())
    );
    assert_eq!(
        sessions.user_for("hash-a", expires).await.expect("lookup"),
        None
    );
    assert_eq!(sessions.user_for("other", now).await.expect("lookup"), None);

    sessions.end("hash-a").await.expect("end");
    assert_eq!(
        sessions.user_for("hash-a", now).await.expect("lookup"),
        None
    );
}
