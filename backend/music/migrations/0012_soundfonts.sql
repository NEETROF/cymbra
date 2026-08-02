-- music module — persisted SoundFont catalog (change: add-soundfont-catalog-db).
--
-- Moves the SoundFont catalog from a hardcoded Rust constant into the database so
-- it is a single source of truth: the delivery route (GET /soundfonts/{id}) and the
-- ListSoundFonts RPC both resolve fonts through this table, and adding a font is a
-- data change (insert a row + upload its object) — not a code change.
--
-- `object_key` is the storage key inside the private SoundFont bucket; `instrument`
-- is the instrument family the font is for (piano, and later guitar/drums/…), which
-- correlates a font to the matching instrument scores. `license`/`attribution`
-- record the required credit for redistributed (e.g. CC-BY) fonts.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

CREATE TABLE IF NOT EXISTS music.soundfonts (
    id           TEXT        PRIMARY KEY,                                   -- client-facing id
    label        TEXT        NOT NULL,                                      -- display name
    object_key   TEXT        NOT NULL,                                      -- key in the private bucket
    instrument   TEXT        NOT NULL DEFAULT 'piano',                      -- instrument family
    license      TEXT        NOT NULL,
    attribution  TEXT,                                                      -- NULL when none required
    size_bytes   BIGINT,                                                    -- informational; may be NULL
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Seed the bundled CC0 default so the catalog is populated out of the box. The app
-- also bundles this font, so it filters this id out of the download list; the row
-- exists so the delivery route (and the back-office preview) can serve it by id.
INSERT INTO music.soundfonts (id, label, object_key, instrument, license, attribution)
VALUES (
    'upright-piano-kw',
    'Upright Piano KW',
    'UprightPianoKW-20220221.sf2',
    'piano',
    'CC0-1.0',
    NULL
)
ON CONFLICT (id) DO NOTHING;
