---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0009 — Wallet History in Postgres, numbered per Wallet

## Context and problem statement

Roadmap step 4 saves Wallets. ADR-0008 made the Wallet a pure decider whose state is
rebuilt from its History, and noted that ADR-0005's `ledger_events` table, keyed per
Account, does not match a Wallet-wide unit of consistency: two Members recording at the
same time on two different Accounts of one Wallet (or a Transfer and a Withdrawal) could
both be saved, each checked against a Wallet the other had already changed.

## Decision drivers

- No Overdraft must hold when two Members record at once.
- History is append-only and gapless, so it can be replayed and audited.
- Every query is scoped by Family (tenant boundary).
- The stored shape must survive domain refactors (renamed Rust fields, new variants).

## Considered options

1. Keep `(account_id, sequence)`; lock all of a Wallet's Accounts in a transaction.
2. Number Events per Wallet, `(wallet_id, sequence)` as primary key, optimistic
   concurrency on the Wallet's Version.
3. Store the Wallet as one JSONB document with a `version` column, no event rows.

## Decision outcome

Chosen option: 2.

```sql
ledger_events (
  family_id   uuid        not null,   -- tenant key, every read filters on it
  wallet_id   uuid        not null,
  sequence    bigint      not null check (sequence > 0),
  kind        text        not null,   -- 'wallet_opened' | 'transaction_recorded' | …
  body        jsonb       not null,   -- {"v": 1, …}
  recorded_at timestamptz not null default now(),
  primary key (wallet_id, sequence)
)
```

- The store loads a Wallet's History in Sequence order, replays it, and reports its
  Version. A decision is saved with `append(wallet, expected_version, events)`: in one
  database transaction the store checks the Wallet is still at that Version and inserts
  the new Events as `Version + 1, + 2, …`. If another save got there first, the primary key
  or the Version check refuses it and the store returns `Conflict`; nothing is saved.
- Before saving, the store folds the new Events into a copy of the Wallet with the domain's
  own `apply`, so an invalid History never reaches the database.
- The JSON body is a stored format owned by the store crate (`body.rs`), separate from the
  domain types, with `"v": 1`. Changing a domain type never silently changes stored data;
  a golden test pins the v1 shape. Old shapes are upcast on read when `v` moves on.
- Text and Amounts are validated again on read with the domain's constructors.
- The `documents` table from ADR-0005 arrives with roadmap step 5, when Members and Kids
  are the first thing to need it.

### Consequences

- Good: one Wallet-wide Conflict covers Deposits, Withdrawals, Transfers and Corrections.
- Good: History rows are the audit trail; Balances are still derived, never stored.
- Bad: two Members recording on one Wallet within the same instant get a Conflict and the
  caller must decide again. Rare for one Family, and retrying is cheap.
- Bad: the JSON mapping is written by hand, one more place to change per new Event.

### Confirmation

`crates/store/tests/wallet_store.rs` runs against real Postgres (`#[sqlx::test]`),
including two concurrent Withdrawals where only one is saved, and Family scoping.
`crates/store/src/body.rs` unit tests round-trip every Event and pin the v1 JSON.

## More information

Supersedes the `ledger_events` key in ADR-0005. Related: ADR-0008.
Glossary: History, Event, Sequence, Version, Conflict, Family.
