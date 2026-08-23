-- Retire the is_piano staff-count proxy (change: add-drums-access).
--
-- Runs at the boot of the binary that no longer reads the column anywhere
-- (backend, crawler, proto and both front ends all moved to `instrument` in
-- the same change), so no reader remains by the time this executes. This is
-- the point of no return: the flag only ever meant "two or more staves" and
-- cannot be reconstructed from `instrument` — which is precisely why it is
-- not worth keeping.

ALTER TABLE music.catalog_scores DROP COLUMN is_piano;
ALTER TABLE music.user_scores DROP COLUMN is_piano;
