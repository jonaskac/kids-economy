---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0005 — PostgreSQL used schema-less: JSONB documents + append-only ledger

## Context and problem statement

We use PostgreSQL for users, families and accounting, and want the data model to be
schema-less so the domain can evolve without a migration for every new field.
Accounting, however, must never lose or silently change money.

## Decision drivers

- Domain evolves fast early on (kid app, goals, allowances…).
- Money correctness: auditability, no lost updates, no floats.
- Keep Postgres as the single datastore.

## Considered options

1. Classic normalised relational schema
2. Fully schema-less: one `documents` table, everything JSONB
3. Thin fixed envelope + JSONB body, with the ledger as append-only events

## Decision outcome

Chosen option: 3 (pending confirmation).

```sql
-- Entities (users, families, kids, accounts): current state as documents
documents (
  id          uuid primary key,
  kind        text not null,          -- 'user' | 'family' | 'kid' | 'account' …
  family_id   uuid,                   -- tenant key, indexed, used for every query
  version     bigint not null,        -- optimistic concurrency
  body        jsonb not null,
  created_at  timestamptz not null,
  updated_at  timestamptz not null
)

-- Ledger: immutable, append-only facts. Balance is derived, never stored as truth.
ledger_events (
  account_id  uuid not null,
  sequence    bigint not null,        -- per account, gapless; (account_id, sequence) is the PK
  kind        text not null,          -- 'deposit' | 'withdrawal' | 'correction' …
  body        jsonb not null,         -- amount in minor units, memo, actor …
  recorded_at timestamptz not null,
  primary key (account_id, sequence)
)
```

- Only identity, tenancy, ordering and concurrency columns are fixed; everything
  else is JSONB and validated in the Rust domain layer, not by the database.
- The `(account_id, sequence)` primary key makes concurrent writes to the same account
  conflict instead of both succeeding — this is how "can't withdraw more than you have"
  holds under concurrency.
- Body shapes carry a `"v"` schema version; the domain upcasts old shapes on read.

### Consequences

- Good: new fields need no migration; the ledger is a full audit trail; balances can be rebuilt.
- Bad: the database no longer enforces field-level constraints — the domain must, and
  tests must cover it. Ad-hoc reporting needs JSONB queries.
