#!/usr/bin/env bash
# Mirror the score corpus off-box for the single-box deploy. The server serves
# score bytes LOCAL-FIRST from SCORES_DIR; S3 is the durable ORIGIN and the read
# FALLBACK (a local miss pulls from S3 and warms the local copy — the
# cymbra-storage port). So this mirror is both DR *and* what makes a rebuilt box
# self-heal on demand.
#
# The score-crawler fan-out (docker-compose.crawler.prod.yml) now writes its
# output DIRECTLY into SCORES_DIR (the served corpus), so freshly crawled scores
# are servable immediately — no merge step here. This script's only job is the
# off-box S3 mirror; it is idempotent and safe to run at the end of a crawl AND
# nightly by cron (see bootstrap.sh / DEPLOY.md):
#
#   MIRROR SCORES_DIR off-box to OVH Object Storage (S3-compatible), same creds
#          convention as backup.sh (/etc/cymbra/backup.env). This is the durable
#          origin the server falls back to on a local miss (S3 key == object_key,
#          safe/<shard>/<uuid>.mxl).
#
# Install alongside the nightly backup cron (bootstrap.sh) and/or call at the end
# of a crawl:
#   . /etc/cymbra/backup.env; /opt/cymbra/backend/deploy/sync-scores.sh
set -euo pipefail

# The unified corpus the app serves from (also where the crawler writes).
SCORES_DIR="${SCORES_DIR:-/var/lib/cymbra/scores}"

mkdir -p "$SCORES_DIR/safe" "$SCORES_DIR/low_confidence"
echo "corpus: $(find "$SCORES_DIR" -name '*.mxl' 2>/dev/null | wc -l | tr -d ' ') .mxl in $SCORES_DIR"

# Off-box mirror to OVH Object Storage (S3-compatible), if configured. `sync`
#    is incremental; no --delete, so an accidental local wipe can't nuke the
#    origin. Mirror to the BUCKET ROOT (not a scores/ prefix): the storage port
#    reads S3 by object_key directly, so the S3 key MUST equal object_key
#    (safe/<shard>/<uuid>.mxl). User uploads share the same bucket under
#    user-scores/. OVH bills stored GB only (no egress/API fees).
#
#    The scores bucket is DISTINCT from the DB-backup bucket (`S3_BUCKET`, used by
#    backup.sh): it MUST equal the server's CYMBRA_SCORE_S3_BUCKET so uploads and
#    the corpus share one keyspace. Set it explicitly (no fallback to S3_BUCKET —
#    that would dump the corpus into the backups bucket):
#      S3_ENDPOINT=https://s3.eu-west-par.io.cloud.ovh.net   SCORES_S3_BUCKET=cymbra-scores
if [[ -n "${SCORES_S3_BUCKET:-}" ]]; then
	aws --endpoint-url "${S3_ENDPOINT:?set S3_ENDPOINT}" \
		s3 sync "$SCORES_DIR" "s3://$SCORES_S3_BUCKET" --no-progress \
		&& echo "mirrored -> s3://$SCORES_S3_BUCKET"
else
	echo "SCORES_S3_BUCKET unset — local corpus only, no off-box mirror"
fi
