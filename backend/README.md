# Cymbra ID — backend

The shared identity/account service for Cymbra Music and Cymbra Live: a
gRPC-first **modular monolith** in Rust (tonic + SQLx/Postgres + Redis), with
OIDC (Google/Apple) + local email/password sign-in, audience-scoped session
tokens, per-app scoped roles, and first-class OpenTelemetry observability.

Spec: [`openspec/changes/add-cymbra-id`](../openspec/changes/add-cymbra-id).

## Layout (ports + dual adapters)

```
backend/
  platform/     cross-cutting: config, telemetry, token codec + interceptor,
                JWKS, OIDC/JWKS verify, argon2, email port, Redis, rate-limit
  auth-port/    contract: AuthPort trait + DTOs + cymbra.auth.v1 proto + gRPC client
  auth/         impl: IdentityVerifier, local credentials, sessions, gRPC server
  user-port/    contract: UserPort trait + DTOs + cymbra.user.v1 proto + gRPC client
  user/         impl: account aggregate (users/identities/roles), gRPC server
  server/       composition root (binary `cymbra-id`): wires everything, serves
                gRPC + the Axum JWKS/health surface
  jobs/         async-job substrate: transactional enqueue seam, channel/retry/
                DLQ policy, recurring scheduler (over sqlxmq; engine swappable)
  worker/       binary `cymbra-worker`: executes queued jobs, runs the scheduler
                + dead-letter sweep, serves /healthz /readyz
```

> **Deployment (change: add-job-infrastructure).** Background work now runs in a
> **separate `cymbra-worker` deployment** — it must be deployed for any async work
> (verification email, scheduled maintenance) to run. The orphan reaper no longer
> runs inside `cymbra-id` (the in-process loop is removed); it is a scheduled job
> in `jobs.schedules`. The worker owns the shared `jobs` schema as `worker_svc`;
> each module role gets only `EXECUTE` on `jobs.enqueue` (a write-only, documented
> exception to per-module isolation — design D3). Run it with
> `cargo run -p cymbra-worker --bin cymbra-worker`.

Each module is a **port** (Rust trait) with two interchangeable adapters: a
**direct** in-process impl and a **gRPC** impl (server + client). Consumers depend
on `<module>-port` only; the dependency rule is enforced in CI
([`scripts/check_boundaries.py`](scripts/check_boundaries.py)). Per-module Postgres
roles confine each module to its own schema. Tokens are signed with an asymmetric
key and published at a JWKS endpoint so Music/Live validate them offline.

### Extraction recipe

To split a module into its own service: give it a `main` that mounts the gRPC
**server** adapter it already has, point it at its own infra, and in `server` swap
the module's **direct** adapter for its `<module>-port` gRPC **client** adapter.
No changes to other modules' domain code.

## Local development

```bash
# infra: Postgres (+ per-module roles), Valkey (Redis-compatible), a mock OIDC issuer, Mailpit
docker compose -f backend/docker-compose.yml up -d

cp backend/.env.example backend/.env   # fill in the signing keypair + OIDC ids
# the server auto-loads backend/.env (dotenvy) — just run it:
cargo run -p cymbra-server --bin cymbra-id
```

Generate the internal-token signing keypair:

```bash
openssl genpkey -algorithm ed25519 -out priv.pem
openssl pkey -in priv.pem -pubout -out pub.pem
# set CYMBRA_TOKEN_SIGNING_KEY_PEM / CYMBRA_TOKEN_PUBLIC_KEY_PEM from these
```

Observability stack + docs: [`observability/README.md`](observability/README.md).

## Smoke test (grpcurl)

Reflection is not enabled, so pass the proto:

```bash
grpcurl -plaintext -proto backend/auth-port/proto/auth.proto \
  -d '{"email":"a@example.com","password":"a-strong-passphrase"}' \
  localhost:50051 cymbra.auth.v1.AuthService/SignUpLocal

curl -s localhost:8081/.well-known/jwks.json   # JWKS for downstream apps
curl -s localhost:8081/readyz                  # readiness (DB + Redis)
```

## Tests

```bash
cargo test --workspace                 # unit tests (fakes, no infra)
# integration (needs the infra above):
cargo test -p cymbra-auth -p cymbra-user -- --ignored
```

CI: [`rust.yml`](../.github/workflows/rust.yml) (fmt/clippy/coverage ≥80% +
boundary check) and [`backend-it.yml`](../.github/workflows/backend-it.yml)
(integration against Postgres + Redis).

## Database roles & operations (change: add-ops-db-access)

Roles + schemas are bootstrapped by [`db/init/00-roles.sh`](db/init/00-roles.sh),
which applies the secret-free, idempotent template
[`db/init/roles.sql.tpl`](db/init/roles.sql.tpl) with role names/passwords from the
environment (defaulting to the dev values, so `docker compose up` and CI need no
config). It is re-runnable, so it also (re)provisions an already-running DB:

```bash
PGHOST=localhost PGPASSWORD=cymbra_dev_pw POSTGRES_USER=cymbra POSTGRES_DB=cymbra_id \
  CYMBRA_ROLES_TEMPLATE=backend/db/init/roles.sql.tpl bash backend/db/init/00-roles.sh
```

**Roles.** Per-module least-privilege login roles (`auth_svc`, `user_svc`,
`worker_svc`) each confined to their own schema (design D0). Plus an **ops role
`admin_svc`** that can read **and** write every schema (current and future) from a
single connection — via Postgres' predefined `pg_read_all_data` + `pg_write_all_data`,
as **pure DML** (it owns nothing and cannot run DDL, so migrations/ownership stay
per-module). `admin_svc` deliberately crosses D0 **for operations only** (runners,
admins, ad-hoc `psql` via `CYMBRA_ADMIN_DATABASE_URL`) and **must never be wired
into an application service**.

> **Depends on `add-job-infrastructure`** — the `jobs` schema + `worker_svc` come
> from that change; this layers `admin_svc` and the env-driven bootstrap on top.

### Production credentials

The committed bootstrap carries **dev defaults only** — no production secret is in
the repo. For prod, replace the `*_dev_pw` defaults via one of:

- **Secret manager** (Vault / AWS Secrets Manager / GCP Secret Manager / K8s Secret
  + External Secrets Operator): inject `CYMBRA_*_DB_PASSWORD` as env at provision
  time; the same `00-roles.sh` applies them. Rotate = update the secret →
  re-run the bootstrap (it `ALTER ROLE … PASSWORD`s) → roll the services.
- **IAM database auth** (RDS / Cloud SQL IAM) — *preferred on managed cloud*: no
  static passwords at all; the connection string carries a short-lived token and
  rotation is automatic.

A single shared `admin_svc` is used for now; **per-operator IAM-authenticated roles**
(individually audited) are the natural prod hardening and are deferred to a later
ops change.
