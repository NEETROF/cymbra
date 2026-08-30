-- Provision the `app_updates` module role + schema on a LIVE database that was
-- initialised before the desktop update feed existed (change:
-- add-desktop-auto-update). Idempotent and TARGETED: it touches updates_svc + the
-- app_updates schema, and adds `app_updates` to the ops role's search_path — it
-- does NOT reset any other role's password (unlike re-running the full
-- 00-roles.sh).
--
-- Mirror of the `app_updates module` block in db/init/roles.sql.tpl. Run as the DB
-- superuser, passing the password as a psql variable so it never lands in a file
-- or shell history:
--
--   docker exec -e UPW='<chosen updates password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v updates_pw="$UPW" -f - < provision-updates-role.sql
--
-- After this: set CYMBRA_UPDATES_DB_PASSWORD (same value) and
-- CYMBRA_UPDATES_DATABASE_URL=postgres://updates_svc:<pw>@postgres:5432/cymbra in
-- .env, then roll the stack — the server's MIGRATOR creates app_updates.releases.
-- The feed stays empty (204) until the release workflow ingests something.
--
-- ⚠️ `/updates/*` must ALSO be in the Caddy `@http` matcher, or the path falls
-- through to tonic and answers 200 with an empty `grpc-status: 12`.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'updates_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'updates_svc')
\gexec

ALTER ROLE "updates_svc" WITH LOGIN PASSWORD :'updates_pw';
CREATE SCHEMA IF NOT EXISTS app_updates AUTHORIZATION "updates_svc";
ALTER ROLE "updates_svc" SET search_path = app_updates;

GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA app_updates TO "updates_svc";
ALTER DEFAULT PRIVILEGES IN SCHEMA app_updates
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "updates_svc";

-- Keep updates_svc out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "updates_svc";

-- The ops role reads every schema from one connection; add `app_updates` to its
-- search_path (idempotent, WITHOUT touching the password). Adjust the role name
-- if your ops role is not `admin_svc`.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'admin_svc') THEN
    EXECUTE 'ALTER ROLE admin_svc SET search_path = auth, user_account, music, jobs, feature_flags, analytics, plans, app_updates, public';
  END IF;
END $$;
