-- music module — per-(player, piece, mode) leaderboard personal bests (change:
-- add-play-leaderboards, task 1.1, design D3).
--
-- One row per (user, accepted-catalog piece, mode) holding that player's BEST
-- result on that board: the highest per-mode synchronization sub-score, its
-- tie-break metric, and when it was achieved. Boards read from HERE (not from the
-- raw `play_sessions`) so a best survives the 90-day detail prune (design D3): it
-- is its own durable summary.
--
-- Maintained on session ingest by a MONOTONIC upsert (the `RecordPlaySession`
-- path): the row is raised only when a new result is strictly better under the
-- ranking order (higher sub-score; ties broken by a smaller tie-break metric, then
-- an earlier `achieved_at`). That makes the update idempotent under #5's
-- at-least-once ingest — a replayed or duplicate session can never lower a best or
-- create a duplicate (PRIMARY KEY (user_id, catalog_score_id, mode)).
--
-- `tiebreak_metric` is normalised so SMALLER is always better, whatever the board:
-- the tempo board stores |mean free-run tempo offset| (tighter timing) and the
-- reaction board stores the mean Wait-Mode reaction time (faster). A candidate
-- whose mode had a sub-score but no timing hits stores a large sentinel, so it
-- sorts last on ties without special-casing the read.
--
-- `mode` is `tempo` (free-run sub-score) or `reaction` (Wait-Mode sub-score); a
-- `mixed` run feeds both boards via its two sub-scores, a pure run feeds one.
--
-- user_id is a PLAIN uuid (no cross-schema FK to the user module — module-role
-- isolation, exactly as score_ratings/user_scores/play_sessions). Account deletion
-- erases these rows explicitly in the `purge_user` worker job (no cross-schema
-- cascade is possible; task 1.3). catalog_score_id DOES FK to catalog_scores (same
-- `music` schema), so a purged catalog piece drops its board via ON DELETE CASCADE
-- and only `accepted` pieces (the ones the ingest path admits) ever get a row.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

CREATE TABLE IF NOT EXISTS music.leaderboard_bests (
    user_id          UUID        NOT NULL,   -- caller's AuthIdentity.user_id; no cross-schema FK
    catalog_score_id UUID        NOT NULL
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    mode             TEXT        NOT NULL CHECK (mode IN ('tempo', 'reaction')),
    best_subscore    REAL        NOT NULL,   -- highest per-mode sync sub-score 0..100 (rank primary, desc)
    tiebreak_metric  REAL        NOT NULL,   -- normalised so SMALLER = better (tie-break, asc)
    achieved_at      TIMESTAMPTZ NOT NULL,   -- when this best was achieved (final tie-break, earliest wins)
    PRIMARY KEY (user_id, catalog_score_id, mode)  -- one best per (player, piece, mode)
);

-- The board read ranks one (piece, mode) by sub-score desc; back that hot path
-- (and the FK's cascade lookups) with a composite index in ranking order.
CREATE INDEX IF NOT EXISTS leaderboard_bests_board_idx
    ON music.leaderboard_bests (catalog_score_id, mode, best_subscore DESC);
