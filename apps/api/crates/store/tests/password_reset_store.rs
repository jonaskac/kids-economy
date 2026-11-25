//! Password reset links against real Postgres (ADR-0012).

use domain::identity::{Email, User};
use domain::ids::UserId;
use domain::text::PersonName;
use sqlx::PgPool;
use store::{PasswordResetStore, SessionStore, StoreError, UserStore};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

const NOW: OffsetDateTime = datetime!(2026-11-25 13:00 UTC);

async fn registered(pool: &PgPool, n: u128, email: &str) -> User {
    let user = User {
        id: UserId::from_uuid(Uuid::from_u128(n)),
        email: Email::new(email).expect("email"),
        name: PersonName::new("Mormor").expect("name"),
        registered_at: NOW,
    };
    UserStore::new(pool.clone())
        .register(&user, "old-hash")
        .await
        .expect("register");
    user
}

#[sqlx::test]
async fn a_reset_link_is_found_by_its_hash_until_it_is_used(pool: PgPool) {
    let resets = PasswordResetStore::new(pool.clone());
    let mormor = registered(&pool, 1, "mormor@example.com").await;
    let expires = NOW + Duration::hours(24);
    resets
        .create("hash-a", mormor.id, expires)
        .await
        .expect("create");

    let found = resets.find("hash-a").await.expect("find").expect("found");
    assert_eq!(found.user_id, mormor.id);
    assert_eq!(found.expires_at, expires);
    assert_eq!(found.used_at, None);
    assert!(resets.find("other").await.expect("find").is_none());
}

#[sqlx::test]
async fn using_a_link_sets_the_password_and_ends_every_session(pool: PgPool) {
    let users = UserStore::new(pool.clone());
    let sessions = SessionStore::new(pool.clone());
    let resets = PasswordResetStore::new(pool.clone());
    let mormor = registered(&pool, 1, "mormor@example.com").await;
    let other = registered(&pool, 2, "jonas@example.com").await;
    sessions
        .start("phone", mormor.id, NOW + Duration::days(30))
        .await
        .expect("start");
    sessions
        .start("jonas-phone", other.id, NOW + Duration::days(30))
        .await
        .expect("start");
    resets
        .create("hash-a", mormor.id, NOW + Duration::hours(24))
        .await
        .expect("create");

    let user_id = resets
        .use_link("hash-a", "new-hash", NOW)
        .await
        .expect("use");

    assert_eq!(user_id, mormor.id);
    let record = users
        .find_by_email(&mormor.email)
        .await
        .expect("find")
        .expect("found");
    assert_eq!(record.password_hash, "new-hash");
    assert_eq!(record.user, mormor);
    assert_eq!(sessions.user_for("phone", NOW).await.expect("lookup"), None);
    assert_eq!(
        sessions.user_for("jonas-phone", NOW).await.expect("lookup"),
        Some(other)
    );
    let used = resets.find("hash-a").await.expect("find").expect("found");
    assert_eq!(used.used_at, Some(NOW));
}

#[sqlx::test]
async fn a_link_works_once_and_not_after_it_expires(pool: PgPool) {
    let resets = PasswordResetStore::new(pool.clone());
    let mormor = registered(&pool, 1, "mormor@example.com").await;
    let expires = NOW + Duration::hours(24);
    resets
        .create("once", mormor.id, expires)
        .await
        .expect("create");
    resets.use_link("once", "h1", NOW).await.expect("first use");
    let again = resets.use_link("once", "h2", NOW).await;
    assert!(
        matches!(again, Err(StoreError::PasswordResetNotPending)),
        "{again:?}"
    );

    resets
        .create("late", mormor.id, expires)
        .await
        .expect("create");
    let late = resets.use_link("late", "h3", expires).await;
    assert!(
        matches!(late, Err(StoreError::PasswordResetNotPending)),
        "{late:?}"
    );

    let unknown = resets.use_link("nope", "h4", NOW).await;
    assert!(
        matches!(unknown, Err(StoreError::PasswordResetNotPending)),
        "{unknown:?}"
    );
}

#[sqlx::test]
async fn a_new_link_replaces_an_unused_one_for_the_same_user(pool: PgPool) {
    let resets = PasswordResetStore::new(pool.clone());
    let mormor = registered(&pool, 1, "mormor@example.com").await;
    let expires = NOW + Duration::hours(24);
    resets
        .create("first", mormor.id, expires)
        .await
        .expect("create");
    resets
        .create("second", mormor.id, expires)
        .await
        .expect("create");
    assert!(resets.find("first").await.expect("find").is_none());
    assert!(resets.find("second").await.expect("find").is_some());
}
