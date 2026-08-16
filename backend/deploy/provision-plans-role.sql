-- Provision the `plans` module role + schema on a LIVE database that was
-- initialised before the plans module existed (change: add-premium-subscription).
-- Idempotent and TARGETED: it touches plans_svc + the plans schema, and adds
-- `plans` to the ops role's search_path — it does NOT reset any other role's
-- password (unlike re-running the full 00-roles.sh).
--
-- Mirror of the `plans module` block in db/init/roles.sql.tpl. Run as the DB
-- superuser, passing the password as a psql variable so it never lands in a file
-- or shell history:
--
--   docker exec -e PPW='<chosen plans password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v plans_pw="$PPW" -f - < provision-plans-role.sql
--
-- After this: set CYMBRA_PLANS_DB_PASSWORD (same value) and
-- CYMBRA_PLANS_DATABASE_URL=postgres://plans_svc:<pw>@postgres:5432/cymbra in .env
-- (server AND worker read it), then roll the stack — the server's MIGRATOR
-- creates the plans tables. Everything stays dark until `plans.enabled` is on.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'plans_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'plans_svc')
\gexec

ALTER ROLE "plans_svc" WITH LOGIN PASSWORD :'plans_pw';
CREATE SCHEMA IF NOT EXISTS plans AUTHORIZATION "plans_svc";
ALTER ROLE "plans_svc" SET search_path = plans;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA plans TO "plans_svc";
ALTER DEFAULT PRIVILEGES IN SCHEMA plans
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "plans_svc";

-- Keep plans_svc out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "plans_svc";

-- The ops role reads every schema from one connection; add `plans` to its
-- search_path (idempotent, WITHOUT touching the password). Adjust the role name
-- if your ops role is not `admin_svc`.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'admin_svc') THEN
    EXECUTE 'ALTER ROLE admin_svc SET search_path = auth, user_account, music, jobs, feature_flags, analytics, plans, public';
  END IF;
END $$;
