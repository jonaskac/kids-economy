//! What a Member (or a User accepting an Invitation) asks a Family to do.

use super::parts::{InvitationLinkHash, Person};
use crate::identity::Email;
use crate::ids::{FamilyId, InvitationId, KidId, MemberId};
use crate::money::Currency;
use crate::role::Role;
use crate::text::{FamilyName, KidName};

/// Founds a Family; the founder becomes its first Owner.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CreateFamily {
    pub family_id: FamilyId,
    pub name: FamilyName,
    pub currency: Currency,
    pub founder: Person,
}

/// A signed-in User opening an Invitation link. They are not a Member yet.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AcceptInvitation {
    pub link_hash: InvitationLinkHash,
    pub person: Person,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilyCommand {
    /// The caller makes the Invitation link's secret and passes only its hash.
    Invite {
        email: Email,
        role: Role,
        link_hash: InvitationLinkHash,
    },
    RevokeInvitation {
        invitation_id: InvitationId,
    },
    ChangeRole {
        member_id: MemberId,
        role: Role,
    },
    /// Removing yourself is leaving the Family.
    RemoveMember {
        member_id: MemberId,
    },
    /// Adds a Kid with a new Wallet id; the caller opens the Wallet in the same save.
    AddKid {
        name: KidName,
    },
    /// Fixes a Kid's name entered by mistake. Their Wallet stays the same.
    RenameKid {
        kid_id: KidId,
        name: KidName,
    },
    /// Hides a Kid from the Family home; their Wallet keeps working (ADR-0013).
    HideKid {
        kid_id: KidId,
    },
    /// Shows a hidden Kid on the Family home again.
    ShowKid {
        kid_id: KidId,
    },
    /// "Remove kid" in the app: the Kid and their Wallet are kept, but no longer active.
    ArchiveKid {
        kid_id: KidId,
    },
}
