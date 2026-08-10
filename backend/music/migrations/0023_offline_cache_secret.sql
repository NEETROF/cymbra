-- music module — per-user offline-cache secret (change: add-offline-score-cache,
-- task 1.1, spec `backend-offline-key`).
--
-- One row per user holding a high-entropy secret (>= 32 bytes) that the app uses
-- as ONE input to its local offline-cache key derivation (HKDF over: an OS-keystore
-- device key, this server secret, the user's uuid, and a per-install seed). The
-- secret is created on first `GetOfflineCacheKey` request and returned unchanged
-- thereafter, so the same favorites decrypt across all of the user's devices.
--
-- `user_id` is a PLAIN uuid — no cross-schema FK to the user module (module-role
-- isolation, exactly like user_scores/user_library/play_sessions). Account deletion
-- erases this row explicitly in the `purge_user` worker job (no DB cascade
-- possible); once gone, the next request mints a NEW secret, so any residual
-- offline cache files on the user's old devices become undecryptable.
--
-- `secret` is sensitive material at rest: it is only ever returned over the
-- authenticated, owner-scoped RPC and never logged or listed cross-user.
--
-- Idempotent DDL + fully-qualified names so a double-apply is safe regardless of
-- the connecting role's search_path.

CREATE TABLE IF NOT EXISTS music.offline_cache_secrets (
    user_id    UUID PRIMARY KEY,                 -- caller's AuthIdentity.user_id; no cross-schema FK
    secret     BYTEA       NOT NULL,             -- >= 32 random bytes; one input to the client KDF
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    rotated_at TIMESTAMPTZ NOT NULL DEFAULT now() -- bumped on every rotation (kill-switch lever)
);
