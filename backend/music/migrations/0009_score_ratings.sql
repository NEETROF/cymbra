-- music module — per-user score ratings (change: add-app-score-rating).
--
-- score_ratings records a signed-in user's rating of a public catalog score: a
-- swipe `verdict` (dislike | like | love) and/or an explicit 1–5 `stars` value.
-- There is AT MOST ONE rating per (user, score): re-rating upserts the row
-- (PRIMARY KEY (user_id, catalog_score_id)), so the per-score aggregate never
-- double-counts a user. Only `accepted` scores are rateable — the gate lives in
-- the module (a rating targets a score resolved through the accepted-only path),
-- exactly as save/fetch do (change: add-score-moderation-gating).
--
-- The aggregate (average effective value + count + verdict breakdown) and the
-- hybrid re-review flag are computed ON DEMAND from this table (design D3/D4): no
-- denormalised column on `catalog_scores` and no `needs_review` column — the
-- corpus/rating volume is small, so a grouped query indexed by `catalog_score_id`
-- is enough. A denormalised column / materialized view can be added later if
-- ranking-by-rating becomes a hot path, without a data migration.
--
-- user_id is a PLAIN uuid (no cross-schema FK to the user module — module-role
-- isolation, exactly as user_library/user_scores). catalog_score_id DOES FK to
-- catalog_scores (same `music` schema), so a purged catalog row drops its ratings
-- via ON DELETE CASCADE.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

CREATE TABLE IF NOT EXISTS music.score_ratings (
    user_id          UUID        NOT NULL,   -- caller's AuthIdentity.user_id; no cross-schema FK
    catalog_score_id UUID        NOT NULL
        REFERENCES music.catalog_scores (id) ON DELETE CASCADE,
    verdict          TEXT        NOT NULL CHECK (verdict IN ('dislike', 'like', 'love')),
    stars            SMALLINT    CHECK (stars IS NULL OR stars BETWEEN 1 AND 5),
    updated_at       TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, catalog_score_id)  -- one updatable rating per (user, score)
);

-- The per-score aggregate groups by the score; back that hot path (and the FK's
-- cascade lookups) with an index on the referencing column.
CREATE INDEX IF NOT EXISTS score_ratings_catalog_score_idx
    ON music.score_ratings (catalog_score_id);
