//! The stored format of Families and Users in `documents`: JSONB with `"v": 1` (ADR-0010).
//!
//! Owned by the store, not the domain: renaming a Rust field never changes stored data.
//! Names, emails and Amounts are validated again on read with the domain's constructors.

use crate::body::{time_in, time_out, BodyError, CURRENT_VERSION};
use domain::family::{
    Family, FamilyParts, FamilyReplayError, Invitation, InvitationLinkHash, InvitationOutcome, Kid,
    Member,
};
use domain::identity::{Email, EmailError, User};
use domain::ids::{FamilyId, InvitationId, KidId, MemberId, UserId, WalletId};
use domain::money::Currency;
use domain::role::Role;
use domain::text::{FamilyName, KidName, PersonName};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use thiserror::Error;
use uuid::Uuid;

#[derive(Debug, Error)]
pub enum DocumentError {
    #[error(transparent)]
    Body(#[from] BodyError),
    #[error("invalid email: {0}")]
    Email(#[from] EmailError),
    #[error("not a valid Family: {0}")]
    Family(#[from] FamilyReplayError),
    #[error("an Invitation link hash is empty")]
    EmptyLinkHash,
}

impl From<serde_json::Error> for DocumentError {
    fn from(error: serde_json::Error) -> Self {
        Self::Body(BodyError::Json(error))
    }
}

impl From<domain::text::TextError> for DocumentError {
    fn from(error: domain::text::TextError) -> Self {
        Self::Body(BodyError::Text(error))
    }
}

impl From<domain::money::MoneyError> for DocumentError {
    fn from(error: domain::money::MoneyError) -> Self {
        Self::Body(BodyError::Money(error))
    }
}

impl From<time::error::Parse> for DocumentError {
    fn from(error: time::error::Parse) -> Self {
        Self::Body(BodyError::TimeParse(error))
    }
}

fn versioned<T: Serialize>(shape: &T) -> Result<Value, DocumentError> {
    let mut value = serde_json::to_value(shape)?;
    match value.as_object_mut() {
        Some(body) => {
            body.insert("v".to_owned(), Value::from(CURRENT_VERSION));
            Ok(value)
        }
        None => Err(BodyError::NotAnObject.into()),
    }
}

fn unversioned(body: Value) -> Result<Value, DocumentError> {
    let Value::Object(mut body) = body else {
        return Err(BodyError::NotAnObject.into());
    };
    match body.remove("v").as_ref().and_then(Value::as_u64) {
        Some(CURRENT_VERSION) => Ok(Value::Object(body)),
        other => Err(BodyError::UnknownVersion(other).into()),
    }
}

// ---------- v1 shapes ----------

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum RoleV1 {
    Owner,
    Parent,
    Viewer,
}

impl From<Role> for RoleV1 {
    fn from(role: Role) -> Self {
        match role {
            Role::Owner => Self::Owner,
            Role::Parent => Self::Parent,
            Role::Viewer => Self::Viewer,
        }
    }
}

impl From<RoleV1> for Role {
    fn from(role: RoleV1) -> Self {
        match role {
            RoleV1::Owner => Self::Owner,
            RoleV1::Parent => Self::Parent,
            RoleV1::Viewer => Self::Viewer,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FamilyV1 {
    name: String,
    currency: String,
    created_at: String,
    members: Vec<MemberV1>,
    invitations: Vec<InvitationV1>,
    kids: Vec<KidV1>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct MemberV1 {
    id: Uuid,
    user_id: Uuid,
    name: String,
    email: String,
    role: RoleV1,
    joined_at: String,
    removed_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct InvitationV1 {
    id: Uuid,
    email: String,
    role: RoleV1,
    link_hash: String,
    invited_by: Uuid,
    sent_at: String,
    expires_at: String,
    outcome: OutcomeV1,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case", deny_unknown_fields)]
enum OutcomeV1 {
    Open,
    Accepted { member_id: Uuid, at: String },
    Revoked { at: String },
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct KidV1 {
    id: Uuid,
    name: String,
    wallet_id: Uuid,
    added_at: String,
    /// Added after v1 shipped; Kids stored before it are shown (ADR-0013).
    #[serde(default)]
    hidden: bool,
    archived_at: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct UserV1 {
    email: String,
    name: String,
    password_hash: String,
    registered_at: String,
}

fn optional_out(at: Option<time::OffsetDateTime>) -> Result<Option<String>, DocumentError> {
    at.map(time_out).transpose().map_err(Into::into)
}

fn optional_in(at: Option<String>) -> Result<Option<time::OffsetDateTime>, DocumentError> {
    at.as_deref().map(time_in).transpose().map_err(Into::into)
}

// ---------- Family ----------

/// The JSONB body for a Family. Its id is the document id.
pub fn encode_family(family: &Family) -> Result<Value, DocumentError> {
    let parts = family.to_parts();
    let members = parts
        .members
        .iter()
        .map(|m| {
            Ok(MemberV1 {
                id: m.id.as_uuid(),
                user_id: m.user_id.as_uuid(),
                name: m.name.as_str().to_owned(),
                email: m.email.as_str().to_owned(),
                role: m.role.into(),
                joined_at: time_out(m.joined_at)?,
                removed_at: optional_out(m.removed_at)?,
            })
        })
        .collect::<Result<_, DocumentError>>()?;
    let invitations = parts
        .invitations
        .iter()
        .map(|i| {
            Ok(InvitationV1 {
                id: i.id.as_uuid(),
                email: i.email.as_str().to_owned(),
                role: i.role.into(),
                link_hash: i.link_hash.as_str().to_owned(),
                invited_by: i.invited_by.as_uuid(),
                sent_at: time_out(i.sent_at)?,
                expires_at: time_out(i.expires_at)?,
                outcome: match i.outcome {
                    InvitationOutcome::Open => OutcomeV1::Open,
                    InvitationOutcome::Accepted { member_id, at } => OutcomeV1::Accepted {
                        member_id: member_id.as_uuid(),
                        at: time_out(at)?,
                    },
                    InvitationOutcome::Revoked { at } => OutcomeV1::Revoked { at: time_out(at)? },
                },
            })
        })
        .collect::<Result<_, DocumentError>>()?;
    let kids = parts
        .kids
        .iter()
        .map(|k| {
            Ok(KidV1 {
                id: k.id.as_uuid(),
                name: k.name.as_str().to_owned(),
                wallet_id: k.wallet_id.as_uuid(),
                added_at: time_out(k.added_at)?,
                hidden: k.hidden,
                archived_at: optional_out(k.archived_at)?,
            })
        })
        .collect::<Result<_, DocumentError>>()?;
    versioned(&FamilyV1 {
        name: parts.name.as_str().to_owned(),
        currency: parts.currency.code().to_owned(),
        created_at: time_out(parts.created_at)?,
        members,
        invitations,
        kids,
    })
}

pub fn decode_family(id: Uuid, body: Value) -> Result<Family, DocumentError> {
    let stored: FamilyV1 = serde_json::from_value(unversioned(body)?)?;
    let members = stored
        .members
        .into_iter()
        .map(|m| {
            Ok(Member {
                id: MemberId::from_uuid(m.id),
                user_id: UserId::from_uuid(m.user_id),
                name: PersonName::new(m.name)?,
                email: Email::new(m.email)?,
                role: m.role.into(),
                joined_at: time_in(&m.joined_at)?,
                removed_at: optional_in(m.removed_at)?,
            })
        })
        .collect::<Result<_, DocumentError>>()?;
    let invitations = stored
        .invitations
        .into_iter()
        .map(|i| {
            Ok(Invitation {
                id: InvitationId::from_uuid(i.id),
                email: Email::new(i.email)?,
                role: i.role.into(),
                link_hash: InvitationLinkHash::new(i.link_hash)
                    .map_err(|_| DocumentError::EmptyLinkHash)?,
                invited_by: MemberId::from_uuid(i.invited_by),
                sent_at: time_in(&i.sent_at)?,
                expires_at: time_in(&i.expires_at)?,
                outcome: match i.outcome {
                    OutcomeV1::Open => InvitationOutcome::Open,
                    OutcomeV1::Accepted { member_id, at } => InvitationOutcome::Accepted {
                        member_id: MemberId::from_uuid(member_id),
                        at: time_in(&at)?,
                    },
                    OutcomeV1::Revoked { at } => InvitationOutcome::Revoked { at: time_in(&at)? },
                },
            })
        })
        .collect::<Result<_, DocumentError>>()?;
    let kids = stored
        .kids
        .into_iter()
        .map(|k| {
            Ok(Kid {
                id: KidId::from_uuid(k.id),
                name: KidName::new(k.name)?,
                wallet_id: WalletId::from_uuid(k.wallet_id),
                added_at: time_in(&k.added_at)?,
                hidden: k.hidden,
                archived_at: optional_in(k.archived_at)?,
            })
        })
        .collect::<Result<_, DocumentError>>()?;
    Ok(Family::from_parts(FamilyParts {
        id: FamilyId::from_uuid(id),
        name: FamilyName::new(stored.name)?,
        currency: Currency::new(&stored.currency)?,
        created_at: time_in(&stored.created_at)?,
        members,
        invitations,
        kids,
    })?)
}

/// The containment pattern that finds Families where a User has (or had) a Membership.
pub fn member_pattern(user_id: UserId) -> Value {
    serde_json::json!({ "members": [{ "user_id": user_id.as_uuid() }] })
}

/// The containment pattern that finds the Family holding an Invitation link.
pub fn invitation_pattern(link_hash: &InvitationLinkHash) -> Value {
    serde_json::json!({ "invitations": [{ "link_hash": link_hash.as_str() }] })
}

// ---------- User ----------

pub fn encode_user(user: &User, password_hash: &str) -> Result<Value, DocumentError> {
    versioned(&UserV1 {
        email: user.email.as_str().to_owned(),
        name: user.name.as_str().to_owned(),
        password_hash: password_hash.to_owned(),
        registered_at: time_out(user.registered_at)?,
    })
}

/// A User and their password hash.
pub fn decode_user(id: Uuid, body: Value) -> Result<(User, String), DocumentError> {
    let stored: UserV1 = serde_json::from_value(unversioned(body)?)?;
    Ok((
        User {
            id: UserId::from_uuid(id),
            email: Email::new(stored.email)?,
            name: PersonName::new(stored.name)?,
            registered_at: time_in(&stored.registered_at)?,
        },
        stored.password_hash,
    ))
}

#[cfg(test)]
#[path = "family_body_tests.rs"]
mod tests;
