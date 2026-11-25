# HTTP API

All routes live under `/api`, take and return JSON, and use the words in
[`glossary.md`](glossary.md). Roadmap step 6 completes this spec; step 5 adds sign-in,
Families, Invitations and Kids (ADR-0010, ADR-0011).

## Conventions

- **Sign-in** sets cookie `session` (`HttpOnly; Secure; SameSite=Lax; Path=/api`, 30 days).
  Routes marked 🔒 answer `401 not_signed_in` without a valid Session.
- **Changing requests** must send `Content-Type: application/json` (otherwise `415`); this,
  with `SameSite=Lax`, is the CSRF guard.
- **Errors** are `{"error": "<code>", "message": "<text>"}`. Codes are stable; messages are not.
- **Tenant boundary**: every `/api/families/{id}/…` route answers `404 family_not_found` to
  anyone who is not an active Member of that Family.
- **Roles** are `"owner" | "parent" | "viewer"`. Amounts are `{"minor": 1250, "currency": "SEK"}`.
- **Conflict**: if another Member saved first, the server decides again up to three times,
  then answers `409 conflict`.

## Users and Sessions

| Method and path | Body | Success | Errors |
| --- | --- | --- | --- |
| `POST /api/users` (sign up) | `{email, name, password}` | `201 {user}` + cookie | `422 invalid_email`, `password_too_short` (under 10), `password_too_long` (over 128), `invalid_text`; `409 email_taken` |
| `POST /api/sessions` (sign in) | `{email, password}` | `200 {user}` + cookie | `401 wrong_email_or_password` |
| `DELETE /api/sessions/current` (sign out) | | `204`, cookie cleared | |
| 🔒 `GET /api/me` | | `200 {user, families: [{id, name, role}]}` | |

`user` is `{id, email, name}`.

## Families

| Method and path | Body | Success | Errors |
| --- | --- | --- | --- |
| 🔒 `POST /api/families` | `{name, currency}` | `201 FamilyView`; you are its Owner | `422 invalid_text`, `invalid_currency` |
| 🔒 `GET /api/families/{id}` | | `200 FamilyView` | |
| 🔒 `POST /api/families/{id}/invitations` | `{email, role}` | `201 {invitation, link}` | `403 not_allowed` (not Owner); `409 already_member`, `invitation_pending` |
| 🔒 `DELETE /api/families/{id}/invitations/{invitation_id}` | | `204` | `403`, `404 unknown_invitation`, `409 invitation_not_pending` |
| 🔒 `PATCH /api/families/{id}/members/{member_id}` | `{role}` | `204` | `403`, `404 unknown_member`, `409 last_owner` |
| 🔒 `DELETE /api/families/{id}/members/{member_id}` | | `204` (yourself: leave) | `403`, `404 unknown_member`, `409 last_owner` |
| 🔒 `POST /api/families/{id}/kids` | `{name}` | `201 {id, name, wallet_id}`; the Wallet opens with a Main account | `403` (Viewer), `422 invalid_text` |
| 🔒 `POST /api/families/{id}/kids/{kid_id}/archive` | `{}` | `204` | `403`, `404 unknown_kid`, `409 kid_archived` |

```json
{
  "id": "…", "name": "The Kacs", "currency": "SEK",
  "you": { "member_id": "…", "role": "owner" },
  "members": [{ "id": "…", "name": "Jonas", "email": "jonas@example.com", "role": "owner", "joined_at": "2026-11-25T13:00:00Z" }],
  "invitations": [{ "id": "…", "email": "mormor@example.com", "role": "viewer", "expires_at": "2026-12-02T13:00:00Z" }],
  "kids": [{ "id": "…", "name": "Elsa", "wallet_id": "…", "wallet_total": { "minor": 0, "currency": "SEK" } }]
}
```

`members` lists active Members; `invitations` lists pending ones and is empty for
non-Owners; `kids` lists active Kids.

## Invitation links

The `link` returned when inviting is `/join/<secret>`, shown once. The web app's `/join/<secret>`
page calls:

| Method and path | Success | Errors |
| --- | --- | --- |
| `GET /api/invitations/{secret}` (no sign-in needed) | `200 {family_name, email, role, state, expires_at}`; `state` is `pending`, `accepted`, `revoked` or `expired` | `404 unknown_invitation` |
| 🔒 `POST /api/invitations/{secret}/accept` | `200 {family_id, member_id, role}` | `403 wrong_email`, `409 already_member`, `invitation_not_pending`, `410 invitation_expired` |

Try it locally (nushell):

```nu
cd apps/api
$env.DATABASE_URL = "postgres://postgres:postgres@localhost:5432/postgres"
cargo run -p server
# in another shell
http post --content-type application/json http://localhost:8080/api/users {email: "jonas@example.com", name: "Jonas", password: "correct horse battery"} --full | get headers.response
```
