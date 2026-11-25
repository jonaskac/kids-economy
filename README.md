# Kids Economy

A fun, simple way for kids in the digital age to understand money and its value.

Each kid in a family gets a fictitious bank account. Parents deposit and withdraw
money, and the kid sees a balance and a transaction history that looks like a
real bank app.

**v1 is parent-only.** A kid-facing app comes later.

## Monorepo layout

| Path                       | What                                      | Status  |
| -------------------------- | ----------------------------------------- | ------- |
| `apps/api`                 | Rust backend (HTTP API, domain, Postgres) | planned |
| `apps/web`                 | Parent web app (React + Tailwind)         | planned |
| `packages/design-system`   | Design tokens and UI primitives           | planned |
| `docs/adr`                 | Architecture Decision Records             | active  |
| `docs/glossary.md`         | Ubiquitous language                       | active  |
| `docs/roadmap.md`          | Ordered plan of work                      | active  |

## Ways of working

- **TDD.** Write a failing test, make it pass, refactor. No production code without a test driving it.
- **Domain first.** Terms come from [`docs/glossary.md`](docs/glossary.md). If a word is not there, add it before using it in code.
- **Decisions are recorded.** Any architecturally significant choice gets an ADR in [`docs/adr`](docs/adr/README.md).
