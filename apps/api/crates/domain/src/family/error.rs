//! Why a Family rejects a command, or refuses a history or stored state.

use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FamilyError {
    #[error("this Member's Role may not do that")]
    NotAllowed,
    #[error("the Actor is not an active Member of this Family")]
    NotAMember,
    #[error("this person is already an active Member of the Family")]
    AlreadyMember,
    #[error("there is already a pending Invitation for this email")]
    InvitationPending,
    #[error("no such Invitation in this Family")]
    UnknownInvitation,
    #[error("the Invitation is no longer pending")]
    InvitationNotPending,
    #[error("the Invitation has expired")]
    InvitationExpired,
    #[error("the Invitation is for another email address")]
    WrongEmail,
    #[error("no such active Member in this Family")]
    UnknownMember,
    #[error("a Family needs at least one active Owner")]
    LastOwner,
    #[error("no such Kid in this Family")]
    UnknownKid,
    #[error("the Kid is archived")]
    KidArchived,
}

/// A history or stored state that does not describe a valid Family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Error)]
pub enum FamilyReplayError {
    #[error("the history does not start with FamilyCreated")]
    NotCreated,
    #[error("FamilyCreated appears twice")]
    AlreadyCreated,
    #[error("an event names a Member, Invitation or Kid the Family does not have")]
    Unknown,
    #[error("a Member, Invitation or Kid id appears twice")]
    Duplicate,
    #[error("the Family has no active Owner")]
    NoOwner,
}
