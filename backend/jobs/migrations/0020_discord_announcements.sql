-- The Discord announcement ledger (change: add-discord-notifications, design D5).
-- Runs as `worker_svc` (owner of `jobs`), search_path = jobs. Re-runnable.
--
-- sqlxmq delivers at least once; a Discord message cannot be taken back. Each
-- announcement claims its dedup key here BEFORE posting and settles it after, so a
-- redelivered or retried `discord_notify` job stops at the claim instead of posting
-- twice. A claim left `claimed` (a crash between claim and post) is taken over after
-- a grace window chosen by the caller.
--
-- It lives in `jobs`, not in a `discord` schema of its own: the worker is its only
-- reader and writer, and `worker_svc` already owns this schema — a dedicated schema
-- would need a new role, pool and bootstrap for one table.
CREATE TABLE IF NOT EXISTS discord_announcements (
    dedup_key  TEXT        PRIMARY KEY,
    status     TEXT        NOT NULL CHECK (status IN ('claimed', 'posted', 'failed')),
    claimed_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    settled_at TIMESTAMPTZ,
    -- Why a post failed for good (a deleted webhook, a refused payload). Never a URL.
    reason     TEXT
);
