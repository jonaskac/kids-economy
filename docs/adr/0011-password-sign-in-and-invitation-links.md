---
status: accepted
date: 2026-11-25
deciders: [Jonas]
---

# 0011 — Sign in with email and password; Invitations as shareable links

## Context and problem statement

Adults need to sign in on a phone, and an Owner needs to invite a second Parent or a
grandparent. We must choose how Users authenticate and how Invitations reach people.

## Decision drivers

- No outside service or secret needed to run v1 (one container on Sliplane, ADR-0007).
- Testable in-process (ADR-0006).
- Safe defaults: no plain passwords stored, no session secret readable by scripts.

## Considered options

1. **Email + password, server-side Sessions, Invitation links shared by the Owner.**
2. Magic links by email (needs an email provider and its API key).
3. Sign in with Google / Apple (OAuth registrations and secrets with both).

## Decision outcome

Chosen option: 1 (Jonas, 2026-11-25).

- Passwords: 10 to 128 characters, hashed with **Argon2id** (`argon2` crate, PHC string).
  Sign-in with an unknown email still runs a hash, so timing does not reveal accounts.
- Sessions: 32 random bytes, sent as cookie `session` with `HttpOnly; Secure; SameSite=Lax;
  Path=/api`, lasting 30 days. Only the SHA-256 of the secret is stored, in a small fixed
  `sessions` table (not a document: it is looked up on every request by its hash).
  Sign-out deletes the row.
- CSRF: changing requests need `Content-Type: application/json` (axum's `Json`), which a
  cross-site form cannot send without a CORS preflight we never allow; plus `SameSite=Lax`.
- Invitations: the server makes a random secret, stores its SHA-256 on the Invitation and
  returns the Invitation link once (`/join/<secret>`). The Owner shares it. Opening it shows
  the Family name and Role; accepting needs a User signed in with the invited email.
  Expires after 7 days.

### Consequences

- Good: nothing external; email sending (option 2) can later deliver the same links.
- Bad: no email verification yet. Mitigated because an Invitation needs both the secret link
  and the matching email; a stranger signing up with someone else's email gains nothing.
- Bad: no password reset without email; until email sending exists, an Owner can remove and
  re-invite. Rate limiting sign-in is left for roadmap step 10.

### Confirmation

`crates/http` tests sign-up, sign-in, sign-out, cookie flags, and the Invitation flow
in-process against real Postgres.

## More information

Related: ADR-0007, ADR-0010. Glossary: User, Session, Invitation, Invitation link.
