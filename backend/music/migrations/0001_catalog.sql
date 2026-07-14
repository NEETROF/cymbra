-- music module — the public score catalog (change: add-score-crawler).
--
-- catalog_scores holds the crawler-ingested, redistributable corpus: one row per
-- retained file, carrying the exact provenance needed to re-publish it plus the
-- search/musical metadata captured at ingest. It has NO owner (public corpus)
-- and NO cross-schema FK; user uploads (user_scores) are added to this schema
-- later by the user-upload change.
--
-- Fully-qualified names so the migration works whether or not the connecting
-- role's search_path is pinned to `music`.

CREATE SCHEMA IF NOT EXISTS music;

CREATE TABLE IF NOT EXISTS music.catalog_scores (
    id                UUID PRIMARY KEY,               -- UUID v7, app-side
    -- provenance / attribution
    title             TEXT,
    composer          TEXT,
    arranger          TEXT,
    source            TEXT        NOT NULL,
    source_url        TEXT        NOT NULL,
    source_item_id    TEXT        NOT NULL,
    license           TEXT        NOT NULL,           -- normalised code, e.g. CC-BY-4.0
    license_url       TEXT,
    confidence        TEXT        NOT NULL CHECK (confidence IN ('verified', 'unverified')),
    sha256            TEXT        NOT NULL UNIQUE,     -- content hash (dedup)
    origin_format     TEXT        NOT NULL
        CHECK (origin_format IN ('music_xml', 'mxl', 'muse_score', 'lily_pond', 'mei')),
    conversion_status TEXT        NOT NULL
        CHECK (conversion_status IN ('converted', 'failed_kept_source', 'failed', 'skipped')),
    object_key        TEXT        NOT NULL,           -- object-store key of the .mxl
    size_bytes        BIGINT      NOT NULL DEFAULT 0,
    -- search / musical metadata (captured at ingest)
    work_key          TEXT        NOT NULL,           -- normalised composer+title (dedup grouping)
    title_norm        TEXT,                            -- accent/case-folded (search)
    is_piano          BOOLEAN     NOT NULL DEFAULT FALSE,
    key_fifths        INTEGER     NOT NULL DEFAULT 0,
    time_sig          TEXT        NOT NULL DEFAULT '',
    measure_count     INTEGER     NOT NULL DEFAULT 0,
    language          TEXT,
    voicing           TEXT,
    level             TEXT        CHECK (level IS NULL OR level IN ('beginner', 'intermediate', 'advanced')),
    level_source      TEXT        CHECK (level_source IS NULL OR level_source IN ('source', 'heuristic', 'manual')),
    metadata          JSONB       NOT NULL DEFAULT '{}',
    created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Facet / filter indexes for the future search API. The fuzzy/full-text
-- (pg_trgm GIN on title_norm/composer) index is deferred to that change and must
-- enable the extension from an admin/ops migration, not this module role.
CREATE INDEX IF NOT EXISTS catalog_scores_source_idx     ON music.catalog_scores (source);
CREATE INDEX IF NOT EXISTS catalog_scores_license_idx    ON music.catalog_scores (license);
CREATE INDEX IF NOT EXISTS catalog_scores_confidence_idx ON music.catalog_scores (confidence);
CREATE INDEX IF NOT EXISTS catalog_scores_level_idx      ON music.catalog_scores (level);
CREATE INDEX IF NOT EXISTS catalog_scores_work_key_idx   ON music.catalog_scores (work_key);
