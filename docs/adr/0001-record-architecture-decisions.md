---
status: accepted
date: 2026-11-25
deciders: [Jonas]
---

# 0001 — Record architecture decisions as MADR files in the repo

## Context and problem statement

We are starting a new product from scratch. Decisions made now (language, storage,
deployment, domain model) will shape everything after. We need their reasoning to
survive beyond chat history and memory.

## Considered options

1. No formal records
2. Wiki / Confluence pages
3. ADRs as Markdown in the repository, MADR format (https://adr.github.io/madr/)

## Decision outcome

Chosen option: 3. Records live next to the code, are reviewed in the same pull
requests as the code they affect, and are readable in a terminal.

Rules:

- One file per decision: `docs/adr/NNNN-kebab-title.md`, numbered sequentially.
- Start from `template.md`. Status starts as `proposed`; becomes `accepted` when agreed.
- Accepted decisions are not rewritten. A changed mind is a new ADR that supersedes the old one.
- `docs/adr/README.md` is the index and is updated in the same commit.

### Consequences

- Good: reasoning is versioned and discoverable; new contributors (and agents) get context.
- Bad: small overhead per decision. Keep ADRs short.
