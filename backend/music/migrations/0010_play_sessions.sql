-- music module — end-of-session play stats (change: add-play-activity-profile,
-- task 1.1, design D1/D2/D3).
--
-- One row per completed play session, keyed by a CLIENT-generated UUID v7 (the
-- session id): the app captures the session durably in a local outbox and
-- retries `RecordPlaySession` until acked, so ingestion must be idempotent —
-- `INSERT ... ON CONFLICT (id) DO NOTHING` makes a resent id a no-op (D2), giving
-- at-least-once + idempotent = effectively-once (no loss, no double-count).
--
-- `user_id` is a PLAIN uuid — no cross-schema FK to the user module (module-role
-- isolation, exactly like user_scores/user_library). Account deletion erases
-- these rows explicitly in the `purge_user` worker job (no DB cascade possible).
--
-- Two tiers of data (design D7):
--   * lightweight summary kept long-term: `overall_sync_pct` + `played_at` +
--     `tz_offset_minutes` drive the heatmap (per-day count + avg success %);
--   * the heavy `session_result` JSONB (the full immutable record incl. per-note
--     judgments, for future replay/leaderboards) is PRUNED after a retention
--     window by the worker, which NULLs it while keeping the summary.
--
-- `played_at` is the wall-clock instant the session ended; `tz_offset_minutes`
-- is the client's UTC offset AT that instant, so the per-day heatmap buckets by
-- the player's LOCAL day (design D3/D4), not UTC.
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path.

CREATE TABLE IF NOT EXISTS music.play_sessions (
    id                UUID PRIMARY KEY,                 -- client-generated UUID v7 (the session id)
    user_id           UUID        NOT NULL,             -- caller's AuthIdentity.user_id; no cross-schema FK
    score_id          TEXT,                             -- catalog/user score identity (opaque; nullable)
    played_at         TIMESTAMPTZ NOT NULL,             -- when the session ended (client wall clock)
    tz_offset_minutes INTEGER     NOT NULL DEFAULT 0,   -- client UTC offset at played_at (local-day bucketing)
    overall_sync_pct  REAL        NOT NULL,             -- the success score 0..100 (summary tier; kept long-term)
    session_result    JSONB,                            -- the full immutable record (heavy tier; pruned after retention)
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Every read is user-scoped and time-ordered (per-day aggregate, activity window).
CREATE INDEX IF NOT EXISTS play_sessions_user_played_idx
    ON music.play_sessions (user_id, played_at);

-- The retention prune scans by age across all users, keeping only rows whose
-- heavy detail is still present.
CREATE INDEX IF NOT EXISTS play_sessions_detail_created_idx
    ON music.play_sessions (created_at)
    WHERE session_result IS NOT NULL;
