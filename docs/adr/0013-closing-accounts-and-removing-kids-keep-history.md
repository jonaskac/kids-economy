---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0013 — Closing an Account moves its money; Kids are hidden, renamed and archived, never deleted

## Context and problem statement

Jonas asked for four things (2026-11-25): hide a Kid, remove a Kid from Family settings,
fix a Kid entered by mistake, and remove an Account so that its money goes to the Main
account or another Account of his choosing, after which the Account is archived.

Until now an Account could close only at a zero Balance, so a Parent first had to make a
Transfer by hand. Ledgers are append-only (AGENTS.md rule 5) and Members are removed,
never deleted, so that past Transactions still name them. We need the same for Kids and
Accounts without losing History.

## Decision drivers

- History is the source of truth (ADR-0008, ADR-0009): nothing recorded is edited or deleted.
- The Wallet total never changes by tidying up; only Deposits and Withdrawals change it.
- One step for the Parent, both or neither for the system.
- Simple words in the app; the glossary words in code.

## Considered options

1. **Close = Transfer what is left, then close, in one decision.** Removing a Kid archives
   them. Hiding is a flag on an active Kid. Fixing a name records a rename.
2. Close = Withdrawal of what is left. Simple, but the money would vanish from the Wallet
   total, which teaches the Kid the wrong thing.
3. Delete Accounts and Kids. Breaks the append-only rule and loses the Kid's History.

## Decision outcome

Chosen option: 1, because it keeps every Transaction and the Wallet total, and a Parent
does it in one tap.

- **Closing an Account** (`CloseAccount { account_id, move_to, memo, occurred_on }`): the
  Wallet checks the Account is open and not the Main account, picks the target (`move_to`,
  or the Main account), and records a normal `TransferRecorded` for the whole Balance (when
  there is one) followed by `AccountClosed`, in one append. The usual Transfer rules apply,
  so the target must be another open Account, and a Savings goal reached by the move is
  celebrated. The Account and its Ledger are kept; the app shows it as closed. "Archive" in
  Jonas's words is **Closed account** in the glossary; there is no reopening in v1.
- **Removing a Kid** is the existing `ArchiveKid`: the Kid, their Wallet and History are
  kept but no longer active, so no Member sees them and nothing can be recorded. The money
  is not moved or withdrawn: it stays in the archived Wallet. There is no restoring in v1;
  it can be added later without a data change, because nothing is deleted.
- **Hiding a Kid** (`HideKid`, `ShowKid`) sets `hidden` on an active Kid. It is shared by
  the whole Family, not per Member. A hidden Kid is left off the Family home, still listed
  in Family settings, and their Wallet keeps working (an older sibling who stopped using it,
  a Kid away for a while). Hiding a hidden Kid, or showing a shown one, records nothing.
- **Fixing a Kid** (`RenameKid`) changes their name. Transactions never store the Kid's
  name, so History stays correct. A Kid's only detail in v1 is their name.
- All four are for Owners and Parents; Viewers are refused (`not_allowed`).

### Consequences

- Good: no Transaction is ever lost; the Kid sees "Lego set closed" as a Transfer in their
  History, and the Wallet total stays the same.
- Good: no new storage. Wallet events are the existing ones; the Family document's Kid gets
  a `hidden` field that defaults to `false`, so Families stored before it still load.
- Bad: an archived Kid cannot be brought back from the app yet.
- Bad: hiding is shared; one Parent hiding a Kid hides them for everyone.

### Confirmation

Domain tests in `wallet/tests.rs` (closing moves the Balance, keeps the total, never into
itself or a closed Account) and `family/tests.rs` (rename, hide, show, Viewer refused,
archived Kid refused); HTTP tests in `crates/http/tests/api.rs`; the stored shape test in
`store/src/family_body_tests.rs`.

## More information

Glossary: Account, Closed account, Kid, Hidden kid. Related: ADR-0008, ADR-0009, ADR-0010.
