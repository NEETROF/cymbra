-- Practice-streak reminder schedule (change: add-practice-streak, task 3.1). A
-- new migration rather than an edit to an earlier one — applied migrations are
-- immutable (sqlx checks their checksum).
--
-- HOURLY, on purpose. The job itself has no idea what time to send: the send hour
-- is a back-office flag (`notifications.category.practice_streak.hour`) applied
-- per user in their OWN timezone by the push platform's selection gate. Running
-- every hour is what lets one flag serve every zone — the sweep resolves the
-- at-risk set, hands it to the platform, and the platform keeps only the users
-- for whom it is currently that local hour. So 23 runs out of 24 select nobody
-- for any given player, which is the intended (and cheap) behaviour.
--
-- `skip` for missed runs: a reminder that was not sent at 20:00 is worthless at
-- 23:00, so a backlog is collapsed rather than replayed. The whole thing is inert
-- until an operator turns the category on in the back office — declaring a
-- category does not enable it.
INSERT INTO schedules (name, module, kind, cron_expr, timezone, enabled, missed_run_policy)
VALUES ('streak_reminder_hourly', 'notifications', 'streak_reminder', '5 * * * *', 'UTC', TRUE, 'skip')
ON CONFLICT (name) DO NOTHING;
