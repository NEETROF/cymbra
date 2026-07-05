-- Session reaper schedule (change: durable-sessions-postgres). A new migration
-- rather than an edit to 0007 — applied migrations are immutable (sqlx checks
-- their checksum). Deletes expired `auth.sessions` rows; lazy expiry enforces
-- correctness on read, so this cadence only bounds table growth. Runtime-tunable
-- thereafter (operators may UPDATE cadence/timezone/enabled without a redeploy).
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('session_reap_hourly', 'auth', 'session_reap', '0 * * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
