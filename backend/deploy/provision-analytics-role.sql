-- Provision the `analytics` module role + schema on a LIVE database that was
-- initialised before the analytics module existed (change: add-feature-usage-
-- analytics). Idempotent and TARGETED: it touches analytics_svc + the analytics
-- schema, and adds `analytics` to the MIGRATOR role's search_path — it does NOT
-- reset any other role's password (unlike re-running the full 00-roles.sh).
--
-- Mirror of the `analytics module` block in db/init/roles.sql.tpl. Run as the DB
-- superuser, passing the password as a psql variable so it never lands in a file
-- or shell history:
--
--   docker exec -e APW='<chosen analytics password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v analytics_pw="$APW" -f - < provision-analytics-role.sql
--
-- After this: set CYMBRA_ANALYTICS_DATABASE_URL=postgres://analytics_svc:<pw>@postgres:5432/cymbra
-- then roll the server — its MIGRATOR creates analytics.usage_events (+ aggregates).

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'analytics_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'analytics_svc')
\gexec

ALTER ROLE "analytics_svc" WITH LOGIN PASSWORD :'analytics_pw';
CREATE SCHEMA IF NOT EXISTS analytics AUTHORIZATION "analytics_svc";
ALTER ROLE "analytics_svc" SET search_path = analytics;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA analytics TO "analytics_svc";
ALTER DEFAULT PRIVILEGES IN SCHEMA analytics
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "analytics_svc";

-- Keep analytics_svc out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "analytics_svc";

-- The MIGRATOR + worker rollup/purge run as admin_svc; add `analytics` to its
-- search_path so it resolves the schema (idempotent, WITHOUT touching the
-- password). Adjust the role name if your migrator/admin role is not `admin_svc`.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'admin_svc') THEN
    EXECUTE 'ALTER ROLE admin_svc SET search_path = auth, user_account, music, jobs, feature_flags, analytics, public';
  END IF;
END $$;

\echo 'analytics_svc + schema analytics + migrator search_path provisioned (idempotent).'
