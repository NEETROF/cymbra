-- push-notification platform — device-token registry, per-category consent and
-- the per-user timezone (change: add-push-notifications, tasks 1.1, design D2/D3/D4).
--
-- These live in the `user_account` schema (owned by `user_svc`) rather than a
-- schema of their own: a device token, a category preference and a timezone are
-- all *account* data whose lifetime is exactly the account's. Keeping them here
-- gives a real FK to `user_account.users`, so account erasure cascades them away
-- for free (RGPD) with no cross-schema cleanup job — the same reason `locale`
-- (0007) is a column on `users`. The `cymbra-notifications` crate reads/writes
-- them with fully-qualified names, so it works from any pool (the server's
-- `user_svc`, the worker's `admin_svc`).

-- Device tokens (design D2). `token` is the PRIMARY KEY: an FCM token identifies
-- one app install, so re-registering the same token after a device changes hands
-- must *move* it to the new user, not duplicate it. One user may hold several
-- (one per device). `platform` is the FCM-capable platform that produced it
-- (ios | android | macos) — Windows/Linux clients never register.
CREATE TABLE IF NOT EXISTS user_account.push_tokens (
    token        TEXT        PRIMARY KEY,
    user_id      UUID        NOT NULL REFERENCES user_account.users(id) ON DELETE CASCADE,
    platform     TEXT        NOT NULL,
    created_at   TIMESTAMPTZ NOT NULL DEFAULT now(),
    last_seen_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The dispatch path loads every candidate token for a user set, so the by-user
-- lookup is the hot one.
CREATE INDEX IF NOT EXISTS push_tokens_user_idx ON user_account.push_tokens (user_id);

-- Per-user, per-category consent (design D4). A row exists only once the user has
-- *expressed* a choice; an absent row means "no explicit choice", and the
-- category's own product default applies (the selection core takes that default
-- as an input). `category` is an opaque identifier owned by the feature that
-- declares the notification type — the platform ships none.
CREATE TABLE IF NOT EXISTS user_account.notification_prefs (
    user_id    UUID        NOT NULL REFERENCES user_account.users(id) ON DELETE CASCADE,
    category   TEXT        NOT NULL,
    enabled    BOOLEAN     NOT NULL,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, category)
);

-- The user's timezone, as an IANA name (e.g. "Europe/Paris"), set and refreshed
-- by the client (design D3) so a schedule expressed in *local* hours fires at the
-- right moment for each user. NULL means "unknown" — the selection core falls
-- back to the configured default timezone rather than an arbitrary hour. A
-- dedicated column next to `locale` for the same reason: it is backend-relevant
-- (the send path reads it), not client-owned `preferences` JSONB.
ALTER TABLE user_account.users ADD COLUMN IF NOT EXISTS timezone TEXT;
