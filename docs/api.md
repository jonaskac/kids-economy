# HTTP API

All routes live under `/api`, take and return JSON, and use the words in
[`glossary.md`](glossary.md). Roadmap step 6 completes this spec; step 5 adds sign-in,
Families, Invitations and Kids (ADR-0010, ADR-0011); Password reset links follow ADR-0012.

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
| 🔒 `POST /api/families/{id}/members/{member_id}/password-reset` | `{}` | `201 {link, expires_at}`; see Password reset links | `403` (not Owner), `404 unknown_member` |
| 🔒 `POST /api/families/{id}/kids` | `{name}` | `201 {id, name, wallet_id}`; the Wallet opens with a Main account | `403` (Viewer), `422 invalid_text` |
| 🔒 `PATCH /api/families/{id}/kids/{kid_id}` | `{name?, hidden?}`; fixes the name and hides or shows the Kid on the Family home, all or nothing | `204` | `403` (Viewer), `404 unknown_kid`, `409 kid_archived`, `422 invalid_text` |
| 🔒 `POST /api/families/{id}/kids/{kid_id}/archive` | `{}`; "Remove kid": the Kid and Wallet are kept but no longer active | `204` | `403`, `404 unknown_kid`, `409 kid_archived` |

```json
{
  "id": "…", "name": "The Kacs", "currency": "SEK",
  "you": { "member_id": "…", "role": "owner" },
  "members": [{ "id": "…", "name": "Jonas", "email": "jonas@example.com", "role": "owner", "joined_at": "2026-11-25T13:00:00Z" }],
  "invitations": [{ "id": "…", "email": "mormor@example.com", "role": "viewer", "expires_at": "2026-12-02T13:00:00Z" }],
  "kids": [{ "id": "…", "name": "Elsa", "wallet_id": "…", "hidden": false, "wallet_total": { "minor": 0, "currency": "SEK" } }]
}
```

`members` lists active Members; `invitations` lists pending ones and is empty for
non-Owners; `kids` lists active Kids, hidden ones too (`hidden: true`; the app leaves them
off the Family home). Archived Kids are not listed (ADR-0013).

## Wallets

Each active Kid has one Wallet. Every route below sits under
`/api/families/{id}/wallets/{wallet_id}`, answers `404 wallet_not_found` for a Wallet that
is not an active Kid's in this Family, and every change answers `201 WalletView`. Viewers
read; changes by a Viewer answer `403 not_allowed`. Dates are `YYYY-MM-DD` and default to today.

| Method and path | Body | Errors |
| --- | --- | --- |
| 🔒 `GET …/{wallet_id}` | | |
| 🔒 `POST …/deposits` | `{account_id?, amount, category, memo, occurred_on?}`; `category` is `allowance`, `chore`, `gift` or `interest`; no `account_id` means the Main account | `422 invalid_amount`, `currency_mismatch`, `invalid_text`, `invalid_date`; `404 unknown_account` |
| 🔒 `POST …/withdrawals` | `{account_id, amount, category, memo, occurred_on?}`; `category` is `purchase` or `donation` | as above, plus `422 insufficient_funds` (no Overdraft) |
| 🔒 `POST …/transfers` | `{from, to, amount, memo, occurred_on?}` | as above, plus `422 same_account` |
| 🔒 `POST …/corrections` | `{transaction_id, reason, occurred_on?}`; `reason` becomes the Correction's Memo; correcting one side of a Transfer corrects both | `404 unknown_transaction`; `409 already_corrected`; `422 invalid_text`, `insufficient_funds` (the reversal would overdraw) |
| 🔒 `POST …/accounts` | `{name, savings_goal?: {name, target, target_date?}}` | `422 invalid_text`, `currency_mismatch` |
| 🔒 `POST …/accounts/{account_id}/close` | `{move_to?, memo, occurred_on?}`; what is left moves to `move_to` (default: the Main account) as one Transfer with `memo`, then the Account closes (ADR-0013) | `404 unknown_account`; `409 main_account`, `account_closed` (either Account); `422 same_account`, `invalid_text` |

```json
{
  "id": "…", "kid": { "id": "…", "name": "Elsa" }, "currency": "SEK",
  "total": { "minor": 3000, "currency": "SEK" },
  "accounts": [{ "id": "…", "name": "Main", "main": true, "balance": { "minor": 2000, "currency": "SEK" }, "savings_goal": null }],
  "closed_accounts": [{ "id": "…", "name": "Lego set" }],
  "transactions": [{ "id": "…", "account_id": "…", "direction": "in", "amount": { "minor": 3000, "currency": "SEK" },
    "category": "gift", "memo": "From mormor", "occurred_on": "2026-11-25", "recorded_at": "2026-11-25T13:00:00Z",
    "transfer_id": null, "corrects": null }]
}
```

`accounts` lists open Accounts, Main first; `closed_accounts` lists closed ones, so their
Transactions can still be named. `transactions` lists every Transaction on every
Account, newest first; the two sides of a Transfer share a `transfer_id`.

## Invitation links

The `link` returned when inviting is `/join/<secret>`, shown once. The web app's `/join/<secret>`
page calls:

| Method and path | Success | Errors |
| --- | --- | --- |
| `GET /api/invitations/{secret}` (no sign-in needed) | `200 {family_name, email, role, state, expires_at}`; `state` is `pending`, `accepted`, `revoked` or `expired` | `404 unknown_invitation` |
| 🔒 `POST /api/invitations/{secret}/accept` | `200 {family_id, member_id, role}` | `403 wrong_email`, `409 already_member`, `invitation_not_pending`, `410 invitation_expired` |

## Password reset links

An Owner makes one for a Member (ADR-0012). The `link` is `/reset/<secret>`, shown once,
working once for 24 hours; a new link for the same User replaces an unused one. The web
app's `/reset/<secret>` page calls:

| Method and path | Body | Success | Errors |
| --- | --- | --- | --- |
| `GET /api/password-resets/{secret}` | | `200 {email, name, state, expires_at}`; `state` is `pending`, `used` or `expired` | `404 unknown_password_reset` |
| `POST /api/password-resets/{secret}` | `{password}` | `200 {user}` + cookie; every other Session of that User ends | `404 unknown_password_reset`, `409 password_reset_used`, `410 password_reset_expired`, `422 password_too_short`, `password_too_long` |

Try it locally (nushell):

```nu
cd apps/api
$env.DATABASE_URL = "postgres://postgres:postgres@localhost:5432/postgres"
cargo run -p server
# in another shell
http post --content-type application/json http://localhost:8080/api/users {email: "jonas@example.com", name: "Jonas", password: "correct horse battery"} --full | get headers.response
```
