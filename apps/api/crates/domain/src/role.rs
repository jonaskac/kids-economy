//! Roles and the Actor behind every command.

use crate::ids::MemberId;

/// What a Member may do in a Family.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Role {
    Owner,
    Parent,
    Viewer,
}

impl Role {
    /// Owners and Parents record money and manage Accounts; Viewers change nothing.
    pub const fn may_change_wallets(self) -> bool {
        matches!(self, Role::Owner | Role::Parent)
    }
}

/// The Member who causes something to happen, with their Role in the Kid's Family.
///
/// The caller resolves the Member and checks they are active in the Wallet's Family
/// before building an Actor; the Wallet checks only what the Role allows.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Actor {
    pub member_id: MemberId,
    pub role: Role,
}
