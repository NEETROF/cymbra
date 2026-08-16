-- Plans maintenance schedules (change: add-premium-subscription). A new migration
-- rather than an edit to an earlier one — applied migrations are immutable.
--
-- Both run daily on the ORDERED `plans.maintenance` channel: reconciliation first
-- (re-read the provider state of paid rows nearing their end, so a missed store /
-- merchant-of-record notification is repaired), then the withdrawal sweep (rotate
-- the offline cache secret of accounts whose plan lapsed past grace — design D13).
-- The order guarantees nobody is withdrawn for a renewal we merely missed.
--
-- `skip` for missed runs: both are idempotent sweeps over current state, so a
-- backlog is collapsed rather than replayed. Inert until `plans.enabled` is on
-- (the handlers answer "0" while the kill-switch is off).
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('plans_reconcile_daily', 'plans', 'plans_reconcile', '10 4 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('plans_withdraw_daily', 'plans', 'plans_withdraw', '40 4 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
