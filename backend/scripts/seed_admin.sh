#!/usr/bin/env bash
# Cymbra — seed the first music/admin (change: add-moderation-back-office).
#
# There is no in-app path to mint the very first administrator, so an operator
# with database access seeds it out-of-band (design D6), exactly as roles are
# bootstrapped today (see backend/db/init/00-roles.sh). After this, that admin
# self-serves further grants from the back office (GrantRole).
#
# Usage (grant music/admin to an existing account by its users.id UUID):
#   PGHOST=localhost PGPASSWORD=… POSTGRES_USER=cymbra POSTGRES_DB=cymbra \
#     bash backend/scripts/seed_admin.sh <user-uuid> [scope] [role]
#
# Defaults: scope=music, role=admin. The account must already exist (sign in once
# to create it, then look up its id: SELECT id FROM user_account.users WHERE …).
# Idempotent: re-running is a no-op, and says so rather than claiming a grant.
#
# This writes a `role_grants` audit row like every other grant path (change:
# harden-module-boundaries, task 1.4). It used to INSERT into `user_roles` and
# nothing else, which meant a review of `role_grants` — the table built to answer
# "who was given what, by whom" — would conclude that nobody held a privileged role,
# while the only production path that creates one left no trace at all.
#
# `acting_admin` is the nil UUID: an operator holding psql credentials has no
# account, and inventing one would be worse than naming the bootstrap for what it
# is. A nil actor in that column reads as "seeded out-of-band".
set -euo pipefail

UID_ARG="${1:?usage: seed_admin.sh <user-uuid> [scope] [role]}"
SCOPE="${2:-music}"
ROLE="${3:-admin}"

# `-q` silences psql's command tags ("DO") so the RAISE NOTICE — the only honest
# report of what changed — is not buried; notices go to stderr and still show.
psql -q -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER:-cymbra}" \
  --dbname "${POSTGRES_DB:-cymbra}" \
  -v uid="$UID_ARG" \
  -v scope="$SCOPE" \
  -v role="$ROLE" <<'SQL'
-- Stash the args as session settings: psql `:var` substitution does NOT reach
-- inside a `$$`-quoted plpgsql body, so the DO block reads them via
-- current_setting instead (substitution here, in plain statements, is fine).
-- `\o /dev/null` discards the SELECT's result grid so only the final notice shows.
\o /dev/null
SELECT set_config('cymbra.seed_uid',   :'uid',   false),
       set_config('cymbra.seed_scope', :'scope', false),
       set_config('cymbra.seed_role',  :'role',  false);
\o

-- Verify the account exists first (a typo'd id RAISEs, and with ON_ERROR_STOP the
-- script exits non-zero) rather than silently seeding a role for no one, then
-- grant idempotently — all in one transaction.
DO $$
DECLARE
  u         uuid := current_setting('cymbra.seed_uid')::uuid;
  s         text := current_setting('cymbra.seed_scope');
  r         text := current_setting('cymbra.seed_role');
  -- No account behind a psql operator; the nil actor means "seeded out-of-band".
  nil_actor uuid := '00000000-0000-0000-0000-000000000000';
BEGIN
  IF NOT EXISTS (SELECT 1 FROM user_account.users WHERE id = u) THEN
    RAISE EXCEPTION 'no account with id %', u;
  END IF;
  -- Reject a typo here rather than at the CHECK constraint, so the message names
  -- what is accepted (migration 0010 enforces the same vocabulary).
  IF s NOT IN ('global', 'music', 'live', 'lingua') THEN
    RAISE EXCEPTION 'unknown scope %; expected global, music, live or lingua', s;
  END IF;
  IF r NOT IN ('user', 'admin', 'moderator') THEN
    RAISE EXCEPTION 'unknown role %; expected user, admin or moderator', r;
  END IF;

  INSERT INTO user_account.user_roles (user_id, scope, role)
  VALUES (u, s, r)
  ON CONFLICT DO NOTHING;

  -- Audit ONLY a grant that actually happened: re-running must not fabricate a
  -- second entry for a role the account already held.
  IF FOUND THEN
    INSERT INTO user_account.role_grants
      (target_user_id, scope, role, action, acting_admin)
    VALUES (u, s, r, 'grant', nil_actor);
    RAISE NOTICE 'granted % in scope % to % (audited)', r, s, u;
  ELSE
    RAISE NOTICE 'no change: % already holds % in scope %', u, r, s;
  END IF;
END $$;
SQL

# The NOTICE above already says what happened — granted, or already held.
echo "cymbra: done (see the notice above for what changed)"
