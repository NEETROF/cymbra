-- music module — the freemium daily-access gate on catalog player-opens and the
-- score audio teaser marker (change: add-score-daily-access-rewards, task 1.3).
--
-- `catalog_day_access`: ONE row per (user, catalog piece, server day) = "this piece
-- is open for this user today". Rows are written idempotently on a served open
-- (a re-open is a no-op, so it never consumes quota twice) and `paid = TRUE` marks
-- a day-slot bought with points. The day is the SERVER (UTC) day — the same clock
-- as the play-award daily cap and for the same reason: a client-offset day would
-- hand out N more free opens per device-clock change (design D1). The free-used
-- count is derived (opened minus paid), so there is no counter to keep in sync.
--
-- Deliberately NOT a permanent grant (contrast `curation_grants`): the access dies
-- with the day; the points spend itself lives in `curation_points` (kind
-- `redeem`, reward_key `score_day_slot`, award_key `score_day_slot:<id>:<day>`,
-- whose unique partial index is the charge-once guard).
--
-- `user_id` is a PLAIN uuid — no cross-schema FK to the user module (module-role
-- isolation, like play_sessions/practice_streaks). Account deletion erases these
-- rows explicitly in the `purge_user` worker job; the piece FK cascades on score
-- deletion; a retention prune drops rows older than the window (only today's rows
-- are ever read).
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path.

CREATE TABLE IF NOT EXISTS music.catalog_day_access (
    user_id    UUID        NOT NULL,                       -- caller's AuthIdentity.user_id; no cross-schema FK
    catalog_id UUID        NOT NULL REFERENCES music.catalog_scores(id) ON DELETE CASCADE,
    day        DATE        NOT NULL,                       -- server (UTC) day
    paid       BOOLEAN     NOT NULL DEFAULT FALSE,         -- TRUE = a points day-slot, FALSE = a free open
    opened_at  TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, catalog_id, day)
);

-- Today's state for a user (opened set + paid set) is one indexed range read.
CREATE INDEX IF NOT EXISTS catalog_day_access_user_day_idx
    ON music.catalog_day_access (user_id, day);

-- The retention prune deletes by day across all users.
CREATE INDEX IF NOT EXISTS catalog_day_access_day_idx
    ON music.catalog_day_access (day);

-- The score audio teaser marker (design D7): stamped when the preview object
-- (`catalog-preview/<id>.wav`) was rendered and stored. The listing truth for
-- "has a sample" — a per-row storage probe (as the SoundFont listing does) does
-- not scale to the corpus. NULL = no preview yet (backfill / regenerate).
ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS preview_rendered_at TIMESTAMPTZ;

-- The backfill and the back-office "no sample" filter list accepted rows without a
-- marker; partial so it stays small once the corpus is rendered.
CREATE INDEX IF NOT EXISTS catalog_scores_missing_preview_idx
    ON music.catalog_scores (id)
    WHERE moderation_status = 'accepted' AND preview_rendered_at IS NULL;
