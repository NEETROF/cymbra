-- Let music_svc enqueue jobs (change: add-score-daily-access-rewards, design D7).
--
-- Accepting a catalog piece enqueues its audio-teaser render (`score_preview_render`)
-- INSIDE the status-write transaction (transactional enqueue, design D3): the job
-- exists iff the acceptance commits. music_svc therefore needs the same narrow
-- EXECUTE on jobs.enqueue that the other module roles have — nothing else
-- (SECURITY DEFINER still runs the insert with worker_svc's rights).
GRANT USAGE ON SCHEMA jobs TO music_svc;
GRANT EXECUTE ON FUNCTION enqueue(
    TEXT, TEXT, TEXT, BOOLEAN, INT, INTERVAL, INTERVAL, TEXT
) TO music_svc;
