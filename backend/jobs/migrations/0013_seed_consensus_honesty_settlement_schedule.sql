-- Consensus honesty-settlement sweep schedule (change: add-curation-rewards, D2 /
-- task 3.2). A new migration rather than an edit to an earlier one — applied
-- migrations are immutable (sqlx checks their checksum). Runs hourly; the handler
-- (`consensus_honesty_settlement`) freezes each score past the consensus minimum as
-- ground truth and settles each still-unsettled rating's honesty bonus. Idempotent
-- (guarded by the per-rating + per-score settlement state), so a missed run is
-- collapsed to the latest (`skip`) with no double-award. Runtime-tunable thereafter
-- (operators may UPDATE cadence/timezone/enabled without a redeploy).
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('consensus_honesty_settlement_hourly', 'music', 'consensus_honesty_settlement', '15 * * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
