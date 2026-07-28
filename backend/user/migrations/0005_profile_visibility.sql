-- user module — public-profile visibility + minor safeguard (change:
-- add-play-activity-profile, tasks 4.1/4.2, design D5/D6).
--
-- `profile_visibility` controls who may see the account's profile. It is
-- **private by default** (opt-in sharing, RGPD-aligned, protects minors): a
-- profile is exposed to other players only after the user explicitly opts in.
--
-- `share_eligible_from` is the ONLY age-derived datum kept: the date on/after
-- which the user is old enough to make the profile public. It is computed
-- server-side at opt-in as `date_of_birth + min_public_sharing_age years`; the
-- date of birth itself is entered once and DISCARDED (data minimization) — this
-- column is never the DOB. NULL means "age never established" (cannot go public).
--
-- Additive + idempotent so a double-apply is safe.

ALTER TABLE users ADD COLUMN profile_visibility  TEXT NOT NULL DEFAULT 'private'
    CHECK (profile_visibility IN ('private', 'limited', 'public'));
ALTER TABLE users ADD COLUMN share_eligible_from DATE;
