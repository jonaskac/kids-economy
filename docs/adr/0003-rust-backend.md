---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0003 — Rust backend with axum, tokio and sqlx

## Context and problem statement

The backend language is decided: Rust. We need a web framework, async runtime and
database access layer.

## Decision drivers

- Testability (TDD): fast unit tests for the domain, real-Postgres integration tests.
- Small, explicit, widely used crates.
- Compile-time safety for SQL where it helps.

## Considered options

- HTTP: **axum** vs actix-web vs rocket
- DB: **sqlx** vs diesel vs sea-orm

## Decision outcome

Chosen: **axum + tokio + sqlx** (Postgres, no ORM).

- axum is built on tower/hyper (tokio team), handlers are plain async functions, and
  the router can be exercised in-process with `tower::ServiceExt::oneshot` — no network
  needed in tests.
- sqlx gives async Postgres, migrations (`sqlx migrate`) and `#[sqlx::test]`, which
  spins up an isolated database per test. Fits JSONB-heavy storage (ADR-0005) without
  an ORM fighting us.

Code shape (hexagonal / ports and adapters):

```
apps/api/crates/
  domain/   pure Rust, no IO — aggregates, invariants, events. Unit tested.
  store/    Postgres adapter implementing domain ports. Integration tested.
  http/     axum routes, DTOs, auth. Tested in-process.
  server/   binary wiring config + adapters.
```

### Consequences

- Good: domain logic is testable in microseconds; adapters are swappable.
- Bad: more crates than a single-binary app; justified once the kid app arrives.
