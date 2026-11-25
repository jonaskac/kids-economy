//! A Family's Members, Invitations and Kids, for its active Members only (ADR-0010).

use crate::auth::{new_secret, SignedIn};
use crate::error::ApiError;
use crate::{AppState, RandomIds};
use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::Json;
use domain::family::{
    CreateFamily, Family, FamilyCommand, FamilyError, FamilyEvent, InvitationLinkHash,
    InvitationState, Person,
};
use domain::identity::{Email, User};
use domain::ids::{FamilyId, InvitationId, KidId, MemberId};
use domain::money::Currency;
use domain::role::{Actor, Context, Role};
use domain::text::{AccountName, FamilyName, KidName};
use domain::wallet::{OpenWallet, Wallet};
use serde::{Deserialize, Serialize};
use store::{LoadedFamily, StoreError};
use time::OffsetDateTime;
use uuid::Uuid;

/// How often a decision is retried when another Member saved first.
const ATTEMPTS: usize = 3;

/// Name of the Main account every new Kid's Wallet opens with.
const MAIN_ACCOUNT_NAME: &str = "Main";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RoleDto {
    Owner,
    Parent,
    Viewer,
}

impl From<RoleDto> for Role {
    fn from(role: RoleDto) -> Self {
        match role {
            RoleDto::Owner => Role::Owner,
            RoleDto::Parent => Role::Parent,
            RoleDto::Viewer => Role::Viewer,
        }
    }
}

pub(crate) const fn role_name(role: Role) -> &'static str {
    match role {
        Role::Owner => "owner",
        Role::Parent => "parent",
        Role::Viewer => "viewer",
    }
}

// ---------- DTOs ----------

#[derive(Debug, Deserialize)]
pub struct NewFamily {
    name: String,
    currency: String,
}

#[derive(Debug, Deserialize)]
pub struct NewInvitation {
    email: String,
    role: RoleDto,
}

#[derive(Debug, Deserialize)]
pub struct NewRole {
    role: RoleDto,
}

#[derive(Debug, Deserialize)]
pub struct NewKid {
    name: String,
}

#[derive(Debug, Serialize)]
pub struct You {
    member_id: Uuid,
    role: &'static str,
}

#[derive(Debug, Serialize)]
pub struct MemberDto {
    id: Uuid,
    name: String,
    email: String,
    role: &'static str,
    #[serde(with = "time::serde::rfc3339")]
    joined_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct InvitationDto {
    id: Uuid,
    email: String,
    role: &'static str,
    #[serde(with = "time::serde::rfc3339")]
    expires_at: OffsetDateTime,
}

#[derive(Debug, Serialize)]
pub struct MoneyDto {
    minor: i64,
    currency: String,
}

#[derive(Debug, Serialize)]
pub struct KidDto {
    id: Uuid,
    name: String,
    wallet_id: Uuid,
    #[serde(skip_serializing_if = "Option::is_none")]
    wallet_total: Option<MoneyDto>,
}

#[derive(Debug, Serialize)]
pub struct FamilyView {
    id: Uuid,
    name: String,
    currency: String,
    you: You,
    members: Vec<MemberDto>,
    /// Pending Invitations; only Owners see them.
    invitations: Vec<InvitationDto>,
    kids: Vec<KidDto>,
}

#[derive(Debug, Serialize)]
pub struct InvitationCreated {
    invitation: InvitationDto,
    /// The Invitation link, shown this once. Only its hash is stored.
    link: String,
}

// ---------- handlers ----------

/// `POST /api/families`: founds a Family; the signed-in User becomes its Owner.
pub async fn create(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Json(body): Json<NewFamily>,
) -> Result<(StatusCode, Json<FamilyView>), ApiError> {
    let command = CreateFamily {
        family_id: FamilyId::from_uuid(Uuid::new_v4()),
        name: FamilyName::new(&body.name)?,
        currency: Currency::new(body.currency.trim())?,
        founder: person(&signed_in.user),
    };
    let created = Family::create(&command, OffsetDateTime::now_utc(), &mut RandomIds);
    let family = Family::replay([&created]).map_err(|_| ApiError::Internal)?;
    state.families.create(&family).await?;
    Ok((
        StatusCode::CREATED,
        Json(view_of(&state, &family, &signed_in.user).await?),
    ))
}

/// `GET /api/families/{id}`: the Family as its Member sees it.
pub async fn view(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path(family_id): Path<Uuid>,
) -> Result<Json<FamilyView>, ApiError> {
    let (loaded, _) = load_as_member(&state, family_id, &signed_in.user).await?;
    Ok(Json(
        view_of(&state, &loaded.family, &signed_in.user).await?,
    ))
}

/// `POST /api/families/{id}/invitations`: an Owner invites an adult; returns the link.
pub async fn invite(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path(family_id): Path<Uuid>,
    Json(body): Json<NewInvitation>,
) -> Result<(StatusCode, Json<InvitationCreated>), ApiError> {
    let email = Email::new(&body.email)?;
    let (secret, hash) = new_secret();
    let link_hash = InvitationLinkHash::new(hash).map_err(|_| ApiError::Internal)?;
    let command = FamilyCommand::Invite {
        email,
        role: body.role.into(),
        link_hash: link_hash.clone(),
    };
    let family = change(&state, family_id, &signed_in.user, |family, context| {
        family.decide(&command, &context, &mut RandomIds)
    })
    .await?;
    let invitation = family
        .invitation_by_link(&link_hash)
        .ok_or(ApiError::Internal)?;
    Ok((
        StatusCode::CREATED,
        Json(InvitationCreated {
            invitation: InvitationDto {
                id: invitation.id.as_uuid(),
                email: invitation.email.as_str().to_owned(),
                role: role_name(invitation.role),
                expires_at: invitation.expires_at,
            },
            link: format!("/join/{secret}"),
        }),
    ))
}

/// `DELETE /api/families/{id}/invitations/{invitation_id}`: an Owner revokes one.
pub async fn revoke(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path((family_id, invitation_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = FamilyCommand::RevokeInvitation {
        invitation_id: InvitationId::from_uuid(invitation_id),
    };
    change(&state, family_id, &signed_in.user, |family, context| {
        family.decide(&command, &context, &mut RandomIds)
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `PATCH /api/families/{id}/members/{member_id}`: an Owner changes a Member's Role.
pub async fn change_role(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path((family_id, member_id)): Path<(Uuid, Uuid)>,
    Json(body): Json<NewRole>,
) -> Result<StatusCode, ApiError> {
    let command = FamilyCommand::ChangeRole {
        member_id: MemberId::from_uuid(member_id),
        role: body.role.into(),
    };
    change(&state, family_id, &signed_in.user, |family, context| {
        family.decide(&command, &context, &mut RandomIds)
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `DELETE /api/families/{id}/members/{member_id}`: an Owner removes a Member, or a
/// Member leaves.
pub async fn remove_member(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path((family_id, member_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = FamilyCommand::RemoveMember {
        member_id: MemberId::from_uuid(member_id),
    };
    change(&state, family_id, &signed_in.user, |family, context| {
        family.decide(&command, &context, &mut RandomIds)
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

/// `POST /api/families/{id}/kids`: adds a Kid and opens their Wallet, both or neither.
pub async fn add_kid(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path(family_id): Path<Uuid>,
    Json(body): Json<NewKid>,
) -> Result<(StatusCode, Json<KidDto>), ApiError> {
    let command = FamilyCommand::AddKid {
        name: KidName::new(&body.name)?,
    };
    let main_account_name = AccountName::new(MAIN_ACCOUNT_NAME)?;
    for _ in 0..ATTEMPTS {
        let (loaded, actor) = load_as_member(&state, family_id, &signed_in.user).await?;
        let context = Context {
            actor,
            now: OffsetDateTime::now_utc(),
        };
        let events = loaded.family.decide(&command, &context, &mut RandomIds)?;
        let next = applied(&loaded.family, &events)?;
        let Some(FamilyEvent::KidAdded { kid }) = events.first() else {
            return Err(ApiError::Internal);
        };
        let opened = Wallet::open(
            &OpenWallet {
                wallet_id: kid.wallet_id,
                family_id: next.id(),
                kid_id: kid.id,
                currency: next.currency(),
                main_account_name: main_account_name.clone(),
            },
            &context,
            &mut RandomIds,
        )?;
        match state
            .families
            .save_with_new_wallet(&next, loaded.version, &opened)
            .await
        {
            Ok(_) => {
                return Ok((
                    StatusCode::CREATED,
                    Json(KidDto {
                        id: kid.id.as_uuid(),
                        name: kid.name.as_str().to_owned(),
                        wallet_id: kid.wallet_id.as_uuid(),
                        wallet_total: None,
                    }),
                ))
            }
            Err(StoreError::Conflict) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ApiError::Conflict)
}

/// `POST /api/families/{id}/kids/{kid_id}/archive`: an Owner or Parent archives a Kid.
pub async fn archive_kid(
    State(state): State<AppState>,
    signed_in: SignedIn,
    Path((family_id, kid_id)): Path<(Uuid, Uuid)>,
) -> Result<StatusCode, ApiError> {
    let command = FamilyCommand::ArchiveKid {
        kid_id: KidId::from_uuid(kid_id),
    };
    change(&state, family_id, &signed_in.user, |family, context| {
        family.decide(&command, &context, &mut RandomIds)
    })
    .await?;
    Ok(StatusCode::NO_CONTENT)
}

// ---------- helpers ----------

pub(crate) fn person(user: &User) -> Person {
    Person {
        user_id: user.id,
        name: user.name.clone(),
        email: user.email.clone(),
    }
}

/// Loads a Family the User is an active Member of; 404 for everyone else.
async fn load_as_member(
    state: &AppState,
    family_id: Uuid,
    user: &User,
) -> Result<(LoadedFamily, Actor), ApiError> {
    let loaded = state
        .families
        .load(FamilyId::from_uuid(family_id))
        .await?
        .ok_or(ApiError::FamilyNotFound)?;
    let actor = loaded
        .family
        .actor_for(user.id)
        .ok_or(ApiError::FamilyNotFound)?;
    Ok((loaded, actor))
}

pub(crate) fn applied(family: &Family, events: &[FamilyEvent]) -> Result<Family, ApiError> {
    let mut next = family.clone();
    for event in events {
        next.apply(event).map_err(|_| ApiError::Internal)?;
    }
    Ok(next)
}

/// Loads, decides and saves, deciding again if another Member saved first.
async fn change<F>(
    state: &AppState,
    family_id: Uuid,
    user: &User,
    decide: F,
) -> Result<Family, ApiError>
where
    F: Fn(&Family, Context) -> Result<Vec<FamilyEvent>, FamilyError>,
{
    for _ in 0..ATTEMPTS {
        let (loaded, actor) = load_as_member(state, family_id, user).await?;
        let context = Context {
            actor,
            now: OffsetDateTime::now_utc(),
        };
        let next = applied(&loaded.family, &decide(&loaded.family, context)?)?;
        match state.families.save(&next, loaded.version).await {
            Ok(_) => return Ok(next),
            Err(StoreError::Conflict) => continue,
            Err(error) => return Err(error.into()),
        }
    }
    Err(ApiError::Conflict)
}

async fn view_of(state: &AppState, family: &Family, user: &User) -> Result<FamilyView, ApiError> {
    let actor = family.actor_for(user.id).ok_or(ApiError::FamilyNotFound)?;
    let now = OffsetDateTime::now_utc();
    let members = family
        .active_members()
        .map(|m| MemberDto {
            id: m.id.as_uuid(),
            name: m.name.as_str().to_owned(),
            email: m.email.as_str().to_owned(),
            role: role_name(m.role),
            joined_at: m.joined_at,
        })
        .collect();
    let invitations = if actor.role == Role::Owner {
        family
            .invitations()
            .filter(|i| i.state_at(now) == InvitationState::Pending)
            .map(|i| InvitationDto {
                id: i.id.as_uuid(),
                email: i.email.as_str().to_owned(),
                role: role_name(i.role),
                expires_at: i.expires_at,
            })
            .collect()
    } else {
        Vec::new()
    };
    let mut kids = Vec::new();
    for kid in family.active_kids() {
        let total = state
            .wallets
            .load(family.id(), kid.wallet_id)
            .await?
            .map(|loaded| loaded.wallet.total());
        kids.push(KidDto {
            id: kid.id.as_uuid(),
            name: kid.name.as_str().to_owned(),
            wallet_id: kid.wallet_id.as_uuid(),
            wallet_total: total.map(|total| MoneyDto {
                minor: total.minor(),
                currency: total.currency().code().to_owned(),
            }),
        });
    }
    Ok(FamilyView {
        id: family.id().as_uuid(),
        name: family.name().as_str().to_owned(),
        currency: family.currency().code().to_owned(),
        you: You {
            member_id: actor.member_id.as_uuid(),
            role: role_name(actor.role),
        },
        members,
        invitations,
        kids,
    })
}
