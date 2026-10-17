-- Piece-step results on a course (change: add-music-piece-lessons).
--
-- A lesson's `piece` step sends the learner to the real player; the run's
-- outcome is kept here beside the completion: `passed_at` is set on the FIRST
-- pass and kept (replays never unset it), `best_sync_pct` only ever rises
-- (GREATEST on write). A row may exist with `completed_at` NULL — a piece step
-- run before the lesson's end — which `list` reports as not completed. Same
-- isolation and erasure as the rest of the table (worker `purge_user`).

ALTER TABLE music.course_progress
    ADD COLUMN IF NOT EXISTS passed_at timestamptz,
    ADD COLUMN IF NOT EXISTS best_sync_pct real;
