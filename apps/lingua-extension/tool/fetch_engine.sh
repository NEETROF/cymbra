#!/usr/bin/env bash
# Fetch the translation engine the Chromium and Firefox packages carry
# (add-lingua-translation-delivery D9) into <dir> (default: apps/lingua-extension/engine).
#
# It takes the artefact of the newest successful lingua-engine-build run on main that still holds
# one, and keeps it only if every file matches engine-pin.json. The build is reproducible, so any
# run of the pinned commit gives the same bytes: when every artefact has expired (they are kept
# 90 days; the workflow also runs monthly so one always exists), dispatch lingua-engine-build and
# run this again. Or build it yourself on Linux: tool/build_engine.sh.
#
# Needs the GitHub CLI, signed in (locally) or GH_TOKEN with `actions: read` (in CI).
#
# Usage: tool/fetch_engine.sh [<dir>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="${1:-$APP/engine}"
ARTEFACT=lingua-translation-engine

runs=$(gh run list --workflow lingua-engine-build.yml --branch main --status success --limit 20 \
  --json databaseId --jq '.[].databaseId')

for run in $runs; do
  tmp=$(mktemp -d)
  if ! gh run download "$run" --name "$ARTEFACT" --dir "$tmp" >/dev/null 2>&1; then
    rm -rf "$tmp"
    continue # expired, or a run from before the artefact existed
  fi
  if node "$SCRIPT_DIR/engine_pin.mjs" "$tmp"; then
    mkdir -p "$OUT"
    cp "$tmp"/bergamot-translator.js "$tmp"/bergamot-translator.wasm "$tmp"/TRANSLATIONS_COMMIT "$OUT"/
    rm -rf "$tmp"
    echo "Engine from lingua-engine-build run $run → $OUT"
    exit 0
  fi
  rm -rf "$tmp"
  echo "::warning::run $run holds an engine that is not the pinned one; trying an older run."
done

echo "::error::no lingua-engine-build run on main still holds the pinned engine. Dispatch the workflow (gh workflow run lingua-engine-build.yml), wait for it, and run this again — or build it on Linux with tool/build_engine.sh." >&2
exit 1
