-- Cymbra ID — dev database bootstrap (task 1.6).
-- Runs once on first Postgres start (mounted into /docker-entrypoint-initdb.d).
--
-- Creates one schema per module and a least-privilege LOGIN role per module whose
-- privileges are confined to its own schema (via schema ownership), with
-- search_path pinned. A module physically cannot read another module's tables —
-- the database rejects it (design D0; asserted by the integration test, task 7.2).
--
-- DEV credentials only. Production provisions roles via secrets / IaC.

-- auth module --------------------------------------------------------------
CREATE ROLE auth_svc LOGIN PASSWORD 'auth_dev_pw';
CREATE SCHEMA IF NOT EXISTS auth AUTHORIZATION auth_svc;
ALTER ROLE auth_svc SET search_path = auth;

-- user module --------------------------------------------------------------
CREATE ROLE user_svc LOGIN PASSWORD 'user_dev_pw';
CREATE SCHEMA IF NOT EXISTS user_account AUTHORIZATION user_svc;
ALTER ROLE user_svc SET search_path = user_account;

-- jobs (shared async-job substrate; change: add-job-infrastructure) ----------
-- Owned by a least-privilege `worker_svc`. The `cymbra-jobs` migrator (run as
-- worker_svc) creates the sqlxmq `mq_*` objects + the `jobs.*` control tables in
-- this schema; module roles get only USAGE + EXECUTE on `jobs.enqueue` (granted
-- by that migration) — the documented, write-only exception to D0 (design D3).
CREATE ROLE worker_svc LOGIN PASSWORD 'worker_dev_pw';
CREATE SCHEMA IF NOT EXISTS jobs AUTHORIZATION worker_svc;
ALTER ROLE worker_svc SET search_path = jobs;
-- sqlxmq's migration uses uuid_nil()/uuid_generate_v4(); CREATE EXTENSION needs
-- superuser, so it is done here (not in the crate migration) and installed into
-- `jobs` so worker_svc resolves it via its pinned search_path.
CREATE EXTENSION IF NOT EXISTS "uuid-ossp" SCHEMA jobs;

-- Keep the module roles out of the shared `public` schema so the only namespaces
-- each can touch are its own (+ the narrow jobs.enqueue grant above).
REVOKE ALL ON SCHEMA public FROM auth_svc, user_svc, worker_svc;
