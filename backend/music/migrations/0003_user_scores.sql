-- music module — authenticated user uploads (change: add-user-score-upload).
--
-- user_scores holds a signed-in user's own contributed scores. It mirrors
-- catalog_scores' column names/types (same schema, so the two read alike), but
-- carries an owner and the rights attestation. All descriptive metadata is
-- DERIVED server-side from the parsed file (design 2b) — the client only chooses
-- the level and confirms the rights basis.
--
-- owner_id is a PLAIN uuid (no cross-schema FK to the user module — module-role
-- isolation, exactly as catalog_scores avoids cross-schema FKs). Ids are UUID v7
-- generated app-side. Idempotent DDL (CREATE ... IF NOT EXISTS, guarded indexes)
-- so a double-apply can never hard-crash; fully-qualified names so it works
-- regardless of the connecting role's search_path.

CREATE TABLE IF NOT EXISTS music.user_scores (
    id             UUID PRIMARY KEY,                  -- UUID v7, app-side
    owner_id       UUID        NOT NULL,              -- caller's AuthIdentity.user_id; no cross-schema FK
    -- user-owned inputs
    level          TEXT        NOT NULL CHECK (level IN ('beginner', 'intermediate', 'advanced')),
    level_source   TEXT        NOT NULL DEFAULT 'manual',
    rights_basis   TEXT        NOT NULL CHECK (rights_basis IN ('own_work', 'public_domain')),
    rights_ack     BOOLEAN     NOT NULL,              -- CGU confirmation; must be true to insert
    -- server-derived metadata (from the parse, never the client)
    title          TEXT,
    composer       TEXT,
    title_norm     TEXT,
    work_key       TEXT        NOT NULL DEFAULT '',
    key_fifths     INTEGER     NOT NULL DEFAULT 0,
    time_sig       TEXT        NOT NULL DEFAULT '',
    measure_count  INTEGER     NOT NULL DEFAULT 0,
    is_piano       BOOLEAN     NOT NULL DEFAULT FALSE,
    sha256         TEXT        NOT NULL,              -- content hash (per-owner dedup)
    size_bytes     BIGINT      NOT NULL DEFAULT 0,
    -- storage / lifecycle
    object_key     TEXT        NOT NULL,              -- user-scores/{owner_id}/{uuid}.mxl
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Uniqueness as guarded indexes (not inline UNIQUE) so a double-apply is safe.
CREATE UNIQUE INDEX IF NOT EXISTS user_scores_object_key_idx ON music.user_scores (object_key);
-- A user can't store the same file twice; two users may upload the same work.
CREATE UNIQUE INDEX IF NOT EXISTS user_scores_owner_sha_idx  ON music.user_scores (owner_id, sha256);
-- Every query is owner-scoped (list, quota count, owner-only delete).
CREATE INDEX IF NOT EXISTS user_scores_owner_idx ON music.user_scores (owner_id, created_at);
