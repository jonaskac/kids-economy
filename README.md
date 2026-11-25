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
| `apps/api`                 | Rust backend (HTTP API, domain, Postgres) | skeleton |
| `apps/web`                 | Parent web app (React + Tailwind)         | skeleton |
| `packages/design-system`   | Our own design tokens and UI primitives   | v0       |
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
