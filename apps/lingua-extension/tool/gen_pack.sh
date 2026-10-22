#!/usr/bin/env bash
# Build the data packs of every pair the extension ships (packs.json, generalise-lingua-pack-build),
# as the package carries them (ship-lingua-pack-bases-once D1, D4): each pair's layer into
# <dir>/packs/<pair>.lingua, and each studied language's base, once, into <dir>/bases/<studied>.lingua:
#   yarn gen:pack       → assets/  from the tiny committed testdata
#                                    (scripts/lingua-data/testdata/<pair>; dogfooding, CI)
#   yarn gen:pack:real  → assets/  from the committed tables, offline — the pair's
#                                    scripts/lingua-data/tables/<pair>/ and its studied language's
#                                    tables/<studied>/ — each part checked against its own
#                                    tables/<pair>/pin.json
# Every pair of a language builds the same base (split-lingua-pack-format): a language's second
# pair builds it again, and a base that differs by one byte from the first pair's is refused,
# naming both pairs, where the files are written. No whole pack is written: the agent plugin's are
# built with scripts/lingua-data/build.sh <pair> <out>. Both outputs are gitignored.
# `yarn gen:fixtures` writes the committed vitest fixtures (test/fixtures/) with build.sh directly.
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
mkdir -p "$OUT_ABS/packs" "$OUT_ABS/bases"
# The first pair of each studied language this run built the base of, one file per language.
SEEN="$(mktemp -d)"
trap 'rm -rf "$SEEN"' EXIT

for pair in $(node "$SCRIPT_DIR/packs.mjs" pairs); do
  studied="${pair%%-*}"
  args=(--parts)
  if [[ "$MODE" == real ]]; then
    if [[ ! -f "$DATA/tables/$pair/pin.json" ]]; then
      echo "error: packs.json ships $pair, but scripts/lingua-data/tables/$pair/ holds no committed tables." >&2
      exit 2
    fi
  else
    if [[ ! -d "$DATA/testdata/$pair" ]]; then
      echo "error: packs.json ships $pair, but scripts/lingua-data/testdata/$pair/ does not exist: add its test sources." >&2
      exit 2
    fi
    args=(--testdata --parts)
  fi
  layer="$OUT_ABS/packs/$pair.lingua"
  base="$OUT_ABS/bases/$studied.lingua"
  if [[ -f "$SEEN/$studied" ]]; then
    first="$(cat "$SEEN/$studied")"
    # Built aside, and the layer moved into place only once its base is the first pair's.
    again="$SEEN/$pair.base.lingua"
    staged="$SEEN/$pair.layer.lingua"
    bash "$DATA/build.sh" "${args[@]}" "$pair" "$again" "$staged"
    if ! cmp -s "$again" "$base"; then
      echo "error: $pair's base differs from $first's ($OUT/bases/$studied.lingua): every pair of a language ships one base." >&2
      echo "  $first and $pair build $studied from different studied tables; the package would carry layers of two bases." >&2
      exit 1
    fi
    mv "$staged" "$layer"
    echo "Built $MODE layer $pair → $OUT/packs/$pair.lingua (its base is $first's, byte for byte)"
  else
    bash "$DATA/build.sh" "${args[@]}" "$pair" "$base" "$layer"
    echo "$pair" >"$SEEN/$studied"
    echo "Built $MODE base $studied → $OUT/bases/$studied.lingua and layer $pair → $OUT/packs/$pair.lingua"
  fi
done
