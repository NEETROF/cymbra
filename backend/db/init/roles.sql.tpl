-- Cymbra ID — database role/schema bootstrap (env-driven; change: add-ops-db-access).
--
-- A secret-free, idempotent template. Role NAMES + PASSWORDS are psql variables
-- set by the `00-roles.sh` entrypoint wrapper (each defaulting to its dev value),
-- so the same bootstrap runs across dev/staging/prod. Run via that wrapper, NOT
-- directly by the Postgres entrypoint — the `.sql.tpl` extension makes the
-- `docker-entrypoint-initdb.d` glob ignore this file (it only runs *.sh / *.sql).
--
-- Idempotent: re-running creates any missing role and (re)sets its password, so
-- it works on a fresh volume AND on an already-provisioned database.
--
-- DEV defaults only. Production injects CYMBRA_*_DB_PASSWORD from a secret store
-- (or uses IAM auth — see backend/README.md). No secret is committed here.

-- Per-module least-privilege roles + schemas (design D0): each role owns and is
-- confined to its own schema; a module physically cannot read another's tables.

-- auth module --------------------------------------------------------------
SELECT format('CREATE ROLE %I LOGIN', :'auth_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'auth_role')
\gexec
ALTER ROLE :"auth_role" WITH LOGIN PASSWORD :'auth_pw';
CREATE SCHEMA IF NOT EXISTS auth AUTHORIZATION :"auth_role";
ALTER ROLE :"auth_role" SET search_path = auth;

-- user module --------------------------------------------------------------
SELECT format('CREATE ROLE %I LOGIN', :'user_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'user_role')
\gexec
ALTER ROLE :"user_role" WITH LOGIN PASSWORD :'user_pw';
CREATE SCHEMA IF NOT EXISTS user_account AUTHORIZATION :"user_role";
ALTER ROLE :"user_role" SET search_path = user_account;

-- music module (the whole music-app domain: scores today, more later) -------
SELECT format('CREATE ROLE %I LOGIN', :'music_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'music_role')
\gexec
ALTER ROLE :"music_role" WITH LOGIN PASSWORD :'music_pw';
CREATE SCHEMA IF NOT EXISTS music AUTHORIZATION :"music_role";
ALTER ROLE :"music_role" SET search_path = music;
-- music_svc owns the schema, so tables it creates via MIGRATOR (user_scores, at
-- server boot) are already its own. Belt-and-braces so it can DML tables the
-- crawler may create here as another privileged role (catalog_scores): grant on
-- what exists now, and default-privilege anything created later in this schema.
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA music TO :"music_role";
ALTER DEFAULT PRIVILEGES IN SCHEMA music
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO :"music_role";
-- Catalog full-text search (change: score-hub-search): the trigram GIN index +
-- the accent-fold backfill need `pg_trgm`/`unaccent`, and CREATE EXTENSION needs
-- superuser, so it is done here (not the least-privilege module migration, which
-- only creates the index that USES the extension). Installed into `music` so
-- music_svc resolves the operator class / functions via its pinned search_path.
CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA music;
CREATE EXTENSION IF NOT EXISTS unaccent SCHEMA music;

-- jobs (shared async-job substrate; owned by worker_svc — design D3) ---------
SELECT format('CREATE ROLE %I LOGIN', :'worker_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'worker_role')
\gexec
ALTER ROLE :"worker_role" WITH LOGIN PASSWORD :'worker_pw';
CREATE SCHEMA IF NOT EXISTS jobs AUTHORIZATION :"worker_role";
ALTER ROLE :"worker_role" SET search_path = jobs;
-- sqlxmq's migration uses uuid_nil()/uuid_generate_v4(); CREATE EXTENSION needs
-- superuser, so it is done here and installed into `jobs` so worker_svc resolves
-- it via its pinned search_path.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" SCHEMA jobs;

-- feature flags (shared, app-agnostic runtime flag/config store; change:
-- add-runtime-feature-flags) — owned by flags_svc, confined to its own schema.
-- Reused by every Cymbra app (music, live, future); the server runs its MIGRATOR
-- and evaluates flags on this connection.
SELECT format('CREATE ROLE %I LOGIN', :'flags_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'flags_role')
\gexec
ALTER ROLE :"flags_role" WITH LOGIN PASSWORD :'flags_pw';
CREATE SCHEMA IF NOT EXISTS feature_flags AUTHORIZATION :"flags_role";
ALTER ROLE :"flags_role" SET search_path = feature_flags;

-- analytics module (first-party feature-usage telemetry; change: add-feature-
-- usage-analytics) — owned by analytics_svc, confined to its own schema. The
-- server's UsageService ingests + reads here on this role; the worker's rollup +
-- purge jobs write it as admin_svc. Deliberately decoupled from identity (rows
-- carry only a hashed user_bucket, never a FK), so no cross-schema grants.
SELECT format('CREATE ROLE %I LOGIN', :'analytics_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'analytics_role')
\gexec
ALTER ROLE :"analytics_role" WITH LOGIN PASSWORD :'analytics_pw';
CREATE SCHEMA IF NOT EXISTS analytics AUTHORIZATION :"analytics_role";
ALTER ROLE :"analytics_role" SET search_path = analytics;
-- analytics_svc owns the schema, so tables it creates via MIGRATOR (at server
-- boot) are already its own. Belt-and-braces DML grants + default privileges.
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA analytics TO :"analytics_role";
ALTER DEFAULT PRIVILEGES IN SCHEMA analytics
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO :"analytics_role";

-- plans module (free/premium entitlement ledger, beta campaigns, access codes,
-- billing events; change: add-premium-subscription) — owned by plans_svc,
-- confined to its own schema. Identifiers only, never billing PII. The server
-- runs its MIGRATOR and serves PlanService on this role; the worker's sweep /
-- reconciliation jobs use it too. No FK to user_account (purge by user_id).
SELECT format('CREATE ROLE %I LOGIN', :'plans_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'plans_role')
\gexec
ALTER ROLE :"plans_role" WITH LOGIN PASSWORD :'plans_pw';
CREATE SCHEMA IF NOT EXISTS plans AUTHORIZATION :"plans_role";
ALTER ROLE :"plans_role" SET search_path = plans;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA plans TO :"plans_role";
ALTER DEFAULT PRIVILEGES IN SCHEMA plans
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO :"plans_role";

-- app_updates module — the desktop update feed (change: add-desktop-auto-update)
-- — owned by updates_svc, confined to its own schema. Stores CI-signed release
-- manifests verbatim; the server serves them anonymously and re-verifies the
-- signature on ingest. Carries no identity and no FK to any other schema (the
-- update check sends no identifier at all), so no cross-schema grants.
SELECT format('CREATE ROLE %I LOGIN', :'updates_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'updates_role')
\gexec
ALTER ROLE :"updates_role" WITH LOGIN PASSWORD :'updates_pw';
CREATE SCHEMA IF NOT EXISTS app_updates AUTHORIZATION :"updates_role";
ALTER ROLE :"updates_role" SET search_path = app_updates;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA app_updates TO :"updates_role";
ALTER DEFAULT PRIVILEGES IN SCHEMA app_updates
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO :"updates_role";

-- Keep the module roles out of the shared `public` schema so the only namespaces
-- each can touch are its own (+ the narrow jobs.enqueue grant from the migration).
REVOKE ALL ON SCHEMA public FROM :"auth_role", :"user_role", :"music_role", :"worker_role", :"flags_role", :"analytics_role", :"plans_role", :"updates_role";

-- Ops role: read+write EVERY schema from a single connection (design OD1/OD2) --
-- `pg_read_all_data` + `pg_write_all_data` cover all current AND future schemas
-- as pure DML — no object ownership, no DDL — so the per-module ownership model
-- is intact. This deliberately crosses D0 for OPERATIONS ONLY (runners/admins/
-- psql); it MUST NEVER be wired into an application module.
SELECT format('CREATE ROLE %I LOGIN', :'admin_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'admin_role')
\gexec
ALTER ROLE :"admin_role" WITH LOGIN PASSWORD :'admin_pw';
GRANT pg_read_all_data, pg_write_all_data TO :"admin_role";
ALTER ROLE :"admin_role" SET search_path = auth, user_account, music, jobs, feature_flags, analytics, plans, app_updates, public;
