---
status: accepted
date: 2026-11-25
deciders: [Jonas]
---

# 0006 — Test-driven development across the stack

## Context and problem statement

The product handles (fictitious) money and is meant to grow. We want confidence to
change it fast.

## Decision outcome

All production code is driven by a failing test first (red → green → refactor).

| Layer                | Tooling                                   | Speed target |
| -------------------- | ----------------------------------------- | ------------ |
| Rust domain          | `cargo test` (pure unit tests)            | ms           |
| Rust store           | `#[sqlx::test]` against real Postgres     | < 1 s / test |
| Rust HTTP            | axum router in-process (`oneshot`)        | ms           |
| Design system / web  | Vitest + React Testing Library            | ms           |
| End-to-end           | Playwright, a handful of critical flows   | seconds      |

- CI blocks merge on: `cargo fmt --check`, `cargo clippy -D warnings`, all tests, `tsc`, lint.
- No mocking of Postgres for store tests: real database, isolated per test.

### Consequences

- Good: safe refactoring, the tests document behaviour in glossary terms.
- Bad: slower start; accepted.
