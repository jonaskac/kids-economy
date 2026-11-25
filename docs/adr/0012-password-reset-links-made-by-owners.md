---
status: accepted
date: 2026-11-25
deciders: [Jonas]
---

# 0012 — Password reset links, made by an Owner for a Member

## Context and problem statement

ADR-0011 left v1 without a password reset, because the app sends no email. A grandparent
who forgets their password today has to be removed and invited again. We need a way back in
that still needs no outside service.

## Decision drivers

- No email provider or other secret in v1 (ADR-0007, ADR-0011).
- Fixable from the Owner's phone, without a shell on the server.
- Same safety as Sessions and Invitation links: secrets stored only as hashes, short-lived.

## Considered options

1. **An Owner makes a Password reset link for a Member of their Family** and shares it,
   like an Invitation link.
2. A server command prints a reset link for an email (only whoever runs the server).
3. No reset until email sending exists.

## Decision outcome

Chosen option: 1 (Jonas, 2026-11-25), because it reuses the
Invitation link pattern and an Owner can do it from the app.

- An Owner of a Family may make a link for any active Member of that Family (the domain
  rule `Family::member_for_password_reset`). Parents and Viewers may not.
- The link is `/reset/<secret>`: 32 random bytes, shown once; only its SHA-256 is stored in
  a small `password_resets` table (like `sessions`), keyed by the hash.
- It works once, for 24 hours. Making a new link for the same User makes earlier unused
  ones stop working.
- Using it sets the new password, ends **all** of that User's Sessions, and signs them in
  on the device that used it, in one database transaction.

### Consequences

- Good: no outside service; the forgotten-password case no longer loses a Member's history
  or Role.
- Bad: a User belongs to every Family they are a Member of, so an Owner of one Family can
  take over a User who is also a Member of another Family. Acceptable for v1, where
  Families are relatives who trust each other; when email sending exists, deliver the link
  to the User's own email instead and drop Owner-made links.
- Bad: the Owner sees the link, so they could use it themselves. Ending every Session on
  use means the Member notices (they are signed out).

### Confirmation

`crates/domain` tests the Owner-only rule; `crates/store` tests single use, expiry and
Session ending against real Postgres; `crates/http` tests the whole flow in-process.

## More information

Refines the "no password reset" consequence of ADR-0011. Glossary: Password reset link,
Owner, Member, Session.
