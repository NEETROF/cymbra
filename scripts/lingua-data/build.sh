#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# Reproducible build of a Cymbra Lingua data pack (pin-lingua-pack-sources D4).
#
#   scripts/lingua-data/build.sh en-fr <out.lingua>            # Build: the committed tables
#   scripts/lingua-data/build.sh --reduce en-fr <out.lingua>   # Re-reduce: the pinned raw sources
#   scripts/lingua-data/build.sh --update en-fr <out.lingua>   # Update: today's raw sources
#   scripts/lingua-data/build.sh --dry en-fr <out.lingua>      # Update into a scratch folder
#   scripts/lingua-data/build.sh --testdata en-fr <out>        # the tiny fixture
#
# Build mode is what every release and every pull request runs: lingua-pack-build on
# tables/<pair>/, then the pack's sha256 against tables/<pair>/pin.json. It reads nothing from the
# network and needs no Python. The other modes reduce raw sources into tables again — the pinned
# bytes when the reduction rules change, today's bytes to take in upstream changes — and are run by
# a person, or by the lingua-pack-update workflow; they write tables/<pair>/ and pin.json, and the
# result reaches a release only through a pull request (see tables/<pair>/README.md).
#
# Raw sources and packs are never committed; the reduced tables are.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON="${LINGUA_PYTHON:-python3}"
TABLE_FILES=(forms.tsv freq.tsv gloss.tsv level.tsv mwe.tsv NOTICE manifest.json)

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# The pack's sha256 as pin.json records it — read without Python: the file is written by
# pack_sources.py, which puts `"sha256"` on its own line inside `"pack"`.
recorded_pack_sha256() {
  sed -n '/"pack": {/,/}/s/.*"sha256": *"\([0-9a-f]\{64\}\)".*/\1/p' "$1" | head -n1
}

build_pack() {
  local input="$1" out="$2"
  # The builder writes the file but not its folder, which a fresh checkout may lack.
  mkdir -p "$(dirname "$out")"
  cargo run --quiet --release -p lingua-pack --bin lingua-pack-build -- "$input" "$out"
}

# reduce <pair> <work> <snapshot>: the reducer over the raw sources in <work>, tables left in <work>.
reduce() {
  local pair="$1" work="$2" snapshot="$3"
  "$PYTHON" "$here/reduce-$pair.py" --work "$work" \
    --max-lemmas "${LINGUA_MAX_LEMMAS:-40000}" \
    --built-at "${snapshot//./-}" --pack-version "$snapshot"
}

copy_tables() {
  local from="$1" to="$2"
  mkdir -p "$to"
  for f in "${TABLE_FILES[@]}"; do
    if [[ -f "$from/$f" ]]; then cp "$from/$f" "$to/$f"; else rm -f "$to/$f"; fi
  done
}

mode=build
case "${1:-}" in
  --testdata | --reduce | --update | --dry) mode="${1#--}"; shift ;;
esac
pair="${1:?pair, e.g. en-fr}"
out="${2:?output path}"
tables="$here/tables/$pair"
pin="$tables/pin.json"
work="$here/work/$pair"

case "$mode" in
  testdata)
    build_pack "$here/testdata/$pair" "$out"
    ;;

  build)
    [[ -f "$pin" ]] || { echo "error: no committed tables for $pair ($pin)." >&2; exit 2; }
    build_pack "$tables" "$out"
    want="$(recorded_pack_sha256 "$pin")"
    got="$(sha256_of "$out")"
    if [[ "$got" != "$want" ]]; then
      echo "error: $out has sha256 $got, but the committed tables build $want (pin.json)." >&2
      echo "  The builder or a dependency changed the pack; if that is intended, update pack.sha256 in the same pull request." >&2
      exit 1
    fi
    echo "Built $out from the committed $pair tables (sha256 $got)."
    ;;

  reduce)
    # The reduction rules changed: the same raw bytes, reduced again. The diff is the rules alone.
    rm -rf "$work" && mkdir -p "$work"
    "$PYTHON" "$here/pack_sources.py" fetch-pinned --pin "$pin" --work "$work"
    snapshot="$("$PYTHON" "$here/pack_sources.py" get --pin "$pin" snapshot)"
    reduce "$pair" "$work" "$snapshot"
    copy_tables "$work" "$tables"
    build_pack "$tables" "$out"
    "$PYTHON" "$here/pack_sources.py" record-build --pin "$pin" --pack "$out" --reducer "$here/reduce-$pair.py"
    ;;

  update | dry)
    # Today's sources. A dry run reduces into a scratch copy and leaves the committed tables alone.
    snapshot="${LINGUA_SNAPSHOT:-$(date -u +%Y.%m.%d)}"
    if [[ "$mode" == dry ]]; then
      tables="${LINGUA_DRY_TABLES:-$here/work/dry/$pair}"
      rm -rf "$tables" && mkdir -p "$tables"
      [[ -f "$pin" ]] && cp "$pin" "$tables/pin.json"
      pin="$tables/pin.json"
    fi
    rm -rf "$work" && mkdir -p "$work"
    "$PYTHON" "$here/pack_sources.py" fetch-live --pin "$pin" --work "$work" --snapshot "$snapshot"
    reduce "$pair" "$work" "$snapshot"
    copy_tables "$work" "$tables"
    build_pack "$tables" "$out"
    "$PYTHON" "$here/pack_sources.py" record-build --pin "$pin" --pack "$out" --reducer "$here/reduce-$pair.py"
    echo "Reduced today's $pair sources into $tables (snapshot $snapshot)."
    ;;
esac
