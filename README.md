# Kids Economy

A fun, simple way for kids in the digital age to understand money and its value.

Each kid in a family gets a fictitious Wallet with one or more Accounts ("Fun stuff",
"Lego set"). Parents deposit, withdraw and transfer money, grandparents can follow
along, and the kid sees balances and a transaction history that looks like a real
bank app.

**v1 is parent-only.** A kid-facing app comes later.

## Monorepo layout

| Path                       | What                                      | Status  |
| -------------------------- | ----------------------------------------- | ------- |
| `apps/api`                 | Rust backend (HTTP API, domain, Postgres) | sign-in, Families, Wallets |
| `apps/web`                 | Parent web app (React + Tailwind)         | first slice |
| `packages/design-system`   | Our own design tokens and UI primitives   | v1 draft |
| `docs/adr`                 | Architecture Decision Records             | active  |
| `docs/glossary.md`         | Ubiquitous language                       | active  |
| `docs/roadmap.md`          | Ordered plan of work                      | active  |

## Ways of working

Full conventions for humans and coding agents: [`AGENTS.md`](AGENTS.md).

- **TDD.** Write a failing test, make it pass, refactor. No production code without a test driving it.
- **Domain first.** Terms come from [`docs/glossary.md`](docs/glossary.md). If a word is not there, add it before using it in code.
- **Decisions are recorded.** Any architecturally significant choice gets an ADR in [`docs/adr`](docs/adr/README.md).

## Getting started

```nu
pnpm install
pnpm test                      # design system + web
cd apps/api; cargo test        # Rust workspace
cargo run -p server            # http://localhost:8080/api/health
```

## Run the app locally with demo data

Three shells (nushell). Needs Docker (for Postgres), Rust and pnpm.

```nu
# 1. Postgres
docker run --name kids-pg -e POSTGRES_PASSWORD=postgres -p 5432:5432 -d postgres:16

# 2. API on :8080 (runs migrations on start)
cd apps/api
$env.DATABASE_URL = "postgres://postgres:postgres@localhost:5432/postgres"
cargo run -p server

# 3. Demo Family, then the web app on :5173
pnpm install
pnpm --filter web seed
pnpm --filter web dev
```

Open http://localhost:5173 (use your browser's phone view) and sign in as
`jonas@example.com` (Owner) or `mormor@example.com` (Viewer), password `correct horse battery`.
Running the seed again does nothing if the Family exists; to start over, `docker rm -f kids-pg`.
