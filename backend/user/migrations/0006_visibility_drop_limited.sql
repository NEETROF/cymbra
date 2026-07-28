-- user module — drop the inert `limited` visibility tier (change:
-- add-play-activity-profile follow-up).
--
-- `limited` was reserved for a future followers-only tier but never had distinct
-- behavior (reads treat anything but `public` as hidden), so it was surfaced as an
-- unexplained third option and left the API accepting a no-op value. We remove it:
-- Private / Public only. A real followers tier can return later with its own
-- semantics + migration.
--
-- Order matters: migrate any existing `limited` rows to `private` BEFORE
-- tightening the CHECK, otherwise the constraint add would fail on legacy data
-- (and the app could no longer parse such a row). Idempotent + safe to re-run.

UPDATE users SET profile_visibility = 'private' WHERE profile_visibility = 'limited';

ALTER TABLE users DROP CONSTRAINT IF EXISTS users_profile_visibility_check;
ALTER TABLE users ADD CONSTRAINT users_profile_visibility_check
    CHECK (profile_visibility IN ('private', 'public'));
