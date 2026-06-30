-- Seed the recurring schedules the worker drives (change: add-job-infrastructure,
-- first slice D10). Runtime-tunable thereafter: operators may UPDATE the cadence,
-- timezone, or `enabled` flag without a redeploy.

-- Orphan reaper: purge handle-less accounts abandoned during onboarding. Runs
-- hourly (matching the old in-process loop's default interval); the grace period
-- comes from the worker's CYMBRA_ORPHAN_REAP_GRACE, applied by the handler.
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('orphan_reap_hourly', 'user', 'orphan_reap', '0 * * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
