-- music module — the GLOBAL leaderboard's per-season stores (change:
-- add-global-leaderboard, tasks 1.1 + 2.2, design D3/D4).
--
-- Two tables:
--
-- 1. `global_season_bests` — one row per (player, season, accepted-catalog piece,
--    mode) holding that player's BEST per-mode sub-score achieved **within that
--    season**. The global season score is a difficulty-weighted best-N aggregation
--    over these rows (design D1/D4), so the read stays a straightforward
--    aggregation and never has to touch the raw `play_sessions` (whose heavy
--    detail is pruned at 90 days).
--
--    Maintained on the SAME ingest path as the per-piece bests (the #6
--    `LeaderboardSink` hook) by a MONOTONIC upsert: the row is raised only when a
--    new result is strictly better (higher sub-score; ties broken by an earlier
--    `achieved_at`). That makes the update idempotent under #5's at-least-once
--    ingest — a replayed or duplicate session can never lower a season best or
--    create a duplicate (PRIMARY KEY (user_id, season_id, catalog_score_id, mode)).
--
--    The season a result belongs to is derived from WHEN it was achieved (UTC
--    fixed-length windows, design D3), so a late delivery lands in the right
--    season and the bucketing is itself idempotent.
--
-- 2. `global_season_snapshots` — the end-of-season hall of fame (design D3). At
--    season rollover the worker folds each mode's final per-user aggregate
--    (weighted score, contributing pieces, when it was reached) into one row per
--    player, which later seasons never overwrite. It stores the RAW aggregate and
--    NOT a rank, so the ranking is recomputed on read.
--
--    `was_listable` freezes the player's CONSENT to be listed as it stood when the
--    season closed. That keeps a past season stable: someone who was public then
--    does not silently vanish from the archive — and shuffle everyone else's rank
--    up — merely because they went private afterwards. The **age** safeguard is
--    deliberately NOT frozen with it: the archive read is
--    `was_listable AND age_eligible_now`, so a player who must not be listed at all
--    disappears from past seasons too and cannot be pinned there by a stale
--    snapshot. A caller always sees their OWN past standing regardless.
--
-- `user_id` is a PLAIN uuid — NO cross-schema FK to the user module. Module-role
-- isolation forbids one (the `music` role cannot reference `user_account`), which
-- is why `play_sessions`, `leaderboard_bests`, `score_ratings` and `user_scores`
-- all do the same. Account deletion therefore erases these rows EXPLICITLY in the
-- `purge_user` worker job (task 3.3) instead of relying on a cascade; task 1.1's
-- "FK to users ON DELETE CASCADE" is not expressible here.
--
-- `catalog_score_id` DOES FK to `catalog_scores` (same `music` schema), so a
-- purged catalog piece drops its season rows via ON DELETE CASCADE, and only the
-- `accepted` pieces the ingest path admits ever get a row.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

CREATE TABLE IF NOT EXISTS music.global_season_bests (
    user_id          UUID        NOT NULL,   -- caller's AuthIdentity.user_id; no cross-schema FK
    season_id        TEXT        NOT NULL,   -- UTC season window id (its start date, `YYYY-MM-DD`)
    catalog_score_id UUID        NOT NULL
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    mode             TEXT        NOT NULL CHECK (mode IN ('tempo', 'reaction')),
    best_subscore    REAL        NOT NULL,   -- best per-mode sync sub-score 0..100 IN THIS SEASON
    achieved_at      TIMESTAMPTZ NOT NULL,   -- when this season best was achieved (tie-break input)
    PRIMARY KEY (user_id, season_id, catalog_score_id, mode)
);

-- The global board aggregates one (season, mode) across every player; back that
-- hot path with a composite index on the aggregation key.
CREATE INDEX IF NOT EXISTS global_season_bests_board_idx
    ON music.global_season_bests (season_id, mode);

CREATE TABLE IF NOT EXISTS music.global_season_snapshots (
    season_id           TEXT        NOT NULL,
    mode                TEXT        NOT NULL CHECK (mode IN ('tempo', 'reaction')),
    user_id             UUID        NOT NULL,  -- no cross-schema FK (see above)
    global_score        REAL        NOT NULL,  -- difficulty-weighted best-N total (rank primary, desc)
    contributing_pieces INT         NOT NULL,  -- how many pieces fed it (first tie-break, desc)
    reached_at          TIMESTAMPTZ NOT NULL,  -- when the final score was reached (last tie-break, asc)
    -- Consent to be listed, FROZEN when the season closed (see above). The age
    -- safeguard stays live and is re-checked on read.
    was_listable        BOOLEAN     NOT NULL DEFAULT FALSE,
    PRIMARY KEY (season_id, mode, user_id)     -- one final standing per (season, mode, player)
);

-- Reading a past season loads one (season, mode) whole, then ranks + gates it.
CREATE INDEX IF NOT EXISTS global_season_snapshots_board_idx
    ON music.global_season_snapshots (season_id, mode);
