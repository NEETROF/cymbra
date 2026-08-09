-- Per-user course completion (change: add-notation-courses, tranche 5).
--
-- Cross-device completion: a signed-in user's finished courses live here and are
-- read back on any device. `completed_at` is set on the first completion and kept
-- thereafter (replays only bump `play_count`), which is how the badge is awarded
-- exactly once. `user_id` is a plain users.id UUID with **no cross-schema FK**
-- (module-role isolation, like play_sessions); erasure is done explicitly by the
-- worker's purge_user job.

CREATE TABLE IF NOT EXISTS music.course_progress (
    user_id      uuid NOT NULL,
    course_id    text NOT NULL,
    completed_at timestamptz,
    play_count   integer NOT NULL DEFAULT 0,
    updated_at   timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (user_id, course_id)
);

CREATE INDEX IF NOT EXISTS course_progress_user_idx
    ON music.course_progress (user_id);
