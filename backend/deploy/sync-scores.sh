#!/usr/bin/env bash
# Publish the score-crawler corpus for the single-box deploy. The server serves
# score bytes LOCAL-FIRST from SCORES_DIR; S3 is the durable ORIGIN and the read
# FALLBACK (a local miss pulls from S3 and warms the local copy — the
# cymbra-storage port). So this mirror is both DR *and* what makes a rebuilt box
# self-heal on demand.
#
# Two steps, both idempotent — safe to run at the end of a crawl AND nightly by
# cron (see bootstrap.sh / DEPLOY.md):
#
#   1. MERGE  each per-source crawler output into one corpus dir the server reads.
#             The crawler writes per-source trees keyed by the score UUID (#82):
#             CRAWL_OUT/<source>/safe/<shard>/<uuid>.mxl, where object_key =
#             "safe/<shard>/<uuid>.mxl" (shard = the uuid's last two hex chars).
#             Merging CRAWL_OUT/*/safe/ -> SCORES_DIR/safe/ makes the on-disk path
#             exactly "SCORES_DIR/ + object_key", so the server resolves bytes from
#             object_key directly. Keys are UUIDs, so sources never collide.
#   2. MIRROR SCORES_DIR off-box to OVH Object Storage (S3-compatible), same creds
#             convention as backup.sh (/etc/cymbra/backup.env). This is the durable
#             origin the server falls back to on a local miss (same object_key).
#
# Install alongside the nightly backup cron (bootstrap.sh) and/or call at the end
# of a crawl:
#   . /etc/cymbra/backup.env; /opt/cymbra/backend/deploy/sync-scores.sh
set -euo pipefail

# Where the crawler's docker-compose wrote its per-source outputs. Absent (crawl
# runs on another box) → the merge is a no-op and we just re-mirror SCORES_DIR.
CRAWL_OUT="${CRAWL_OUT:-/opt/cymbra/score-crawler/output}"
# The unified corpus the app serves from (mounted read-only into the server).
SCORES_DIR="${SCORES_DIR:-/var/lib/cymbra/scores}"

mkdir -p "$SCORES_DIR/safe" "$SCORES_DIR/low_confidence"

# 1) Merge per-source subtrees into the unified corpus (rsync: incremental, keeps
#    existing files, never deletes — the corpus only grows).
if [[ -d "$CRAWL_OUT" ]]; then
	shopt -s nullglob
	for d in "$CRAWL_OUT"/*/safe/; do
		rsync -a "$d" "$SCORES_DIR/safe/"
	done
	for d in "$CRAWL_OUT"/*/low_confidence/; do
		rsync -a "$d" "$SCORES_DIR/low_confidence/"
	done
	shopt -u nullglob
fi
echo "corpus: $(find "$SCORES_DIR" -name '*.mxl' 2>/dev/null | wc -l | tr -d ' ') .mxl in $SCORES_DIR"

# 2) Off-box mirror to OVH Object Storage (S3-compatible), if configured. `sync`
#    is incremental; no --delete, so an accidental local wipe can't nuke the
#    origin. Mirror to the BUCKET ROOT (not a scores/ prefix): the storage port
#    reads S3 by object_key directly, so the S3 key MUST equal object_key
#    (safe/<shard>/<uuid>.mxl). User uploads share the same bucket under
#    user-scores/. OVH bills stored GB only (no egress/API fees).
#      S3_ENDPOINT=https://s3.gra.io.cloud.ovh.net   S3_BUCKET=cymbra-scores
if [[ -n "${S3_BUCKET:-}" ]]; then
	aws --endpoint-url "${S3_ENDPOINT:?set S3_ENDPOINT}" \
		s3 sync "$SCORES_DIR" "s3://$S3_BUCKET" --no-progress \
		&& echo "mirrored -> s3://$S3_BUCKET"
else
	echo "S3_BUCKET unset — local corpus only, no off-box mirror"
fi
