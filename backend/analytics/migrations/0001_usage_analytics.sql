-- analytics module — first-party feature-usage telemetry (change:
-- add-feature-usage-analytics, tasks 1.1–1.3, design D3/D8/D10).
--
-- A dedicated `analytics` schema (design D10): the data is deliberately decoupled
-- from identity — every row carries a hashed, period-salted `user_bucket`, never a
-- FK to `users`, so there is nothing to relate to `music`/`user_account`. A
-- separate schema gives it its own retention policy and targeted grants, and makes
-- a future move to a dedicated analytics DB a clean cut.
--
-- Three tiers (design D3):
--   * usage_events       — raw, one row per action, all dimensions, TTL'd. Powers
--     free-form cross-filtering over the retention window.
--   * usage_action_daily — (day, action, variant, platform, device_class,
--     app_version, locale) → event_count. Permanent volume archive.
--   * usage_user_daily   — (day, user_bucket, platform, device_class) presence.
--     Permanent; makes exact COUNT(DISTINCT user_bucket) over any window possible
--     (daily distinct counts CANNOT be summed into a period distinct count).
--
-- Idempotent, fully-qualified DDL so a double-apply is safe regardless of the
-- connecting role's search_path. The `analytics` schema itself is provisioned by
-- ops (`roles.sql.tpl` / `provision-analytics-role.sql`,
-- `CREATE SCHEMA ... AUTHORIZATION analytics_svc`), NOT here — a least-privilege
-- module role has no CREATE-on-database and `CREATE SCHEMA IF NOT EXISTS` checks
-- that privilege *before* the existence short-circuit. Migrations only create
-- objects inside the schema.

-- Raw events (tier 1): one row per reported action, every dimension preserved.
-- `subject_id` (high-cardinality, e.g. a score UUID) and `variant` (low-cardinality
-- qualifier) are the two optional context fields (design D8). `occurred_at` is the
-- client wall clock (clamped server-side); `received_at` is stamped on ingest.
CREATE TABLE IF NOT EXISTS analytics.usage_events (
    id            UUID        PRIMARY KEY,             -- server-generated UUID v7
    user_bucket   TEXT        NOT NULL,                -- period-salted pseudonymous id (never a raw user_id)
    action        TEXT        NOT NULL,                -- shape-validated taxonomy action
    variant       TEXT,                                -- optional low-cardinality qualifier (play mode, settings category)
    subject_id    TEXT,                                -- optional high-cardinality ref (e.g. score UUID); raw-only
    platform      TEXT        NOT NULL,                -- ios|android|macos|windows|linux|web
    device_class  TEXT        NOT NULL,                -- phone|tablet|desktop
    app_version   TEXT        NOT NULL,
    locale        TEXT        NOT NULL,
    occurred_at   TIMESTAMPTZ NOT NULL,                -- client wall clock (clamped)
    received_at   TIMESTAMPTZ NOT NULL DEFAULT now()   -- server ingest stamp
);

-- The purge job scans by age; the rollup and free-form filtering scan by time.
CREATE INDEX IF NOT EXISTS usage_events_occurred_at_idx
    ON analytics.usage_events (occurred_at);
CREATE INDEX IF NOT EXISTS usage_events_action_idx
    ON analytics.usage_events (action);
-- Composite for the common back-office filter combination (a period, sliced by
-- action / platform / device class).
CREATE INDEX IF NOT EXISTS usage_events_filter_idx
    ON analytics.usage_events (occurred_at, action, platform, device_class);

-- Action volume aggregate (tier 2): permanent, tiny (all grain columns are
-- low-cardinality). `variant` participates in the grain (design D8) but is
-- NOT-NULL-with-'' default so it can be part of a unique key (SQL NULLs never
-- compare equal); the rollup coalesces a missing variant to ''. `subject_id` is
-- high-cardinality and deliberately absent from the grain (raw-only).
CREATE TABLE IF NOT EXISTS analytics.usage_action_daily (
    day          DATE   NOT NULL,
    action       TEXT   NOT NULL,
    variant      TEXT   NOT NULL DEFAULT '',
    platform     TEXT   NOT NULL,
    device_class TEXT   NOT NULL,
    app_version  TEXT   NOT NULL,
    locale       TEXT   NOT NULL,
    event_count  BIGINT NOT NULL,
    PRIMARY KEY (day, action, variant, platform, device_class, app_version, locale)
);

-- Per-user daily presence (tier 3): permanent, one row per (day, bucket, platform,
-- device_class). Enables exact COUNT(DISTINCT user_bucket) over any window.
CREATE TABLE IF NOT EXISTS analytics.usage_user_daily (
    day          DATE NOT NULL,
    user_bucket  TEXT NOT NULL,
    platform     TEXT NOT NULL,
    device_class TEXT NOT NULL,
    PRIMARY KEY (day, user_bucket, platform, device_class)
);

-- Distinct-user counts filter/group by day (and optionally platform/device).
CREATE INDEX IF NOT EXISTS usage_user_daily_day_idx
    ON analytics.usage_user_daily (day);
