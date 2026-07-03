#!/usr/bin/env bash
# Nightly Postgres backup for the single-box deploy.
#
# Dumps the whole `cymbra` DB from the compose Postgres container, gzips it, keeps
# a local rolling window, and (optionally) ships it off-box to a Hetzner Storage
# Box over SFTP so a lost server does not mean lost data.
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

# Off-box copy (optional). Set STORAGEBOX_TARGET, e.g.:
#   STORAGEBOX_TARGET=u123456@u123456.your-storagebox.de:/backups/cymbra
# Uses SSH key auth (add the box's key to the Storage Box).
if [[ -n "${STORAGEBOX_TARGET:-}" ]]; then
	scp -q "$OUT" "$STORAGEBOX_TARGET/" && echo "shipped off-box -> $STORAGEBOX_TARGET"
fi

# Prune local backups older than the retention window.
find "$LOCAL_DIR" -name 'cymbra-*.sql.gz' -mtime "+$RETENTION_DAYS" -delete
echo "pruned local backups older than ${RETENTION_DAYS}d"
