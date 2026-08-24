-- The stored instrument family replacing the is_piano staff-count proxy
-- (change: add-drums-access).
--
-- Additive only: the tables hold an object_key, not the score bytes, so no SQL
-- can classify a row — the real values come from the application-level
-- re-derivation pass (`backfill-instruments`), which parses each stored object
-- with the shared classifier. Until that pass has run, every row reads
-- 'unknown', and the drum gate is NOT a boundary (it withholds only rows
-- recorded 'percussion'). `is_piano` is dropped in a later migration once no
-- reader remains.
--
-- NOT NULL by construction: the column never holds NULL, so the gate and the
-- instrument filter test exactly one undeterminable value ('unknown').

ALTER TABLE music.catalog_scores
    ADD COLUMN instrument TEXT NOT NULL DEFAULT 'unknown'
        CHECK (instrument IN ('keyboard', 'percussion', 'unknown'));

ALTER TABLE music.user_scores
    ADD COLUMN instrument TEXT NOT NULL DEFAULT 'unknown'
        CHECK (instrument IN ('keyboard', 'percussion', 'unknown'));

-- The search predicate and the drum gate both filter catalog rows by
-- instrument on every catalog read; the existing search indexes are all
-- single-purpose btrees on low-cardinality columns (level, source), so a
-- matching one is consistent with the table's index shape.
CREATE INDEX idx_catalog_scores_instrument ON music.catalog_scores (instrument);
