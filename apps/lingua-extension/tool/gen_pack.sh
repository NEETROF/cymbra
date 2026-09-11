#!/usr/bin/env bash
# Build an EN→FR data pack for the extension at the given output path.
#
#   yarn gen:pack       → assets/pack.lingua        (shipped/dogfooding pack, gitignored)
#   yarn gen:fixtures   → test/fixtures/en-fr.testdata.lingua  (committed vitest fixture)
#
# Today both use the tiny committed testdata sources (scripts/lingua-data/testdata/en-fr):
# real-source fetching (build.sh en-fr <out>) is still an unwired stub in this checkout.
# When real sources are wired (scripts/lingua-data/SOURCES.md), the dogfooding pack switches
# to the full lexicon; the fixture stays on --testdata so vitest is hermetic and small.
set -euo pipefail

OUT="${1:?usage: gen_pack.sh <output-path>}"

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"

# Resolve OUT relative to the app dir when it is not absolute.
case "$OUT" in
  /*) OUT_ABS="$OUT" ;;
  *) OUT_ABS="$APP_DIR/$OUT" ;;
esac

mkdir -p "$(dirname "$OUT_ABS")"
bash "$REPO_ROOT/scripts/lingua-data/build.sh" --testdata en-fr "$OUT_ABS"
echo "Built pack → $OUT"
