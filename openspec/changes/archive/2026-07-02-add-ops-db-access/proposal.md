## Why

Operators and maintenance runners need to read and write data across **every**
module schema (`auth`, `user_account`, `jobs`, and any future module) from a
**single** connection — today that would mean juggling one confined role/
connection per schema. Separately, `backend/db/init/01-schemas-roles.sql`
hardcodes dev passwords (`auth_dev_pw`, …) and is dev-only, so there is no way to
run the **same** bootstrap across dev/staging/prod or to source credentials from
a secret store.

## What Changes

- **Add an ops-only `admin_svc` Postgres role** that can read **and** write data
  in every schema (current and future) via the predefined roles
  `pg_read_all_data` + `pg_write_all_data`. Pure DML: it owns no objects and runs
  no DDL/migrations, so the per-module ownership/isolation model is untouched.
  `search_path` spans all module schemas for convenience. This is a **separate
  ops trust tier** that deliberately crosses the D0 module-isolation invariant
  **for operations only** — it MUST NOT be used by an application module (each
  module keeps its confined per-schema role).
- **Make the role bootstrap environment-driven.** Source role **names** and
  **passwords** from environment variables (defaulting to the current dev values
  so local `docker compose up` still works out of the box) instead of literals,
  so the same bootstrap runs across environments. Keep a pure-SQL template;
  inject variables via a small `.sh` entrypoint wrapper (psql `-v`) or
  `envsubst`.
- **Document the production credential strategy** that replaces the `*_dev_pw`
  literals: passwords from a secret manager (Vault / AWS Secrets Manager / GCP
  Secret Manager / K8s + External Secrets), or **IAM database auth**
  (RDS/Cloud SQL IAM) for no static passwords at all, with rotation via
  `ALTER ROLE` or short-lived IAM tokens.
- Keep CI (`backend-it.yml`, which applies the bootstrap via `psql`) green —
  exercise the env-driven path with the dev defaults.

## Capabilities

### New Capabilities
- `ops-db-access`: A cross-schema operations DB role (read/write all module
  schemas from one connection, ops-only, no DDL) and an environment-driven,
  multi-environment role/schema bootstrap with a documented production secret
  strategy.

### Modified Capabilities
<!-- No existing archived spec governs the dev DB bootstrap, so there is no
     requirement delta to record here; the bootstrap rework is captured as this
     change's own capability + tasks. -->

## Impact

- **Database**: a new `admin_svc` login role (granted `pg_read_all_data` +
  `pg_write_all_data`); the `backend/db/init/01-schemas-roles.sql` bootstrap
  reworked into an env-driven template + entrypoint wrapper.
- **Ops**: a new `CYMBRA_ADMIN_DATABASE_URL` (ops/psql only — not read by any
  service); documented secret/IAM strategy for production role provisioning.
- **CI**: `backend-it.yml` bootstrap step continues to work via the env-driven
  path with dev defaults.
- **Dependency**: layers on top of the in-flight `add-job-infrastructure` change
  (which introduces the `jobs` schema + `worker_svc`); this change should land
  after it so the bootstrap covers all four roles.
- **Out of scope**: the application services' own config (they already read
  `CYMBRA_*_DATABASE_URL`); changing the per-module isolation model itself; a
  full Vault/External-Secrets rollout (documented, not implemented here).
