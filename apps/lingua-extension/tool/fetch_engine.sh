#!/usr/bin/env bash
# Fetch the translation engine the Chromium and Firefox packages carry
# (add-lingua-translation-delivery D9) into <dir> (default: apps/lingua-extension/engine), and keep
# it only if every file matches engine-pin.json.
#
# Two places hold it, tried in this order:
#   1. The GitHub Release `node tool/engine_pin.mjs --release-tag` names, which lingua-engine-build
#      publishes once from main. It never expires — the packages still build the day the upstream
#      sources, a submodule or the Emscripten SDK can no longer be fetched.
#   2. The artefact of the newest successful lingua-engine-build run on main that still holds one
#      (kept 90 days; the workflow runs monthly). Only until that release exists.
# If neither does: dispatch lingua-engine-build, or build it on Linux with tool/build_engine.sh.
#
# Needs the GitHub CLI, signed in (locally) or GH_TOKEN with `contents: read` and `actions: read`
# (in CI).
#
# Usage: tool/fetch_engine.sh [<dir>]
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP="$(cd "$SCRIPT_DIR/.." && pwd)"
OUT="${1:-$APP/engine}"
ARTEFACT=lingua-translation-engine
TAG=$(node "$SCRIPT_DIR/engine_pin.mjs" --release-tag)

# Keep the files of $1 if they are the pinned engine; say where they came from.
keep() {
  local from=$1 origin=$2
  node "$SCRIPT_DIR/engine_pin.mjs" "$from" || return 1
  mkdir -p "$OUT"
  cp "$from"/bergamot-translator.js "$from"/bergamot-translator.wasm "$from"/TRANSLATIONS_COMMIT "$OUT"/
  echo "Engine from $origin → $OUT"
}

tmp=$(mktemp -d)
if gh release download "$TAG" --pattern 'bergamot-translator.*' --pattern TRANSLATIONS_COMMIT --dir "$tmp" >/dev/null 2>&1; then
  if keep "$tmp" "the $TAG release"; then
    rm -rf "$tmp"
    exit 0
  fi
  echo "::warning::the $TAG release does not hold the pinned engine; trying the workflow's artefacts."
fi
rm -rf "$tmp"

runs=$(gh run list --workflow lingua-engine-build.yml --branch main --status success --limit 20 \
  --json databaseId --jq '.[].databaseId')

for run in $runs; do
  tmp=$(mktemp -d)
  if ! gh run download "$run" --name "$ARTEFACT" --dir "$tmp" >/dev/null 2>&1; then
    rm -rf "$tmp"
    continue # expired, or a run from before the artefact existed
  fi
  if keep "$tmp" "lingua-engine-build run $run"; then
    rm -rf "$tmp"
    exit 0
  fi
  rm -rf "$tmp"
  echo "::warning::run $run holds an engine that is not the pinned one; trying an older run."
done

echo "::error::neither the $TAG release nor any lingua-engine-build artefact on main holds the pinned engine. Dispatch the workflow (gh workflow run lingua-engine-build.yml), wait for it, and run this again — or build it on Linux with tool/build_engine.sh." >&2
exit 1
