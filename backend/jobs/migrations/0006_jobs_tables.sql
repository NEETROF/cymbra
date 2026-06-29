-- Cymbra job infrastructure — control tables, the transactional-enqueue seam,
-- the cross-module grants (design D3 + spike refinement), and observability views.
-- Runs as `worker_svc` (owner of the `jobs` schema), search_path = jobs.

-- --- Runtime-tunable retry policy per (module, kind) (design D6) -------------
CREATE TABLE retry_policy (
    module        TEXT     NOT NULL,
    kind          TEXT     NOT NULL,
    max_attempts  INT      NOT NULL CHECK (max_attempts >= 1),
    base_backoff  INTERVAL NOT NULL DEFAULT INTERVAL '1 second',
    max_backoff   INTERVAL NOT NULL DEFAULT INTERVAL '1 hour',
    ordered       BOOLEAN  NOT NULL DEFAULT FALSE,
    PRIMARY KEY (module, kind)
);

-- --- Recurring schedules (design D5) ----------------------------------------
CREATE TABLE schedules (
    name               TEXT        PRIMARY KEY,
    module             TEXT        NOT NULL,
    kind               TEXT        NOT NULL,
    cron_expr          TEXT        NOT NULL,
    timezone           TEXT        NOT NULL DEFAULT 'UTC',
    enabled            BOOLEAN     NOT NULL DEFAULT TRUE,
    -- 'skip' collapses missed occurrences to the latest; 'catch_up' enqueues each.
    missed_run_policy  TEXT        NOT NULL DEFAULT 'skip'
        CHECK (missed_run_policy IN ('skip', 'catch_up')),
    payload_json       TEXT        NOT NULL DEFAULT '{}',
    last_evaluated_at  TIMESTAMPTZ
);

-- Idempotency ledger: one row per (schedule, occurrence) so that even with many
-- worker replicas evaluating the same cron, exactly one job is enqueued. The
-- enqueue and this insert happen in one transaction; `ON CONFLICT DO NOTHING`
-- makes the loser a no-op (design D5).
CREATE TABLE schedule_occurrences (
    dedup_key    TEXT        PRIMARY KEY,   -- '<name>:<occurrence-unix>'
    enqueued_at  TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- --- Dead-letter store (design D6) ------------------------------------------
CREATE TABLE dead_letter (
    id               UUID        PRIMARY KEY,   -- original mq_msgs.id
    name             TEXT        NOT NULL,
    channel_name     TEXT        NOT NULL,
    channel_args     TEXT        NOT NULL DEFAULT '',
    payload_json     JSONB,
    attempts         INT         NOT NULL,
    last_error       TEXT,
    dead_lettered_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
CREATE INDEX dead_letter_at_idx ON dead_letter (dead_lettered_at DESC);

-- --- Transactional-enqueue seam (design D3 + spike finding #2) ---------------
-- A SECURITY DEFINER wrapper owned by `worker_svc`: module roles call it inside
-- their own business transaction to enqueue a job, but never touch the `mq_*`
-- tables directly and gain no read access to any other schema. This is the
-- narrow, deliberate exception to the per-module isolation invariant (D0).
CREATE FUNCTION enqueue(
    p_name          TEXT,
    p_channel_name  TEXT,
    p_channel_args  TEXT,
    p_ordered       BOOLEAN,
    p_retries       INT,
    p_retry_backoff INTERVAL,
    p_delay         INTERVAL,
    p_payload_json  TEXT
) RETURNS UUID
LANGUAGE plpgsql
SECURITY DEFINER
SET search_path = jobs
AS $$
DECLARE
    v_id UUID := uuid_generate_v4();
BEGIN
    PERFORM mq_insert(ARRAY[(
        v_id,
        COALESCE(p_delay, INTERVAL '0'),
        p_retries,
        p_retry_backoff,
        p_channel_name,
        COALESCE(p_channel_args, ''),
        NULL,                       -- commit_interval: NULL = single-phase (committed)
        p_ordered,
        p_name,
        p_payload_json,
        NULL                        -- payload_bytes
    )::mq_new_t]);
    RETURN v_id;
END;
$$;

-- --- Observability views (design D8; task 7.1) ------------------------------
-- Ready to run now (committed, not blocked by an unfinished predecessor).
CREATE VIEW pending AS
SELECT id, channel_name, channel_args, attempts, created_at, attempt_at
FROM mq_msgs
WHERE id != uuid_nil()
  AND commit_interval IS NULL
  AND attempt_at IS NOT NULL
  AND attempt_at <= NOW()
  AND NOT mq_uuid_exists(after_message_id);

-- Live but not yet runnable: future/retry-waiting messages with attempts left.
CREATE VIEW inflight AS
SELECT id, channel_name, channel_args, attempts, created_at, attempt_at
FROM mq_msgs
WHERE id != uuid_nil()
  AND attempt_at IS NOT NULL
  AND attempt_at > NOW()
  AND attempts > 0;

-- Exhausted-but-not-yet-reaped messages (attempts spent, never to run again)
-- unioned with the dead-letter store.
CREATE VIEW failed AS
SELECT id, channel_name, name AS job_name, attempts, NULL::TIMESTAMPTZ AS dead_lettered_at
FROM mq_msgs
WHERE id != uuid_nil() AND attempt_at IS NULL AND attempts <= 0
UNION ALL
SELECT id, channel_name, name AS job_name, attempts, dead_lettered_at
FROM dead_letter;

-- Per-channel queue depth and age of the oldest waiting message.
CREATE VIEW channel_depth AS
SELECT
    channel_name,
    COUNT(*) AS depth,
    NOW() - MIN(created_at) AS oldest_age
FROM mq_msgs
WHERE id != uuid_nil()
  AND attempt_at IS NOT NULL
GROUP BY channel_name;

-- --- Cross-module grants (design D3 + spike finding #2) ---------------------
-- Module roles get USAGE on the schema and EXECUTE on the single enqueue
-- function — nothing else. No table privileges, no read access to mq_* or any
-- other schema. SECURITY DEFINER runs the insert with `worker_svc`'s rights.
GRANT USAGE ON SCHEMA jobs TO auth_svc, user_svc;
GRANT EXECUTE ON FUNCTION enqueue(
    TEXT, TEXT, TEXT, BOOLEAN, INT, INTERVAL, INTERVAL, TEXT
) TO auth_svc, user_svc;
