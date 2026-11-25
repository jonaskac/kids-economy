# Architecture Decision Records

Format: [MADR](https://adr.github.io/madr/). Process: see [ADR-0001](0001-record-architecture-decisions.md).
New ADR: copy [`template.md`](template.md), take the next number, add it below.

| #    | Decision                                                                 | Status   |
| ---- | ------------------------------------------------------------------------ | -------- |
| 0001 | [Record architecture decisions](0001-record-architecture-decisions.md)   | accepted |
| 0002 | [Monorepo](0002-monorepo.md)                                             | accepted |
| 0003 | [Rust backend: axum, tokio, sqlx](0003-rust-backend.md)                  | proposed |
| 0004 | [React + TypeScript + Tailwind, Vite](0004-react-tailwind-frontend.md)   | proposed |
| 0005 | [Postgres schema-less: JSONB documents + ledger](0005-postgres-document-storage.md) | proposed |
| 0006 | [Test-driven development](0006-test-driven-development.md)               | accepted |
| 0007 | [Deploy as one container on Sliplane](0007-deploy-on-sliplane.md)        | accepted |
| 0008 | [Aggregates are pure deciders over their history](0008-aggregates-as-pure-deciders.md) | proposed |
| 0009 | [Wallet History in Postgres, numbered per Wallet](0009-wallet-history-in-postgres.md) | proposed |
| 0010 | [Families and Users as documents, decided like the Wallet](0010-families-and-users-as-documents.md) | proposed |
| 0011 | [Sign in with email and password; Invitations as links](0011-password-sign-in-and-invitation-links.md) | accepted |
| 0012 | [Password reset links, made by an Owner for a Member](0012-password-reset-links-made-by-owners.md) | accepted |
| 0013 | [Closing an Account moves its money; Kids are hidden, renamed and archived, never deleted](0013-closing-accounts-and-removing-kids-keep-history.md) | proposed |
| 0014 | [Release one public image; install with a "Deploy to Sliplane" button](0014-release-one-image-install-with-a-button.md) | proposed |
