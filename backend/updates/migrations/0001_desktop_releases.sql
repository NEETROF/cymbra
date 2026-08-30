-- app_updates module — the desktop update feed (change: add-desktop-auto-update,
-- tasks 2.1/2.2, design D1/D3).
--
-- One table. A row is a CI-signed release: the manifest bytes and the signature
-- are stored VERBATIM and served verbatim (the signature covers exactly those
-- bytes — design D2), while `key_id`, `rollout_percent` and `paused` are policy
-- fields deliberately OUTSIDE the signature, so ops can stage or kill a release
-- with a plain UPDATE without touching anything the client verifies.
--
-- `version_order` is the sortable projection of `version` (major.minor.patch+build)
-- computed at ingest by `cymbra_updates::core::version_order`. Storing it makes
-- "the highest servable release" a plain index scan instead of a text sort that
-- would put 1.9.0 above 1.10.0.
--
-- Dedicated schema (design D0, as for `analytics`): the feed carries no identity
-- and no FK to any other schema — the update check sends no identifier at all.
-- Idempotent, fully-qualified DDL so a double-apply is safe regardless of the
-- connecting role's search_path. The schema itself is provisioned by ops
-- (`roles.sql.tpl`, `CREATE SCHEMA ... AUTHORIZATION updates_svc`), NOT here.

CREATE TABLE IF NOT EXISTS app_updates.releases (
    product         TEXT        NOT NULL,             -- 'music' (later: 'live')
    channel         TEXT        NOT NULL,             -- 'stable'
    version         TEXT        NOT NULL,             -- 'major.minor.patch+build', as signed
    version_order   BIGINT      NOT NULL,             -- sortable projection of `version`
    manifest        TEXT        NOT NULL,             -- base64 of the EXACT signed manifest bytes
    signature       TEXT        NOT NULL,             -- base64 Ed25519 signature over those bytes
    key_id          TEXT        NOT NULL,             -- which trusted key signed it
    rollout_percent SMALLINT    NOT NULL DEFAULT 0,   -- 0 = kill-switch; evaluated client-side
    paused          BOOLEAN     NOT NULL DEFAULT FALSE,
    created_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at      TIMESTAMPTZ NOT NULL DEFAULT now(),
    CONSTRAINT releases_pkey PRIMARY KEY (product, channel, version),
    CONSTRAINT releases_rollout_range CHECK (rollout_percent BETWEEN 0 AND 100)
);

-- The one read the public endpoint makes: highest servable release for a
-- product/channel. Partial index — a paused or rollout-0 row is never a candidate.
CREATE INDEX IF NOT EXISTS releases_servable_idx
    ON app_updates.releases (product, channel, version_order DESC)
    WHERE NOT paused AND rollout_percent > 0;
