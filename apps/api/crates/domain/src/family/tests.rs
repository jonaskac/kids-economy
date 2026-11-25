//! Behaviour of the Family aggregate, in glossary terms.

use super::*;
use crate::identity::Email;
use crate::ids::{FamilyId, NewIds, UserId};
use crate::money::Currency;
use crate::role::{Actor, Context, Role};
use crate::text::{FamilyName, KidName, PersonName};
use time::macros::datetime;
use time::{Duration, OffsetDateTime};
use uuid::Uuid;

// ---------- fixture ----------

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

fn link(secret: &str) -> InvitationLinkHash {
    InvitationLinkHash::new(format!("hash-of-{secret}")).expect("hash")
}

struct Fixture {
    family: Family,
    ids: SequentialIds,
    history: Vec<FamilyEvent>,
}

impl Fixture {
    /// A Family founded by `owner@example.com`.
    fn new() -> Self {
        let mut ids = SequentialIds(0);
        let created = Family::create(
            &CreateFamily {
                family_id: FamilyId::from_uuid(Uuid::from_u128(900)),
                name: FamilyName::new("The Kacs").expect("name"),
                currency: Currency::new("SEK").expect("currency"),
                founder: person(1, "owner@example.com"),
            },
            NOW,
            &mut ids,
        );
        let history = vec![created];
        let family = Family::replay(&history).expect("replay");
        Fixture {
            family,
            ids,
            history,
        }
    }

    fn owner(&self) -> Actor {
        self.actor(&person(1, "owner@example.com"))
    }

    fn actor(&self, who: &Person) -> Actor {
        self.family
            .actor_for(who.user_id)
            .expect("an active Member")
    }

    fn try_decide(
        &mut self,
        actor: Actor,
        command: FamilyCommand,
    ) -> Result<Vec<FamilyEvent>, FamilyError> {
        self.try_decide_at(actor, command, NOW)
    }

    fn try_decide_at(
        &mut self,
        actor: Actor,
        command: FamilyCommand,
        now: OffsetDateTime,
    ) -> Result<Vec<FamilyEvent>, FamilyError> {
        let events = self
            .family
            .decide(&command, &Context { actor, now }, &mut self.ids)?;
        self.record(&events);
        Ok(events)
    }

    fn decide(&mut self, actor: Actor, command: FamilyCommand) -> Vec<FamilyEvent> {
        self.try_decide(actor, command).expect("accepted")
    }

    fn record(&mut self, events: &[FamilyEvent]) {
        for event in events {
            self.family.apply(event).expect("apply");
            self.history.push(event.clone());
        }
    }

    fn invite(&mut self, email: &str, role: Role, secret: &str) -> InvitationId {
        let owner = self.owner();
        let events = self.decide(
            owner,
            FamilyCommand::Invite {
                email: Email::new(email).expect("email"),
                role,
                link_hash: link(secret),
            },
        );
        match events.as_slice() {
            [FamilyEvent::InvitationSent { invitation }] => invitation.id,
            other => panic!("unexpected events {other:?}"),
        }
    }

    fn try_accept_at(
        &mut self,
        secret: &str,
        who: &Person,
        now: OffsetDateTime,
    ) -> Result<Vec<FamilyEvent>, FamilyError> {
        let events = self.family.accept(
            &AcceptInvitation {
                link_hash: link(secret),
                person: who.clone(),
            },
            now,
            &mut self.ids,
        )?;
        self.record(&events);
        Ok(events)
    }

    fn try_accept(&mut self, secret: &str, who: &Person) -> Result<Vec<FamilyEvent>, FamilyError> {
        self.try_accept_at(secret, who, NOW)
    }

    /// Invites and accepts: `who` becomes a Member with `role`.
    fn join(&mut self, who: &Person, role: Role) -> Actor {
        let secret = format!("join-{}", who.email.as_str());
        self.invite(who.email.as_str(), role, &secret);
        self.try_accept(&secret, who).expect("joined");
        self.actor(who)
    }

    fn add_kid(&mut self, actor: Actor, name: &str) -> Result<Kid, FamilyError> {
        let events = self.try_decide(
            actor,
            FamilyCommand::AddKid {
                name: KidName::new(name).expect("name"),
            },
        )?;
        match events.as_slice() {
            [FamilyEvent::KidAdded { kid }] => Ok(kid.clone()),
            other => panic!("unexpected events {other:?}"),
        }
    }
}

// ---------- creating a Family ----------

#[test]
fn the_founder_of_a_family_is_its_owner() {
    let f = Fixture::new();
    let members: Vec<_> = f.family.active_members().collect();
    assert_eq!(members.len(), 1);
    assert_eq!(members[0].role, Role::Owner);
    assert_eq!(members[0].email.as_str(), "owner@example.com");
    assert_eq!(members[0].joined_at, NOW);
    assert_eq!(f.family.name().as_str(), "The Kacs");
    assert_eq!(f.family.currency().code(), "SEK");
}

#[test]
fn a_stranger_is_no_actor_in_the_family() {
    let f = Fixture::new();
    assert_eq!(f.family.actor_for(person(9, "x@example.com").user_id), None);
}

#[test]
fn an_actor_who_is_not_an_active_member_is_rejected() {
    let mut f = Fixture::new();
    let stranger = Actor {
        member_id: MemberId::from_uuid(Uuid::from_u128(77)),
        role: Role::Owner,
    };
    assert_eq!(f.add_kid(stranger, "Elsa"), Err(FamilyError::NotAMember));
}

// ---------- Invitations ----------

#[test]
fn an_owner_invites_an_adult_with_a_role_for_seven_days() {
    let mut f = Fixture::new();
    let id = f.invite("Grandma@Example.com", Role::Viewer, "s1");
    let invitation = f.family.invitation(id).expect("invitation");
    assert_eq!(invitation.email.as_str(), "grandma@example.com");
    assert_eq!(invitation.role, Role::Viewer);
    assert_eq!(invitation.sent_at, NOW);
    assert_eq!(invitation.expires_at, NOW + Duration::days(7));
    assert_eq!(invitation.state_at(NOW), InvitationState::Pending);
    assert_eq!(
        invitation.state_at(NOW + Duration::days(7)),
        InvitationState::Expired
    );
}

#[test]
fn only_an_owner_may_invite() {
    let mut f = Fixture::new();
    let parent = f.join(&person(2, "parent@example.com"), Role::Parent);
    let viewer = f.join(&person(3, "viewer@example.com"), Role::Viewer);
    for actor in [parent, viewer] {
        let result = f.try_decide(
            actor,
            FamilyCommand::Invite {
                email: Email::new("new@example.com").expect("email"),
                role: Role::Viewer,
                link_hash: link("s"),
            },
        );
        assert_eq!(result, Err(FamilyError::NotAllowed));
    }
}

#[test]
fn one_pending_invitation_per_email() {
    let mut f = Fixture::new();
    f.invite("grandma@example.com", Role::Viewer, "s1");
    let owner = f.owner();
    let again = f.try_decide(
        owner,
        FamilyCommand::Invite {
            email: Email::new("GRANDMA@example.com").expect("email"),
            role: Role::Parent,
            link_hash: link("s2"),
        },
    );
    assert_eq!(again, Err(FamilyError::InvitationPending));
}

#[test]
fn an_expired_or_revoked_invitation_can_be_sent_again() {
    let mut f = Fixture::new();
    f.invite("grandma@example.com", Role::Viewer, "s1");
    let owner = f.owner();
    let later = NOW + Duration::days(8);
    f.try_decide_at(
        owner,
        FamilyCommand::Invite {
            email: Email::new("grandma@example.com").expect("email"),
            role: Role::Viewer,
            link_hash: link("s2"),
        },
        later,
    )
    .expect("expired one does not block");

    let id = f.invite("aunt@example.com", Role::Viewer, "s3");
    f.decide(owner, FamilyCommand::RevokeInvitation { invitation_id: id });
    f.invite("aunt@example.com", Role::Viewer, "s4");
}

#[test]
fn an_active_member_cannot_be_invited() {
    let mut f = Fixture::new();
    let owner = f.owner();
    let result = f.try_decide(
        owner,
        FamilyCommand::Invite {
            email: Email::new("owner@example.com").expect("email"),
            role: Role::Viewer,
            link_hash: link("s"),
        },
    );
    assert_eq!(result, Err(FamilyError::AlreadyMember));
}

#[test]
fn accepting_an_invitation_makes_a_member_with_the_invited_role() {
    let mut f = Fixture::new();
    let id = f.invite("grandma@example.com", Role::Viewer, "s1");
    let grandma = person(4, "grandma@example.com");
    let later = NOW + Duration::days(1);
    let events = f.try_accept_at("s1", &grandma, later).expect("accepted");

    let [FamilyEvent::MemberJoined {
        member,
        invitation_id,
    }] = events.as_slice()
    else {
        panic!("unexpected events {events:?}");
    };
    assert_eq!(*invitation_id, id);
    assert_eq!(member.user_id, grandma.user_id);
    assert_eq!(member.role, Role::Viewer);
    assert_eq!(member.joined_at, later);
    assert_eq!(f.actor(&grandma).role, Role::Viewer);
    assert_eq!(
        f.family.invitation(id).map(|i| i.state_at(later)),
        Some(InvitationState::Accepted)
    );
}

#[test]
fn an_invitation_is_accepted_only_by_the_invited_email() {
    let mut f = Fixture::new();
    f.invite("grandma@example.com", Role::Viewer, "s1");
    assert_eq!(
        f.try_accept("s1", &person(5, "someone@example.com")),
        Err(FamilyError::WrongEmail)
    );
}

#[test]
fn an_unknown_link_is_no_invitation() {
    let mut f = Fixture::new();
    assert_eq!(
        f.try_accept("nope", &person(5, "x@example.com")),
        Err(FamilyError::UnknownInvitation)
    );
}

#[test]
fn an_expired_invitation_cannot_be_accepted() {
    let mut f = Fixture::new();
    f.invite("grandma@example.com", Role::Viewer, "s1");
    assert_eq!(
        f.try_accept_at(
            "s1",
            &person(4, "grandma@example.com"),
            NOW + Duration::days(7)
        ),
        Err(FamilyError::InvitationExpired)
    );
}

#[test]
fn a_revoked_or_used_invitation_cannot_be_accepted() {
    let mut f = Fixture::new();
    let id = f.invite("grandma@example.com", Role::Viewer, "s1");
    let owner = f.owner();
    f.decide(owner, FamilyCommand::RevokeInvitation { invitation_id: id });
    assert_eq!(
        f.try_accept("s1", &person(4, "grandma@example.com")),
        Err(FamilyError::InvitationNotPending)
    );

    let uncle = person(6, "uncle@example.com");
    f.invite("uncle@example.com", Role::Viewer, "s2");
    f.try_accept("s2", &uncle).expect("first time");
    assert_eq!(
        f.try_accept("s2", &uncle),
        Err(FamilyError::InvitationNotPending)
    );
}

#[test]
fn only_an_owner_revokes_and_only_pending_invitations() {
    let mut f = Fixture::new();
    let id = f.invite("grandma@example.com", Role::Viewer, "s1");
    let parent = f.join(&person(2, "parent@example.com"), Role::Parent);
    assert_eq!(
        f.try_decide(
            parent,
            FamilyCommand::RevokeInvitation { invitation_id: id }
        ),
        Err(FamilyError::NotAllowed)
    );
    let owner = f.owner();
    let events = f.decide(owner, FamilyCommand::RevokeInvitation { invitation_id: id });
    assert_eq!(
        events,
        vec![FamilyEvent::InvitationRevoked {
            invitation_id: id,
            revoked_at: NOW
        }]
    );
    assert_eq!(
        f.try_decide(owner, FamilyCommand::RevokeInvitation { invitation_id: id }),
        Err(FamilyError::InvitationNotPending)
    );
    assert_eq!(
        f.try_decide(
            owner,
            FamilyCommand::RevokeInvitation {
                invitation_id: InvitationId::from_uuid(Uuid::from_u128(1))
            }
        ),
        Err(FamilyError::UnknownInvitation)
    );
}

// ---------- Roles and removing Members ----------

#[test]
fn an_owner_changes_a_members_role() {
    let mut f = Fixture::new();
    let parent = f.join(&person(2, "parent@example.com"), Role::Parent);
    let owner = f.owner();
    f.decide(
        owner,
        FamilyCommand::ChangeRole {
            member_id: parent.member_id,
            role: Role::Owner,
        },
    );
    assert_eq!(f.actor(&person(2, "parent@example.com")).role, Role::Owner);
}

#[test]
fn only_an_owner_changes_roles() {
    let mut f = Fixture::new();
    let parent = f.join(&person(2, "parent@example.com"), Role::Parent);
    assert_eq!(
        f.try_decide(
            parent,
            FamilyCommand::ChangeRole {
                member_id: parent.member_id,
                role: Role::Owner
            }
        ),
        Err(FamilyError::NotAllowed)
    );
}

#[test]
fn the_last_owner_cannot_step_down_or_leave() {
    let mut f = Fixture::new();
    let owner = f.owner();
    assert_eq!(
        f.try_decide(
            owner,
            FamilyCommand::ChangeRole {
                member_id: owner.member_id,
                role: Role::Parent
            }
        ),
        Err(FamilyError::LastOwner)
    );
    assert_eq!(
        f.try_decide(
            owner,
            FamilyCommand::RemoveMember {
                member_id: owner.member_id
            }
        ),
        Err(FamilyError::LastOwner)
    );
}

#[test]
fn with_two_owners_one_may_step_down() {
    let mut f = Fixture::new();
    f.join(&person(2, "second@example.com"), Role::Owner);
    let owner = f.owner();
    f.decide(
        owner,
        FamilyCommand::ChangeRole {
            member_id: owner.member_id,
            role: Role::Parent,
        },
    );
    assert_eq!(f.owner().role, Role::Parent);
}

#[test]
fn an_owner_removes_a_member_who_is_kept_but_no_longer_acts() {
    let mut f = Fixture::new();
    let viewer_person = person(3, "viewer@example.com");
    let viewer = f.join(&viewer_person, Role::Viewer);
    let owner = f.owner();
    let events = f.decide(
        owner,
        FamilyCommand::RemoveMember {
            member_id: viewer.member_id,
        },
    );
    assert_eq!(
        events,
        vec![FamilyEvent::MemberRemoved {
            member_id: viewer.member_id,
            removed_at: NOW
        }]
    );
    assert_eq!(f.family.actor_for(viewer_person.user_id), None);
    let kept = f.family.member(viewer.member_id).expect("still named");
    assert_eq!(kept.removed_at, Some(NOW));
    assert_eq!(
        f.try_decide(
            owner,
            FamilyCommand::RemoveMember {
                member_id: viewer.member_id
            }
        ),
        Err(FamilyError::UnknownMember)
    );
}

#[test]
fn a_member_may_leave_but_not_remove_others() {
    let mut f = Fixture::new();
    let parent = f.join(&person(2, "parent@example.com"), Role::Parent);
    let viewer = f.join(&person(3, "viewer@example.com"), Role::Viewer);
    assert_eq!(
        f.try_decide(
            viewer,
            FamilyCommand::RemoveMember {
                member_id: parent.member_id
            }
        ),
        Err(FamilyError::NotAllowed)
    );
    f.decide(
        viewer,
        FamilyCommand::RemoveMember {
            member_id: viewer.member_id,
        },
    );
    assert_eq!(f.family.active_members().count(), 2);
}

#[test]
fn a_removed_member_can_be_invited_and_join_again() {
    let mut f = Fixture::new();
    let viewer_person = person(3, "viewer@example.com");
    let viewer = f.join(&viewer_person, Role::Viewer);
    f.decide(
        viewer,
        FamilyCommand::RemoveMember {
            member_id: viewer.member_id,
        },
    );
    f.invite("viewer@example.com", Role::Parent, "again");
    f.try_accept("again", &viewer_person).expect("rejoined");
    let back = f.actor(&viewer_person);
    assert_eq!(back.role, Role::Parent);
    assert_ne!(back.member_id, viewer.member_id);
}

#[test]
fn a_user_who_is_already_an_active_member_cannot_accept_another_invitation() {
    let mut f = Fixture::new();
    let parent = person(2, "parent@example.com");
    f.invite("parent@example.com", Role::Viewer, "a");
    f.invite("parent.work@example.com", Role::Parent, "b");
    f.try_accept("a", &parent).expect("joined");
    // Same User, signed in with a changed email address.
    let same_user = Person {
        email: Email::new("parent.work@example.com").expect("email"),
        ..parent
    };
    assert_eq!(
        f.try_accept("b", &same_user),
        Err(FamilyError::AlreadyMember)
    );
}

// ---------- Kids ----------

#[test]
fn an_owner_or_parent_adds_a_kid_with_a_new_wallet() {
    let mut f = Fixture::new();
    let owner = f.owner();
    let elsa = f.add_kid(owner, "Elsa").expect("added");
    assert_eq!(elsa.name.as_str(), "Elsa");
    assert_eq!(elsa.added_at, NOW);
    assert_ne!(elsa.wallet_id.as_uuid(), elsa.id.as_uuid());

    let parent = f.join(&person(2, "parent@example.com"), Role::Parent);
    f.add_kid(parent, "Theo").expect("added");
    let names: Vec<_> = f.family.active_kids().map(|k| k.name.as_str()).collect();
    assert_eq!(names, ["Elsa", "Theo"]);
}

#[test]
fn a_viewer_cannot_add_or_archive_kids() {
    let mut f = Fixture::new();
    let owner = f.owner();
    let elsa = f.add_kid(owner, "Elsa").expect("added");
    let viewer = f.join(&person(3, "viewer@example.com"), Role::Viewer);
    assert_eq!(f.add_kid(viewer, "Theo"), Err(FamilyError::NotAllowed));
    assert_eq!(
        f.try_decide(viewer, FamilyCommand::ArchiveKid { kid_id: elsa.id }),
        Err(FamilyError::NotAllowed)
    );
}

#[test]
fn an_archived_kid_is_kept_but_not_active() {
    let mut f = Fixture::new();
    let owner = f.owner();
    let elsa = f.add_kid(owner, "Elsa").expect("added");
    f.decide(owner, FamilyCommand::ArchiveKid { kid_id: elsa.id });
    assert_eq!(f.family.active_kids().count(), 0);
    assert_eq!(f.family.kid(elsa.id).and_then(|k| k.archived_at), Some(NOW));
    assert_eq!(
        f.try_decide(owner, FamilyCommand::ArchiveKid { kid_id: elsa.id }),
        Err(FamilyError::KidArchived)
    );
    assert_eq!(
        f.try_decide(
            owner,
            FamilyCommand::ArchiveKid {
                kid_id: KidId::from_uuid(Uuid::from_u128(1))
            }
        ),
        Err(FamilyError::UnknownKid)
    );
}

// ---------- rebuilding ----------

#[test]
fn replaying_the_history_rebuilds_the_same_family() {
    let mut f = Fixture::new();
    let owner = f.owner();
    f.join(&person(2, "parent@example.com"), Role::Parent);
    f.invite("grandma@example.com", Role::Viewer, "g");
    f.add_kid(owner, "Elsa").expect("added");
    assert_eq!(Family::replay(&f.history), Ok(f.family.clone()));
}

#[test]
fn a_history_must_start_with_the_family_being_created() {
    let mut f = Fixture::new();
    let owner = f.owner();
    f.add_kid(owner, "Elsa").expect("added");
    assert_eq!(
        Family::replay(&f.history[1..]),
        Err(FamilyReplayError::NotCreated)
    );
}

#[test]
fn a_family_round_trips_through_its_parts() {
    let mut f = Fixture::new();
    let owner = f.owner();
    f.join(&person(2, "parent@example.com"), Role::Parent);
    f.invite("grandma@example.com", Role::Viewer, "g");
    f.add_kid(owner, "Elsa").expect("added");
    assert_eq!(
        Family::from_parts(f.family.to_parts()),
        Ok(f.family.clone())
    );
}

#[test]
fn parts_without_an_active_owner_are_no_family() {
    let f = Fixture::new();
    let mut parts = f.family.to_parts();
    parts.members[0].role = Role::Parent;
    assert_eq!(Family::from_parts(parts), Err(FamilyReplayError::NoOwner));
}
