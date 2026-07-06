#!/usr/bin/env bash
# Deploy a pinned cymbra-backend image version to the single-box prod.
#
#   deploy.sh <version>     e.g.  deploy.sh 0.2.2   |   deploy.sh latest
#
# Pins CYMBRA_IMAGE in the local .env (so the .env stays the record of what is
# running), pulls, rolls server+worker, and waits for /healthz to go green.
# Rollback = re-run with the previous version. Runs as the `ubuntu` user (docker
# group + owns the .env), no sudo. Invoked manually or by the CI deploy workflow
# via the forced-command wrapper (deploy-forced.sh).
set -euo pipefail

VERSION="${1:?usage: deploy.sh <version|latest>}"
if ! [[ "$VERSION" =~ ^([0-9]+\.[0-9]+\.[0-9]+|latest)$ ]]; then
  echo "[deploy] refused: invalid version '$VERSION' (expected X.Y.Z or latest)" >&2
  exit 1
fi

IMAGE_REPO="ghcr.io/neetrof/cymbra-backend"
DIR="/opt/cymbra/backend/deploy"
COMPOSE=(docker compose -f "$DIR/docker-compose.prod.yml")
cd "$DIR"

PREV="$(grep '^CYMBRA_IMAGE=' .env || true)"
echo "[deploy] pin  $IMAGE_REPO:$VERSION   (was: ${PREV#CYMBRA_IMAGE=})"
sed -i "s|^CYMBRA_IMAGE=.*|CYMBRA_IMAGE=$IMAGE_REPO:$VERSION|" .env

echo "[deploy] pull…"
"${COMPOSE[@]}" pull server worker

echo "[deploy] up -d…"
"${COMPOSE[@]}" up -d

echo "[deploy] waiting for /healthz…"
for _ in $(seq 1 20); do
  if curl -fsS "https://api.cymbra.app/healthz" >/dev/null 2>&1; then
    echo "[deploy] OK — /healthz green"
    "${COMPOSE[@]}" ps --format 'table {{.Name}}\t{{.Image}}\t{{.Status}}' | grep -E 'server|worker' || true
    exit 0
  fi
  sleep 3
done

echo "[deploy] ERROR — /healthz not green after ~60s. Recent server logs:" >&2
"${COMPOSE[@]}" logs --tail=40 server >&2 || true
echo "[deploy] rollback: re-run with the previous version (${PREV#CYMBRA_IMAGE=*:})" >&2
exit 1
