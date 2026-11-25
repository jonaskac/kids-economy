---
status: accepted
date: 2026-11-25
deciders: [Jonas]
---

# 0002 — Single monorepo for backend, frontend, design system and docs

## Context and problem statement

The product has a Rust backend, a React parent app, a design system, and later a
kid app. How do we organise the code?

## Considered options

1. One repo per component
2. One monorepo (`kids-economy`)

## Decision outcome

Chosen option: 2. One change (e.g. a new field) can touch API, UI and docs in a
single reviewable commit, and there is one CI pipeline.

Layout:

```
apps/
  api/                 Rust (Cargo workspace root lives here)
  web/                 Parent app
  kid/                 (future) kid app
packages/
  design-system/       Tokens + React primitives, consumed by apps/web and apps/kid
docs/
  adr/  glossary.md  roadmap.md
```

- JS packages are managed as a pnpm workspace (root `pnpm-workspace.yaml`), see ADR-0004.
- Rust code is a Cargo workspace under `apps/api`, so JS tooling and Cargo do not collide.

### Consequences

- Good: atomic cross-cutting changes, shared tooling, one source of truth for the API contract.
- Bad: CI must be path-aware to stay fast (only build what changed).
