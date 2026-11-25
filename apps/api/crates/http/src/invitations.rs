//! Opening and accepting an Invitation link (ADR-0011).

use crate::auth::{hash_secret, SignedIn};
use crate::error::ApiError;
use crate::families::{applied, person, role_name};
use crate::{AppState, RandomIds};
use axum::extract::{Path, State};
use axum::Json;
use domain::family::{
    AcceptInvitation, FamilyError, FamilyEvent, InvitationLinkHash, InvitationState,
};
use serde::Serialize;
use store::StoreError;
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Debug, Serialize)]
pub struct Preview {
    family_name: String,
    email: String,
    role: &'static str,
    state: &'static str,
    #[serde(with = "time::serde::rfc3339")]
    expires_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct Joined {
    family_id: Uuid,
    member_id: Uuid,
    role: &'static str,
}

fn link_hash(secret: &str) -> Result<InvitationLinkHash, ApiError> {
    InvitationLinkHash::new(hash_secret(secret)).map_err(|_| ApiError::Internal)
}

/// `GET /api/invitations/{secret}`: what the link offers, so it can be shown before
/// signing in. Anyone with the link may see it.
pub async fn preview(
    State(state): State<AppState>,
    Path(secret): Path<String>,
) -> Result<Json<Preview>, ApiError> {
    let link_hash = link_hash(&secret)?;
    let loaded = state
        .families
        .by_invitation_link(&link_hash)
        .await?
        .ok_or(FamilyError::UnknownInvitation)?;
    let invitation = loaded
        .family
        .invitation_by_link(&link_hash)
        .ok_or(FamilyError::UnknownInvitation)?;
    let state_name = match invitation.state_at(OffsetDateTime::now_utc()) {
        InvitationState::Pending => "pending",
        InvitationState::Accepted => "accepted",
        InvitationState::Revoked => "revoked",
        InvitationState::Expired => "expired",
    };
    Ok(Json(Preview {
        family_name: loaded.family.name().as_str().to_owned(),
        email: invitation.email.as_str().to_owned(),
        role: role_name(invitation.role),
        state: state_name,
        expires_at: invitation.expires_at,
    }))
}

/// `POST /api/invitations/{secret}/accept`: the signed-in User joins the Family.
pub async fn accept(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path(secret): Path<String>,
) -> Result<Json<Joined>, ApiError> {
    let command = AcceptInvitation {
        link_hash: link_hash(&secret)?,
        person: person(&signed_in.user),
    };
    for _ in 0..3 {
        let loaded = state
            .families
            .by_invitation_link(&command.link_hash)
            .await?
            .ok_or(FamilyError::UnknownInvitation)?;
        let events = loaded
            .family
            .accept(&command, OffsetDateTime::now_utc(), &mut RandomIds)?;
        let next = applied(&loaded.family, &events)?;
        let Some(FamilyEvent::MemberJoined { member, .. }) = events.first() else {
            return Err(ApiError::Internal);
        };
        match state.families.save(&next, loaded.version).await {
            Ok(_) => {
                return Ok(Json(Joined {
                    family_id: next.id().as_uuid(),
                    member_id: member.id.as_uuid(),
                    role: role_name(member.role),
                }))
            }
            Err(StoreError::Conflict) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ApiError::Conflict)
}
