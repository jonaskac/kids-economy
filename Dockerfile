# syntax=docker/dockerfile:1
# Kids Economy: one image with the Rust server and the built web app (ADR-0007, ADR-0014).
# Build:  docker build -t kids-economy .
# Run:    needs DATABASE_URL; see docs/deploy.md and compose.yml.

# 1. Web app (React + Vite) -> /web/apps/web/dist
FROM node:22-bookworm-slim AS web
WORKDIR /web
RUN corepack enable
COPY package.json pnpm-lock.yaml pnpm-workspace.yaml tsconfig.base.json ./
COPY apps/web/package.json apps/web/
COPY packages/design-system/package.json packages/design-system/
RUN pnpm install --frozen-lockfile
COPY apps/web apps/web
COPY packages packages
RUN pnpm --filter web build

# 2. Server (Rust, release build) -> /api/target/release/server
FROM rust:1-bookworm AS api
WORKDIR /api
COPY apps/api ./
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/api/target \
    cargo build --release --locked -p server \
    && cp target/release/server /usr/local/bin/server

# 3. Runtime: just the binary and the static files, no shell, as a non-root user.
#    Postgres TLS trusts the bundled public certificate authorities (webpki roots).
FROM gcr.io/distroless/cc-debian12:nonroot AS runtime
WORKDIR /app
COPY --from=api /usr/local/bin/server /app/server
COPY --from=web /web/apps/web/dist /app/web
ENV PORT=8080 \
    WEB_DIR=/app/web \
    RUST_LOG=info
USER nonroot
EXPOSE 8080
# Migrations run on start, before the server accepts traffic. Health: GET /api/health.
CMD ["/app/server"]
