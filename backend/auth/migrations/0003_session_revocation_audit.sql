-- Append-only audit of admin-initiated session revocations (change: add-session-
-- management). The role's search_path is pinned to `auth`, so this is
-- `auth.session_revocation_audit`. A durable, queryable trail — not just a log line
-- lost in the stream: who revoked whose sessions, when, and how many were cut.

CREATE TABLE session_revocation_audit (
    id             UUID        PRIMARY KEY,          -- UUIDv7, generated app-side
    target_user_id TEXT        NOT NULL,             -- account whose sessions were revoked
    acting_admin   TEXT        NOT NULL,             -- admin who performed the revocation
    revoked_count  INTEGER     NOT NULL,             -- live sessions cut at the time
    at             TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Audit lookup by target account, most recent first.
CREATE INDEX session_revocation_audit_target_idx
    ON session_revocation_audit (target_user_id, at DESC);
