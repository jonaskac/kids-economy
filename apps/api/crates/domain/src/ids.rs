//! Typed identifiers. A `KidId` can never be passed where an `AccountId` is expected.
//!
//! The domain never generates ids itself (no IO, no randomness): new ids come from a
//! [`NewIds`] source handed in by the caller, so every decision is deterministic in tests.

use uuid::Uuid;

macro_rules! id {
    ($(#[$doc:meta])* $name:ident) => {
        $(#[$doc])*
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
        pub struct $name(Uuid);

        impl $name {
            pub const fn from_uuid(uuid: Uuid) -> Self {
                Self(uuid)
            }

            pub const fn as_uuid(&self) -> Uuid {
                self.0
            }
        }
    };
}

id!(
    /// A Family: the tenant boundary.
    FamilyId
);
id!(
    /// A Member of a Family (an adult with a sign-in).
    MemberId
);
id!(
    /// A Kid in a Family.
    KidId
);
id!(
    /// A Kid's Wallet.
    WalletId
);
id!(
    /// An Account inside a Wallet.
    AccountId
);
id!(
    /// A Transaction on an Account.
    TransactionId
);
id!(
    /// Links the two Transactions of a Transfer (or of a corrected Transfer).
    TransferId
);

/// Source of fresh ids, supplied by the caller (UUID v4/v7 in production, a counter in tests).
pub trait NewIds {
    fn next_id(&mut self) -> Uuid;
}
