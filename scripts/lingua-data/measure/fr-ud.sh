#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# The fr-en pack, built from the committed tables, measured with the real analyser
# (add-lingua-french-forms-tables D9): on UD French-PUD, held to the programme's gates (Spanish's:
# 98.5 % of words resolved, 93.5 % of content lemmas, 97 % of auxiliary lemmas), and on UD
# French-GSD's test section, reported beside it and not gated — the reduction reads GSD's training
# and development sections, so a held-out section of the same treebank is the weaker test. Both are
# fetched at a pinned commit and checked by sha256; neither is committed (PUD is CC BY-SA 3.0), and
# the reduction never reads them. The exit status is PUD's.
#
#   scripts/lingua-data/measure/fr-ud.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# UD French-PUD, the default branch's head of 2026-05-06.
PUD_COMMIT=db260db10fe728853c549760801229ef4e7b16e1
PUD_SHA256=4dfed37b83d76e77fd2e9963d0be00d723e9a010e7e2a746f8b7640c48063c10
# UD French-GSD at the commit fr-en's reduction reads its other two sections at
# (pack_sources.py PINNED["fr-en"]).
GSD_COMMIT=94d5b68e185fc22a9ef292040e84f476d36d9b0e
GSD_TEST_SHA256=eee5a599b429658b6ee8582fae9993eb07161247fc64bad57d16b2050ed4eb1a
work="$here/../work/measure-fr"
pud="$work/fr_pud-ud-test.conllu"
gsd="$work/fr_gsd-ud-test.conllu"

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# fetch_checked <file> <url> <sha256>: the file, fetched once, and refused unless it is those bytes.
fetch_checked() {
  local file="$1" url="$2" want="$3"
  if [[ ! -f "$file" ]]; then
    curl -sSL --fail --retry 3 -o "$file.part" "$url"
    mv "$file.part" "$file"
  fi
  if [[ "$(sha256_of "$file")" != "$want" ]]; then
    echo "error: $file is not $url (sha256 differs)." >&2
    exit 1
  fi
}

mkdir -p "$work"
fetch_checked "$pud" \
  "https://raw.githubusercontent.com/UniversalDependencies/UD_French-PUD/$PUD_COMMIT/fr_pud-ud-test.conllu" "$PUD_SHA256"
fetch_checked "$gsd" \
  "https://raw.githubusercontent.com/UniversalDependencies/UD_French-GSD/$GSD_COMMIT/fr_gsd-ud-test.conllu" "$GSD_TEST_SHA256"
bash "$here/../build.sh" fr-en "$work/fr-en.lingua"
measure() {
  cargo run --quiet --release -p lingua-pack --bin lingua-pack-measure -- "$work/fr-en.lingua" "$1"
}
status=0
echo "UD French-PUD at $PUD_COMMIT (gated):"
measure "$pud" || status=$?
echo "UD French-GSD's test section at $GSD_COMMIT (reported, not gated):"
measure "$gsd" || echo "  (reported only: GSD's test section decides nothing)"
exit "$status"
