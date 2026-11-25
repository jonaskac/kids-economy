---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0010 — Families and Users as documents, decided like the Wallet

## Context and problem statement

Roadmap step 5 adds Users, Families, Members, Invitations and Kids. ADR-0005 planned a
`documents` table for entities (current state as JSONB), and ADR-0008 made the Wallet a
pure decider. We need to decide which aggregates exist, how they are decided, and how they
are stored, including adding a Kid, which touches both a Family and a new Wallet.

## Decision drivers

- Domain stays pure (no IO, no clock, no randomness) and every rule is a unit test.
- A Family always keeps at least one active Owner, even when two Owners act at once.
- Every query is scoped by Family (tenant boundary); no data crosses Families.
- Adding a Kid and opening the Kid's Wallet happen both or neither.

## Considered options

1. One aggregate per entity (Member, Invitation, Kid documents of their own).
2. **Family as the aggregate** holding its Members, Invitations and Kids; User separate.
3. Event-sourced Family in `ledger_events`, like the Wallet.

## Decision outcome

Chosen option: 2.

- The **Family** is the unit of consistency for people: "at least one active Owner", "one
  pending Invitation per email", "Viewers change nothing" are all checked inside it. It is a
  decider like the Wallet: `Family::decide(&self, &FamilyCommand, &Context, &mut impl NewIds)`
  returns `FamilyEvent`s, `Family::apply` folds them. `Family::accept` handles an Invitation,
  whose User is not a Member yet. A Family's size (a handful of adults and Kids) keeps it small.
- The **User** is separate (one User can be a Member of several Families) and holds only
  identity: email, name, password hash.
- Both are stored in `documents` (ADR-0005) as their **current state**, not as events: the
  Family's history of who joined when is kept in the state itself (`joined_at`, `removed_at`,
  Invitation status), and money history stays in `ledger_events`. `version` gives optimistic
  concurrency: a save is `update … where version = expected`; zero rows is a `Conflict`.
- Body formats are owned by the store with `"v": 1`, separate from domain types (as ADR-0009).
- Adding a Kid saves the Family and the new Wallet's `WalletOpened` in **one database
  transaction**.
- Lookups by email (Users) and by Invitation link or Member (Families) use expression and
  GIN indexes on the JSONB body.

### Consequences

- Good: same decide/apply shape everywhere; rules such as "last Owner" are plain unit tests.
- Good: no migration per new field; Family reads are one row.
- Bad: two Members changing the same Family at the same moment get a Conflict and retry.
- Bad: no full event history for Family changes; if an audit log is needed, add events then.

### Confirmation

`crates/domain/src/family/tests.rs` drives every Family rule. `crates/store/tests/family_store.rs`
runs against real Postgres, including a Conflict and Family scoping.

## More information

Related: ADR-0005, ADR-0008, ADR-0009, ADR-0011. Glossary: Family, User, Member, Role,
Invitation, Kid.
