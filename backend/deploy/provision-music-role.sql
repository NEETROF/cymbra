-- Provision the `music` module role + schema on a LIVE database that was
-- initialised before the music module existed (so db/init's roles.sql.tpl never
-- created them). Idempotent and TARGETED: it touches music_svc + the music
-- schema, and adds `music` to the MIGRATOR role's search_path (see below) — it
-- does NOT reset any role's password (unlike re-running the full 00-roles.sh).
--
-- Mirror of the `music module` block in db/init/roles.sql.tpl. Run as the DB
-- superuser, passing the password as a psql variable so it never lands in a file
-- or shell history:
--
--   docker exec -e MPW='<chosen music password>' -i cymbra-prod-postgres-1 \
--     psql -U cymbra -d cymbra -v music_pw="$MPW" -f - < provision-music-role.sql
--
-- After this: set CYMBRA_MUSIC_DATABASE_URL=postgres://music_svc:<pw>@postgres:5432/cymbra
-- (+ the CYMBRA_SCORE_S3_* keys), then roll the server — its MIGRATOR creates
-- music.catalog_scores (0001) and music.user_scores (0003).

\set ON_ERROR_STOP on

SELECT format('CREATE ROLE %I LOGIN', 'music_svc')
WHERE NOT EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'music_svc')
\gexec

ALTER ROLE "music_svc" WITH LOGIN PASSWORD :'music_pw';
CREATE SCHEMA IF NOT EXISTS music AUTHORIZATION "music_svc";
ALTER ROLE "music_svc" SET search_path = music;

-- Belt-and-braces DML grants (the crawler may create catalog_scores here as the
-- superuser; default-privilege anything created later in this schema).
GRANT SELECT, INSERT, UPDATE, DELETE ON ALL TABLES IN SCHEMA music TO "music_svc";
ALTER DEFAULT PRIVILEGES IN SCHEMA music
  GRANT SELECT, INSERT, UPDATE, DELETE ON TABLES TO "music_svc";

-- Keep music_svc out of the shared public schema (design D0).
REVOKE ALL ON SCHEMA public FROM "music_svc";

-- Catalog full-text search (change: score-hub-search): pg_trgm/unaccent are
-- needed by the trigram GIN index + accent-fold backfill; CREATE EXTENSION needs
-- superuser, so enable it here (mirrors the roles.sql.tpl music block). Installed
-- into `music` so music_svc resolves it via its pinned search_path.
CREATE EXTENSION IF NOT EXISTS pg_trgm SCHEMA music;
CREATE EXTENSION IF NOT EXISTS unaccent SCHEMA music;

-- The MIGRATOR role (admin_svc) runs the module migrations. Migration 0004 calls
-- `unaccent(...)` and uses `gin_trgm_ops` UNqualified, so `music` must be on the
-- migrator's search_path too — roles.sql.tpl lists it (auth, user_account, music,
-- jobs, public), but a DB provisioned before the music module has an admin
-- search_path WITHOUT `music`. Without this, `0004` fails on a live upgrade with
-- "function unaccent(text) does not exist" even though the extensions above exist
-- (music_svc resolves them, but the migrator role does not). Re-assert the
-- canonical list here, idempotently, WITHOUT touching the password. Adjust the
-- role name if your migrator role is not `admin_svc`.
DO $$
BEGIN
  IF EXISTS (SELECT 1 FROM pg_roles WHERE rolname = 'admin_svc') THEN
    EXECUTE 'ALTER ROLE admin_svc SET search_path = auth, user_account, music, jobs, public';
  END IF;
END $$;

\echo 'music_svc + schema music + migrator search_path provisioned (idempotent).'
