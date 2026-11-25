# Glossary — the ubiquitous language

These words mean exactly this in code, API, UI and conversation.
If you need a new word, add it here first. "Avoid" lists synonyms we do not use.

v1 of this glossary follows the domain model agreed on 2026-11-25: several adult
logins per Family with Roles, and several Accounts per Kid inside one Wallet.

## People and groups

**Family**
: The group of Members and Kids that share an economy. The tenant boundary: no data
  is ever visible across Families. *Avoid:* household, group, team.

**User**
: Technical identity that can authenticate (email, credentials). A User joins a Family as a
  Member; one User can be a Member of several Families (a grandparent with grandkids in two
  Families). Kept separate so a Kid can become a User later.

**Member**
: An adult with their own sign-in in a Family: a parent, a grandparent, an aunt. Has exactly
  one Role. Members are removed, never deleted, so their past Transactions still name them.
  *Avoid:* user (for this meaning), account holder.

**Role**
: What a Member may do in a Family. One of **Owner**, **Parent** or **Viewer**.

**Owner**
: Role that may do everything a Parent can, plus invite, remove and change the Role of
  Members. A Family always has at least one active Owner; two Owners (both parents) is the
  normal case. *Avoid:* admin.

**Parent**
: Role that may add and archive Kids, open and close Accounts, set Savings goals and record
  Deposits, Withdrawals, Transfers and Corrections. *Avoid:* guardian, admin.

**Viewer**
: Role that may see every Kid's Wallet in the Family but change nothing (grandparents,
  relatives). Any command from a Viewer is rejected as not allowed. *Avoid:* guest, read-only user.

**Invitation**
: A pending offer, sent by an Owner to an email address, to join a Family with a given Role.
  Accepted by signing in; expires after a set time.

**Kid**
: A child in a Family who owns exactly one Wallet. Archived, never deleted. In v1 a Kid
  cannot sign in. *Avoid:* child (in code), user, member.

## Money

**Wallet**
: All of one Kid's Accounts together. The unit of consistency: a Transfer between two
  Accounts of the same Kid happens inside one Wallet in one step. *Avoid:* portfolio, profile.

**Account**
: One named pot of fictitious money in a Wallet ("Fun stuff", "Lego set"). Holds a Ledger;
  its Balance is derived. Closed only when its Balance is zero, never deleted.
  *Avoid:* jar, piggy bank (in code — fine as UI copy).

**Main account**
: The Account where money lands when no Account is chosen. Every Wallet has exactly one,
  opened together with the Kid. It cannot be closed.

**Savings goal**
: An optional target on an Account: a name, a target Amount and optionally a date
  ("Lego set, 400 kr by summer"). An Account with a Savings goal is a savings Account.
  Reaching the goal is celebrated, but does not lock the money.

**Amount**
: A non-negative quantity of money in **minor units** (öre, cents) as an integer, plus a Currency.
  `12,50 kr` is `{ "minor": 1250, "currency": "SEK" }`. Never a float. A zero Amount is
  never recorded.

**Currency**
: ISO 4217 code (e.g. `SEK`). One Currency per Family in v1; every Wallet uses it.

**Balance**
: The sum of all Transactions on one Account at a point in time. Derived from the Ledger,
  never edited directly. *Avoid:* saldo (in code).

**Wallet total**
: The sum of the Balances of all Accounts in a Wallet: "how much the Kid has". Transfers never
  change it.

**Transaction**
: One recorded movement of money on one Account: a Deposit, a Withdrawal, one side of a
  Transfer, or a Correction. Immutable once recorded. Shown like a row in a bank app.

**Deposit**
: A Transaction that brings money into a Wallet from outside (pocket money, a gift, a reward).
  Lands in the Main account unless another Account is chosen. *Avoid:* credit, add, top-up.

**Withdrawal**
: A Transaction that takes money out of a Wallet (the Kid bought something, cash out).
  Not allowed if it would make the Account's Balance negative (see Overdraft).
  *Avoid:* debit, remove, spend.

**Transfer**
: Moving money between two different, open Accounts in the same Wallet. Recorded as a linked
  pair of Transactions with the same transfer id: out of the source, into the target, same
  Amount. Both or neither. Not allowed between siblings in v1. *Avoid:* move, shift.

**Correction**
: A Transaction that reverses an earlier, mistaken Transaction. We never delete or edit
  Transactions; we correct them, like a real bank. A Transaction can be corrected once;
  correcting one side of a Transfer corrects both.

**Category**
: Why money moved, as a teaching label (not accounting): `allowance`, `chore`, `gift`,
  `interest`, `purchase`, `donation`, `transfer`, `correction`.

**Memo**
: A short, Kid-readable text on a Transaction ("Pocket money week 48", "LEGO"). *Avoid:* note, comment.

**Ledger**
: The append-only, ordered log of Transactions for an Account. Source of truth for Balance.

**Overdraft**
: A Balance below zero. Not allowed on any Account in v1.

## System

**Occurred on**
: The calendar day a Transaction happened for the Kid ("I got it on my birthday").
  History is sorted by it.

**Recorded at**
: When the system stored a Transaction (server time). Audit order.

**Actor**
: The Member who caused something to happen. Stored on every Transaction; must be an
  active Owner or Parent of the Kid's Family.
