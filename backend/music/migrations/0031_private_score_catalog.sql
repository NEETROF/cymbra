-- music module — the private score catalog (change: add-private-score-catalog).
--
-- Three additions on top of `music.user_scores`, which already IS the private
-- per-user library (owner-scoped rows, per-owner sha256 dedup, objects in the
-- private bucket). Nothing here touches the public catalog.
--
-- 1. `private_use` rights basis. The attestation offered `own_work` and
--    `public_domain` only, so a user storing a score they legally bought or
--    transcribed FOR THEMSELVES had to misdeclare — an attestation everyone
--    violates protects no one. The third basis says "strictly personal use" and
--    is structurally unshareable: `propose` refuses it by reading THIS column,
--    never the proposal's own licence declaration (design D3). Existing rows are
--    untouched: both legacy bases stay proposable.
--
-- 2. Collections — a many-to-many tagging of one's own uploads. Two tables
--    rather than a `collection` column on user_scores: a score belongs to
--    several collections, and renaming/deleting a collection must not rewrite
--    score rows (design D4). Both FKs are IN-SCHEMA (the no-FK rule is about
--    CROSS-schema references to the user module, which `owner_id` still honours
--    as a plain uuid).
--
-- 3. `user_score_takedowns` — the audit trail of an admin removing a private
--    score on an illicit-content notice. Written BEFORE the row and object are
--    deleted, and deliberately outlives them: it carries the identifying
--    metadata (sha256, title) precisely because the score itself is gone
--    (design D5). It is the in-repo half of notice-and-takedown.
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path. New tables inherit music_svc's privileges
-- from ALTER DEFAULT PRIVILEGES (backend/deploy/provision-music-role.sql).

-- 1. Widen the rights basis. Drop-then-add is the only way to edit a CHECK; the
--    guarded drop makes the pair idempotent. The constraint name is the one
--    Postgres generated for the inline CHECK in 0003.
ALTER TABLE music.user_scores
    DROP CONSTRAINT IF EXISTS user_scores_rights_basis_check;
ALTER TABLE music.user_scores
    ADD CONSTRAINT user_scores_rights_basis_check
        CHECK (rights_basis IN ('own_work', 'public_domain', 'private_use'));

-- 2. Collections.
CREATE TABLE IF NOT EXISTS music.user_score_collections (
    id         UUID PRIMARY KEY,                    -- UUID v7, app-side
    owner_id   UUID        NOT NULL,                -- caller's AuthIdentity.user_id; no cross-schema FK
    name       TEXT        NOT NULL CHECK (length(btrim(name)) > 0),
    created_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- One "Chopin" per owner, whatever the casing they typed. Functional unique
-- index (not an inline UNIQUE) so the case-folding is part of the constraint and
-- a double-apply is safe.
CREATE UNIQUE INDEX IF NOT EXISTS user_score_collections_owner_name_idx
    ON music.user_score_collections (owner_id, lower(name));
-- Every listing is "my collections, newest first".
CREATE INDEX IF NOT EXISTS user_score_collections_owner_idx
    ON music.user_score_collections (owner_id, created_at);

CREATE TABLE IF NOT EXISTS music.user_score_collection_items (
    collection_id UUID        NOT NULL
        REFERENCES music.user_score_collections (id) ON DELETE CASCADE,
    user_score_id UUID        NOT NULL
        REFERENCES music.user_scores (id) ON DELETE CASCADE,
    added_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (collection_id, user_score_id)
);

-- The PK covers "what is in this collection"; this one covers the reverse read
-- ("which collections hold this score") and the cascade on score deletion.
CREATE INDEX IF NOT EXISTS user_score_collection_items_score_idx
    ON music.user_score_collection_items (user_score_id);

-- 3. Takedown audit. No FK to user_scores: the row it describes is deleted
--    moments later, which is the whole point — this table must survive it.
CREATE TABLE IF NOT EXISTS music.user_score_takedowns (
    id            UUID PRIMARY KEY,               -- UUID v7, app-side
    user_score_id UUID        NOT NULL,           -- the removed score (row is gone)
    owner_id      UUID        NOT NULL,           -- who had uploaded it
    admin_id      UUID        NOT NULL,           -- who removed it
    sha256        TEXT        NOT NULL,           -- identifies the content after deletion
    title         TEXT,                           -- may be NULL, like user_scores.title
    reason        TEXT        NOT NULL CHECK (length(btrim(reason)) > 0),
    created_at    TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Answering "was this content ever taken down / what did we do about this owner"
-- without scanning the table.
CREATE INDEX IF NOT EXISTS user_score_takedowns_sha_idx
    ON music.user_score_takedowns (sha256);
CREATE INDEX IF NOT EXISTS user_score_takedowns_owner_idx
    ON music.user_score_takedowns (owner_id, created_at);
