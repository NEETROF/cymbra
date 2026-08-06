-- Global-leaderboard season rollover schedule (change: add-global-leaderboard,
-- D3 / task 2.2). A new migration rather than an edit to an earlier one — applied
-- migrations are immutable (sqlx checks their checksum).
--
-- Runs daily just after midnight UTC (season boundaries are UTC): the handler
-- (`global_season_snapshot`) freezes the season that has just CLOSED into the
-- hall of fame and lets the new one accumulate from scratch. Most runs find the
-- previous season already snapshotted and do nothing — the job is idempotent
-- (guarded by the existing snapshot), so a daily cadence simply guarantees a
-- rollover is never missed by more than a day, and a missed run is collapsed to
-- the latest (`skip`) with no double-write. Runtime-tunable thereafter (operators
-- may UPDATE cadence/timezone/enabled without a redeploy).
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('global_season_snapshot_daily', 'music', 'global_season_snapshot', '20 0 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
