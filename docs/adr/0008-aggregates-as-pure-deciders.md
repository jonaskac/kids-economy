---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0008 — Aggregates are pure deciders over their history

## Context and problem statement

Roadmap step 3 builds the Ledger in `crates/domain`. We need one shape for aggregates
that keeps the domain free of IO (ADR-0003), testable in milliseconds (ADR-0006), and a
natural fit for an append-only ledger (ADR-0005).

## Decision drivers

- Ledgers are append-only; Balance is derived, never stored as truth.
- A Transfer touches two Accounts and must be both or neither.
- Tests must be deterministic: no clock, no random ids inside the domain.

## Considered options

1. Mutable entities with methods (`account.withdraw(..)`) saved as current state.
2. **Decider**: `decide(state, command) -> events | error`, `apply(state, event) -> state`,
   state rebuilt by replaying the history.

## Decision outcome

Chosen option: 2, the decider, with the **Wallet** as the aggregate.

- The Wallet (all of one Kid's Accounts) is the unit of consistency, so a Transfer and a
  corrected Transfer are a single decision. The Account is an entity inside it.
- `Wallet::decide(&self, &WalletCommand, &Context, &mut impl NewIds)` returns the events to
  record or a typed `WalletError`; it never changes state. `Wallet::apply` folds one event;
  `Wallet::replay` rebuilds from the full history, which must start with `WalletOpened`.
- Time (`Context.now`) and new ids (`NewIds`) come from the caller. The Actor carries the
  Member's Role; the caller has already checked the Member is active in the Wallet's Family.
- Balances are computed from the Ledger on read. Caching them is a later optimisation in
  the store, never the truth.

### Consequences

- Good: every rule is a plain unit test; the store only appends events and replays them.
- Good: the events are exactly what `ledger_events` will hold.
- Bad: ADR-0005 sketches `ledger_events` keyed per Account; with the Wallet as the unit of
  consistency, step 4 should key the sequence per Wallet (`wallet_id, sequence`) so two
  concurrent commands on one Wallet conflict. To be settled when ADR-0005 is accepted.
- Bad: replay cost grows with history; fine for a family's volume, snapshot later if needed.

### Confirmation

`crates/domain/src/wallet/tests.rs` drives every Wallet rule, including
`replaying_the_history_rebuilds_the_same_wallet`.

## More information

Glossary: Wallet, Account, Transaction, Transfer, Correction, Command, Event, History.
