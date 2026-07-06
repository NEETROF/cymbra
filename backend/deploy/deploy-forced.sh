#!/usr/bin/env bash
# Forced-command wrapper for the CI deploy SSH key.
#
# The deploy key's authorized_keys entry pins:
#   command="/opt/cymbra/backend/deploy/deploy-forced.sh",no-pty,no-*-forwarding …
# so a client presenting that key can ONLY run this script — it cannot get a
# shell or run arbitrary commands. The command the client requested arrives in
# $SSH_ORIGINAL_COMMAND; we accept exactly `deploy <version>` and nothing else.
set -euo pipefail

read -r action version _rest <<<"${SSH_ORIGINAL_COMMAND:-}"

if [[ "$action" != "deploy" || -n "${_rest:-}" ]]; then
  echo "refused: only 'deploy <version>' is allowed (got: '${SSH_ORIGINAL_COMMAND:-}')" >&2
  exit 1
fi
if ! [[ "$version" =~ ^([0-9]+\.[0-9]+\.[0-9]+|latest)$ ]]; then
  echo "refused: invalid version '$version' (expected X.Y.Z or latest)" >&2
  exit 1
fi

exec /opt/cymbra/backend/deploy/deploy.sh "$version"
