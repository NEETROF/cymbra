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
# Idempotent: re-running is a no-op (ON CONFLICT DO NOTHING).
set -euo pipefail

UID_ARG="${1:?usage: seed_admin.sh <user-uuid> [scope] [role]}"
SCOPE="${2:-music}"
ROLE="${3:-admin}"

psql -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER:-cymbra}" \
  --dbname "${POSTGRES_DB:-cymbra}" \
  -v uid="$UID_ARG" \
  -v scope="$SCOPE" \
  -v role="$ROLE" <<'SQL'
-- Verify the account exists first, so a typo'd id fails loudly rather than
-- silently seeding a role for a non-existent user.
DO $$
BEGIN
  IF NOT EXISTS (SELECT 1 FROM user_account.users WHERE id = :'uid'::uuid) THEN
    RAISE EXCEPTION 'no account with id %', :'uid';
  END IF;
END $$;

INSERT INTO user_account.user_roles (user_id, scope, role)
VALUES (:'uid'::uuid, :'scope', :'role')
ON CONFLICT DO NOTHING;
SQL

echo "cymbra: seeded ${ROLE} in scope ${SCOPE} for ${UID_ARG}"
