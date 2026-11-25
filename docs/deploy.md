# Install Kids Economy

Every Install is one container image plus a Postgres database (ADR-0007, ADR-0014).
The image is `ghcr.io/jonaskac/kids-economy`, published by `.github/workflows/release.yml`:

| Tag       | What                                     |
| --------- | ---------------------------------------- |
| `X.Y.Z`   | A Release (`git tag vX.Y.Z`). Use these. |
| `latest`  | The newest Release                       |
| `edge`    | Whatever is on `main` now                |

## On Sliplane, with one click

[![Deploy to Sliplane](https://sliplane.opendeploy.cloud/button.svg)](https://sliplane.opendeploy.cloud/?repo=https://github.com/jonaskac/kids-economy)

1. Click the button. It opens Sliplane Deployer, which reads [`sliplane.yaml`](../sliplane.yaml).
2. Sign in with your Sliplane account and pick (or create) a server. The smallest one is enough.
3. Confirm. The deployer creates, in your account:
   - a managed **Postgres 16** database (`kids_economy`, backed up by Sliplane),
   - the **kids-economy** service from the image, with `DATABASE_URL` pointing at it and
     the health check on `/api/health`.
4. When it is done you get `https://<name>.sliplane.app`, already on HTTPS. Open it, tap
   **Create an account**, then **Start a Family**. You are the Family's Owner and invite the
   other grown-ups from Family settings.

Sliplane Deployer is a third-party tool, not made by Sliplane. If you would rather not use
it, do the same by hand:

1. In Sliplane, **Databases → Create Database**: Postgres 16. Copy its connection URL.
2. **Add service → Registry**: image `ghcr.io/jonaskac/kids-economy:<version>`, public,
   port `8080`, health check `/api/health`.
3. Environment: `DATABASE_URL` = the URL from step 1 (mark it secret). Deploy.

### Your own domain

The `*.sliplane.app` address keeps working. To add yours (say `pengar.example.com`):

1. In Sliplane, open the service → **Domains** → **Add Domain**, enter the domain.
2. At your DNS provider, add the record Sliplane shows:
   - a subdomain (`pengar.example.com`): `CNAME` to your `<name>.sliplane.app`;
   - the bare domain (`example.com`): `ALIAS`/`ANAME` to `<name>.sliplane.app` if your
     provider has it, otherwise the `A`/`AAAA` addresses Sliplane shows.
3. Wait for DNS. Sliplane gets a Let's Encrypt certificate and renews it; nothing to do in
   the app. Sign-ins are per address, so everyone signs in again on the new domain.

See [Sliplane: custom domains](https://docs.sliplane.io/services/custom-domains/).

### Upgrading

Change the service's image tag to the new Release and redeploy. Migrations run on start;
the Ledger is append-only, so a Release never rewrites history.

## Anywhere with Docker

[`compose.yml`](../compose.yml) runs the image with a Postgres container and a volume:

```nu
$env.POSTGRES_PASSWORD = (random chars --length 32)
docker compose up -d
http get http://localhost:8080/api/health
```

Put HTTPS in front with your own reverse proxy (Caddy, Traefik, your host's load balancer).
The Session cookie is `Secure`, so browsers keep it only over HTTPS (or on `localhost`).

## Configuration

Only environment variables; no file, no hard-coded credentials.

| Variable       | Required | Default    | What                                                        |
| -------------- | -------- | ---------- | ----------------------------------------------------------- |
| `DATABASE_URL` | yes      |            | Postgres 16+. TLS URLs (`sslmode=verify-full`, also `sslrootcert=system`) work. |
| `PORT`         | no       | `8080`     | Port to listen on, on all interfaces. Plain HTTP.           |
| `RUST_LOG`     | no       | `info`     | Log filter                                                  |
| `WEB_DIR`      | no       | `/app/web` in the image | Folder with the built web app. Unset: API only. |

At start the server waits up to about two minutes for Postgres, runs migrations, then
listens. It stops cleanly on `SIGTERM`.

## Adding another hosting provider

Providers are recipes next to the app, never code in it (ADR-0014). Add the provider's
file (like `sliplane.yaml`) that runs the same image with a Postgres and `DATABASE_URL`,
a section here, and its button in the README.

## For maintainers: making a Release

```nu
git tag v0.1.0
git push origin v0.1.0
```

The workflow pushes `:0.1.0`, `:0.1` and `:latest`. The first time, set the package to
**public** on GitHub (Your profile → Packages → kids-economy → Package settings → Change
visibility), or strangers cannot pull it. The button uses the newest GitHub release and
falls back to `edge` while there is none.
