# Glossary — the ubiquitous language

These words mean exactly this in code, API, UI and conversation.
If you need a new word, add it here first. "Avoid" lists synonyms we do not use.

## People and groups

**Parent**
: A grown-up who manages a Family. Can sign in, add Kids, and record Deposits and Withdrawals.
  In v1 the only user type. *Avoid:* admin, guardian, owner.

**Kid**
: A child in a Family who owns one Account. In v1 a Kid cannot sign in.
  *Avoid:* child (in code), user, member.

**Family**
: The group of Parents and Kids that share an economy. The tenant boundary: no data
  is ever visible across Families. *Avoid:* household, group, team.

**User**
: Technical identity that can authenticate (email, credentials). In v1 every User is a Parent.
  Kept separate so a Kid can become a User later.

## Money

**Account**
: A Kid's fictitious bank account. Holds an ordered list of Transactions. One per Kid in v1.
  *Avoid:* wallet, piggy bank (in code — fine as UI copy later).

**Amount**
: A non-negative quantity of money in **minor units** (öre, cents) as an integer, plus a Currency.
  `12,50 kr` is `{ "minor": 1250, "currency": "SEK" }`. Never a float.

**Currency**
: ISO 4217 code (e.g. `SEK`). One Currency per Family in v1.

**Balance**
: The sum of all Transactions on an Account at a point in time. Derived from the Ledger,
  never edited directly. *Avoid:* total, saldo (in code).

**Transaction**
: One recorded movement of money on an Account: a Deposit, a Withdrawal or a Correction.
  Immutable once recorded. Shown like a row in a bank app.

**Deposit**
: A Transaction that increases the Balance. Recorded by a Parent (pocket money, a gift, a reward).
  *Avoid:* credit, add, top-up.

**Withdrawal**
: A Transaction that decreases the Balance (the Kid bought something, cash out).
  Not allowed if it would make the Balance negative (see Overdraft).
  *Avoid:* debit, remove, spend.

**Correction**
: A Transaction that reverses an earlier, mistaken Transaction. We never delete or edit
  Transactions; we correct them, like a real bank.

**Memo**
: A short, Kid-readable text on a Transaction ("Pocket money week 48", "LEGO"). *Avoid:* note, comment.

**Ledger**
: The append-only, ordered log of Transactions for an Account. Source of truth for Balance.

**Overdraft**
: A Balance below zero. Not allowed in v1. *(Open question — see roadmap.)*

## System

**Recorded at**
: When the system stored a Transaction (server time).

**Actor**
: Who caused something to happen (the Parent's User id). Stored on every Transaction.
