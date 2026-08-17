-- Provision the `feature_flags` module role + schema on a LIVE database that was
-- initialised before the flags module existed (change: add-runtime-feature-flags).
-- Idempotent and TARGETED: it touches flags_svc + the feature_flags schema only —
-- it does NOT reset any other role's password (unlike re-running 00-roles.sh).
--
-- Mirror of the `feature flags` block in db/init/roles.sql.tpl. Run as the DB
-- superuser, passing the password as a psql variable so it never lands in a file
-- or shell history:
--
--   docker exec -e FPW='<chosen flags password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v flags_pw="$FPW" -f - < provision-flags-role.sql
--
-- After this: set CYMBRA_FLAGS_DB_PASSWORD (same value) and
-- CYMBRA_FLAGS_DATABASE_URL=postgres://flags_svc:<pw>@postgres:5432/cymbra in .env
-- (server AND worker read it), then roll the stack — the server's MIGRATOR creates
-- the feature_flags tables and the back office Flags screen becomes writable.
-- Without it the server runs "defaults-only" and no flag (plans.enabled included)
-- can be changed at runtime.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'flags_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'flags_svc')
\gexec

ALTER ROLE "flags_svc" WITH LOGIN PASSWORD :'flags_pw';
CREATE SCHEMA IF NOT EXISTS feature_flags AUTHORIZATION "flags_svc";
ALTER ROLE "flags_svc" SET search_path = feature_flags;

-- Keep flags_svc out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "flags_svc";
