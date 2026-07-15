-- Let admin_svc enqueue jobs (change: add-user-score-upload).
--
-- The `purge_user` job runs as admin_svc and, as part of a complete account
-- erasure, must enqueue one `purge_score_object` job per stored score object
-- INSIDE its erasure transaction (transactional enqueue, design D3) so the row
-- deletes and the object-cleanup jobs commit atomically. admin_svc therefore
-- needs the same narrow EXECUTE on jobs.enqueue that the module roles have —
-- nothing else (SECURITY DEFINER still runs the insert with worker_svc's rights).
GRANT USAGE ON SCHEMA jobs TO admin_svc;
GRANT EXECUTE ON FUNCTION enqueue(
    TEXT, TEXT, TEXT, BOOLEAN, INT, INTERVAL, INTERVAL, TEXT
) TO admin_svc;
