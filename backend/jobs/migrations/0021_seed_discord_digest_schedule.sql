-- Schedule the Discord reports run (change: add-discord-notifications, D7).
-- Runs as `worker_svc` (owner of `jobs`), search_path = jobs. Re-runnable.
--
-- Once a day at 00:30 UTC: every report decides for itself whether its closed period is
-- due today (Music daily by default, ID and the weekly top pieces on Mondays, Lingua on
-- Tuesdays since its figures arrive by device sync). Nothing is posted while the
-- `discord.enabled` flag or the report's own flag is off (both default off).
-- `skip`: a missed day is not caught up — each report is due on one day only, and a
-- late report of an old period is worth less than the noise.
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('discord_digest_daily', 'discord', 'discord_digest', '30 0 * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
