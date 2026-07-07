#!/usr/bin/env bash
# Nightly Postgres backup for the single-box deploy.
#
# Dumps the whole `cymbra` DB from the compose Postgres container, gzips it, keeps
# a local rolling window, and (optionally) ships it off-box to OVH Object Storage
# (S3-compatible) so a lost server does not mean lost data.
#
# Install as a cron job on the host (see DEPLOY.md):
#   0 3 * * *  /opt/cymbra/backend/deploy/backup.sh >> /var/log/cymbra-backup.log 2>&1
#
# Restore (into a fresh empty DB):
#   gunzip -c cymbra-YYYYmmdd-HHMM.sql.gz | \
#     docker compose -f docker-compose.prod.yml exec -T postgres psql -U cymbra -d cymbra
set -euo pipefail

COMPOSE_DIR="$(cd "$(dirname "$0")" && pwd)"
COMPOSE_FILE="$COMPOSE_DIR/docker-compose.prod.yml"
LOCAL_DIR="${BACKUP_DIR:-/var/backups/cymbra}"
RETENTION_DAYS="${BACKUP_RETENTION_DAYS:-14}"
STAMP="$(date +%Y%m%d-%H%M)"
OUT="$LOCAL_DIR/cymbra-$STAMP.sql.gz"

mkdir -p "$LOCAL_DIR"

# pg_dump as the superuser inside the container; stream straight to a gzip file.
docker compose -f "$COMPOSE_FILE" exec -T postgres \
	pg_dump -U cymbra -d cymbra --format=plain --no-owner \
	| gzip -9 > "$OUT"
echo "backup written: $OUT ($(du -h "$OUT" | cut -f1))"

# Off-box copy to OVH Object Storage (S3-compatible), optional. Provide S3_BUCKET +
# S3_ENDPOINT and OVH S3 creds (AWS_ACCESS_KEY_ID / AWS_SECRET_ACCESS_KEY) in the
# environment. OVH bills only stored GB — no ingress/egress/API fees.
#   S3_ENDPOINT=https://s3.gra.io.cloud.ovh.net   S3_BUCKET=cymbra-backups
if [[ -n "${S3_BUCKET:-}" ]]; then
	aws --endpoint-url "${S3_ENDPOINT:?set S3_ENDPOINT}" \
		s3 cp "$OUT" "s3://$S3_BUCKET/$(basename "$OUT")" \
		&& echo "shipped off-box -> s3://$S3_BUCKET"
fi

# Off-box copy of the .env (DB passwords + token signing key), ENCRYPTED client-side
# so a leaked S3 object / OVH-side access can't read it — redundancy for the
# password-manager vault. gpg symmetric (AES256) with BACKUP_ENV_PASSPHRASE (kept in
# this env file AND your vault; a lost box means restoring the passphrase from the
# vault). Fixed object name: the secrets don't rotate, we keep one current copy.
# Restore:  gpg --batch --pinentry-mode loopback --passphrase "<pass>" -d cymbra-env.gpg > .env
if [[ -n "${S3_BUCKET:-}" && -n "${BACKUP_ENV_PASSPHRASE:-}" && -f "$COMPOSE_DIR/.env" ]]; then
	ENC="$(mktemp)"
	gpg --batch --yes --symmetric --cipher-algo AES256 \
		--pinentry-mode loopback --passphrase "$BACKUP_ENV_PASSPHRASE" \
		-o "$ENC" "$COMPOSE_DIR/.env"
	aws --endpoint-url "${S3_ENDPOINT:?set S3_ENDPOINT}" \
		s3 cp "$ENC" "s3://$S3_BUCKET/env/cymbra-env.gpg" \
		&& echo "shipped encrypted .env -> s3://$S3_BUCKET/env/cymbra-env.gpg"
	rm -f "$ENC"
fi

# Prune local backups older than the retention window.
find "$LOCAL_DIR" -name 'cymbra-*.sql.gz' -mtime "+$RETENTION_DAYS" -delete
echo "pruned local backups older than ${RETENTION_DAYS}d"
