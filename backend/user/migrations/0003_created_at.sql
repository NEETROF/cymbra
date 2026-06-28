-- Account creation timestamp (change: fix-handle-onboarding-escape).
--
-- Needed by the orphan reaper: handle-less accounts created longer ago than a
-- grace period are purged (covers hard app-kills during handle onboarding where
-- the client cannot delete the just-provisioned account). Existing rows default
-- to now() at migration time, so they get a fresh grace window.

ALTER TABLE users ADD COLUMN created_at TIMESTAMPTZ NOT NULL DEFAULT now();

CREATE INDEX users_orphan_idx ON users (created_at) WHERE handle IS NULL;
