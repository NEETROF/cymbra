#!/usr/bin/env bash
# Build the data pack of every pair the extension ships (packs.json, generalise-lingua-pack-build)
# into <dir>/<pair>.lingua:
#   yarn gen:pack       → assets/packs/  from the tiny committed testdata
#                                          (scripts/lingua-data/testdata/<pair>; dogfooding, CI)
#   yarn gen:pack:real  → assets/packs/  from the committed tables, offline, each checked against
#                                          its own scripts/lingua-data/tables/<pair>/pin.json
# Both outputs are gitignored. `yarn gen:fixtures` writes the committed vitest fixture
# (test/fixtures/en-fr.testdata.lingua) with build.sh directly.
#
# Usage: tool/gen_pack.sh [--real] <dir>
set -euo pipefail
MODE=testdata
if [[ "${1:-}" == "--real" ]]; then
  MODE=real
  shift
fi
OUT="${1:?usage: gen_pack.sh [--real] <dir>}"
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
APP_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
REPO_ROOT="$(cd "$APP_DIR/../.." && pwd)"
DATA="$REPO_ROOT/scripts/lingua-data"
case "$OUT" in
  /*) OUT_ABS="$OUT" ;;
  *) OUT_ABS="$APP_DIR/$OUT" ;;
esac
mkdir -p "$OUT_ABS"

for pair in $(node "$SCRIPT_DIR/packs.mjs" pairs); do
  if [[ "$MODE" == real ]]; then
    if [[ ! -f "$DATA/tables/$pair/pin.json" ]]; then
      echo "error: packs.json ships $pair, but scripts/lingua-data/tables/$pair/ holds no committed tables." >&2
      exit 2
    fi
    bash "$DATA/build.sh" "$pair" "$OUT_ABS/$pair.lingua"
  else
    if [[ ! -d "$DATA/testdata/$pair" ]]; then
      echo "error: packs.json ships $pair, but scripts/lingua-data/testdata/$pair/ does not exist: add its test sources." >&2
      exit 2
    fi
    bash "$DATA/build.sh" --testdata "$pair" "$OUT_ABS/$pair.lingua"
  fi
  echo "Built $MODE pack $pair → $OUT/$pair.lingua"
done
