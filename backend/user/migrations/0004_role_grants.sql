-- user module — role-grant audit trail (change: add-moderation-back-office).
--
-- Append-only history of every role grant/revoke: who was granted/revoked which
-- role in which scope, by which admin, and when. The current authorization state
-- stays in `user_roles`; this table is history and is never consulted for access
-- decisions. It deliberately has NO foreign keys to `users`: the audit must survive
-- account deletion (so "who granted whom" stays answerable), and it is written by
-- the same least-privilege `user_svc` role (search_path = `user_account`).

CREATE TABLE role_grants (
    id             BIGINT      GENERATED ALWAYS AS IDENTITY PRIMARY KEY,
    target_user_id UUID        NOT NULL,                 -- the account granted/revoked
    scope          TEXT        NOT NULL,                 -- global | music | live
    role           TEXT        NOT NULL,                 -- user | admin | moderator
    action         TEXT        NOT NULL CHECK (action IN ('grant', 'revoke')),
    acting_admin   UUID        NOT NULL,                 -- the admin who performed it
    created_at     TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- The audit is queried per target account, most recent first.
CREATE INDEX role_grants_target_idx ON role_grants (target_user_id, created_at DESC);
