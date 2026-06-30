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

-- Keep the module roles out of the shared `public` schema so the only namespaces
-- each can touch are its own (+ the narrow jobs.enqueue grant from the migration).
REVOKE ALL ON SCHEMA public FROM :"auth_role", :"user_role", :"worker_role";

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
ALTER ROLE :"admin_role" SET search_path = auth, user_account, jobs, public;
