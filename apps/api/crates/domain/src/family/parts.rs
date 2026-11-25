//! The people and Kids inside a Family.

use crate::identity::Email;
use crate::ids::{InvitationId, KidId, MemberId, UserId, WalletId};
use crate::role::Role;
use crate::text::{KidName, PersonName};
use thiserror::Error;
use time::{Duration, OffsetDateTime};

/// How long an Invitation stays pending.
pub const INVITATION_LIFETIME: Duration = Duration::days(7);

/// The User behind a new Member: who founds a Family or accepts an Invitation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Person {
    pub user_id: UserId,
    pub name: PersonName,
    pub email: Email,
}

/// An adult in a Family. Removed, never deleted, so past Transactions still name them.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Member {
    pub id: MemberId,
    pub user_id: UserId,
    pub name: PersonName,
    pub email: Email,
    pub role: Role,
    pub joined_at: OffsetDateTime,
    pub removed_at: Option<OffsetDateTime>,
}

impl Member {
    pub const fn is_active(&self) -> bool {
        self.removed_at.is_none()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
#[error("an Invitation link hash cannot be empty")]
pub struct EmptyLinkHash;

/// The hash of an Invitation link's secret. The domain never sees the secret itself.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct InvitationLinkHash(String);

impl InvitationLinkHash {
    pub fn new(hash: impl Into<String>) -> Result<Self, EmptyLinkHash> {
        let hash = hash.into();
        if hash.is_empty() {
            Err(EmptyLinkHash)
        } else {
            Ok(Self(hash))
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// What has happened to an Invitation, as recorded. Expiry is derived from the time.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvitationOutcome {
    Open,
    Accepted {
        member_id: MemberId,
        at: OffsetDateTime,
    },
    Revoked {
        at: OffsetDateTime,
    },
}

/// An Invitation's state at a point in time (glossary).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InvitationState {
    Pending,
    Accepted,
    Revoked,
    Expired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Invitation {
    pub id: InvitationId,
    pub email: Email,
    pub role: Role,
    pub link_hash: InvitationLinkHash,
    pub invited_by: MemberId,
    pub sent_at: OffsetDateTime,
    pub expires_at: OffsetDateTime,
    pub outcome: InvitationOutcome,
}

impl Invitation {
    pub fn state_at(&self, now: OffsetDateTime) -> InvitationState {
        match self.outcome {
            InvitationOutcome::Accepted { .. } => InvitationState::Accepted,
            InvitationOutcome::Revoked { .. } => InvitationState::Revoked,
            InvitationOutcome::Open if now >= self.expires_at => InvitationState::Expired,
            InvitationOutcome::Open => InvitationState::Pending,
        }
    }
}

/// A child in the Family with exactly one Wallet. Archived, never deleted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Kid {
    pub id: KidId,
    pub name: KidName,
    pub wallet_id: WalletId,
    pub added_at: OffsetDateTime,
    /// Left off the Family home, but still active: the Wallet keeps working.
    pub hidden: bool,
    pub archived_at: Option<OffsetDateTime>,
}

impl Kid {
    pub const fn is_active(&self) -> bool {
        self.archived_at.is_none()
    }
}
