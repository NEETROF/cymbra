## Context

Cymbra ID is **one Postgres database** (`cymbra_id`, `postgres:16`) with
schema-isolated modules, each owned by a confined login role (`auth_svc`,
`user_svc`, and — from `add-job-infrastructure` — `worker_svc`). A module
physically cannot read another module's schema (design D0). The dev bootstrap
`backend/db/init/01-schemas-roles.sql` creates the schemas, roles, the `uuid-ossp`
extension (in `jobs`), and the cross-module grants, with **hardcoded dev
passwords**; it runs once via the Postgres `docker-entrypoint-initdb.d` hook and
is applied in CI (`backend-it.yml`) with `psql`.

Two operational gaps: (1) there is no single role an operator/runner can use to
read+write across all schemas — D0 confinement is correct for the app but
obstructive for ops; (2) the bootstrap is dev-only with literal passwords, so it
cannot be reused across environments or wired to a secret store.

## Goals / Non-Goals

**Goals:**
- One `admin_svc` connection that can read+write **all** module schemas (present
  and future), for ops/runners/psql — without weakening the app's per-module
  confinement.
- A single bootstrap that runs across dev/staging/prod by sourcing role names +
  passwords from the environment, with dev defaults preserved.
- A documented production credential strategy (secret manager / IAM auth).

**Non-Goals:**
- Changing the per-module isolation model (D0) for application roles.
- Granting `admin_svc` DDL/ownership or letting any application module use it.
- Implementing a full Vault/External-Secrets integration (documented, not built).
- The services' own runtime config (already env-driven via `CYMBRA_*_DATABASE_URL`).

## Decisions

### OD1 — `admin_svc` via predefined roles, not membership or superuser
Grant the ops role Postgres 14+'s **`pg_read_all_data` + `pg_write_all_data`**.
These give SELECT (read) and INSERT/UPDATE/DELETE (write) plus schema `USAGE` on
**all** tables/sequences in **all** schemas, **including future ones**, with zero
per-table or per-schema maintenance. Crucially it is **pure DML**: `admin_svc`
owns no objects and cannot run DDL, so migrations and object ownership stay with
each module's role — the isolation/ownership model is intact.
*Alternatives:* **role membership** (`GRANT auth_svc, user_svc, worker_svc TO
admin_svc`) — works via INHERIT but must be re-granted for every new module, and
mixes in ownership/DDL rights; rejected as higher-maintenance and broader.
**SUPERUSER / the existing `cymbra` owner** — simplest but bypasses all safety
(RLS, every privilege) and invites accidental DDL; reserved for true DBA tasks.

### OD2 — Ops is a separate trust tier, not a D0 regression
`admin_svc` deliberately crosses module isolation, but only for **operations** —
it is never wired into an application module (modules keep their confined role).
The D0 invariant is about *modules* not reading each other; a privileged ops
identity is a different, explicitly-trusted tier. Enforced by convention +
documentation (no service reads `CYMBRA_ADMIN_DATABASE_URL`) and, optionally, a
note in the boundary docs.

**Decision: a single shared `admin_svc` for now.** We keep one shared ops role
rather than per-operator IAM-authenticated roles. It is the smallest step that
meets the need (one connection, all schemas) and works in every environment
including plain Postgres. Per-operator roles (one IAM-authenticated login per
human, individually audited) are the natural prod hardening and are deferred to a
later ops change — `admin_svc` does not preclude them.

### OD3 — Env-driven bootstrap = SQL template + thin entrypoint, psql `-v`
Keep a **pure-SQL template** (no secrets inlined) and pass values as psql
variables. A small `.sh` wrapper in `docker-entrypoint-initdb.d` reads env vars
(with dev defaults) and invokes `psql -v auth_pw="$CYMBRA_AUTH_DB_PASSWORD" …
-f template.sql`, the template referencing `:'auth_pw'` etc. Postgres runs `.sh`
and `.sql` files in `initdb.d` in name order, so a `00-roles.sh` wrapper replaces
direct execution of the `.sql`. *Alternatives:* **`envsubst`** over a `*.sql.tpl`
— also fine, fewer quoting rules than psql vars but needs the template to use
`${VAR}` markers and a gettext dependency; **a Rust/CLI provisioner** — most
flexible but a new moving part for what is declarative DDL. psql `-v` is chosen as
the smallest, dependency-free step that keeps the SQL reviewable.

### OD4 — Dev defaults preserved; prod credentials from a secret store
Every env var defaults to its current dev literal, so `docker compose up` and CI
are unchanged. For prod the literals are **not** used: passwords come from a
secret manager (Vault / AWS Secrets Manager / GCP Secret Manager / K8s Secret +
External Secrets Operator) injected as env at provision time, or — preferred on a
managed cloud — **IAM database auth** (RDS/Cloud SQL IAM) so there are no static
passwords at all and rotation is automatic. Rotation otherwise = update the
secret → `ALTER ROLE … PASSWORD` → rolling restart. This realizes the
"external-secrets/Vault decision" previously parked as a separate ops topic.

## Risks / Trade-offs

- **Broad-privilege credential** (`admin_svc` reads/writes everything) → mitigate
  by treating it as ops-only (never in a service config), protecting its secret
  like any high-value credential, and preferring IAM auth in prod.
- **Predefined roles require PG ≥ 14** → satisfied (repo uses `postgres:16`).
- **psql `-v` quoting** for passwords with special characters → use the
  `:'var'` (quoted-literal) form and document allowed charset, or switch to
  `envsubst` if quoting proves fragile.
- **Bootstrap is `initdb.d` (first-run only)** → for existing volumes the role
  must be applied out-of-band; document an idempotent `ALTER ROLE`/`CREATE ROLE
  IF NOT EXISTS`-style apply path for re-runs and for already-provisioned DBs.
- **Ordering vs `add-job-infrastructure`** → this change assumes `worker_svc` +
  `jobs` exist; land it after that change (or guard the worker bits).

## Migration Plan

Additive and dev-safe. (1) Refactor the bootstrap into `template.sql` + a
`00-roles.sh` entrypoint wrapper with env defaults. (2) Add `admin_svc` +
`pg_read_all_data`/`pg_write_all_data` grants. (3) Add `CYMBRA_ADMIN_DATABASE_URL`
to `.env.example` (ops note). (4) Update CI to apply via the wrapper. (5) Document
the prod secret/IAM strategy in `backend/README.md` / an ops doc. Rollback =
revert to the literal `.sql` (dev only). For already-running dev DBs, apply
`admin_svc` idempotently (as was done manually) rather than recreating the volume.

## Open Questions

- psql `-v` vs `envsubst` for the template (quoting ergonomics).
- Whether to also parametrize the **database name** and schema names, or only
  role credentials.
- Where the prod secret strategy is owned (this change documents it; a separate
  IaC/secrets change implements it).

*Resolved:* shared `admin_svc` vs per-operator IAM roles → keep one shared
`admin_svc` for now; per-operator IAM roles deferred to a later ops change (OD2).
