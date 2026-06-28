-- Unique account handle (change: add-music-account-access, task 1.1).
--
-- `handle` stores the display form exactly as the user entered it; `handle_key`
-- stores the app-computed normalized form (Unicode NFC + case-fold, see
-- `handle_core::normalize`). Uniqueness is enforced **case-insensitively** on the
-- key. Both are nullable: accounts created before onboarding (and OIDC auto-
-- provisioned accounts) have no handle until the user picks one. A unique index
-- permits many NULL keys, so unclaimed accounts never collide.

ALTER TABLE users ADD COLUMN handle     TEXT;
ALTER TABLE users ADD COLUMN handle_key TEXT;

CREATE UNIQUE INDEX users_handle_key_uniq ON users (handle_key);
