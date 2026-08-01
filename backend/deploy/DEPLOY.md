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

> **Automated / disaster recovery:** [`bootstrap.sh`](bootstrap.sh) does everything in
> this section idempotently (ufw, key-only SSH on a custom port, fail2ban,
> unattended-upgrades, Docker, the `/opt/cymbra` dirs, the nightly backup cron). Rebuild
> a lost box with: provision → `git clone`/scp the repo → `SSH_PORT=… DEPLOY_PUBKEY="…"
> sudo -E bash backend/deploy/bootstrap.sh` → restore the vaulted `.env` →
> `docker login ghcr.io` → first launch (§5) → restore the latest DB dump. It is
> **secret-free**; every secret is restored separately, never committed.

- Order an **OVH VPS** in an EU datacenter (Gravelines/Roubaix/Strasbourg):
  - **VPS-2** (4 vCore / 8 GB / 75 GB NVMe) is the recommended size; VPS-1 (2 vCore
    / 4 GB) also works at this scale.
- OVH VPS has no Hetzner-style cloud firewall, so lock the box down with the host
  firewall (`ufw`). SSH runs on a **non-standard port** (`$SSH_PORT`, e.g. `49222`)
  with **key-only** auth + `fail2ban`; open it from anywhere rather than pinning to a
  single source IP (a home connection is usually dynamic — pinning risks locking
  yourself out). Allow ONLY:
  ```bash
  SSH_PORT=49222
  ufw default deny incoming
  ufw allow 443/tcp                 # gRPC + JWKS over TLS
  ufw allow 80/tcp                  # Let's Encrypt HTTP-01 + redirect
  ufw allow "$SSH_PORT"/tcp         # SSH (custom port)
  ufw enable
  ```
  Postgres/Valkey/gRPC are never published to the host anyway — the compose file uses
  `expose`, not `ports`, so they stay on the internal Docker network.
- **Move SSH off port 22 — carefully.** On Ubuntu 24.04/26.04 sshd is **socket-activated**
  (`ssh.socket`), so the `Port` directive in `sshd_config` is **ignored**; change the port
  in the socket instead. Two traps that WILL lock you out if ignored:
  1. A bare `ListenStream=<port>` binds **IPv6-only** on some boxes — always bind both
     families explicitly:
     ```ini
     # /etc/systemd/system/ssh.socket.d/override.conf
     [Socket]
     ListenStream=
     ListenStream=0.0.0.0:49222
     ListenStream=[::]:49222
     ```
  2. OVH VPS IPv6 inbound is frequently **not routed** (ping6 fails), so you cannot rely
     on IPv6 as a fallback. Keep the box reachable on IPv4.
  Do it lock-out-safe: open the new port in `ufw` and make sshd listen on **both** 22 and
  the new port first, verify a **fresh** connection on the new port, and only then drop 22.
  Arm a self-reverting timer while you work, so a mistake auto-restores access:
  ```bash
  # reverts to default (22) + re-opens ufw:22 in 5 min unless you cancel it
  printf '#!/bin/bash\nrm -f /etc/systemd/system/ssh.socket.d/override.conf\nsystemctl daemon-reload\nsystemctl restart ssh.socket\nufw allow 22/tcp\n' \
    | sudo tee /usr/local/sbin/ssh-revert.sh >/dev/null && sudo chmod +x /usr/local/sbin/ssh-revert.sh
  sudo systemd-run --unit=ssh-revert --on-active=300 /usr/local/sbin/ssh-revert.sh
  # ... apply change, verify new port from a NEW terminal, then:
  sudo systemctl stop ssh-revert.timer   # cancel the net once confirmed
  ```
  If you do get locked out: recover via the **OVH KVM console** (or rescue mode) and delete
  the override drop-in. Finally, point `fail2ban`'s `[sshd]` jail at the new port
  (`port = 49222` in `/etc/fail2ban/jail.local`).
- Install Docker Engine + the compose plugin. On a brand-new Ubuntu LTS the third-party
  Docker repo may not have a build for the release codename yet — the Ubuntu-archive
  packages work out of the box: `apt install docker.io docker-compose-v2`.

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

### Back office console (`bo.cymbra.app`)

The moderation console is a static SPA on **Cloudflare Pages** (not this box) — see
[apps/back-office/README.md](../../apps/back-office/README.md) "Deploy". It reaches the
same `api.<your-domain>` over gRPC-web + the web-auth cookie endpoints, so the backend
needs two things (both already in `.env.prod.example`):

- `CYMBRA_BACK_OFFICE_ORIGINS=https://bo.<your-domain>` — CORS allow-list for the
  gRPC-web CorsLayer **and** the web-auth cookie. Empty (default) = the console is
  blocked.
- `CYMBRA_WEB_AUTH_COOKIE_DOMAIN=<your-domain>` — the registrable domain shared by
  `api.` and `bo.`, so the refresh cookie is first-party for the console. Host-only
  (unset) is never sent by `bo.*` → sign-in won't persist.

DNS for `bo.<your-domain>` is a **custom domain on the Pages project**, not an A record
to this box. The Caddyfile already routes `/web/auth/*` + gRPC-web (incl. the CORS
preflight) correctly; roll the server after setting the two vars:
`docker compose -f docker-compose.prod.yml up -d`.

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

release-please tags `backend-vX.Y.Z` on merge → the `backend-image` workflow builds
and pushes `ghcr.io/neetrof/cymbra-backend:X.Y.Z` (+ `:latest`). Deploy that version
one of two ways:

**One-click CI (recommended)** — Actions → **deploy** → *Run workflow* → enter the
version (`X.Y.Z`). The runner SSHes to the box with a dedicated deploy key locked to a
forced-command and runs [`deploy.sh`](deploy.sh) (pins `CYMBRA_IMAGE` in `.env` → pull
→ `up -d` → waits for `/healthz`). Rollback = run it again with the previous version.
Needs repo secrets `DEPLOY_SSH_KEY` / `DEPLOY_HOST` / `DEPLOY_USER` / `DEPLOY_SSH_PORT`
and `deploy.sh` + `deploy-forced.sh` present on the box.

**Manual on the box** — pin the tag yourself and roll:

```bash
cd /opt/cymbra/backend/deploy
./deploy.sh X.Y.Z        # or, by hand:
# sed -i 's|^CYMBRA_IMAGE=.*|CYMBRA_IMAGE=ghcr.io/neetrof/cymbra-backend:X.Y.Z|' .env
# docker compose -f docker-compose.prod.yml pull && docker compose -f docker-compose.prod.yml up -d
```

The pinned tag in `.env` is the record of what is running. Migrations run automatically
on the new container's boot. Single node, so there is no multi-replica migration race.

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

## 11. Score corpus (crawler → local serve → S3 origin/fallback)

The score-crawler harvests redistributable scores and ingests their provenance
into `catalog_scores` (Postgres). The `.mxl` **bytes** are served **local-first**
from a **local folder** under `/srv/cymbra/scores`, with an **S3 fallback**: on a
local miss the server pulls the object from S3 and warms the local copy (the
`cymbra-storage` port, change `add-user-score-upload`). So S3 is the durable
**origin** (and it also receives user uploads), not merely an off-box backup —
a rebuilt/empty box still serves everything.

`object_key` in the catalog is `<prefix>/<shard>/<uuid>.mxl` — `prefix` is
`safe`/`low_confidence`, `shard` is the score UUID's last two hex chars, `uuid`
is the catalog PK (keyed by the immutable id since #82, matching the user-upload
store). The on-disk path is exactly `SCORES_DIR/ + object_key`, so the reader
resolves bytes from `object_key` with no per-source branching.

**No source is needed on the box.** The `crawler-image` CI workflow publishes two
images to GHCR — `cymbra-score-crawler` and `cymbra-musescore` (headless MuseScore
4, for OpenScore's `.mscx`) — and `docker-compose.crawler.prod.yml` (ships with
`backend/deploy/`) pulls and runs them. It joins the backend's private network so
`postgres` resolves, so **start the backend stack first**.

The crawler writes its `.mxl` bytes **directly into the served corpus dir**
(`SCORES_DIR`, the same path the server reads local-first) — it writes every file
before inserting the `catalog_scores` rows, so a row visible in the hub is already
servable, with no deferred merge. `SCORES_DIR` must therefore be writable by the
crawler container UID (the server owns it as 1000:1000 — see the `chown` in the
user-upload section below).

```bash
cd /opt/cymbra/backend/deploy
docker login ghcr.io                     # once (or reuse the deploy pull creds)
docker compose -f docker-compose.crawler.prod.yml pull

# smoke test: a few scores per source into the prod catalog (--env-file for the DB pw)
LIMIT=5 docker compose --env-file .env -f docker-compose.crawler.prod.yml up

# scores are served locally right after the run; mirror them off-box to S3:
. /etc/cymbra/backup.env; ./sync-scores.sh
```

Verify: `select count(*) from music.catalog_scores;` grows, and
`find $SCORES_DIR -name '*.mxl' | wc -l` matches. Then run unbounded (drop `LIMIT`),
or one source at a time (`... up mutopia`). `pdmx` downloads ~2 GB (the 222k-score
Zenodo dataset) — run it last. `openscore` converts via a sibling `cymbra-musescore`
container over the mounted docker socket (already wired in the compose).

The crawler connects as the DB superuser by default (it runs its own `score`
migrations then ingests); override `CYMBRA_SCORE_DATABASE_URL` for a dedicated role.

**Nightly** `bootstrap.sh` installs a cron (04:00) that runs `sync-scores.sh`:
it mirrors `SCORES_DIR` to the **bucket root** `s3://$SCORES_S3_BUCKET` — same
`/etc/cymbra/backup.env` creds as the DB backup. Set `SCORES_DIR` there (default
`/var/lib/cymbra/scores`). Since the crawler now writes straight into `SCORES_DIR`,
the script only mirrors off-box (no merge). Mirroring to the root (not a `scores/`
prefix) keeps the S3 key equal to `object_key`, so the server's S3 fallback and user
uploads (`user-scores/…`) share one keyspace.

> **`SCORES_S3_BUCKET` is DISTINCT from the DB-backup `S3_BUCKET`.** The scores
> bucket must equal the server's `CYMBRA_SCORE_S3_BUCKET` (e.g. `cymbra-scores`),
> NOT the backups bucket (`cymbra-backups`). Add `SCORES_S3_BUCKET=cymbra-scores`
> to `/etc/cymbra/backup.env`; `sync-scores.sh` no longer falls back to
> `S3_BUCKET`, so an unset value means "local corpus only" rather than silently
> dumping the corpus into the backups bucket.

> The server-side reader is the `cymbra-storage` local-first port (change
> `add-user-score-upload`): `CYMBRA_SCORE_LOCAL_ROOT` roots the local cache
> (default `/srv/cymbra/scores`) and `CYMBRA_SCORE_S3_*` configures the S3
> origin/fallback. The corpus layout above is exactly what it expects.

### Maintenance: backfill catalog titles

If catalog titles show an opaque id (e.g. `lc28971056`) instead of the real name,
those rows were ingested before the crawler preferred the embedded `<work-title>`
over the source filename. **Re-running the crawler will NOT fix them** — ingestion
dedups on the content SHA-256, and a title-only fix leaves the `.mxl` bytes
unchanged, so every existing row is skipped. Repair them with the `backfill-titles`
maintenance bin, which re-reads each stored `.mxl`, re-derives the title, and
rewrites `title` + `title_norm` + `work_key` together (so the score stays findable
by its real title — catalog search matches `title_norm`).

It ships in the backend image, so run it as a **one-off container from the `server`
service** (same `.env`, same DB + S3 + score-volume access) — the running server is
untouched:

```bash
cd /opt/cymbra/backend/deploy
./backup.sh                                                   # snapshot the DB first

# dry run — writes nothing, prints what WOULD change
docker compose -f docker-compose.prod.yml run --rm server backfill-titles --source openscore

# apply once the counts look right
docker compose -f docker-compose.prod.yml run --rm server backfill-titles --apply --source openscore
```

Verify: re-run the dry run (idempotent — `would rewrite: 0` when done), or
`select title, title_norm from music.catalog_scores where source='openscore' limit 10;`,
then search a real title in the app. Drop `--source openscore` to sweep every
source (it only rewrites titles that actually differ). Idempotent and resumable —
per-row failures are logged and skipped, never fatal.

**Mutopia is the exception.** Its titles show as `bwv 1001 1` (the source
filename), and `backfill-titles` above will NOT fix them: the LilyPond→MusicXML
conversion drops the title, so the stored `.mxl` has no `<work-title>` to re-derive
from (those rows count as `no title`). The real title lives only in the source
`.ly` `\header`, so a dedicated bin re-reads it from a fresh MutopiaProject
checkout. It ships in the **crawler** image (which has git + the DB env), so run it
as a one-off from the crawler compose, overriding the entrypoint:

```bash
cd /opt/cymbra/backend/deploy
./backup.sh                                                   # snapshot the DB first

# dry run — clones MutopiaProject, prints what WOULD change, writes nothing
docker compose --env-file .env -f docker-compose.crawler.prod.yml \
  run --rm --entrypoint backfill-mutopia-titles mutopia

# apply once the counts look right
docker compose --env-file .env -f docker-compose.crawler.prod.yml \
  run --rm --entrypoint backfill-mutopia-titles mutopia --apply
```

It only rewrites the `title` (the composer already came from the MusicXML) and its
`title_norm`/`work_key`, never a curator-edited row. Idempotent — a second dry run
reports `updated: 0`.

### Enabling user score upload (the ScoreService)

Off by default — the server logs `score-upload disabled` until `CYMBRA_SCORE_S3_BUCKET`
is set. To turn it on:

1. **Provision the `music` role** (only if the box was initialised before the module
   existed — check with `\du`): run `provision-music-role.sql` (idempotent, targeted;
   it does NOT reset other roles' passwords):
   ```bash
   docker exec -e MPW='<music pw>' -i <postgres-container> \
     psql -U <superuser> -d cymbra -v music_pw="$MPW" -f - < provision-music-role.sql
   ```
2. **Make the score root writable by the container UID (1000)** — the store
   `create_dir_all()`s it at boot and warms the cache into it (so `:ro` is not enough):
   ```bash
   sudo chown -R 1000:1000 "${SCORES_DIR:-/var/lib/cymbra/scores}"
   ```
   The compose already bind-mounts it (writable) into **both** `server` and `worker`.
3. **Fill the `.env` block** (see `.env.prod.example` → "User score upload"):
   `CYMBRA_MUSIC_DB_PASSWORD` + `CYMBRA_MUSIC_DATABASE_URL` (must match the role
   password from step 1) and the `CYMBRA_SCORE_S3_*` keys. Leave `CYMBRA_SCORE_LOCAL_ROOT`
   unset (defaults to the container path `/srv/cymbra/scores`).
4. Roll: `./deploy.sh <version>` — the server's MIGRATOR then creates
   `music.catalog_scores` + `music.user_scores`, and the log shows the ScoreService
   mounted instead of `score-upload disabled`.

## Enabling SoundFont delivery (`/soundfonts/*`)

Turned on by the `CYMBRA_SOUNDFONT_S3_BUCKET` block in `.env` (see `.env.prod.example`
→ "SoundFont delivery"). Like the score store, the server builds a **LocalFirstStore**
over S3 and warms a **writable local cache** — a store SEPARATE from scores, rooted at
`CYMBRA_SOUNDFONT_LOCAL_ROOT` (default container path `/srv/cymbra/soundfonts`).

> ⚠️ The store `create_dir_all()`s + writes this root at boot. `/srv/cymbra` is
> root-owned in the image, so the container UID (1000) cannot create the dir there —
> the server would crash-loop with `local root /srv/cymbra/soundfonts: Permission
> denied (os error 13)` and the deploy rolls back. The compose bind-mounts the host
> `SOUNDFONTS_DIR` there; make it writable by 1000 (a fresh `bootstrap.sh` does this
> for you — step 10):
> ```bash
> sudo mkdir -p "${SOUNDFONTS_DIR:-/var/lib/cymbra/soundfonts}"
> sudo chown -R 1000:1000 "${SOUNDFONTS_DIR:-/var/lib/cymbra/soundfonts}"
> ```

Only `server` mounts it (the worker builds no soundfont store). Leave the bucket unset
to disable the route (it then responds 503). Seed the default font once — see the
`aws s3 cp` snippet in `.env.prod.example`.

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
      unless-stopped`), SSH **key-only (no password) on a non-standard port**, `fail2ban`
      enabled and pointed at that port.
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
