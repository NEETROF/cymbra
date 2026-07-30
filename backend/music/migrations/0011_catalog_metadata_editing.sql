-- music module — moderator/admin editing of catalog curatorial metadata
-- (change: add-catalog-metadata-editing).
--
-- Lets a moderator/admin correct a public-corpus score's CURATORIAL fields
-- (title/composer/arranger/level) instead of only accept/reject. Two additions:
--
--   1. `catalog_edits` — an append-only audit trail (one row per changed field),
--      mirroring `role_grants` and the moderation `reviewed_by` traceability, so
--      "who changed which field of which score, and when" is always answerable.
--   2. `edited_by` / `edited_at` on `catalog_scores` — a manual-edit provenance
--      marker. Any future automated metadata refresh (e.g. the title backfill)
--      MUST skip rows where it is set, so curated metadata is never clobbered.
--
-- The edit itself (in pg.rs) also recomputes the DERIVED search keys
-- (`title_norm`, `composer_norm`, `work_key`) via the shared `normalize_text`, so
-- the trigram search + same-work grouping follow the edit — the SQL there, not here.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

-- Provenance marker on the corpus row (NULL until a manual edit happens).
ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS edited_by UUID,          -- last manual editor's users.id
    ADD COLUMN IF NOT EXISTS edited_at TIMESTAMPTZ;   -- when last manually edited

-- Append-only audit trail: one row per changed field. Never UPDATEd/DELETEd (the
-- current metadata is the source of truth on the score row; this is history).
CREATE TABLE IF NOT EXISTS music.catalog_edits (
    id                BIGSERIAL   PRIMARY KEY,
    catalog_score_id  UUID        NOT NULL REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    editor            UUID        NOT NULL,           -- moderator/admin users.id
    field             TEXT        NOT NULL
        CHECK (field IN ('title', 'composer', 'arranger', 'level')),
    old_value         TEXT,                            -- value before the edit (NULL allowed)
    new_value         TEXT,                            -- value after the edit (NULL allowed)
    edited_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Query the edit history of a score, newest first.
CREATE INDEX IF NOT EXISTS catalog_edits_score_idx
    ON music.catalog_edits (catalog_score_id, edited_at DESC);
