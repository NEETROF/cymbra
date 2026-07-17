-- music module — catalog full-text search (change: score-hub-search).
--
-- Realises the fuzzy/full-text search the 0001 catalog migration deferred: a
-- trigram GIN index over the normalised title AND a normalised composer, so the
-- hub's search-as-you-type (title OR composer, typo/accent tolerant) is
-- index-backed rather than a full-table scan.
--
-- The `pg_trgm`/`unaccent` extensions are enabled by OPS (roles.sql.tpl /
-- provision-music-role.sql), NOT here: CREATE EXTENSION needs superuser, which the
-- least-privilege music_svc role lacks (same reason CREATE SCHEMA is done by ops).
-- This migration only creates the column + index that USE them.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

-- composer had no normalised column (only title_norm existed). Add an
-- accent/case-folded composer key so the trigram index can match composer
-- fragments the same way it matches the title.
ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS composer_norm TEXT;

-- Backfill existing rows to parity with the crawler's Rust `normalize_text`
-- (NFD accent-strip + lowercase + whitespace-collapse). `unaccent` covers the
-- accent-strip; `lower` + the whitespace regex cover the rest. The crawler
-- populates composer_norm directly for rows ingested from now on.
UPDATE music.catalog_scores
   SET composer_norm =
         regexp_replace(btrim(lower(unaccent(composer))), '\s+', ' ', 'g')
 WHERE composer IS NOT NULL
   AND composer_norm IS NULL;

-- Trigram GIN indexes: substring/similarity matching on the normalised search
-- columns. `gin_trgm_ops` resolves via music_svc's pinned `search_path = music`
-- (pg_trgm is installed into the `music` schema).
CREATE INDEX IF NOT EXISTS catalog_scores_title_norm_trgm_idx
    ON music.catalog_scores USING gin (title_norm gin_trgm_ops);
CREATE INDEX IF NOT EXISTS catalog_scores_composer_norm_trgm_idx
    ON music.catalog_scores USING gin (composer_norm gin_trgm_ops);
