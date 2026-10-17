-- change: add-music-piece-stars — the player's best OVERALL synchronisation on a
-- piece, kept beside the per-mode board bests so the app can draw one to three
-- stars from the same read that drives the score-card badge. Monotonic: the
-- ingest only ever raises it (GREATEST), whatever the per-mode ranking decides.
ALTER TABLE music.leaderboard_bests
    ADD COLUMN IF NOT EXISTS best_overall_sync_pct REAL;
