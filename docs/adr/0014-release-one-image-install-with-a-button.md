---
status: proposed
date: 2026-11-25
deciders: [Jonas]
---

# 0014 — Release one public image; anyone installs it with a "Deploy to Sliplane" button

## Context and problem statement

Kids Economy should be installable by anyone on their own hosting, not only by us. The
first host is Sliplane: a visitor of the GitHub page clicks a button, lands in Sliplane,
and gets a running Install with its database and HTTPS, with nothing to set up by hand.
More hosting providers may follow.

## Decision drivers

- One click from the README, every dependency (Postgres) created for the user.
- An HTTPS address at once (`*.sliplane.app`), own domain optional.
- Provider-agnostic: adding a host must not change the app.
- No credentials in the repo; everything through environment variables.

## Considered options

1. Sliplane's own presets gallery: curated by Sliplane, no third-party format or link.
2. Sliplane's "Docker Command Parser" (paste a `docker run`/compose): manual, partial
   compose support, no database creation.
3. Sliplane Deployer (`sliplane.opendeploy.cloud`): a "Deploy to Sliplane" button reads
   `sliplane.yaml` in the repo and creates the managed Postgres and the service through
   the user's own Sliplane account.
4. Sliplane building from our GitHub repo: needs admin access to the repo, so it does not
   work for strangers.

## Decision outcome

Chosen option: 3, on top of a provider-agnostic release:

- **The Release is one image**, `ghcr.io/jonaskac/kids-economy`, built from the root
  `Dockerfile` by `.github/workflows/release.yml`: `main` → `:edge`, tag `vX.Y.Z` →
  `:X.Y.Z`, `:X.Y`, `:latest`. The package must be public.
- **Providers are recipes next to the app**, never code in it: `sliplane.yaml` (Sliplane
  Deployer) and `compose.yml` (any Docker host). A new provider adds one more recipe and a
  section in `docs/deploy.md`.
- The image needs only `DATABASE_URL` (and optionally `PORT`, `RUST_LOG`).

### Consequences

- Good: one click, Postgres included, HTTPS on `*.sliplane.app`, own domain in Sliplane's
  Domains tab; the same image runs anywhere.
- Bad: Sliplane Deployer is a third-party tool, not made by Sliplane. The user signs in
  to it with their Sliplane account. If it goes away, `docs/deploy.md` keeps the manual
  Sliplane steps (create a database, deploy the image, set `DATABASE_URL`).
- Bad: the first Install must wait for the first image to be published.

### Confirmation

- The `Release image` workflow builds the Dockerfile on PRs that change it.
- Before announcing a Release, one test install through the button on a real account.

## More information

ADR-0007 (one container, managed Postgres), glossary: Install, Release.
[Sliplane Deployer example](https://github.com/nielsbosma/sliplane-helloworld).
