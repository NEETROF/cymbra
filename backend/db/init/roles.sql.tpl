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

-- lingua module (Cymbra Lingua: word statuses, cards, daily stat aggregates;
-- change: add-lingua-backend) — owned by lingua_svc, confined to its own schema.
-- Consumes the platform (UserPort for account state — never a read of user_account),
-- so no cross-schema grants. Inert until CYMBRA_LINGUA_DATABASE_URL is set; the server
-- then runs its MIGRATOR and serves the cymbra.lingua.v1 services on this role. No FK
-- to user_account (purge by user_id).
SELECT format('CREATE ROLE %I LOGIN', :'lingua_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'lingua_role')
\gexec
ALTER ROLE :"lingua_role" WITH LOGIN PASSWORD :'lingua_pw';
CREATE SCHEMA IF NOT EXISTS lingua AUTHORIZATION :"lingua_role";
ALTER ROLE :"lingua_role" SET search_path = lingua;
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA lingua TO :"lingua_role";
ALTER DEFAULT PRIVILEGES IN SCHEMA lingua
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO :"lingua_role";

-- Keep the module roles out of the shared `public` schema so the only namespaces
-- each can touch are its own (+ the narrow jobs.enqueue grant from the migration).
REVOKE ALL ON SCHEMA public FROM :"auth_role", :"user_role", :"music_role", :"worker_role", :"flags_role", :"analytics_role", :"plans_role", :"lingua_role";

-- Ops role: read+write EVERY schema from a single connection (design OD1/OD2) --
-- `pg_read_all_data` + `pg_write_all_data` cover all current AND future schemas
-- as pure DML — no object ownership, no DDL — so the per-module ownership model
-- is intact. This deliberately crosses D0.
--
-- Two permitted actors, and the second is not "operations" (change:
-- harden-module-boundaries, task 7.5 — the comment used to say "MUST NEVER be wired
-- into an application module", which the worker has always contradicted):
--
--   1. Operators: runners, admins, psql.
--   2. `cymbra-worker`, on a SECOND pool. Its queue pool is `worker_svc`; the job
--      handlers reach every module's schema through this one. That is a decision,
--      not drift: a job body legitimately spans modules (erasure touches user,
--      music and plans in one transaction), and splitting it into per-module pools
--      would only move the cross-schema write, not remove it.
--
-- Consequence, and it is accepted: INSIDE THE WORKER, NO DATABASE GRANT CONFINES A
-- MODULE. The worker's own code is the boundary there. A server module must still
-- never be given this role.
--
-- ---------------------------------------------------------------------------
-- Named cross-schema exceptions (change: harden-module-boundaries, task 8.4)
--
-- Two module crates carry SQL naming ANOTHER module's schema. Both are listed here
-- so they are exceptions on the record rather than discoveries; an audit that finds
-- a third has found something new. Verified 2026-09-08 by grepping the module
-- crates for `FROM|JOIN|INTO|UPDATE <schema>.<table>` outside their own schema.
--
--   1. `music/src/pg_streak.rs` — `LEFT JOIN user_account.users` (two columns:
--      locale and timezone), for the streak reminder. VERIFIED worker-only: the
--      join sits behind `live_streaks`, reachable only through
--      `StreakModule::reminder_candidates`/`reminder_groups`, and the only caller
--      is the worker on the ops connection. The server builds the same module on
--      `music_svc` but never calls them — and if it ever did, the query would fail
--      rather than leak, because that role cannot read `user_account`. Fail-closed,
--      which is why this is an acceptable exception and not a hole.
--
--   2. `notifications/src/pg.rs` — 11 statements on `user_account.*`, including
--      `UPDATE user_account.users SET timezone`. NAMED DEBT, deliberately deferred.
--      This is a schema-OWNERSHIP problem, not a request-path leak: the
--      notifications module has no schema and no migrations of its own, and its
--      tables are created by `user/migrations/0008_push_notifications.sql`. Giving
--      it a schema is its own change — folding it in here would mix a migration,
--      a role grant and a data move into an unrelated one.
-- ---------------------------------------------------------------------------
SELECT format('CREATE ROLE %I LOGIN', :'admin_role')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = :'admin_role')
\gexec
ALTER ROLE :"admin_role" WITH LOGIN PASSWORD :'admin_pw';
GRANT pg_read_all_data, pg_write_all_data TO :"admin_role";
ALTER ROLE :"admin_role" SET search_path = auth, user_account, music, jobs, feature_flags, analytics, plans, lingua, public;
