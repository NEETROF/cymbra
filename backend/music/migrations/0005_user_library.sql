-- music module — per-user saved-catalog library (change: score-hub-search).
--
-- user_library records which public catalog scores a signed-in user has pinned
-- to their personal library from the Score Hub. One owner-scoped row per save;
-- the saved set is the backend source of truth, so it syncs across the account's
-- devices. Removing a save deletes ONLY the row here — never the catalog entry.
--
-- owner_id is a PLAIN uuid (no cross-schema FK to the user module — module-role
-- isolation, exactly as user_scores/catalog_scores avoid cross-schema FKs).
-- catalog_id DOES FK to catalog_scores: both live in the `music` schema, so a
-- real FK is allowed and gives referential cleanup (a purged catalog row drops
-- the dangling saves via ON DELETE CASCADE). A crawler re-ingest can still change
-- ids, so the read path joins to catalog_scores and omits saves whose entry is
-- gone — the FK is a safety net, not the whole story.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

CREATE TABLE IF NOT EXISTS music.user_library (
    owner_id   UUID        NOT NULL,   -- caller's AuthIdentity.user_id; no cross-schema FK
    catalog_id UUID        NOT NULL
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (owner_id, catalog_id)   -- idempotent save; one row per (owner, score)
);

-- Every read is owner-scoped and newest-saved-first.
CREATE INDEX IF NOT EXISTS user_library_owner_idx
    ON music.user_library (owner_id, created_at DESC);
