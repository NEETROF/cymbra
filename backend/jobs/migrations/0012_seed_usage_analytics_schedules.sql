-- Feature-usage-analytics rollup + purge schedules (change: add-feature-usage-
-- analytics, D3/D4). A new migration rather than an edit to an earlier one —
-- applied migrations are immutable (sqlx checks their checksum).
--
-- Two daily maintenance jobs on the ORDERED `analytics.maintenance` channel:
--   * usage_rollup — folds each closed day of analytics.usage_events into the two
--     permanent daily aggregates (idempotent upserts).
--   * usage_purge  — deletes raw events older than the BO-configured retention
--     window (default 180 days), leaving the aggregates intact.
--
-- Ordering (design D4): the shared ordered channel serialises the two, and rollup
-- runs at an EARLIER cron time (03:10) than purge (03:40), so purge never removes a
-- day the rollup has not yet aggregated. Runtime-tunable thereafter (operators may
-- UPDATE cadence/timezone/enabled without a redeploy).
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES
  ('usage_rollup_daily', 'analytics', 'usage_rollup', '10 3 * * *', 'UTC', TRUE, 'skip'),
  ('usage_purge_daily',  'analytics', 'usage_purge',  '40 3 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
