---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0007 — Deploy as one container on Sliplane with a managed Postgres service

## Context and problem statement

The app is deployed on Sliplane on our own server, with a Postgres database.

## Considered options

1. Two services: API container + static frontend container
2. One container: Rust binary serves the API under `/api` and the built SPA as static files

## Decision outcome

Chosen option: 2 (pending confirmation).

- Multi-stage `Dockerfile` at the repo root: build web (Node) → build api (Rust) →
  small runtime image with the binary and `dist/`.
- Postgres runs as a separate Sliplane service on the same server; connected via `DATABASE_URL`.
- Migrations run on startup (`sqlx::migrate!`), before the server accepts traffic.
- Config only through environment variables (12-factor); `.env.example` documents them.
- Health endpoint `GET /api/health` for Sliplane's health check.

### Consequences

- Good: one origin (no CORS, simple cookies), one deploy, cheap.
- Bad: frontend and backend deploy together — acceptable at this size.
