-- music module — favorite flag on user uploads (change: favorites-home).
--
-- The signed-in home screen shows the user's FAVORITES: saved catalog scores
-- (music.user_library) plus their own uploads that are favorited. An upload is
-- auto-favorited on creation (DEFAULT true); un-favoriting hides it from the home
-- but keeps it in the hub's "mes partitions" (and never deletes it).
--
-- Idempotent + fully-qualified, matching the existing migrations.

ALTER TABLE music.user_scores
    ADD COLUMN IF NOT EXISTS favorite BOOLEAN NOT NULL DEFAULT TRUE;

-- The home lists the caller's favorited uploads, newest first.
CREATE INDEX IF NOT EXISTS user_scores_owner_favorite_idx
    ON music.user_scores (owner_id, favorite, created_at DESC);
