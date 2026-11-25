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
  Families). Kept separate so a Kid can become a User later. Signs in with an email
  address and a password (ADR-0011).

**Session**
: One User signed in on one device. A secret held in a cookie; only its hash is stored.
  Ends at sign-out or 30 days after sign-in. *Avoid:* login (as a noun), token (in UI).

**Member**
: An adult with their own sign-in in a Family: a parent, a grandparent, an aunt. Has exactly
  one Role. Members are removed, never deleted, so their past Transactions still name them.
  A Member may remove themselves (leave the Family); only an Owner removes others.
  *Avoid:* user (for this meaning), account holder.

**Role**
: What a Member may do in a Family. One of **Owner**, **Parent** or **Viewer**.

**Owner**
: Role that may do everything a Parent can, plus invite, remove and change the Role of
  Members. A Family always has at least one active Owner; two Owners (both parents) is the
  normal case. *Avoid:* admin.

**Parent**
: Role that may add, rename, hide and archive Kids, open and close Accounts, set Savings goals and record
  Deposits, Withdrawals, Transfers and Corrections. *Avoid:* guardian, admin.

**Viewer**
: Role that may see every Kid's Wallet in the Family but change nothing (grandparents,
  relatives). Any command from a Viewer is rejected as not allowed. *Avoid:* guest, read-only user.

**Invitation**
: An offer, made by an Owner for one email address, to join a Family with a given Role.
  It is **pending** until it is **accepted** (a User signed in with that email opens the
  Invitation link and joins), **revoked** by an Owner, or **expired** (7 days after it was
  made). Only one pending Invitation per email address in a Family.

**Invitation link**
: The secret link that carries an Invitation. Shown to the Owner once, who shares it any way
  they like (SMS, WhatsApp, in person); only a hash of its secret is stored.

**Password reset link**
: A secret link an Owner makes for a Member of their Family who forgot their password
  (ADR-0012). Whoever opens it chooses a new password; that signs the User out on every
  other device. Works once, for 24 hours; only a hash of its secret is stored.

**Kid**
: A child in a Family who owns exactly one Wallet. Archived, never deleted: "Remove kid" in
  the app archives them, keeping their Wallet and History, and no Member sees them after.
  A Kid's name can be fixed (renamed) while they are active. In v1 a Kid cannot sign in.
  *Avoid:* child (in code), user, member.

**Hidden kid**
: An active Kid left off the Family home, for the whole Family, but still listed in Family
  settings. Their Wallet keeps working. Shown again with one tap (ADR-0013).
  *Avoid:* inactive, disabled.

## Money

**Wallet**
: All of one Kid's Accounts together. The unit of consistency: a Transfer between two
  Accounts of the same Kid happens inside one Wallet in one step. *Avoid:* portfolio, profile.

**Account**
: One named pot of fictitious money in a Wallet ("Fun stuff", "Lego set"). Holds a Ledger;
  its Balance is derived. Closed, never deleted (see Closed account).
  *Avoid:* jar, piggy bank (in code — fine as UI copy).

**Closed account**
: An Account that no longer takes Transactions. Closing moves whatever is left, as one
  Transfer, to the Main account or another open Account the Parent picks, then closes it in
  the same step; the Wallet total stays (ADR-0013). Its Ledger is kept and its Transactions
  still show in History under its name. No reopening in v1. *Avoid:* archived account,
  deleted account (in code — "Close account" is the UI copy).

**Main account**
: The Account where money lands when no Account is chosen. Every Wallet has exactly one,
  opened together with the Kid. It cannot be closed.

**Savings goal**
: An optional target on an Account: a name, a target Amount and optionally a date
  ("Lego set, 400 kr by summer"). An Account with a Savings goal is a savings Account.
  Reaching the goal is celebrated, but does not lock the money.

**Goal reached**
: The moment an Account's Balance first reaches its Savings goal. Recorded once per goal,
  so the app can celebrate it with the Kid. Setting a new goal starts over.

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

**Direction**
: Whether a Transaction brings money **in** to its Account or takes it **out**. Amounts are
  always positive; the Direction carries the sign. *Avoid:* sign, credit/debit.

**Category**
: Why money moved, as a teaching label (not accounting): `allowance`, `chore`, `gift`,
  `interest`, `purchase`, `donation`, `transfer`, `correction`. A Deposit carries one of
  `allowance`, `chore`, `gift`, `interest`; a Withdrawal one of `purchase`, `donation`;
  `transfer` and `correction` are set by the system.

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

**Command**
: Something a Member asks the domain to do ("Deposit 20 kr"). Accepted or rejected as a
  whole, with a reason when rejected.

**Event**
: A fact the domain has recorded ("Transfer recorded"). Past tense, immutable.

**History**
: The ordered list of a Wallet's Events, starting with the Wallet being opened. The Wallet's
  state is rebuilt from it; nothing else is the source of truth.

**Sequence**
: The position of an Event in its Wallet's History: 1 for the Wallet being opened, then
  2, 3, … with no gaps. Numbered per Wallet, not per Account, because the Wallet is the
  unit of consistency.

**Version**
: How many Events a Wallet's History holds, which is also the Sequence of its last Event.
  A decision is made against one Version and saved only if the Wallet is still at it.

**Conflict**
: Saving a decision when the Wallet has moved past the Version it was decided on (another
  Member recorded something first). Nothing is saved; the Command is decided again against
  the fresh Wallet. *Avoid:* race, collision (in code).

**Install**
: One running copy of Kids Economy with its own database, set up by whoever hosts it
  (for example from the "Deploy to Sliplane" button). Families on different Installs never
  see each other. *Avoid:* instance, tenant (in code).

**Release**
: A published, numbered version of the app (`v0.1.0`), shipped as one container image that
  every Install runs. An Install upgrades by moving to a newer Release; migrations run on start.
