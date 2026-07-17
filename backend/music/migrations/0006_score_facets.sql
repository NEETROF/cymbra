-- music module — musical facet columns (change: score-catalog-facets).
--
-- Derived purely at ingest by the shared ScoreSummary (smallest note value,
-- rhythmic/textural flags, ambitus, staff/note counts, tempo, dynamics), so the
-- hub can filter the catalog (and "mes partitions") by musical traits. All
-- nullable: a signal absent from a file stays NULL (never fabricated), and older
-- rows are backfilled in place by re-reading their stored objects (no re-crawl).
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

-- catalog_scores ------------------------------------------------------------
ALTER TABLE music.catalog_scores
    ADD COLUMN IF NOT EXISTS min_note_value SMALLINT,   -- power-of-two denominator (16 = sixteenth)
    ADD COLUMN IF NOT EXISTS has_tuplets    BOOLEAN,
    ADD COLUMN IF NOT EXISTS has_dotted     BOOLEAN,
    ADD COLUMN IF NOT EXISTS has_chords     BOOLEAN,
    ADD COLUMN IF NOT EXISTS lowest_midi    SMALLINT,
    ADD COLUMN IF NOT EXISTS highest_midi   SMALLINT,
    ADD COLUMN IF NOT EXISTS staff_count    SMALLINT,
    ADD COLUMN IF NOT EXISTS note_count     INTEGER,
    ADD COLUMN IF NOT EXISTS tempo_bpm      INTEGER,    -- marked per-minute; NULL when unmarked
    ADD COLUMN IF NOT EXISTS has_dynamics   BOOLEAN;

CREATE INDEX IF NOT EXISTS catalog_scores_min_note_value_idx ON music.catalog_scores (min_note_value);
CREATE INDEX IF NOT EXISTS catalog_scores_staff_count_idx    ON music.catalog_scores (staff_count);
CREATE INDEX IF NOT EXISTS catalog_scores_tempo_bpm_idx      ON music.catalog_scores (tempo_bpm);
CREATE INDEX IF NOT EXISTS catalog_scores_ambitus_idx        ON music.catalog_scores (lowest_midi, highest_midi);

-- user_scores (parity so "mes partitions" filters identically) ---------------
ALTER TABLE music.user_scores
    ADD COLUMN IF NOT EXISTS min_note_value SMALLINT,
    ADD COLUMN IF NOT EXISTS has_tuplets    BOOLEAN,
    ADD COLUMN IF NOT EXISTS has_dotted     BOOLEAN,
    ADD COLUMN IF NOT EXISTS has_chords     BOOLEAN,
    ADD COLUMN IF NOT EXISTS lowest_midi    SMALLINT,
    ADD COLUMN IF NOT EXISTS highest_midi   SMALLINT,
    ADD COLUMN IF NOT EXISTS staff_count    SMALLINT,
    ADD COLUMN IF NOT EXISTS note_count     INTEGER,
    ADD COLUMN IF NOT EXISTS tempo_bpm      INTEGER,
    ADD COLUMN IF NOT EXISTS has_dynamics   BOOLEAN;

CREATE INDEX IF NOT EXISTS user_scores_min_note_value_idx ON music.user_scores (min_note_value);
CREATE INDEX IF NOT EXISTS user_scores_staff_count_idx    ON music.user_scores (staff_count);
CREATE INDEX IF NOT EXISTS user_scores_tempo_bpm_idx      ON music.user_scores (tempo_bpm);
