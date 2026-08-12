-- music module — the consecutive-day practice streak (change: add-practice-streak,
-- task 1.1).
--
-- ONE row per user holding the whole streak: the running `current_streak`, the
-- monotonic `longest_streak` (what the streak badges are measured against, so it
-- is never decreased), and `last_played_date` — the player's LOCAL calendar day
-- of their most recent play, the same day convention the heatmap buckets by
-- (`play_core::local_day`, from the client's UTC offset at the session).
--
-- Deliberately a small derived-state table rather than a query over
-- `music.play_sessions`: the streak is read on every app launch and advanced on
-- every ingest, and the freeze (below) makes it *not* a pure function of the
-- sessions — a recovered day has no session behind it.
--
-- `last_played_date` doubles as the freeze/recovery anchor. A break is only
-- **materialised** by the next play (`advance` resets the run to 1 then), so
-- while the user has not played since the break the pre-break `current_streak`
-- is still standing here — that is exactly what a confirmed points freeze
-- restores, with no extra "previous streak" column to keep consistent.
--
-- `user_id` is a PLAIN uuid — no cross-schema FK to the user module (module-role
-- isolation, like play_sessions/leaderboard_bests). Account deletion erases this
-- row explicitly in the `purge_user` worker job.
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path.

CREATE TABLE IF NOT EXISTS music.practice_streaks (
    user_id          UUID PRIMARY KEY,               -- caller's AuthIdentity.user_id; no cross-schema FK
    current_streak   INTEGER     NOT NULL DEFAULT 0, -- consecutive local days ending at last_played_date
    longest_streak   INTEGER     NOT NULL DEFAULT 0, -- monotonic all-time best (badge metric)
    last_played_date DATE,                           -- player's LOCAL day of the most recent play
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The reminder job's candidate sweep is "everyone with a live streak", ordered by
-- how stale it is. Partial on `current_streak > 0` so it stays tiny: users with no
-- streak are never at risk and never scanned.
CREATE INDEX IF NOT EXISTS practice_streaks_at_risk_idx
    ON music.practice_streaks (last_played_date)
    WHERE current_streak > 0;
