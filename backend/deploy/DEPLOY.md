# Cymbra backend — single-box production deploy (testers / early access)

A pragmatic, non-HA production for handing the apps to real users. One VPS (e.g. an
OVH VPS-2 — 4 vCore / 8 GB) runs Postgres + Valkey + `cymbra-server` + `cymbra-worker`
+ Caddy (auto-HTTPS) via
`docker-compose.prod.yml`. Images are built by GitHub Actions and pulled from GHCR.

> Not highly available: a hardware failure means a **manual restore** from backup
> (minutes). That is acceptable at this scale. The code is already HA-ready (stateless
> server, idempotent worker), so moving to managed HA later is an infra change, not a
> code change.

## 0. What you need first

- A domain you control (managed via Google — Google Domains/Squarespace or Cloud DNS).
- An OVHcloud account.
- A transactional email provider with SMTP creds (Brevo/Postmark/SES) for verification
  emails. (Mailpit is dev-only.)
- Your Google + Apple OIDC client IDs (for `CYMBRA_GOOGLE_AUDIENCE` / Apple vars).

## 1. Provision the box

- Order an **OVH VPS** in an EU datacenter (Gravelines/Roubaix/Strasbourg):
  - **VPS-2** (4 vCore / 8 GB / 75 GB NVMe) is the recommended size; VPS-1 (2 vCore
    / 4 GB) also works at this scale.
- OVH VPS has no Hetzner-style cloud firewall, so lock the box down with the host
  firewall (`ufw`) — allow ONLY:
  ```bash
  ufw default deny incoming
  ufw allow 443/tcp                                    # gRPC + JWKS over TLS
  ufw allow 80/tcp                                     # Let's Encrypt HTTP-01 + redirect
  ufw allow from <your-ip> to any port 22 proto tcp    # SSH from your IP only
  ufw enable
  ```
  Postgres/Valkey/gRPC are never published to the host anyway — the compose file uses
  `expose`, not `ports`, so they stay on the internal Docker network.
- Install Docker Engine + the compose plugin (`docker`, `docker compose`).

## 2. DNS (in Google domain management)

Create an **A record**:

```
api.<your-domain>   A   <the box's public IPv4>
```

(Optionally an `AAAA` for the IPv6.) Wait for it to resolve (`dig api.<your-domain>`).
This host is what `API_DOMAIN` and the apps will use.

## 3. Get the code + env onto the box

```bash
sudo mkdir -p /opt/cymbra && sudo chown "$USER" /opt/cymbra
git clone https://github.com/NEETROF/cymbra.git /opt/cymbra
cd /opt/cymbra/backend/deploy
cp .env.prod.example .env
chmod 600 .env
```

Only `backend/db/init`, `backend/deploy/*`, and the GHCR image are actually used at
runtime — the clone is just the convenient way to get the compose file + init scripts.

## 4. Generate secrets and fill `.env`

```bash
# DB passwords (one per role) — put each value in BOTH the CYMBRA_*_DB_PASSWORD
# var AND the matching *_DATABASE_URL.
openssl rand -base64 24     # repeat for superuser + auth/user/worker/admin

# Internal-token signing keypair (Ed25519):
openssl genpkey -algorithm ed25519 -out priv.pem
openssl pkey -in priv.pem -pubout -out pub.pem
```

Paste the PEM contents into `CYMBRA_TOKEN_SIGNING_KEY_PEM` / `CYMBRA_TOKEN_PUBLIC_KEY_PEM`.
Multi-line PEM in a dotenv file: wrap the whole value in double quotes and keep the
newlines, e.g.

```
CYMBRA_TOKEN_SIGNING_KEY_PEM="-----BEGIN PRIVATE KEY-----
MC4CAQ...
-----END PRIVATE KEY-----"
```

Set `API_DOMAIN`, the SMTP vars, and the Google/Apple OIDC vars. Delete `priv.pem`/
`pub.pem` from the box once pasted.

## 5. First launch

```bash
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
docker compose -f docker-compose.prod.yml logs -f caddy server
```

On first boot: Postgres runs `db/init` (creates per-module roles with your passwords),
then `server` runs auth+user migrations and `worker` runs jobs migrations. Caddy fetches
the TLS cert for `api.<your-domain>` automatically.

Smoke test:

```bash
curl -s https://api.<your-domain>/healthz
curl -s https://api.<your-domain>/.well-known/jwks.json
```

## 6. Point the OIDC providers + the apps at prod

- **Google / Apple consoles:** add `https://api.<your-domain>` (and any required
  redirect URIs) as authorized origins/redirects. Set `CYMBRA_GOOGLE_AUDIENCE` and the
  Apple `AUDIENCE`/`TEAM_ID`/`KEY_ID`/`P8_KEY_PEM` in `.env`.
- **Music/Live apps:** build pointing at the prod endpoint, over TLS on 443:

  ```bash
  flutter build ... \
    --dart-define=CYMBRA_GRPC_HOST=api.<your-domain> \
    --dart-define=CYMBRA_GRPC_PORT=443
  ```

  Note: `cymbraEndpoint` currently only wires host/port from `--dart-define`; the channel
  is secure only when the provider's `secure` is true. For prod, override
  `cymbraEndpointProvider` to `CymbraEndpoint(host: …, port: 443, secure: true)` (or extend
  it to read a `CYMBRA_GRPC_SECURE` define). See
  [grpc_client.dart](../../apps/music/lib/services/grpc_client.dart).

## 7. Backups (do this before you invite users)

```bash
chmod +x /opt/cymbra/backend/deploy/backup.sh
apt install -y awscli    # for the off-box S3 copy
# Off-box target = OVH Object Storage (S3). Create a bucket + an S3 user in the OVH
# console, then export the creds. OVH bills only stored GB — no egress/API fees.
#   export S3_ENDPOINT=https://s3.gra.io.cloud.ovh.net S3_BUCKET=cymbra-backups
#   export AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=...
crontab -e
# 0 3 * * *  S3_ENDPOINT=https://s3.gra.io.cloud.ovh.net S3_BUCKET=cymbra-backups AWS_ACCESS_KEY_ID=... AWS_SECRET_ACCESS_KEY=... /opt/cymbra/backend/deploy/backup.sh >> /var/log/cymbra-backup.log 2>&1
```

**Test a restore once** into a throwaway DB — an untested backup is not a backup.

## 8. Updating

Push to `main` (or tag `v*`) → the `backend-image` workflow builds and pushes the image.
Then on the box:

```bash
cd /opt/cymbra/backend/deploy
docker compose -f docker-compose.prod.yml pull
docker compose -f docker-compose.prod.yml up -d
```

Migrations run automatically on the new container's boot. Single node, so there is no
multi-replica migration race to worry about.

## 9. Email deliverability (SPF / DKIM / DMARC) — don't skip

Verification and password-reset emails will land in **spam** without domain auth.
After creating the sender domain in your provider (Brevo/Resend/Postmark), add the
DNS records it gives you (in Google domain management):

- **SPF** — a TXT record authorizing the provider to send for your domain.
- **DKIM** — the CNAME/TXT keys the provider generates (signs your mail).
- **DMARC** — a TXT at `_dmarc.<domain>` (start with `p=none` to monitor).

`CYMBRA_SMTP_FROM` must be on that authenticated domain. Send yourself a test
sign-up and confirm the mail arrives in the inbox (check the DKIM=pass header).

## 10. Observability (optional — off by default)

Docker logs (`docker compose logs -f server worker`) are enough at this scale, and
OTel ships **off**. When you want traces/metrics without self-hosting the heavy
Grafana/Tempo/Loki/Prometheus stack on this box:

1. Create a free **Grafana Cloud** stack; grab its OTLP endpoint + an API token.
2. In `.env`: set `GRAFANA_OTLP_ENDPOINT`, `GRAFANA_OTLP_AUTH` (base64 of
   `instanceID:token`), `CYMBRA_OTLP_ENABLED=true`,
   `CYMBRA_OTLP_ENDPOINT=http://otel-collector:4317`.
3. Start the collector profile:
   ```bash
   docker compose -f docker-compose.prod.yml --profile observability up -d
   ```
   Both binaries then export OTLP to the local collector, which forwards to Grafana
   Cloud. The self-hosted stack under `backend/observability/` stays for local dev.

## Before you invite testers — checklist (the easy-to-forget bits)

- [ ] **Uptime monitor** — there's no HA/alerting. Point a free UptimeRobot/BetterStack
      check at `https://api.<domain>/healthz` so you learn about downtime before users do.
- [ ] **Backups tested** — run `backup.sh` once and actually restore the dump into a
      throwaway DB. Confirm the off-box copy lands in OVH Object Storage.
- [ ] **Email deliverability** — SPF+DKIM+DMARC set, test mail hits the inbox (§9).
- [ ] **Apple Sign in** — the current implementation is the **native flow**: the app sends
      Apple's `id_token`, the backend verifies it against Apple's JWKS (`aud` = bundle ID).
      **No `.p8` client-secret JWT is used** — `CYMBRA_APPLE_P8_KEY_PEM`/`_TEAM_ID`/`_KEY_ID`
      are unused placeholders today, and there is nothing to rotate every 6 months. Just set
      `CYMBRA_APPLE_AUDIENCE` (= the bundle ID) + `CYMBRA_APPLE_ISSUER`. The `.p8` only becomes
      needed IF you later add the web authorization-code exchange OR Apple's server-to-server
      **token revocation** (which the account-deletion requirement below may pull in — that
      flow does need a `.p8`-signed client secret, valid ≤6 months).
- [ ] **OIDC audiences** — `CYMBRA_GOOGLE_AUDIENCE` / `CYMBRA_APPLE_AUDIENCE` match the
      app bundle IDs; `CYMBRA_ALLOWED_AUDIENCES=music,live` matches what the apps send.
- [ ] **Account deletion path** — Apple & Google **require** apps with account creation to
      offer in-app account deletion. The orphan reaper only cleans abandoned onboarding,
      not user-requested deletion — you need a real delete flow + a privacy policy / ToS
      URL before store review (even for TestFlight/closed testing at scale).
- [ ] **App distribution** — the backend is half of "giving the apps to users": set up
      **TestFlight** (iOS) and **Play Console internal/closed testing** (Android). Build the
      apps pointing at prod (§6).
- [ ] **Box hardening** — enable `unattended-upgrades` (auto security patches), confirm
      `systemctl enable docker` (services come back after reboot via `restart:
      unless-stopped`), SSH key-only (no password), fail2ban optional.
- [ ] **Disk headroom** — Postgres data + WAL + backups grow; container logs are capped
      (10 MB × 3). Keep an eye on `df -h` for the first weeks.
- [ ] **Secrets hygiene** — `.env` is `chmod 600`; you deleted `priv.pem`/`pub.pem` after
      pasting; the `.p8` and signing key exist **only** in `.env` (and a password manager
      backup), not in git.
- [ ] **GDPR basics** — you store EU user accounts: a privacy policy, a data-deletion
      route, and a contact. Minimal but expected for a French-facing app.

## Rotating DB passwords later

`db/init` only runs on the FIRST Postgres init. To change a role password afterwards,
re-run the bootstrap against the live DB (see `backend/README.md` §Database roles) and
update the matching `*_DATABASE_URL`, then `up -d` to roll the services.
