---
status: accepted
date: 2026-11-25
deciders: [Jonas]
---

# 0007 — Deploy as one container on Sliplane with a managed Postgres

## Context and problem statement

The app runs on Sliplane, with a Postgres database. We need one shape for the running app
that is cheap, simple to operate and easy to install for anyone (ADR-0014).

## Considered options

1. Two services: API container + static frontend container
2. One container: Rust binary serves the API under `/api` and the built SPA as static files

## Decision outcome

Chosen option: 2.

- Multi-stage `Dockerfile` at the repo root: build web (Node) → build server (Rust) →
  distroless runtime image with the binary and the web app in `/app/web` (`WEB_DIR`).
- Postgres is a Sliplane managed database (created by the Deploy button, ADR-0014),
  connected through `DATABASE_URL`. Its TLS URL (`sslmode=verify-full&sslrootcert=system`)
  is supported. Any other Postgres 16+ works the same way.
- At start the server waits for Postgres (about two minutes of retries), runs migrations
  (`sqlx::migrate!`), and only then accepts traffic.
- Config only through environment variables (12-factor); `.env.example` documents them.
- Health endpoint `GET /api/health` for Sliplane's health check.
- HTTPS is Sliplane's: every public service gets `<name>.sliplane.app` and custom domains
  with Let's Encrypt certificates. The app itself speaks plain HTTP on `PORT`.

### Consequences

- Good: one origin (no CORS, simple `Secure` cookies), one deploy, cheap, a ~15 MB image.
- Bad: frontend and backend deploy together — acceptable at this size.

### Confirmation

`crates/server` tests cover config, the managed-Postgres URL and serving the web app;
the `Release image` workflow builds the Dockerfile on every change to it.

## More information

- 2026-11-25: accepted. Postgres moved from "a separate container service" to Sliplane's
  managed database, because the Deploy button can create it with backups (ADR-0014).
- [Sliplane: custom domains](https://docs.sliplane.io/services/custom-domains/),
  [SSL](https://docs.sliplane.io/more/ssl/)
