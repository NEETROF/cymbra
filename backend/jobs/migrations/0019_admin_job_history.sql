-- History, per-kind figures and schedules for the Jobs console (change:
-- add-jobs-console-history). Runs as `worker_svc` (owner of `jobs`), search_path = jobs.
-- Re-runnable: every object is created with OR REPLACE / IF NOT EXISTS.
--
-- Nothing new is recorded: `job_attempts` (0018) already holds one row per attempt,
-- `dead_letter` and `cancellations` the other two outcomes, `schedules` (0006) the
-- recurring cadences. The console role still holds no table privilege, so it reads them
-- through the SECURITY DEFINER functions below, none of which returns a payload column
-- or error text (`schedules.payload_json`, `dead_letter.payload_json/last_error`).

-- The kind-filtered history: newest finished attempts of one kind.
CREATE INDEX IF NOT EXISTS job_attempts_name_finished_idx
    ON job_attempts (job_name, finished_at)
    WHERE finished_at IS NOT NULL;

-- Finished attempts over [p_from, p_to), newest first. `running` is never listed: those
-- attempts belong to the queue table. `p_name` / `p_outcome` NULL = no filter.
CREATE OR REPLACE FUNCTION admin_list_attempts(
    p_from    TIMESTAMPTZ,
    p_to      TIMESTAMPTZ,
    p_name    TEXT,
    p_outcome TEXT,
    p_limit   INT,
    p_offset  INT
)
RETURNS TABLE (
    job_id       UUID,
    job_name     TEXT,
    channel_name TEXT,
    attempt      INT,
    outcome      TEXT,
    started_at   TIMESTAMPTZ,
    finished_at  TIMESTAMPTZ
)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    SELECT a.job_id, a.job_name, a.channel_name, a.attempt, a.outcome,
           a.started_at, a.finished_at
    FROM job_attempts a
    WHERE a.outcome <> 'running'
      AND a.finished_at >= p_from AND a.finished_at < p_to
      AND (p_name IS NULL OR a.job_name = p_name)
      AND (p_outcome IS NULL OR a.outcome = p_outcome)
    ORDER BY a.finished_at DESC, a.id DESC
    LIMIT p_limit OFFSET p_offset
$$;

-- How many attempts `admin_list_attempts` matches across all pages.
CREATE OR REPLACE FUNCTION admin_count_attempts(
    p_from    TIMESTAMPTZ,
    p_to      TIMESTAMPTZ,
    p_name    TEXT,
    p_outcome TEXT
)
RETURNS BIGINT
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    SELECT COUNT(*)
    FROM job_attempts a
    WHERE a.outcome <> 'running'
      AND a.finished_at >= p_from AND a.finished_at < p_to
      AND (p_name IS NULL OR a.job_name = p_name)
      AND (p_outcome IS NULL OR a.outcome = p_outcome)
$$;

-- `admin_period_stats` per kind, over the same window and filter. One grouped aggregate
-- per source, combined and summed per kind, so each figure adds up to the total
-- (`completed` counts distinct jobs, and a job has exactly one kind). Only kinds with at
-- least one non-zero figure are returned, ordered by kind; the console adds the others.
CREATE OR REPLACE FUNCTION admin_period_stats_by_kind(
    p_from TIMESTAMPTZ,
    p_to   TIMESTAMPTZ,
    p_name TEXT
)
RETURNS TABLE (
    job_name         TEXT,
    completed        BIGINT,
    failed_attempts  BIGINT,
    dead_lettered    BIGINT,
    cancelled        BIGINT,
    avg_run_ms       BIGINT,
    last_finished_at TIMESTAMPTZ
)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    WITH per_source AS (
        SELECT a.job_name,
               COUNT(DISTINCT a.job_id) FILTER (WHERE a.outcome = 'succeeded') AS completed,
               COUNT(*) FILTER (WHERE a.outcome = 'failed')                    AS failed_attempts,
               0::BIGINT                                                        AS dead_lettered,
               0::BIGINT                                                        AS cancelled,
               ROUND(AVG(EXTRACT(EPOCH FROM (a.finished_at - a.started_at)) * 1000)
                     FILTER (WHERE a.outcome = 'succeeded'))::BIGINT          AS avg_run_ms,
               MAX(a.finished_at)                                               AS last_finished_at
        FROM job_attempts a
        WHERE a.outcome <> 'running'
          AND a.finished_at >= p_from AND a.finished_at < p_to
          AND (p_name IS NULL OR a.job_name = p_name)
        GROUP BY a.job_name
        UNION ALL
        SELECT d.name, 0, 0, COUNT(*), 0, NULL, NULL
        FROM dead_letter d
        WHERE d.dead_lettered_at >= p_from AND d.dead_lettered_at < p_to
          AND (p_name IS NULL OR d.name = p_name)
        GROUP BY d.name
        UNION ALL
        SELECT c.job_name, 0, 0, 0, COUNT(*), NULL, NULL
        FROM cancellations c
        WHERE c.cancelled_at >= p_from AND c.cancelled_at < p_to
          AND (p_name IS NULL OR c.job_name = p_name)
        GROUP BY c.job_name
    )
    SELECT s.job_name,
           SUM(s.completed)::BIGINT,
           SUM(s.failed_attempts)::BIGINT,
           SUM(s.dead_lettered)::BIGINT,
           SUM(s.cancelled)::BIGINT,
           MAX(s.avg_run_ms),
           MAX(s.last_finished_at)
    FROM per_source s
    GROUP BY s.job_name
    HAVING SUM(s.completed) + SUM(s.failed_attempts) + SUM(s.dead_lettered)
           + SUM(s.cancelled) > 0
        OR MAX(s.last_finished_at) IS NOT NULL
    ORDER BY s.job_name
$$;

-- The recurring schedules, without their payload (a scheduled `push_dispatch` carries its
-- message there) and without the scheduler's bookkeeping columns.
CREATE OR REPLACE FUNCTION admin_list_schedules()
RETURNS TABLE (
    name      TEXT,
    kind      TEXT,
    cron_expr TEXT,
    timezone  TEXT,
    enabled   BOOLEAN
)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    SELECT s.name, s.kind, s.cron_expr, s.timezone, s.enabled
    FROM schedules s
    ORDER BY s.kind, s.name
$$;

REVOKE EXECUTE ON FUNCTION admin_list_attempts(TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT, INT, INT) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION admin_count_attempts(TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION admin_period_stats_by_kind(TIMESTAMPTZ, TIMESTAMPTZ, TEXT) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION admin_list_schedules() FROM PUBLIC;

-- --- Grants to the console role -------------------------------------------------
-- Conditional, like 0018: the role may be provisioned after this migration ran, and
-- `provision-jobs-admin-role.sql` then grants the same functions.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'jobs_admin_svc') THEN
        GRANT EXECUTE ON FUNCTION
            admin_list_attempts(TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT, INT, INT),
            admin_count_attempts(TIMESTAMPTZ, TIMESTAMPTZ, TEXT, TEXT),
            admin_period_stats_by_kind(TIMESTAMPTZ, TIMESTAMPTZ, TEXT),
            admin_list_schedules()
        TO jobs_admin_svc;
    END IF;
END $$;
