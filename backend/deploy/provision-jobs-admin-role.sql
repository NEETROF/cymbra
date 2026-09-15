-- Provision the Jobs-console role on a LIVE database (change: add-admin-jobs-console).
-- Idempotent and TARGETED: it creates or updates `jobs_admin_svc` and, if the worker has
-- already migrated them, grants it the four queue-administration functions. It touches
-- no other role's password. Mirror of the `jobs console` block in db/init/roles.sql.tpl.
--
-- The role owns nothing and gets no table privilege: USAGE on schema `jobs` and EXECUTE
-- on the SECURITY DEFINER `jobs.admin_*` functions only, so the server can list queue
-- metadata and cancel a job but never read a job's payload.
--
-- Run as the DB superuser, passing the password as a psql variable so it never lands in
-- a file or shell history:
--
--   docker exec -e JPW='<chosen jobs admin password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v jobs_admin_pw="$JPW" -f - < provision-jobs-admin-role.sql
--
-- After this: set CYMBRA_JOBS_ADMIN_DATABASE_URL=postgres://jobs_admin_svc:<pw>@postgres:5432/cymbra
-- (the SERVER reads it; the worker does not), then roll the server — JobsAdminService
-- mounts. Order with the worker deploy does not matter: its migration grants to this role
-- when the role exists, and this script grants on the functions when they exist.

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'jobs_admin_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'jobs_admin_svc')
\gexec

ALTER ROLE "jobs_admin_svc" WITH LOGIN PASSWORD :'jobs_admin_pw';
ALTER ROLE "jobs_admin_svc" SET search_path = jobs;

-- Keep it out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "jobs_admin_svc";

DO $$
BEGIN
  IF to_regprocedure('jobs.admin_cancel(uuid, text, text[])') IS NOT NULL THEN
    GRANT USAGE ON SCHEMA jobs TO jobs_admin_svc;
    GRANT EXECUTE ON FUNCTION
      jobs.admin_list_jobs(text, text, integer, integer),
      jobs.admin_queue_counts(text),
      jobs.admin_period_stats(timestamptz, timestamptz, text),
      jobs.admin_cancel(uuid, text, text[])
    TO jobs_admin_svc;
    RAISE NOTICE 'jobs_admin_svc: queue-administration functions granted';
  ELSE
    RAISE NOTICE 'jobs_admin_svc: functions not migrated yet; the worker migration will grant them';
  END IF;
END $$;

\echo 'jobs_admin_svc provisioned (idempotent).'
