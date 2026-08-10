-- music module — scoreless practice sessions (change: add-measure-range-practice,
-- task 2.2, design D4).
--
-- A **selective run** (a chosen measure range) is practice: it is never scored,
-- so it must never land in `music.play_sessions` — that table's rows carry an
-- `overall_sync_pct` and feed the leaderboards and the heatmap's success colour.
-- Practice lives in its own table so it can count as *activity* on the profile
-- while staying structurally incapable of polluting the scored stats: there is
-- no score column to be misread, and nothing here is ever handed to the
-- leaderboard sink.
--
-- Idempotency mirrors the scored path: the id is the CLIENT-generated UUID v7,
-- the app captures the practice in the same durable outbox and retries
-- `RecordPractice` until acked, so `INSERT ... ON CONFLICT (id) DO NOTHING`
-- makes a resent id a no-op (at-least-once + idempotent = effectively-once).
-- A looping practice session is recorded ONCE on stop, not per lap, so the
-- per-day count reflects sessions rather than repetitions.
--
-- `user_id` is a PLAIN uuid — no cross-schema FK to the user module (module-role
-- isolation, like play_sessions). Account deletion erases these rows explicitly
-- in the `purge_user` worker job.
--
-- `practiced_at` is the wall-clock instant the practice ended and
-- `tz_offset_minutes` the client's UTC offset AT that instant, so the per-day
-- grid buckets by the player's LOCAL day, exactly like the scored path.
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path.

CREATE TABLE IF NOT EXISTS music.practice_sessions (
    id                UUID PRIMARY KEY,               -- client-generated UUID v7 (the session id)
    user_id           UUID        NOT NULL,           -- caller's AuthIdentity.user_id; no cross-schema FK
    score_id          TEXT,                           -- catalog/user score identity (opaque; nullable)
    practiced_at      TIMESTAMPTZ NOT NULL,           -- when the practice session ended (client wall clock)
    tz_offset_minutes INTEGER     NOT NULL DEFAULT 0, -- client UTC offset at practiced_at (local-day bucketing)
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Every read is user-scoped and time-ordered (per-day aggregate, activity window).
CREATE INDEX IF NOT EXISTS practice_sessions_user_practiced_idx
    ON music.practice_sessions (user_id, practiced_at);
