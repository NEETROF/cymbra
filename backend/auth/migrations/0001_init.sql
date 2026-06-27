-- auth module schema (task 4.2). The role's search_path is pinned to `auth`.
-- Sessions/refresh tokens live in Redis (not here); this stores the local
-- credential secret + the single-use verification / reset tokens.

CREATE TABLE local_credentials (
    email              TEXT PRIMARY KEY,
    password_hash      TEXT        NOT NULL,          -- argon2id PHC string
    email_verified     BOOLEAN     NOT NULL DEFAULT false,
    verification_token TEXT,
    verification_expires_at BIGINT,                   -- unix seconds
    reset_token        TEXT,
    reset_expires_at   BIGINT,
    created_at         TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX local_credentials_verification_token_idx
    ON local_credentials (verification_token);
CREATE INDEX local_credentials_reset_token_idx
    ON local_credentials (reset_token);
