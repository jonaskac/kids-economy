//! The Family aggregate: its Members, Invitations and Kids (ADR-0010).
//!
//! A pure decider like the Wallet: [`Family::decide`] checks a command and returns the
//! events to record, [`Family::apply`] folds one in. [`Family::accept`] handles an
//! Invitation, whose User is not a Member yet. The store keeps the current state, read
//! and written through [`FamilyParts`].

mod command;
mod error;
mod event;
mod parts;

pub use command::{AcceptInvitation, CreateFamily, FamilyCommand};
pub use error::{FamilyError, FamilyReplayError};
pub use event::FamilyEvent;
pub use parts::{
    EmptyLinkHash, Invitation, InvitationLinkHash, InvitationOutcome, InvitationState, Kid, Member,
    Person, INVITATION_LIFETIME,
};

use crate::identity::Email;
use crate::ids::{FamilyId, InvitationId, KidId, MemberId, NewIds, UserId, WalletId};
use crate::money::Currency;
use crate::role::{Actor, Context, Role};
use crate::text::FamilyName;
use std::collections::HashSet;
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Family {
    id: FamilyId,
    name: FamilyName,
    currency: Currency,
    created_at: OffsetDateTime,
    /// In joining order; removed Members are kept.
    members: Vec<Member>,
    /// In sending order; every outcome is kept.
    invitations: Vec<Invitation>,
    /// In adding order; archived Kids are kept.
    kids: Vec<Kid>,
}

/// A Family's whole state as plain data, for storing and loading it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FamilyParts {
    pub id: FamilyId,
    pub name: FamilyName,
    pub currency: Currency,
    pub created_at: OffsetDateTime,
    pub members: Vec<Member>,
    pub invitations: Vec<Invitation>,
    pub kids: Vec<Kid>,
}

impl Family {
    /// Decides founding a Family; the founder becomes its first Owner.
    pub fn create(
        command: &CreateFamily,
        now: OffsetDateTime,
        ids: &mut impl NewIds,
    ) -> FamilyEvent {
        FamilyEvent::FamilyCreated {
            family_id: command.family_id,
            name: command.name.clone(),
            currency: command.currency,
            owner: new_member(&command.founder, Role::Owner, now, ids),
            created_at: now,
        }
    }

    /// Rebuilds a Family from its full, ordered history.
    pub fn replay<'a>(
        history: impl IntoIterator<Item = &'a FamilyEvent>,
    ) -> Result<Self, FamilyReplayError> {
        let mut events = history.into_iter();
        let mut family = match events.next() {
            Some(FamilyEvent::FamilyCreated {
                family_id,
                name,
                currency,
                owner,
                created_at,
            }) => Family {
                id: *family_id,
                name: name.clone(),
                currency: *currency,
                created_at: *created_at,
                members: vec![owner.clone()],
                invitations: Vec::new(),
                kids: Vec::new(),
            },
            _ => return Err(FamilyReplayError::NotCreated),
        };
        for event in events {
            family.apply(event)?;
        }
        Ok(family)
    }

    /// Rebuilds a Family from stored state, checking it is a valid one.
    pub fn from_parts(parts: FamilyParts) -> Result<Self, FamilyReplayError> {
        let unique = |ids: Vec<uuid::Uuid>| {
            let count = ids.len();
            ids.into_iter().collect::<HashSet<_>>().len() == count
        };
        if !unique(parts.members.iter().map(|m| m.id.as_uuid()).collect())
            || !unique(parts.invitations.iter().map(|i| i.id.as_uuid()).collect())
            || !unique(parts.kids.iter().map(|k| k.id.as_uuid()).collect())
        {
            return Err(FamilyReplayError::Duplicate);
        }
        let family = Family {
            id: parts.id,
            name: parts.name,
            currency: parts.currency,
            created_at: parts.created_at,
            members: parts.members,
            invitations: parts.invitations,
            kids: parts.kids,
        };
        if family.active_owners() == 0 {
            return Err(FamilyReplayError::NoOwner);
        }
        Ok(family)
    }

    pub fn to_parts(&self) -> FamilyParts {
        FamilyParts {
            id: self.id,
            name: self.name.clone(),
            currency: self.currency,
            created_at: self.created_at,
            members: self.members.clone(),
            invitations: self.invitations.clone(),
            kids: self.kids.clone(),
        }
    }

    pub const fn id(&self) -> FamilyId {
        self.id
    }
    pub const fn name(&self) -> &FamilyName {
        &self.name
    }
    pub const fn currency(&self) -> Currency {
        self.currency
    }
    pub const fn created_at(&self) -> OffsetDateTime {
        self.created_at
    }
    pub fn members(&self) -> impl Iterator<Item = &Member> {
        self.members.iter()
    }
    pub fn active_members(&self) -> impl Iterator<Item = &Member> {
        self.members.iter().filter(|m| m.is_active())
    }
    pub fn member(&self, id: MemberId) -> Option<&Member> {
        self.members.iter().find(|m| m.id == id)
    }
    pub fn invitations(&self) -> impl Iterator<Item = &Invitation> {
        self.invitations.iter()
    }
    pub fn invitation(&self, id: InvitationId) -> Option<&Invitation> {
        self.invitations.iter().find(|i| i.id == id)
    }
    pub fn invitation_by_link(&self, link_hash: &InvitationLinkHash) -> Option<&Invitation> {
        self.invitations.iter().find(|i| &i.link_hash == link_hash)
    }
    pub fn kids(&self) -> impl Iterator<Item = &Kid> {
        self.kids.iter()
    }
    pub fn active_kids(&self) -> impl Iterator<Item = &Kid> {
        self.kids.iter().filter(|k| k.is_active())
    }
    pub fn kid(&self, id: KidId) -> Option<&Kid> {
        self.kids.iter().find(|k| k.id == id)
    }

    /// The Actor for a signed-in User, if they are an active Member here.
    pub fn actor_for(&self, user_id: UserId) -> Option<Actor> {
        self.active_members()
            .find(|m| m.user_id == user_id)
            .map(|m| Actor {
                member_id: m.id,
                role: m.role,
            })
    }

    /// The Member an Owner may make a Password reset link for: any active Member.
    /// Nothing is recorded in the Family; the link lives with the User (ADR-0012).
    pub fn member_for_password_reset(
        &self,
        actor: Actor,
        member_id: MemberId,
    ) -> Result<&Member, FamilyError> {
        if !actor.role.may_manage_members() {
            return Err(FamilyError::NotAllowed);
        }
        self.active_member(member_id)
    }

    /// Checks `command` against the Family's rules and returns the events to record.
    pub fn decide(
        &self,
        command: &FamilyCommand,
        context: &Context,
        ids: &mut impl NewIds,
    ) -> Result<Vec<FamilyEvent>, FamilyError> {
        let actor = self.ensure_active(context.actor)?;
        let now = context.now;
        match command {
            FamilyCommand::Invite {
                email,
                role,
                link_hash,
            } => {
                ensure(actor.role.may_manage_members())?;
                if self.active_members().any(|m| &m.email == email) {
                    return Err(FamilyError::AlreadyMember);
                }
                if self.pending_invitation_for(email, now).is_some() {
                    return Err(FamilyError::InvitationPending);
                }
                Ok(vec![FamilyEvent::InvitationSent {
                    invitation: Invitation {
                        id: InvitationId::from_uuid(ids.next_id()),
                        email: email.clone(),
                        role: *role,
                        link_hash: link_hash.clone(),
                        invited_by: actor.member_id,
                        sent_at: now,
                        expires_at: now + INVITATION_LIFETIME,
                        outcome: InvitationOutcome::Open,
                    },
                }])
            }
            FamilyCommand::RevokeInvitation { invitation_id } => {
                ensure(actor.role.may_manage_members())?;
                let invitation = self
                    .invitation(*invitation_id)
                    .ok_or(FamilyError::UnknownInvitation)?;
                if invitation.state_at(now) != InvitationState::Pending {
                    return Err(FamilyError::InvitationNotPending);
                }
                Ok(vec![FamilyEvent::InvitationRevoked {
                    invitation_id: *invitation_id,
                    revoked_at: now,
                }])
            }
            FamilyCommand::ChangeRole { member_id, role } => {
                ensure(actor.role.may_manage_members())?;
                let member = self.active_member(*member_id)?;
                if member.role == Role::Owner && *role != Role::Owner {
                    self.ensure_another_owner()?;
                }
                Ok(vec![FamilyEvent::RoleChanged {
                    member_id: *member_id,
                    role: *role,
                }])
            }
            FamilyCommand::RemoveMember { member_id } => {
                ensure(actor.member_id == *member_id || actor.role.may_manage_members())?;
                let member = self.active_member(*member_id)?;
                if member.role == Role::Owner {
                    self.ensure_another_owner()?;
                }
                Ok(vec![FamilyEvent::MemberRemoved {
                    member_id: *member_id,
                    removed_at: now,
                }])
            }
            FamilyCommand::AddKid { name } => {
                ensure(actor.role.may_manage_kids())?;
                Ok(vec![FamilyEvent::KidAdded {
                    kid: Kid {
                        id: KidId::from_uuid(ids.next_id()),
                        name: name.clone(),
                        wallet_id: WalletId::from_uuid(ids.next_id()),
                        added_at: now,
                        hidden: false,
                        archived_at: None,
                    },
                }])
            }
            FamilyCommand::RenameKid { kid_id, name } => {
                ensure(actor.role.may_manage_kids())?;
                self.active_kid(*kid_id)?;
                Ok(vec![FamilyEvent::KidRenamed {
                    kid_id: *kid_id,
                    name: name.clone(),
                }])
            }
            FamilyCommand::HideKid { kid_id } => {
                ensure(actor.role.may_manage_kids())?;
                let kid = self.active_kid(*kid_id)?;
                Ok(if kid.hidden {
                    Vec::new()
                } else {
                    vec![FamilyEvent::KidHidden { kid_id: *kid_id }]
                })
            }
            FamilyCommand::ShowKid { kid_id } => {
                ensure(actor.role.may_manage_kids())?;
                let kid = self.active_kid(*kid_id)?;
                Ok(if kid.hidden {
                    vec![FamilyEvent::KidShown { kid_id: *kid_id }]
                } else {
                    Vec::new()
                })
            }
            FamilyCommand::ArchiveKid { kid_id } => {
                ensure(actor.role.may_manage_kids())?;
                let kid = self.kid(*kid_id).ok_or(FamilyError::UnknownKid)?;
                if !kid.is_active() {
                    return Err(FamilyError::KidArchived);
                }
                Ok(vec![FamilyEvent::KidArchived {
                    kid_id: *kid_id,
                    archived_at: now,
                }])
            }
        }
    }

    /// Decides a signed-in User accepting the Invitation behind a link.
    pub fn accept(
        &self,
        command: &AcceptInvitation,
        now: OffsetDateTime,
        ids: &mut impl NewIds,
    ) -> Result<Vec<FamilyEvent>, FamilyError> {
        let invitation = self
            .invitation_by_link(&command.link_hash)
            .ok_or(FamilyError::UnknownInvitation)?;
        match invitation.state_at(now) {
            InvitationState::Pending => {}
            InvitationState::Expired => return Err(FamilyError::InvitationExpired),
            InvitationState::Accepted | InvitationState::Revoked => {
                return Err(FamilyError::InvitationNotPending)
            }
        }
        if self.actor_for(command.person.user_id).is_some() {
            return Err(FamilyError::AlreadyMember);
        }
        if command.person.email != invitation.email {
            return Err(FamilyError::WrongEmail);
        }
        Ok(vec![FamilyEvent::MemberJoined {
            member: new_member(&command.person, invitation.role, now, ids),
            invitation_id: invitation.id,
        }])
    }

    /// Folds one recorded event into the Family.
    pub fn apply(&mut self, event: &FamilyEvent) -> Result<(), FamilyReplayError> {
        match event {
            FamilyEvent::FamilyCreated { .. } => return Err(FamilyReplayError::AlreadyCreated),
            FamilyEvent::InvitationSent { invitation } => {
                if self.invitation(invitation.id).is_some() {
                    return Err(FamilyReplayError::Duplicate);
                }
                self.invitations.push(invitation.clone());
            }
            FamilyEvent::InvitationRevoked {
                invitation_id,
                revoked_at,
            } => {
                self.invitation_mut(*invitation_id)?.outcome =
                    InvitationOutcome::Revoked { at: *revoked_at };
            }
            FamilyEvent::MemberJoined {
                member,
                invitation_id,
            } => {
                if self.member(member.id).is_some() {
                    return Err(FamilyReplayError::Duplicate);
                }
                self.invitation_mut(*invitation_id)?.outcome = InvitationOutcome::Accepted {
                    member_id: member.id,
                    at: member.joined_at,
                };
                self.members.push(member.clone());
            }
            FamilyEvent::RoleChanged { member_id, role } => {
                self.member_mut(*member_id)?.role = *role;
            }
            FamilyEvent::MemberRemoved {
                member_id,
                removed_at,
            } => {
                self.member_mut(*member_id)?.removed_at = Some(*removed_at);
            }
            FamilyEvent::KidAdded { kid } => {
                if self.kid(kid.id).is_some() {
                    return Err(FamilyReplayError::Duplicate);
                }
                self.kids.push(kid.clone());
            }
            FamilyEvent::KidRenamed { kid_id, name } => {
                self.kid_mut(*kid_id)?.name = name.clone();
            }
            FamilyEvent::KidHidden { kid_id } => self.kid_mut(*kid_id)?.hidden = true,
            FamilyEvent::KidShown { kid_id } => self.kid_mut(*kid_id)?.hidden = false,
            FamilyEvent::KidArchived {
                kid_id,
                archived_at,
            } => {
                self.kid_mut(*kid_id)?.archived_at = Some(*archived_at);
            }
        }
        Ok(())
    }

    // ---------- rules ----------

    fn ensure_active(&self, actor: Actor) -> Result<Actor, FamilyError> {
        // The Role comes from the Family itself, not from what the caller claims.
        self.active_members()
            .find(|m| m.id == actor.member_id)
            .map(|m| Actor {
                member_id: m.id,
                role: m.role,
            })
            .ok_or(FamilyError::NotAMember)
    }

    fn active_member(&self, id: MemberId) -> Result<&Member, FamilyError> {
        self.active_members()
            .find(|m| m.id == id)
            .ok_or(FamilyError::UnknownMember)
    }

    fn active_kid(&self, id: KidId) -> Result<&Kid, FamilyError> {
        let kid = self.kid(id).ok_or(FamilyError::UnknownKid)?;
        if kid.is_active() {
            Ok(kid)
        } else {
            Err(FamilyError::KidArchived)
        }
    }

    fn active_owners(&self) -> usize {
        self.active_members()
            .filter(|m| m.role == Role::Owner)
            .count()
    }

    fn ensure_another_owner(&self) -> Result<(), FamilyError> {
        if self.active_owners() > 1 {
            Ok(())
        } else {
            Err(FamilyError::LastOwner)
        }
    }

    fn pending_invitation_for(&self, email: &Email, now: OffsetDateTime) -> Option<&Invitation> {
        self.invitations
            .iter()
            .find(|i| &i.email == email && i.state_at(now) == InvitationState::Pending)
    }

    // ---------- state changes ----------

    fn member_mut(&mut self, id: MemberId) -> Result<&mut Member, FamilyReplayError> {
        self.members
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or(FamilyReplayError::Unknown)
    }

    fn kid_mut(&mut self, id: KidId) -> Result<&mut Kid, FamilyReplayError> {
        self.kids
            .iter_mut()
            .find(|k| k.id == id)
            .ok_or(FamilyReplayError::Unknown)
    }

    fn invitation_mut(&mut self, id: InvitationId) -> Result<&mut Invitation, FamilyReplayError> {
        self.invitations
            .iter_mut()
            .find(|i| i.id == id)
            .ok_or(FamilyReplayError::Unknown)
    }
}

fn new_member(person: &Person, role: Role, now: OffsetDateTime, ids: &mut impl NewIds) -> Member {
    Member {
        id: MemberId::from_uuid(ids.next_id()),
        user_id: person.user_id,
        name: person.name.clone(),
        email: person.email.clone(),
        role,
        joined_at: now,
        removed_at: None,
    }
}

fn ensure(allowed: bool) -> Result<(), FamilyError> {
    if allowed {
        Ok(())
    } else {
        Err(FamilyError::NotAllowed)
    }
}

#[cfg(test)]
mod tests;
