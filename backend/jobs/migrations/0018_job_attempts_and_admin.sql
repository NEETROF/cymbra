-- Attempt history, operator cancellations and the queue-administration surface
-- (change: add-admin-jobs-console). Runs as `worker_svc` (owner of `jobs`),
-- search_path = jobs.
--
-- Why a history at all: sqlxmq deletes a job when it completes and writes nothing
-- when a handler fails, so "how many jobs completed this week" has no answer in
-- `mq_msgs`, and a running job looks exactly like one waiting for its retry (both
-- have `attempt_at` in the future). The worker records each attempt around its
-- handler (design D1); these tables are what it writes.

-- --- Attempt history (design D1) --------------------------------------------
CREATE TABLE job_attempts (
    id           BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    -- mq_msgs.id. No FK: the message is deleted when the job completes.
    job_id       UUID        NOT NULL,
    job_name     TEXT        NOT NULL,
    channel_name TEXT        NOT NULL,
    attempt      INT         NOT NULL CHECK (attempt >= 1),
    started_at   TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    -- When the attempt was closed. For `abandoned` that is when the crash was
    -- noticed, not when the handler stopped, so run times come from `succeeded` only.
    finished_at  TIMESTAMPTZ,
    outcome      TEXT        NOT NULL DEFAULT 'running'
        CHECK (outcome IN ('running', 'succeeded', 'failed', 'abandoned')),
    CHECK ((outcome = 'running') = (finished_at IS NULL))
);
CREATE INDEX job_attempts_job_idx ON job_attempts (job_id);
CREATE INDEX job_attempts_running_idx ON job_attempts (job_id) WHERE outcome = 'running';
CREATE INDEX job_attempts_finished_idx ON job_attempts (finished_at) WHERE finished_at IS NOT NULL;
CREATE INDEX job_attempts_started_idx ON job_attempts (started_at);

-- --- Operator cancellations (design D3) ---------------------------------------
CREATE TABLE cancellations (
    job_id       UUID        PRIMARY KEY,
    job_name     TEXT        NOT NULL,
    channel_name TEXT        NOT NULL,
    -- The admin's user id, taken from the auth interceptor, never from a request body.
    cancelled_by TEXT        NOT NULL,
    cancelled_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX cancellations_at_idx ON cancellations (cancelled_at);

-- --- What "running" means (design D2) ------------------------------------------
-- How long a FINAL attempt counts as running. sqlxmq sets `attempt_at = NULL` when it
-- claims the last attempt, so there is no lease left to expire; past this grace the
-- attempt is presumed crashed. The dead-letter sweep reads the same value.
CREATE FUNCTION running_grace() RETURNS INTERVAL
LANGUAGE sql IMMUTABLE
AS $$ SELECT INTERVAL '1 hour' $$;

-- Whether a `running` attempt that started at `p_started_at` still holds its message,
-- whose next attempt time is `p_attempt_at`. A non-final attempt holds it while the
-- runner's keep-alive pushes `attempt_at` forward; once that time passes, sqlxmq may
-- hand the job to another worker. One definition for the view, the cancellation and
-- the sweep, so the three cannot disagree about whether a job is running.
CREATE FUNCTION attempt_holds_lease(p_started_at TIMESTAMPTZ, p_attempt_at TIMESTAMPTZ)
RETURNS BOOLEAN
LANGUAGE sql STABLE
AS $$
    SELECT p_started_at IS NOT NULL
       AND (p_attempt_at > NOW()
            OR (p_attempt_at IS NULL AND p_started_at > NOW() - running_grace()))
$$;

-- One row per queued job with a single derived state; the first matching rule wins.
-- Filtering, paging and per-state counts all read this, so the rule lives here once.
CREATE VIEW admin_queue AS
SELECT
    m.id,
    COALESCE(p.name, '')    AS job_name,
    m.channel_name,
    m.created_at,
    m.attempt_at,
    GREATEST(m.attempts, 0) AS attempts_left,
    COALESCE(made.n, 0)     AS attempts_made,
    run.started_at          AS running_started_at,
    CASE
        WHEN attempt_holds_lease(run.started_at, m.attempt_at) THEN 'running'
        WHEN m.attempt_at IS NULL                              THEN 'exhausted'
        WHEN mq_uuid_exists(m.after_message_id)                THEN 'blocked'
        WHEN m.attempt_at <= NOW()                             THEN 'ready'
        WHEN COALESCE(made.n, 0) > 0                           THEN 'retry_wait'
        ELSE                                                        'scheduled'
    END                     AS state
FROM mq_msgs m
LEFT JOIN mq_payloads p ON p.id = m.id
LEFT JOIN LATERAL (
    SELECT COUNT(*)::INT AS n FROM job_attempts a WHERE a.job_id = m.id
) made ON TRUE
LEFT JOIN LATERAL (
    SELECT a.started_at FROM job_attempts a
    WHERE a.job_id = m.id AND a.outcome = 'running'
    ORDER BY a.started_at DESC
    LIMIT 1
) run ON TRUE
WHERE m.id != uuid_nil();

-- --- Queue-administration functions (design D4) ------------------------------
-- SECURITY DEFINER so the console role reads queue metadata and cancels without any
-- table privilege: none of these returns a payload column, and the role cannot select
-- one itself.

CREATE FUNCTION admin_list_jobs(p_state TEXT, p_name TEXT, p_limit INT, p_offset INT)
RETURNS TABLE (
    id                 UUID,
    job_name           TEXT,
    channel_name       TEXT,
    state              TEXT,
    attempts_made      INT,
    attempts_left      INT,
    created_at         TIMESTAMPTZ,
    attempt_at         TIMESTAMPTZ,
    running_started_at TIMESTAMPTZ
)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    SELECT q.id, q.job_name, q.channel_name, q.state, q.attempts_made, q.attempts_left,
           q.created_at, q.attempt_at, q.running_started_at
    FROM admin_queue q
    WHERE (p_state IS NULL OR q.state = p_state)
      AND (p_name IS NULL OR q.job_name = p_name)
    ORDER BY q.created_at ASC, q.id ASC
    LIMIT p_limit OFFSET p_offset
$$;

CREATE FUNCTION admin_queue_counts(p_name TEXT)
RETURNS TABLE (state TEXT, n BIGINT)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    SELECT q.state, COUNT(*)
    FROM admin_queue q
    WHERE p_name IS NULL OR q.job_name = p_name
    GROUP BY q.state
$$;

-- Figures over [p_from, p_to). `history_since` is the oldest attempt on record: before
-- it the figures are unknown, not zero.
CREATE FUNCTION admin_period_stats(p_from TIMESTAMPTZ, p_to TIMESTAMPTZ, p_name TEXT)
RETURNS TABLE (
    completed       BIGINT,
    failed_attempts BIGINT,
    dead_lettered   BIGINT,
    cancelled       BIGINT,
    avg_run_ms      BIGINT,
    history_since   TIMESTAMPTZ
)
LANGUAGE sql STABLE SECURITY DEFINER
SET search_path = jobs
AS $$
    SELECT
        (SELECT COUNT(DISTINCT a.job_id) FROM job_attempts a
          WHERE a.outcome = 'succeeded'
            AND a.finished_at >= p_from AND a.finished_at < p_to
            AND (p_name IS NULL OR a.job_name = p_name)),
        (SELECT COUNT(*) FROM job_attempts a
          WHERE a.outcome = 'failed'
            AND a.finished_at >= p_from AND a.finished_at < p_to
            AND (p_name IS NULL OR a.job_name = p_name)),
        (SELECT COUNT(*) FROM dead_letter d
          WHERE d.dead_lettered_at >= p_from AND d.dead_lettered_at < p_to
            AND (p_name IS NULL OR d.name = p_name)),
        (SELECT COUNT(*) FROM cancellations c
          WHERE c.cancelled_at >= p_from AND c.cancelled_at < p_to
            AND (p_name IS NULL OR c.job_name = p_name)),
        (SELECT ROUND(AVG(EXTRACT(EPOCH FROM (a.finished_at - a.started_at)) * 1000))::BIGINT
           FROM job_attempts a
          WHERE a.outcome = 'succeeded'
            AND a.finished_at >= p_from AND a.finished_at < p_to
            AND (p_name IS NULL OR a.job_name = p_name)),
        (SELECT MIN(a.started_at) FROM job_attempts a)
$$;

-- Cancel one queued job (design D3). Returns 'cancelled', 'gone', 'protected' or
-- 'running'. `p_protected` is the registry's list of non-cancellable kinds, passed by
-- the caller so the list has one home (`cymbra_jobs::registry`) and the check happens
-- under the same row lock as the delete.
CREATE FUNCTION admin_cancel(p_job_id UUID, p_actor TEXT, p_protected TEXT[])
RETURNS TEXT
LANGUAGE plpgsql SECURITY DEFINER
SET search_path = jobs
AS $$
DECLARE
    v_channel    TEXT;
    v_attempt_at TIMESTAMPTZ;
    v_after      UUID;
    v_name       TEXT;
    v_running    TIMESTAMPTZ;
BEGIN
    IF p_job_id IS NULL OR p_job_id = uuid_nil() THEN
        RETURN 'gone';
    END IF;

    -- The worker locks the same row before it starts an attempt, so a claim and a
    -- cancellation serialise: either this sees the new `running` attempt and refuses,
    -- or the worker finds the message gone and skips the handler.
    SELECT m.channel_name, m.attempt_at, m.after_message_id
      INTO v_channel, v_attempt_at, v_after
      FROM mq_msgs m
     WHERE m.id = p_job_id
       FOR UPDATE;
    IF NOT FOUND THEN
        RETURN 'gone';
    END IF;

    SELECT p.name INTO v_name FROM mq_payloads p WHERE p.id = p_job_id;
    IF v_name = ANY(COALESCE(p_protected, '{}'::TEXT[])) THEN
        RETURN 'protected';
    END IF;

    SELECT a.started_at INTO v_running
      FROM job_attempts a
     WHERE a.job_id = p_job_id AND a.outcome = 'running'
     ORDER BY a.started_at DESC
     LIMIT 1;
    IF attempt_holds_lease(v_running, v_attempt_at) THEN
        RETURN 'running';
    END IF;

    INSERT INTO cancellations (job_id, job_name, channel_name, cancelled_by)
    VALUES (p_job_id, COALESCE(v_name, ''), v_channel, p_actor)
    ON CONFLICT (job_id) DO NOTHING;

    -- A `running` attempt that no longer holds the lease belonged to a crashed worker.
    UPDATE job_attempts
       SET outcome = 'abandoned', finished_at = NOW()
     WHERE job_id = p_job_id AND outcome = 'running';

    -- Ordered channels are a linked list with a unique (channel, args, predecessor)
    -- index and `ON DELETE SET DEFAULT` (nil). Deleting a message in the middle would
    -- point its successor at nil: a unique violation when the head already holds nil,
    -- or a successor overtaking the head. So the successor is pointed at this message's
    -- predecessor first, after this message releases its slot in the index.
    IF v_after IS NOT NULL THEN
        UPDATE mq_msgs SET after_message_id = NULL WHERE id = p_job_id;
        UPDATE mq_msgs SET after_message_id = v_after WHERE after_message_id = p_job_id;
    END IF;

    PERFORM mq_delete(ARRAY[p_job_id]);
    -- mq_delete only notifies when it removes a head, and the relink above cleared that
    -- marker; wake the runners so a promoted successor starts now, not at the next poll.
    PERFORM pg_notify(CONCAT('mq_', v_channel), '');
    PERFORM pg_notify('mq', '');
    RETURN 'cancelled';
END;
$$;

REVOKE EXECUTE ON FUNCTION admin_list_jobs(TEXT, TEXT, INT, INT) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION admin_queue_counts(TEXT) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION admin_period_stats(TIMESTAMPTZ, TIMESTAMPTZ, TEXT) FROM PUBLIC;
REVOKE EXECUTE ON FUNCTION admin_cancel(UUID, TEXT, TEXT[]) FROM PUBLIC;
-- `enqueue` (0006) kept Postgres's default EXECUTE for PUBLIC. It never mattered, since
-- only the module roles had USAGE on `jobs` and each holds an explicit grant; but the
-- console role now has USAGE too and must not be able to enqueue. Every intended caller
-- keeps its explicit grant (0006, 0010, 0016).
REVOKE EXECUTE ON FUNCTION enqueue(TEXT, TEXT, TEXT, BOOLEAN, INT, INTERVAL, INTERVAL, TEXT) FROM PUBLIC;

-- --- Grants to the console role (design D4) ------------------------------------
-- Conditional: on a live database the role may be provisioned after this migration
-- ran. `provision-jobs-admin-role.sql` then grants the same functions, so whichever
-- runs first, the second completes the grants.
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'jobs_admin_svc') THEN
        GRANT USAGE ON SCHEMA jobs TO jobs_admin_svc;
        GRANT EXECUTE ON FUNCTION
            admin_list_jobs(TEXT, TEXT, INT, INT),
            admin_queue_counts(TEXT),
            admin_period_stats(TIMESTAMPTZ, TIMESTAMPTZ, TEXT),
            admin_cancel(UUID, TEXT, TEXT[])
        TO jobs_admin_svc;
    END IF;
END $$;
