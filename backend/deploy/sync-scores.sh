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
# nightly by cron (see bootstrap.sh / DEPLOY.md).
#
# It mirrors an ALLOW-LIST of servable prefixes, never `$SCORES_DIR` as a whole.
# That is deliberately redundant with the crawler now keeping its working files
# outside the corpus (change: fix-crawler-corpus-isolation): this job is billed,
# nightly and unattended, so it must fail closed. A deny-list would only block
# the cases someone thought of — in the incident that motivated this, 4.4 GB of
# git checkouts appeared under SCORES_DIR and `aws s3 sync`, which does not skip
# dot-directories, would have shipped every byte of them to the bucket.
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

# The ONLY prefixes that are mirrored. These are the object_key namespaces: the
# crawler's two confidence corpora plus user uploads. Anything else under
# SCORES_DIR is not servable and is not our business to ship off-box.
SERVABLE_PREFIXES=(safe low_confidence user-scores)

mkdir -p "$SCORES_DIR/safe" "$SCORES_DIR/low_confidence"
echo "corpus: $(find "${SCORES_DIR}/safe" "${SCORES_DIR}/low_confidence" -name '*.mxl' 2>/dev/null | wc -l | tr -d ' ') .mxl in $SCORES_DIR"

# Warn (don't fail) when something non-servable is sitting at the corpus root: it
# will NOT be mirrored, but it does not belong there and usually means a crawler
# work dir is misconfigured.
for entry in "$SCORES_DIR"/* "$SCORES_DIR"/.[!.]*; do
	[[ -e "$entry" ]] || continue
	name="$(basename "$entry")"
	skip=""
	for p in "${SERVABLE_PREFIXES[@]}"; do [[ "$name" == "$p" ]] && skip=1; done
	[[ -n "$skip" ]] || echo "WARNING: $name is not a servable prefix — not mirrored; check CYMBRA_SCORE_WORK_DIR"
done

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
	for p in "${SERVABLE_PREFIXES[@]}"; do
		[[ -d "$SCORES_DIR/$p" ]] || continue
		# One sync per prefix, mirroring `<prefix>/…` onto `s3://bucket/<prefix>/…`
		# so the S3 key stays exactly equal to object_key.
		aws --endpoint-url "${S3_ENDPOINT:?set S3_ENDPOINT}" \
			s3 sync "$SCORES_DIR/$p" "s3://$SCORES_S3_BUCKET/$p" --no-progress
	done
	echo "mirrored ${SERVABLE_PREFIXES[*]} -> s3://$SCORES_S3_BUCKET"
else
	echo "SCORES_S3_BUCKET unset — local corpus only, no off-box mirror"
fi
