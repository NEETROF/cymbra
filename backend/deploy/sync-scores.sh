#!/usr/bin/env bash
# Publish the score-crawler corpus for the single-box deploy (Option A: the app
# serves score bytes from a LOCAL folder; S3 is a durable off-box backup).
#
# Two steps, both idempotent — safe to run at the end of a crawl AND nightly by
# cron (see bootstrap.sh / DEPLOY.md):
#
#   1. MERGE  each per-source crawler output into one corpus dir the app reads.
#             The crawler's docker-compose writes per-source trees
#             (CRAWL_OUT/<source>/safe/<source>/<author>/<title>.mxl), while the
#             catalog stores object_key = "safe/<source>/<author>/<title>.mxl".
#             Merging CRAWL_OUT/*/safe/ -> SCORES_DIR/safe/ makes the on-disk
#             path exactly "SCORES_DIR/ + object_key", so the app resolves bytes
#             from object_key directly. Sources are namespaced, so no collisions.
#   2. MIRROR SCORES_DIR off-box to OVH Object Storage (S3-compatible), same
#             creds convention as backup.sh (/etc/cymbra/backup.env). Durable
#             backup / DR — NOT the app's read path.
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
#    backup. OVH bills stored GB only (no egress/API fees).
#      S3_ENDPOINT=https://s3.gra.io.cloud.ovh.net   S3_BUCKET=cymbra-scores
if [[ -n "${S3_BUCKET:-}" ]]; then
	aws --endpoint-url "${S3_ENDPOINT:?set S3_ENDPOINT}" \
		s3 sync "$SCORES_DIR" "s3://$S3_BUCKET/scores" --no-progress \
		&& echo "mirrored -> s3://$S3_BUCKET/scores"
else
	echo "S3_BUCKET unset — local corpus only, no off-box mirror"
fi
