-- Musical content fingerprint (change: add-score-crawler) — dedup that survives
-- re-encoding. The exact-content dedup key is `sha256` (UNIQUE); this fingerprint
-- hashes the notes themselves, so the same piece exported by a different editor
-- (or found on another site) is detectable. Non-unique index: it is used to
-- skip near-duplicates at ingest and to find them for curation, but it is NOT a
-- hard constraint (different works can legitimately share a tune).

ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS content_fingerprint TEXT NOT NULL DEFAULT '';

CREATE INDEX IF NOT EXISTS catalog_scores_fingerprint_idx
    ON music.catalog_scores (content_fingerprint);
