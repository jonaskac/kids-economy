//! Facts a Family has recorded.

use super::parts::{Invitation, Kid, Member};
use crate::ids::{FamilyId, InvitationId, KidId, MemberId};
use crate::money::Currency;
use crate::role::Role;
use crate::text::{FamilyName, KidName};
use time::OffsetDateTime;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FamilyEvent {
    /// Always first. `owner` is the founder.
    FamilyCreated {
        family_id: FamilyId,
        name: FamilyName,
        currency: Currency,
        owner: Member,
        created_at: OffsetDateTime,
    },
    InvitationSent {
        invitation: Invitation,
    },
    InvitationRevoked {
        invitation_id: InvitationId,
        revoked_at: OffsetDateTime,
    },
    MemberJoined {
        member: Member,
        invitation_id: InvitationId,
    },
    RoleChanged {
        member_id: MemberId,
        role: Role,
    },
    MemberRemoved {
        member_id: MemberId,
        removed_at: OffsetDateTime,
    },
    KidAdded {
        kid: Kid,
    },
    KidRenamed {
        kid_id: KidId,
        name: KidName,
    },
    KidHidden {
        kid_id: KidId,
    },
    KidShown {
        kid_id: KidId,
    },
    KidArchived {
        kid_id: KidId,
        archived_at: OffsetDateTime,
    },
}
