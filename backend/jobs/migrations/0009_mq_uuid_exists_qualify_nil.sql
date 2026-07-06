-- Cold-restore fix for pg_dump backups (backend/deploy/backup.sh).
--
-- pg_dump restores with an empty search_path and dumps function bodies verbatim.
-- mq_uuid_exists() (from the vendored sqlxmq 0.6.0 setup, 0001) called uuid_nil()
-- UNqualified. uuid_nil() is provided by the uuid-ossp extension, which lives in
-- schema `jobs` (created by the dev bootstrap, db/init). At runtime the pinned
-- search_path=jobs resolves it, but a cold `pg_dump | psql` into an empty DB does
-- not: creating the mq_msgs polling partial index — whose predicate inlines
-- mq_uuid_exists — failed with "function uuid_nil() does not exist", so a restored
-- database silently lost that index.
--
-- Schema-qualify the call so the body resolves under any search_path. Kept a plain
-- SQL IMMUTABLE function with NO `SET search_path` clause, so it stays inlinable in
-- the index predicate (a SET clause would make it non-inlinable). CREATE OR REPLACE
-- keeps the same OID, so the existing indexes/DEFAULT that reference it stay valid.
-- Runs as worker_svc with search_path=jobs, so `mq_uuid_exists` targets jobs.*.
CREATE OR REPLACE FUNCTION mq_uuid_exists(
    id UUID
) RETURNS BOOLEAN AS $$
	SELECT id IS NOT NULL AND id != jobs.uuid_nil()
$$ LANGUAGE SQL IMMUTABLE;
