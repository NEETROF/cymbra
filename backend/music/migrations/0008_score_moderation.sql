-- music module — score moderation gating (change: add-score-moderation-gating).
--
-- Introduces the moderation lifecycle on the public catalog: a `moderation_status`
-- (`pending` | `accepted` | `rejected`, default `pending`) plus a review audit trail
-- (`reviewed_by` / `reviewed_at`). Only `accepted` scores are publicly visible; the
-- read-side gate lives in the search/fetch SQL (see `pg.rs`). `reviewed_by`/`_at`
-- stay NULL while `pending`; a later accept/reject decision is traceable to a
-- specific reviewer and time.
--
-- BREAKING (data): adding the column with `NOT NULL DEFAULT 'pending'` to the
-- already-populated `catalog_scores` backfills EVERY existing row to `pending`, so
-- the hub is empty until scores are validated one by one (change #3's back office).
-- This is intentional — no bulk-accept of the existing corpus, and licensing
-- `confidence` is NOT editorial validation, so no subset is pre-accepted.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS moderation_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (moderation_status IN ('pending', 'accepted', 'rejected')),
    ADD COLUMN IF NOT EXISTS reviewed_by UUID,          -- moderator's users.id; NULL while pending
    ADD COLUMN IF NOT EXISTS reviewed_at TIMESTAMPTZ;   -- when the status was last set; NULL while pending

-- Backs the hub's hot path (`WHERE moderation_status = 'accepted'`) on every search.
CREATE INDEX IF NOT EXISTS catalog_scores_moderation_status_idx
    ON music.catalog_scores (moderation_status);

-- The backfill to `pending` is implicit: `ADD COLUMN ... NOT NULL DEFAULT 'pending'`
-- fills every pre-existing row with the default. No explicit UPDATE is needed, and
-- deliberately no subset is pre-accepted (each score is reviewed individually).
