-- Play-detail retention prune schedule (change: add-play-activity-profile, D7).
-- A new migration rather than an edit to an earlier one — applied migrations are
-- immutable (sqlx checks their checksum). Runs daily; the handler NULLs the heavy
-- `session_result` JSONB on `music.play_sessions` rows past the configured
-- retention window (CYMBRA_PLAY_DETAIL_RETENTION_DAYS, default 90), keeping the
-- lightweight summary + per-day aggregate. Runtime-tunable thereafter (operators
-- may UPDATE cadence/timezone/enabled without a redeploy).
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('play_detail_prune_daily', 'music', 'play_detail_prune', '30 3 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
