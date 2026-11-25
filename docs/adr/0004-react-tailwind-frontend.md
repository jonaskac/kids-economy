---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0004 — Parent app: React + TypeScript + Tailwind CSS, built with Vite

## Context and problem statement

Frontend is decided as "vanilla React and Tailwind CSS", i.e. no meta-framework
(Next.js, Remix). We need build tooling, a language choice and a test setup that
supports TDD.

## Decision outcome

- **React** as a client-side SPA, **Vite** as dev server and bundler (no meta-framework).
- **TypeScript** (strict). Still "vanilla React" — no framework on top.
- **Tailwind CSS**, with design tokens defined once in `packages/design-system` (ADR to follow).
- **Vitest + React Testing Library** for component tests; **Playwright** for a few end-to-end flows.
- **pnpm** workspaces for JS packages.
- Served as static files by the Rust server in production (ADR-0007), so one origin, no CORS.

### Consequences

- Good: minimal moving parts, fast tests, SPA is fine for an authenticated parent app (no SEO).
- Bad: routing and data fetching are our own choices; keep them small (e.g. `react-router`, plain `fetch`).
