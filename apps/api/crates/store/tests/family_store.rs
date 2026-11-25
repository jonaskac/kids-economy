//! The Family store against real Postgres (ADR-0010).

use domain::family::{
    AcceptInvitation, CreateFamily, Family, FamilyCommand, InvitationLinkHash, Kid, Person,
};
use domain::identity::Email;
use domain::ids::{FamilyId, NewIds, UserId};
use domain::money::Currency;
use domain::role::{Context, Role};
use domain::text::{AccountName, FamilyName, KidName, PersonName};
use domain::wallet::{OpenWallet, Wallet};
use sqlx::PgPool;
use store::{FamilyStore, StoreError, Version, WalletStore};
use time::macros::datetime;
use time::OffsetDateTime;
use uuid::Uuid;

struct SequentialIds(u128);

impl NewIds for SequentialIds {
    fn next_id(&mut self) -> Uuid {
        self.0 += 1;
        Uuid::from_u128(self.0)
    }
}

const NOW: OffsetDateTime = datetime!(2026-11-25 13:00 UTC);

fn person(n: u128, email: &str) -> Person {
    Person {
        user_id: UserId::from_uuid(Uuid::from_u128(5_000 + n)),
        name: PersonName::new(format!("Adult {n}")).expect("name"),
        email: Email::new(email).expect("email"),
    }
}

fn founded(family: u128, founder: &Person, ids: &mut SequentialIds) -> Family {
    let created = Family::create(
        &CreateFamily {
            family_id: FamilyId::from_uuid(Uuid::from_u128(family)),
            name: FamilyName::new("The Kacs").expect("name"),
            currency: Currency::new("SEK").expect("currency"),
            founder: founder.clone(),
        },
        NOW,
        ids,
    );
    Family::replay([&created]).expect("replay")
}

fn decided(
    family: &Family,
    founder: &Person,
    command: FamilyCommand,
    ids: &mut SequentialIds,
) -> Family {
    let actor = family.actor_for(founder.user_id).expect("member");
    let events = family
        .decide(&command, &Context { actor, now: NOW }, ids)
        .expect("decide");
    let mut next = family.clone();
    for event in &events {
        next.apply(event).expect("apply");
    }
    next
}

fn link(secret: &str) -> InvitationLinkHash {
    InvitationLinkHash::new(format!("hash-{secret}")).expect("hash")
}

#[sqlx::test]
async fn a_created_family_is_loaded_back_at_version_one(pool: PgPool) {
    let store = FamilyStore::new(pool);
    let mut ids = SequentialIds(0);
    let founder = person(1, "owner@example.com");
    let family = founded(900, &founder, &mut ids);

    let version = store.create(&family).await.expect("create");
    assert_eq!(version, Version::from_count(1));

    let loaded = store.load(family.id()).await.expect("load").expect("found");
    assert_eq!(loaded.family, family);
    assert_eq!(loaded.version, version);
}

#[sqlx::test]
async fn an_unknown_family_is_not_found(pool: PgPool) {
    let store = FamilyStore::new(pool);
    let loaded = store
        .load(FamilyId::from_uuid(Uuid::from_u128(404)))
        .await
        .expect("load");
    assert!(loaded.is_none());
}

#[sqlx::test]
async fn a_save_moves_the_version_on_and_a_stale_save_is_a_conflict(pool: PgPool) {
    let store = FamilyStore::new(pool);
    let mut ids = SequentialIds(0);
    let founder = person(1, "owner@example.com");
    let family = founded(900, &founder, &mut ids);
    let v1 = store.create(&family).await.expect("create");

    let invite = |email: &str, secret: &str| FamilyCommand::Invite {
        email: Email::new(email).expect("email"),
        role: Role::Viewer,
        link_hash: link(secret),
    };
    let first = decided(&family, &founder, invite("a@example.com", "a"), &mut ids);
    let second = decided(&family, &founder, invite("b@example.com", "b"), &mut ids);

    let v2 = store.save(&first, v1).await.expect("save");
    assert_eq!(v2, Version::from_count(2));
    let stale = store.save(&second, v1).await;
    assert!(matches!(stale, Err(StoreError::Conflict)), "{stale:?}");

    let loaded = store.load(family.id()).await.expect("load").expect("found");
    assert_eq!(loaded.family, first);
}

#[sqlx::test]
async fn families_of_a_user_are_those_they_are_an_active_member_of(pool: PgPool) {
    let store = FamilyStore::new(pool);
    let mut ids = SequentialIds(0);
    let jonas = person(1, "jonas@example.com");
    let grandma = person(2, "grandma@example.com");
    let ours = founded(900, &jonas, &mut ids);
    let theirs = founded(901, &grandma, &mut ids);
    let v = store.create(&ours).await.expect("create");
    store.create(&theirs).await.expect("create");

    let found = store.of_user(jonas.user_id).await.expect("of user");
    assert_eq!(found, vec![ours.clone()]);

    // Jonas joins grandma's Family as a Viewer, then leaves his own? Not allowed (last
    // Owner), so a second Owner joins first.
    let with_invite = decided(
        &ours,
        &jonas,
        FamilyCommand::Invite {
            email: Email::new("partner@example.com").expect("email"),
            role: Role::Owner,
            link_hash: link("p"),
        },
        &mut ids,
    );
    let partner = person(3, "partner@example.com");
    let events = with_invite
        .accept(
            &AcceptInvitation {
                link_hash: link("p"),
                person: partner,
            },
            NOW,
            &mut ids,
        )
        .expect("accept");
    let mut joined = with_invite.clone();
    joined.apply(&events[0]).expect("apply");
    let jonas_member = joined.actor_for(jonas.user_id).expect("member").member_id;
    let left = decided(
        &joined,
        &jonas,
        FamilyCommand::RemoveMember {
            member_id: jonas_member,
        },
        &mut ids,
    );
    store.save(&left, v).await.expect("save");

    let found = store.of_user(jonas.user_id).await.expect("of user");
    assert!(
        found.is_empty(),
        "a removed Member no longer sees the Family"
    );
}

#[sqlx::test]
async fn a_family_is_found_by_an_invitation_link(pool: PgPool) {
    let store = FamilyStore::new(pool);
    let mut ids = SequentialIds(0);
    let founder = person(1, "owner@example.com");
    let family = founded(900, &founder, &mut ids);
    let v1 = store.create(&family).await.expect("create");
    let invited = decided(
        &family,
        &founder,
        FamilyCommand::Invite {
            email: Email::new("grandma@example.com").expect("email"),
            role: Role::Viewer,
            link_hash: link("secret"),
        },
        &mut ids,
    );
    store.save(&invited, v1).await.expect("save");

    let found = store
        .by_invitation_link(&link("secret"))
        .await
        .expect("lookup")
        .expect("found");
    assert_eq!(found.family.id(), family.id());
    assert_eq!(found.version, Version::from_count(2));
    assert!(store
        .by_invitation_link(&link("other"))
        .await
        .expect("lookup")
        .is_none());
}

#[sqlx::test]
async fn adding_a_kid_opens_their_wallet_in_the_same_save(pool: PgPool) {
    let families = FamilyStore::new(pool.clone());
    let wallets = WalletStore::new(pool);
    let mut ids = SequentialIds(0);
    let founder = person(1, "owner@example.com");
    let family = founded(900, &founder, &mut ids);
    let v1 = families.create(&family).await.expect("create");

    let with_kid = decided(
        &family,
        &founder,
        FamilyCommand::AddKid {
            name: KidName::new("Elsa").expect("name"),
        },
        &mut ids,
    );
    let kid: Kid = with_kid.active_kids().next().cloned().expect("kid");
    let actor = with_kid.actor_for(founder.user_id).expect("member");
    let opened = Wallet::open(
        &OpenWallet {
            wallet_id: kid.wallet_id,
            family_id: family.id(),
            kid_id: kid.id,
            currency: family.currency(),
            main_account_name: AccountName::new("Main").expect("name"),
        },
        &Context { actor, now: NOW },
        &mut ids,
    )
    .expect("open");

    families
        .save_with_new_wallet(&with_kid, v1, &opened)
        .await
        .expect("save");

    let wallet = wallets
        .load(family.id(), kid.wallet_id)
        .await
        .expect("load")
        .expect("found");
    assert_eq!(wallet.wallet.kid_id(), kid.id);

    // A stale save writes neither the Family nor the Wallet.
    let other = decided(
        &family,
        &founder,
        FamilyCommand::AddKid {
            name: KidName::new("Theo").expect("name"),
        },
        &mut ids,
    );
    let theo = other.active_kids().next().cloned().expect("kid");
    let theo_opened = Wallet::open(
        &OpenWallet {
            wallet_id: theo.wallet_id,
            family_id: family.id(),
            kid_id: theo.id,
            currency: family.currency(),
            main_account_name: AccountName::new("Main").expect("name"),
        },
        &Context { actor, now: NOW },
        &mut ids,
    )
    .expect("open");
    let stale = families
        .save_with_new_wallet(&other, v1, &theo_opened)
        .await;
    assert!(matches!(stale, Err(StoreError::Conflict)), "{stale:?}");
    assert!(wallets
        .load(family.id(), theo.wallet_id)
        .await
        .expect("load")
        .is_none());
}

#[sqlx::test]
async fn a_wallet_for_another_family_is_refused(pool: PgPool) {
    let families = FamilyStore::new(pool);
    let mut ids = SequentialIds(0);
    let founder = person(1, "owner@example.com");
    let family = founded(900, &founder, &mut ids);
    let v1 = families.create(&family).await.expect("create");
    let actor = family.actor_for(founder.user_id).expect("member");
    let foreign = Wallet::open(
        &OpenWallet {
            wallet_id: domain::ids::WalletId::from_uuid(Uuid::from_u128(77)),
            family_id: FamilyId::from_uuid(Uuid::from_u128(999)),
            kid_id: domain::ids::KidId::from_uuid(Uuid::from_u128(78)),
            currency: family.currency(),
            main_account_name: AccountName::new("Main").expect("name"),
        },
        &Context { actor, now: NOW },
        &mut ids,
    )
    .expect("open");
    let result = families.save_with_new_wallet(&family, v1, &foreign).await;
    assert!(matches!(result, Err(StoreError::WrongFamily)), "{result:?}");
}
