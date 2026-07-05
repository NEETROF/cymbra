-- Durable refresh-token sessions (change: durable-sessions-postgres). Replaces the
-- Redis-only session store: one row per session *family*. The role's search_path
-- is pinned to `auth`, so this table is `auth.sessions`.
--
-- A refresh token handed to a client is `"{id}.{secret}"`; only the SHA-256 hash of
-- the whole token is stored (`current_rt_hash`), so a DB dump leaks no usable token.
-- Rotation is a single guarded UPDATE on (id, current_rt_hash); a replayed rotated
-- token still resolves to its family (id is in the token) so theft revokes it.

CREATE TABLE sessions (
    id               UUID        PRIMARY KEY,     -- session family id (UUIDv7)
    user_id          TEXT        NOT NULL,
    audience         TEXT        NOT NULL,        -- bound at sign-in (one login per app)
    current_rt_hash  TEXT        NOT NULL,        -- SHA-256 hex of the current refresh token
    expires_at       TIMESTAMPTZ NOT NULL,        -- sliding refresh-token lifetime
    created_at       TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Rotate lookup + integrity (a token hash identifies at most one live family).
CREATE UNIQUE INDEX sessions_current_rt_hash_idx ON sessions (current_rt_hash);
-- revoke-all / active-session enumeration by account.
CREATE INDEX sessions_user_id_idx ON sessions (user_id);
-- Reap of expired rows.
CREATE INDEX sessions_expires_at_idx ON sessions (expires_at);
