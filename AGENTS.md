# AGENTS.md — how to work in this repository

Instructions for coding agents (Claude Code, Codex, Cursor, …) and humans alike.
`CLAUDE.md` only imports this file, so this is the single source of truth.

## What we are building

Kids Economy: each Kid in a Family gets a fictitious Wallet with one or more Accounts.
Adults (Members) record money in and out, and show the Kid their Balance and a
bank-style history. v1 is for adults only, used on a phone next to the Kid.
Product goal and order of work: [`docs/roadmap.md`](docs/roadmap.md).

## Ground rules

1. **Domain first.** Model the domain before code. Use only the words in
   [`docs/glossary.md`](docs/glossary.md) for names in code, API, UI copy and docs.
   A new concept gets a glossary entry *before* it appears in code.
2. **TDD is mandatory** ([ADR-0006](docs/adr/0006-test-driven-development.md)). Write or change a
   test first, see it fail, then implement, then refactor. No production code without a test
   driving it.
3. **Decisions are recorded** ([ADR-0001](docs/adr/0001-record-architecture-decisions.md)).
   Architecturally significant choices get an ADR in `docs/adr/` from `template.md`, numbered
   sequentially and added to `docs/adr/README.md` in the same commit. Never rewrite an accepted
   ADR's decision; supersede it.
4. **Money is never a float.** Amounts are integer minor units plus a Currency (glossary: Amount).
5. **Ledgers are append-only.** Transactions are never edited or deleted; mistakes get a Correction.
6. **Small, reviewable changes.** One concern per PR; docs, tests and code change together.

## Stack and layout

| Path | What | Decided in |
| --- | --- | --- |
| `apps/api` | Rust Cargo workspace: axum + tokio + sqlx, Postgres | ADR-0002, ADR-0003 |
| `apps/api/crates/domain` | Pure domain: aggregates, invariants, events. No IO. | ADR-0003 |
| `apps/api/crates/store` | Postgres adapter (JSONB documents + append-only ledger) | ADR-0005 |
| `apps/api/crates/http` | axum routes, DTOs, auth | ADR-0003 |
| `apps/api/crates/server` | Binary: config + wiring, serves `/api` and the SPA | ADR-0007 |
| `apps/web` | Parent app: React + TypeScript (strict) + Tailwind, Vite, no meta-framework | ADR-0004 |
| `packages/design-system` | Our own design tokens and React primitives | ADR-0004 |
| `docs/` | ADRs, glossary, roadmap | ADR-0001 |

JS packages are a pnpm workspace (root `pnpm-workspace.yaml`). Rust lives entirely under
`apps/api`, so Cargo and pnpm never collide.

## Design and UI

- **We use our own design system** in `packages/design-system`. Do **not** use the SHFTRS
  design system or `@shftrs/*` packages: this is a private project.
- **Phone first.** Design every screen for a ~375 px wide phone, then let it grow. Touch targets
  at least 44 px. The Balance must be readable by a Kid from across the table.
- Styling only through design tokens (Tailwind theme from `packages/design-system`); no ad-hoc
  hex colours in apps.
- Money in and out always shows a sign and a word, not colour alone ("+ 20 kr allowance").
  Text meets WCAG AA contrast in light and dark.
- No emoji in UI copy unless the design system defines it.

## Commands

The maintainer uses **nushell**. Shell examples in docs, PRs and comments are written for
nushell (e.g. `$env.DATABASE_URL = "postgres://…"`, `ls | where size > 1mb`), not bash.

```nu
# JavaScript (from the repo root)
pnpm install
pnpm -r typecheck
pnpm -r lint
pnpm -r test
pnpm -r build

# Rust (from apps/api)
cd apps/api
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
cargo run -p server    # GET http://localhost:8080/api/health
```

CI (`.github/workflows/ci.yml`) runs exactly these and blocks merge on failure.

## Code conventions

**TypeScript**
- `strict` on, no `any`, no non-null `!` without a comment saying why.
- Prefer plain functions and discriminated unions over classes. Components are function components.
- Tests sit next to code as `*.test.ts(x)`, using Vitest + React Testing Library. Test behaviour
  through what the user sees (roles, labels), not implementation details.

**Rust**
- `domain` has no IO and no async; every rule is a unit test there.
- Errors are typed enums (`thiserror` style), not strings. No `unwrap()` outside tests.
- HTTP handlers are tested in-process with `tower::ServiceExt::oneshot`; store tests run against
  real Postgres via `#[sqlx::test]`, never a mock.

**Git**
- Conventional commits (`feat:`, `fix:`, `docs:`, `chore:`, `test:`), imperative mood.
- Never commit secrets; configuration comes from environment variables, documented in
  `.env.example`.
