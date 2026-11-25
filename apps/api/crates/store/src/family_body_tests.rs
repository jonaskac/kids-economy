//! The v1 stored format of Families and Users: round-trips, and the JSON shape is pinned.

use super::*;
use domain::family::FamilyParts;
use serde_json::json;
use time::macros::datetime;

fn id(n: u128) -> Uuid {
    Uuid::from_u128(n)
}

fn family() -> Family {
    let at = datetime!(2026-11-25 13:00 UTC);
    Family::from_parts(FamilyParts {
        id: FamilyId::from_uuid(id(900)),
        name: FamilyName::new("The Kacs").expect("name"),
        currency: Currency::new("SEK").expect("currency"),
        created_at: at,
        members: vec![Member {
            id: MemberId::from_uuid(id(1)),
            user_id: UserId::from_uuid(id(2)),
            name: PersonName::new("Jonas").expect("name"),
            email: Email::new("jonas@example.com").expect("email"),
            role: Role::Owner,
            joined_at: at,
            removed_at: None,
        }],
        invitations: vec![
            Invitation {
                id: InvitationId::from_uuid(id(3)),
                email: Email::new("grandma@example.com").expect("email"),
                role: Role::Viewer,
                link_hash: InvitationLinkHash::new("abc").expect("hash"),
                invited_by: MemberId::from_uuid(id(1)),
                sent_at: at,
                expires_at: datetime!(2026-12-02 13:00 UTC),
                outcome: InvitationOutcome::Open,
            },
            Invitation {
                id: InvitationId::from_uuid(id(4)),
                email: Email::new("aunt@example.com").expect("email"),
                role: Role::Parent,
                link_hash: InvitationLinkHash::new("def").expect("hash"),
                invited_by: MemberId::from_uuid(id(1)),
                sent_at: at,
                expires_at: datetime!(2026-12-02 13:00 UTC),
                outcome: InvitationOutcome::Revoked { at },
            },
        ],
        kids: vec![Kid {
            id: KidId::from_uuid(id(5)),
            name: KidName::new("Elsa").expect("name"),
            wallet_id: WalletId::from_uuid(id(6)),
            added_at: at,
            hidden: true,
            archived_at: Some(at),
        }],
    })
    .expect("family")
}

#[test]
fn a_family_round_trips() {
    let family = family();
    let body = encode_family(&family).expect("encode");
    assert_eq!(decode_family(id(900), body).expect("decode"), family);
}

#[test]
fn the_v1_family_shape_is_pinned() {
    let body = encode_family(&family()).expect("encode");
    assert_eq!(
        body,
        json!({
            "v": 1,
            "name": "The Kacs",
            "currency": "SEK",
            "created_at": "2026-11-25T13:00:00Z",
            "members": [{
                "id": id(1), "user_id": id(2), "name": "Jonas", "email": "jonas@example.com",
                "role": "owner", "joined_at": "2026-11-25T13:00:00Z", "removed_at": null
            }],
            "invitations": [{
                "id": id(3), "email": "grandma@example.com", "role": "viewer",
                "link_hash": "abc", "invited_by": id(1),
                "sent_at": "2026-11-25T13:00:00Z", "expires_at": "2026-12-02T13:00:00Z",
                "outcome": { "kind": "open" }
            }, {
                "id": id(4), "email": "aunt@example.com", "role": "parent",
                "link_hash": "def", "invited_by": id(1),
                "sent_at": "2026-11-25T13:00:00Z", "expires_at": "2026-12-02T13:00:00Z",
                "outcome": { "kind": "revoked", "at": "2026-11-25T13:00:00Z" }
            }],
            "kids": [{
                "id": id(5), "name": "Elsa", "wallet_id": id(6),
                "added_at": "2026-11-25T13:00:00Z", "hidden": true,
                "archived_at": "2026-11-25T13:00:00Z"
            }]
        })
    );
}

#[test]
fn a_kid_stored_before_hiding_existed_is_shown() {
    let mut body = encode_family(&family()).expect("encode");
    body["kids"][0]
        .as_object_mut()
        .expect("kid")
        .remove("hidden");
    let decoded = decode_family(id(900), body).expect("decode");
    assert!(!decoded.kid(KidId::from_uuid(id(5))).expect("kid").hidden);
}

#[test]
fn a_family_body_without_an_owner_is_refused() {
    let mut body = encode_family(&family()).expect("encode");
    body["members"][0]["role"] = json!("parent");
    assert!(matches!(
        decode_family(id(900), body),
        Err(DocumentError::Family(FamilyReplayError::NoOwner))
    ));
}

#[test]
fn an_unknown_version_is_refused() {
    let mut body = encode_family(&family()).expect("encode");
    body["v"] = json!(2);
    assert!(matches!(
        decode_family(id(900), body),
        Err(DocumentError::Body(BodyError::UnknownVersion(Some(2))))
    ));
}

#[test]
fn a_user_round_trips_with_a_pinned_shape() {
    let user = User {
        id: UserId::from_uuid(id(2)),
        email: Email::new("jonas@example.com").expect("email"),
        name: PersonName::new("Jonas").expect("name"),
        registered_at: datetime!(2026-11-25 13:00 UTC),
    };
    let body = encode_user(&user, "$argon2id$x").expect("encode");
    assert_eq!(
        body,
        json!({
            "v": 1, "email": "jonas@example.com", "name": "Jonas",
            "password_hash": "$argon2id$x", "registered_at": "2026-11-25T13:00:00Z"
        })
    );
    assert_eq!(
        decode_user(id(2), body).expect("decode"),
        (user, "$argon2id$x".to_owned())
    );
}
