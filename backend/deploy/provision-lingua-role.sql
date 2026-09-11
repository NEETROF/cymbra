-- Provision the `lingua` module role + schema on a LIVE database that was initialised
-- before the lingua module existed (change: add-lingua-backend). Idempotent and
-- TARGETED: it touches lingua_svc + the lingua schema and adds `lingua` to the ops
-- role's search_path — it does NOT reset any other role's password (unlike re-running
-- the full 00-roles.sh). Mirror of the `lingua module` block in db/init/roles.sql.tpl.
--
-- Run as the DB superuser, passing the password as a psql variable so it never lands in
-- a file or shell history:
--
--   docker exec -e LPW='<chosen lingua password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v lingua_pw="$LPW" -f - < provision-lingua-role.sql
--
-- After this: set CYMBRA_LINGUA_DATABASE_URL=postgres://lingua_svc:<pw>@postgres:5432/cymbra
-- (server AND worker read it), then roll the stack — the server's MIGRATOR creates the
-- lingua tables and the cymbra.lingua.v1 services mount.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'lingua_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'lingua_svc')
\gexec

ALTER ROLE "lingua_svc" WITH LOGIN PASSWORD :'lingua_pw';
CREATE SCHEMA IF NOT EXISTS lingua AUTHORIZATION "lingua_svc";
ALTER ROLE "lingua_svc" SET search_path = lingua;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA lingua TO "lingua_svc";
ALTER DEFAULT PRIVILEGES IN SCHEMA lingua
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "lingua_svc";

-- Keep lingua_svc out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "lingua_svc";

-- The ops role (MIGRATOR + worker erasure) reads/writes every schema from one
-- connection; add `lingua` to its search_path (idempotent, WITHOUT touching the
-- password). Adjust the role name if your ops role is not `admin_svc`.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'admin_svc') THEN
    EXECUTE 'ALTER ROLE admin_svc SET search_path = auth, user_account, music, jobs, feature_flags, analytics, plans, lingua, public';
  END IF;
END $$;

\echo 'lingua_svc + schema lingua + migrator search_path provisioned (idempotent).'
