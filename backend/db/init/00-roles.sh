#!/usr/bin/env bash
# Cymbra ID — role/schema bootstrap entrypoint (change: add-ops-db-access).
#
# Runs first in docker-entrypoint-initdb.d (name "00-…"), reads role names +
# passwords from the environment (each defaulting to its dev value), and applies
# the secret-free `roles.sql.tpl` with psql variables. The template is idempotent,
# so this is safe on a fresh volume and on re-apply.
#
# Also reused by CI and by hand to (re)provision an existing DB:
#   PGHOST=localhost PGPASSWORD=… POSTGRES_USER=cymbra POSTGRES_DB=cymbra \
#     CYMBRA_ROLES_TEMPLATE=backend/db/init/roles.sql.tpl bash backend/db/init/00-roles.sh
#
# Production: set CYMBRA_*_DB_PASSWORD from a secret store (or use IAM auth and
# ignore the password vars) — see backend/README.md. No secret is committed.
set -euo pipefail

TEMPLATE="${CYMBRA_ROLES_TEMPLATE:-/docker-entrypoint-initdb.d/roles.sql.tpl}"

psql -v ON_ERROR_STOP=1 \
  --username "${POSTGRES_USER:-cymbra}" \
  --dbname "${POSTGRES_DB:-cymbra}" \
  -v auth_role="${CYMBRA_AUTH_DB_ROLE:-auth_svc}" \
  -v auth_pw="${CYMBRA_AUTH_DB_PASSWORD:-auth_dev_pw}" \
  -v user_role="${CYMBRA_USER_DB_ROLE:-user_svc}" \
  -v user_pw="${CYMBRA_USER_DB_PASSWORD:-user_dev_pw}" \
  -v worker_role="${CYMBRA_WORKER_DB_ROLE:-worker_svc}" \
  -v worker_pw="${CYMBRA_WORKER_DB_PASSWORD:-worker_dev_pw}" \
  -v admin_role="${CYMBRA_ADMIN_DB_ROLE:-admin_svc}" \
  -v admin_pw="${CYMBRA_ADMIN_DB_PASSWORD:-admin_dev_pw}" \
  -f "$TEMPLATE"

echo "cymbra: roles + schemas bootstrapped (auth_svc, user_svc, worker_svc, admin_svc)"
