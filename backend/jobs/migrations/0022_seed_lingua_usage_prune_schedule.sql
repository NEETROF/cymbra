-- Schedule the Lingua usage counters' retention (change: add-lingua-usage-counts, D9).
-- Runs as `worker_svc` (owner of `jobs`), search_path = jobs. Re-runnable.
--
-- Once a day at 04:15 UTC the worker deletes the anonymous counters
-- (`lingua.usage_counts_daily`, `lingua.leave_reasons_weekly`) more than 25 months old,
-- over the `lingua_svc` pool. It is its own run on purpose: retention must hold whether the
-- counts' kill switch is on or off, and whether any report arrives. `skip`: a missed day is
-- caught up by the next one, which deletes the same rows.
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('lingua_usage_prune_daily', 'lingua', 'lingua_usage_prune', '15 4 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
