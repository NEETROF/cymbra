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

-- Keep both roles out of the shared `public` schema so the only namespace each
-- can touch is its own.
REVOKE ALL ON SCHEMA public FROM auth_svc, user_svc;
