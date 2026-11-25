# Roadmap

Worked backwards from the v1 goal:

> A Parent signs in on the deployed app, sees every Kid in the Family with a Balance,
> opens a Kid's Account, records a Deposit or Withdrawal, and sees it appear in a
> bank-style Transaction list.

Each step is done TDD-first and ends in a pushed, reviewable commit.

| #  | Step                                                                                   | Output                                   | Status |
| -- | -------------------------------------------------------------------------------------- | ---------------------------------------- | ------ |
| 0  | Foundations: ADR process, initial ADRs, glossary v0, roadmap                           | `docs/`                                  | done   |
| 1  | Domain model: bounded contexts, aggregates, invariants, events (Identity, Family, Ledger) | `docs/domain-model.md`, glossary v1, ADRs | next   |
| 2  | Rust workspace + CI: crates skeleton, fmt/clippy/test in GitHub Actions                 | `apps/api`, `.github/workflows`          |        |
| 3  | Ledger domain, pure Rust, TDD: Amount, Deposit, Withdrawal, Balance, no-Overdraft rule  | `crates/domain`                          |        |
| 4  | Persistence: migrations for `documents` + `ledger_events`, store adapter, concurrency tests | `crates/store`                         |        |
| 5  | Identity & Family: Parent sign-up/sign-in, sessions, create Family, add Kid             | domain + store + http                    |        |
| 6  | HTTP API: endpoints + contract tests, API spec                                          | `crates/http`, `docs/api.md`             |        |
| 7  | Design system v0: tokens, Tailwind preset, primitives (Button, Card, AmountText, TransactionRow, Avatar, Sheet) | `packages/design-system` |        |
| 8  | Parent app: sign-in, Family overview, Account view, Deposit/Withdrawal sheet           | `apps/web`                               |        |
| 9  | Packaging & deploy: Dockerfile, Sliplane service + Postgres, migrations on boot         | `Dockerfile`, `docs/deploy.md`           |        |
| 10 | Hardening: E2E (Playwright), backups, logging, error pages                              |                                          |        |

Later (not v1): Kid app, savings goals, recurring pocket money, interest, multiple Parents invite flow.
