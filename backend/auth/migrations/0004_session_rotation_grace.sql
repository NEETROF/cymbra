-- Tolerate a client killed before it stored a rotated token (change:
-- fix-interrupted-refresh-signouts). A refresh token is rotated on use and the old one
-- dies at once, so a client suspended between the server's answer and its own write wakes
-- holding a token the server has already replaced. Replaying it looked exactly like theft,
-- and revoked the whole family — which is how a Safari extension, suspended whenever it is
-- idle, signed its reader out several times a day.
--
-- The row now remembers the token it last replaced and when, so `rotate` can tell "my
-- answer never arrived" (within the grace) from a replay to revoke. Both columns are
-- nullable with no backfill: a session created before this migration simply has no
-- previous token until its next rotation fills them in.

ALTER TABLE sessions ADD COLUMN IF NOT EXISTS prev_rt_hash     TEXT;
ALTER TABLE sessions ADD COLUMN IF NOT EXISTS prev_replaced_at TIMESTAMPTZ;
