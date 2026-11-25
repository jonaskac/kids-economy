# Roadmap

Worked backwards from the v1 goal:

> Anyone installs the app with one click, then a Parent signs in, sees every Kid in the Family with a Wallet total,
> opens a Kid's Wallet, records a Deposit, Withdrawal or Transfer between the Kid's Accounts,
> and sees it appear in a bank-style Transaction list. A second Parent or a Viewer
> (grandparent) can be invited and sees the same Family.

Each step is done TDD-first and ends in a pushed, reviewable commit.

| #  | Step                                                                                   | Output                                   | Status |
| -- | -------------------------------------------------------------------------------------- | ---------------------------------------- | ------ |
| 0  | Foundations: ADR process, initial ADRs, glossary v0, roadmap                           | `docs/`                                  | done   |
| 1  | Domain model: bounded contexts, aggregates, invariants, events (Identity, Family, Ledger) | `docs/domain-model.md`, glossary v1, ADRs | glossary v1 done |
| 2  | Rust workspace + CI: crates skeleton, fmt/clippy/test in GitHub Actions; pnpm workspace skeleton | `apps/api`, `apps/web`, `.github/workflows` | done   |
| 3  | Ledger domain, pure Rust, TDD: Amount, Wallet, Accounts, Deposit, Withdrawal, Transfer, Balance, no-Overdraft rule | `crates/domain` | done |
| 4  | Persistence: `ledger_events` numbered per Wallet (ADR-0009), Wallet store, concurrency tests; `documents` moves to step 5 | `crates/store`                         | done |
| 5  | Identity & Family (+ `documents` table): sign-up/sign-in, sessions, create Family, Members with Roles, Invitations, add Kid (ADR-0010, ADR-0011); Password reset links (ADR-0012) | domain + store + http, `docs/api.md` | done |
| 6  | HTTP API: endpoints + contract tests, API spec                                          | `crates/http`, `docs/api.md`             | done |
| 7  | Design system v0: tokens, Tailwind preset, primitives (Button, Card, AmountText, TransactionRow, Avatar, Sheet) | `packages/design-system` | done |
| 8  | Parent app: sign-in, Family overview, Account view, Deposit/Withdrawal sheet           | `apps/web`                               | done |
| 9  | Release & install: one public image, "Deploy to Sliplane" button with managed Postgres and HTTPS, migrations on boot (ADR-0014) | `Dockerfile`, `sliplane.yaml`, `compose.yml`, `docs/deploy.md` | in review |
| 10 | Hardening: E2E (Playwright), backups, logging, error pages                              |                                          |        |

Later (not v1): Kid app, recurring pocket money, interest, gifts from Viewers, transfers between siblings.
