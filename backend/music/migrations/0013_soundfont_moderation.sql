-- music module — SoundFont moderation + private per-user library
-- (change: add-soundfont-moderation).
--
-- Brings SoundFonts to parity with score moderation and splits user imports from the
-- curated public catalog:
--   1. `music.soundfonts` gains a `pending`/`accepted`/`rejected` lifecycle with
--      reviewer attribution (`reviewed_by`/`reviewed_at`), the uploader (`uploaded_by`),
--      and an exact-byte content digest (`content_sha256`) for identical-content
--      detection across uploads. Only `accepted` fonts are publicly visible; the
--      read-side gate lives in the repo/delivery layer.
--   2. `music.user_soundfonts` is a NEW private, per-user store (owner-only, unmoderated,
--      server-backed so it syncs across the owner's devices).
--
-- `uploaded_by` / `user_id` are PLAIN uuids (the caller's AuthIdentity.user_id) — no
-- cross-schema FK to the user module, matching the isolation of the other music tables
-- (see 0010_play_sessions.sql).
--
-- Backfill: the bundled default `upright-piano-kw` (also shipped inside the app) becomes
-- `accepted` so it stays available out of the box; every other pre-existing catalog row
-- becomes `pending` and drops out of the public listing until a moderator validates it.
--
-- Idempotent DDL + fully-qualified names, matching the existing migrations.

-- 1. Moderation lifecycle on the public catalog ----------------------------------
ALTER TABLE music.soundfonts
    ADD COLUMN IF NOT EXISTS moderation_status TEXT NOT NULL DEFAULT 'pending'
        CHECK (moderation_status IN ('pending', 'accepted', 'rejected')),
    ADD COLUMN IF NOT EXISTS reviewed_by    UUID,          -- reviewer's users.id; NULL while pending
    ADD COLUMN IF NOT EXISTS reviewed_at    TIMESTAMPTZ,   -- when the status was last set; NULL while pending
    ADD COLUMN IF NOT EXISTS uploaded_by    UUID,          -- who contributed the font; NULL for pre-existing seeds
    ADD COLUMN IF NOT EXISTS content_sha256 TEXT;          -- exact-byte digest for dedup; NULL until (re)hashed

-- Backs the public hot path (`WHERE moderation_status = 'accepted'`) on every listing.
CREATE INDEX IF NOT EXISTS soundfonts_moderation_status_idx
    ON music.soundfonts (moderation_status);

-- Content-identity lookup on upload/propose (dedup against existing fonts).
CREATE INDEX IF NOT EXISTS soundfonts_content_sha256_idx
    ON music.soundfonts (content_sha256);

-- Seed statuses: the bundled default is pre-accepted; the ADD COLUMN default already
-- put every other existing row at `pending`, so only the default needs an explicit set.
UPDATE music.soundfonts SET moderation_status = 'accepted'
    WHERE id = 'upright-piano-kw';

-- 2. Private, per-user soundfont library -----------------------------------------
CREATE TABLE IF NOT EXISTS music.user_soundfonts (
    id             UUID        PRIMARY KEY,                -- server-generated row id
    user_id        UUID        NOT NULL,                   -- owner's AuthIdentity.user_id; no cross-schema FK
    label          TEXT        NOT NULL,                   -- display name
    object_key     TEXT        NOT NULL,                   -- key in the private bucket (per-user prefix)
    content_sha256 TEXT        NOT NULL,                   -- exact-byte digest; idempotent re-import guard
    size_bytes     BIGINT      NOT NULL,
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Owner-scoped listing.
CREATE INDEX IF NOT EXISTS user_soundfonts_user_id_idx
    ON music.user_soundfonts (user_id);

-- One copy of a given content per user (backs the idempotent re-import).
CREATE UNIQUE INDEX IF NOT EXISTS user_soundfonts_user_content_uidx
    ON music.user_soundfonts (user_id, content_sha256);
