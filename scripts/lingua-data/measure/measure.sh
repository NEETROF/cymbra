#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# What the working tables change for a reader, against the tables of a git ref, read by the
# extension's own engine over the measurement corpus (see README.md).
#
#   scripts/lingua-data/measure/measure.sh [<base ref, default origin/main>] [<pair, default en-fr>]
#
# Needs the extension's wasm glue (`yarn gen:wasm` in apps/lingua-extension) and the network, once,
# for the corpus's text. Writes into scripts/lingua-data/work/measure/ (git-ignored).
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../../.." && pwd)"
base="${1:-origin/main}"
pair="${2:-en-fr}"
out="$here/../work/measure"
glue="$repo/apps/lingua-extension/src/wasm/pkg"
[[ -f "$glue/lingua_wasm.js" ]] || { echo "error: no wasm glue in $glue — run yarn gen:wasm in apps/lingua-extension." >&2; exit 2; }

studied="${pair%%-*}"
mkdir -p "$out/base"
rm -rf "${out:?}/base/"*
# The ref's tables: the pair's folder and its studied language's (split-lingua-pack-tables-by-
# language); a ref older than the split holds both sides in the pair's folder.
if git -C "$repo" cat-file -e "$base:scripts/lingua-data/tables/$studied/studied.json" 2>/dev/null; then
  git -C "$repo" archive "$base" "scripts/lingua-data/tables/$pair" "scripts/lingua-data/tables/$studied" \
    | tar -x -C "$out/base" --strip-components=3
  base_studied="$out/base/$studied"
else
  git -C "$repo" archive "$base" "scripts/lingua-data/tables/$pair" | tar -x -C "$out/base" --strip-components=3
  base_studied="$out/base/$pair"
fi
[[ -f "$out/corpus.json" ]] || python3 "$here/fetch_corpus.py" "$here/corpus.json" "$out/corpus.json"

build() { cargo run --quiet --release -p lingua-pack --bin lingua-pack-build -- --studied "$1" "$2" "$3"; }
build "$base_studied" "$out/base/$pair" "$out/base.lingua"
build "$here/../tables/$studied" "$here/../tables/$pair" "$out/working.lingua"
node "$here/compare_packs.mjs" "$glue/lingua_wasm.js" "$glue/lingua_wasm_bg.wasm" \
  "$out/base.lingua" "$out/working.lingua" "$out/corpus.json" \
  "$base_studied/freq.tsv" "$here/../tables/$studied/freq.tsv" "$out/report.json"
echo "Details (top lemma changes, gains, losses): $out/report.json"
