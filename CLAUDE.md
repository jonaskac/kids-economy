# Conventions for this repository

- Monorepo: backend in `apps/api` (Rust), parent app in `apps/web` (React + TypeScript + Tailwind, no meta-framework), shared UI in `packages/design-system`.
- TDD is mandatory: add or change a test first, see it fail, then implement.
- Use the terms in `docs/glossary.md` for names in code, API, UI and docs. Add new terms there first.
- Architecturally significant decisions get an ADR in `docs/adr/` using `docs/adr/template.md` (MADR). Number sequentially, never edit an accepted ADR's decision — supersede it with a new one.
- Money is never a float. See the glossary entry for **Amount**.
