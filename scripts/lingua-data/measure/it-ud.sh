#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# The it-fr pack, built from the committed tables, measured with the real analyser — Italian's
# pre-pass and cascade (add-lingua-italian-analysis D6, add-lingua-italian-forms-tables D8): on UD
# Italian-PUD, held to Italian's gates — Spanish's 98.5 % of words resolved and 97 % of auxiliary
# lemmas, and 90.5 % of content lemmas, the owner's gate of 2026-10-23 (`MIN_CONTENT` below: MarkIT's
# counts are about sixteen times thinner than GSD's, UD reads a participle as its verb where the
# dictionary keeps the noun or the adjective, and no measured variant reached Spanish's 93.5 %) —,
# and on MarkIT's test section, reported beside it and not gated —
# the reduction reads MarkIT's training and development sections, so a held-out section of the same
# treebank is the weaker test, and its lemmas are automatic with corrections. Both are fetched at a
# pinned commit and checked by sha256; neither is committed (PUD is CC BY-SA 3.0, MarkIT CC BY 4.0),
# and the reduction never reads them. The exit status is PUD's.
#
# Italian's tables are committed by add-lingua-italian-forms-tables (change 9), its first update of
# it-fr writing them: until then there is no it-fr pack to build, and the script says so.
# LINGUA_IT_PACK=<pack.lingua> measures a pack built elsewhere instead — the prototype
# add-lingua-italian-analysis measured its cascade on, or change 9's tables reduced locally.
#
#   scripts/lingua-data/measure/it-ud.sh
#   LINGUA_IT_PACK=/path/to/it-fr.lingua scripts/lingua-data/measure/it-ud.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
# UD Italian-PUD, the commit the Italian marks corpus pins (apps/lingua-extension/tool/marks/pud.mjs).
PUD_COMMIT=71d932aa096368331ba2fd9bc906ec8dd853da8e
PUD_SHA256=ad5d302bdfd05194155d9ad43dc54c3cb915076d66b3eb6ef33eca9a11f1372a
# UD Italian-MarkIT at the commit it-fr's reduction reads its other two sections at
# (pack_sources.py PINNED["it-fr"]), the default branch's head of 2026-10-23.
MARKIT_COMMIT=e210079df0aea331ec1dcaa90648d815ce94c3ce
MARKIT_TEST_SHA256=9048ed0064e3989e2c37377755a28c25c7e462b1ad916e5808dc0a7ce95fc3fc
work="$here/../work/measure-it"
# Italian's content-word gate (add-lingua-italian-forms-tables D8), the owner's; the other two are the
# programme's, lingua-pack-measure's own.
MIN_CONTENT=90.5
pud="$work/it_pud-ud-test.conllu"
markit="$work/it_markit-ud-test.conllu"

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

pack="${LINGUA_IT_PACK:-$work/it-fr.lingua}"
if [[ -z "${LINGUA_IT_PACK:-}" && ! -f "$here/../tables/it-fr/pin.json" ]]; then
  echo "error: Italian's tables are not committed yet (tables/it-fr/, add-lingua-italian-forms-tables);" >&2
  echo "  this measures them once they are, or a pack named by LINGUA_IT_PACK." >&2
  exit 2
fi

mkdir -p "$work"
fetch_checked "$pud" \
  "https://raw.githubusercontent.com/UniversalDependencies/UD_Italian-PUD/$PUD_COMMIT/it_pud-ud-test.conllu" "$PUD_SHA256"
fetch_checked "$markit" \
  "https://raw.githubusercontent.com/UniversalDependencies/UD_Italian-MarkIT/$MARKIT_COMMIT/it_markit-ud-test.conllu" "$MARKIT_TEST_SHA256"
if [[ -z "${LINGUA_IT_PACK:-}" ]]; then
  bash "$here/../build.sh" it-fr "$pack"
fi
measure() {
  cargo run --quiet --release -p lingua-pack --bin lingua-pack-measure -- --min-content "$MIN_CONTENT" "$pack" "$1"
}
status=0
echo "UD Italian-PUD at $PUD_COMMIT (gated):"
measure "$pud" || status=$?
echo "MarkIT's test section at $MARKIT_COMMIT (reported, not gated):"
measure "$markit" || echo "  (reported only: MarkIT's test section decides nothing)"
exit "$status"
