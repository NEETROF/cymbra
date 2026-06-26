-- user module schema (task 3.2). The role's search_path is pinned to
-- `user_account`, so unqualified objects are created there.

CREATE TABLE users (
    id           UUID PRIMARY KEY,                 -- UUID v7, generated app-side
    display_name TEXT,
    preferences  JSONB       NOT NULL DEFAULT '{}',
    version      BIGINT      NOT NULL DEFAULT 1,
    updated_at   TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE TABLE user_identities (
    id        UUID PRIMARY KEY,
    user_id   UUID        NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    provider  TEXT        NOT NULL,                 -- local | google | apple
    subject   TEXT        NOT NULL,                 -- email (local) or OIDC sub
    linked_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    UNIQUE (provider, subject)                      -- an identity belongs to one account
);

CREATE INDEX user_identities_user_id_idx ON user_identities (user_id);

CREATE TABLE user_roles (
    user_id UUID NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    scope   TEXT NOT NULL,                          -- global | music | live
    role    TEXT NOT NULL,
    UNIQUE (user_id, scope, role)
);
