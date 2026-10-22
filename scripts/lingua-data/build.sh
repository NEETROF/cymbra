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
#   LINGUA_KEEP_EDITIONS=en scripts/lingua-data/build.sh --update fr-en <out.lingua>
#                                                     # … keeping the English edition's recorded bytes
#   scripts/lingua-data/build.sh --dry en-fr <out.lingua>      # Update into a scratch folder
#   scripts/lingua-data/build.sh --testdata en-fr <out>        # the tiny fixture
#   scripts/lingua-data/build.sh --parts en-fr <base.lingua> <layer.lingua>
#                                       # Build, as the package carries it: the base and the layer
#   scripts/lingua-data/build.sh --testdata --parts en-fr <base.lingua> <layer.lingua>
#                                       # … of the tiny fixture
#
# Build mode is what every release and every pull request runs: lingua-pack-build on
# tables/<pair>/ and its studied language's tables/<studied>/, then the pack's sha256 against
# tables/<pair>/pin.json. With --parts (ship-lingua-pack-bases-once D3), read after the mode's flag,
# the same tables are built as the studied language's base and the pair's layer — what a package
# carries — each checked against its own record of pin.json (`base`, `layer`). It reads nothing
# from the network and needs no Python. The other modes
# reduce raw sources into tables again — the pinned bytes when the reduction rules change, today's
# bytes to take in upstream changes — and are run by a person, or by the lingua-pack-update
# workflow; they write tables/<pair>/ and pin.json — and tables/<studied>/ when the pair is its
# studied language's reference (tables/<studied>/studied.json) — and the result reaches a release
# only through a pull request (see tables/<pair>/README.md).
#
# Raw sources and packs are never committed; the reduced tables are.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PYTHON="${LINGUA_PYTHON:-python3}"
# A reducer writes both sides of a pair's tables into its work folder; `pack_sources.py split`
# files them (split-lingua-pack-tables-by-language, M24): the native side into tables/<pair>/, the
# studied side into tables/<studied>/ only when the pair is that language's reference, with the
# dictionary words (`lexical.tsv`, which every reference's reducer writes: its glossed lemmas less
# those it glosses as names alone). The studied folder's inputs no
# reducer writes — the pinned tag pool `tags.tsv` and `studied.json` (pack_sources.py KEPT_INPUTS) —
# are never touched, so reducing a pair again keeps them; a dry run copies the studied folder into
# its scratch root.

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

# The pack's sha256 as pin.json records it — read without Python: the file is written by
# pack_sources.py, which puts `"sha256"` on its own line inside `"pack"`.
recorded_pack_sha256() {
  recorded_sha256 "$1" pack
}

# recorded_sha256 <pin> <record>: the sha256 of the `pack`, `base` or `layer` record, read the same
# way; empty when the pin has no such record.
recorded_sha256() {
  sed -n "/\"$2\": {/,/}/s/.*\"sha256\": *\"\([0-9a-f]\{64\}\)\".*/\1/p" "$1" | head -n1
}

# build_pack <studied dir> <pair dir> <out>; the testdata fixture holds both sides in one folder.
build_pack() {
  local studied="$1" input="$2" out="$3"
  # The builder writes the file but not its folder, which a fresh checkout may lack.
  mkdir -p "$(dirname "$out")"
  cargo run --quiet --release -p lingua-pack --bin lingua-pack-build -- --studied "$studied" "$input" "$out"
}

# build_parts <studied dir> <pair dir> <base> <layer>: the same inputs as build_pack, written as the
# studied language's base and the pair's layer (split-lingua-pack-format D8): neither is written
# when either fails.
build_parts() {
  local studied="$1" input="$2" base="$3" layer="$4"
  mkdir -p "$(dirname "$base")" "$(dirname "$layer")"
  cargo run --quiet --release -p lingua-pack --bin lingua-pack-build -- --split "$studied" "$input" "$base" "$layer"
}

# check_part <pin> <record> <file>: the part against its record, a mismatch named with its file.
check_part() {
  local pin="$1" record="$2" file="$3" want got
  want="$(recorded_sha256 "$pin" "$record")"
  got="$(sha256_of "$file")"
  if [[ -z "$want" ]]; then
    echo "error: $pair: tables/$pair/pin.json records no $record: record the parts the committed tables build" >&2
    echo "  (pack_sources.py record-parts --pin tables/$pair/pin.json --base <base> --layer <layer>)." >&2
    return 1
  fi
  if [[ "$got" != "$want" ]]; then
    echo "error: $pair: $file has sha256 $got, but the committed tables build the $record $want (tables/$pair/pin.json)." >&2
    echo "  The builder, a dependency or the tables changed the $record; if that is intended, record it in the same" >&2
    echo "  pull request (pack_sources.py record-parts, or reduce $pair again)." >&2
    return 1
  fi
}

# How many lemmas a pair keeps by default: Spanish keeps 60,000, for which the forms of the
# commonest lemmas pass the programme's gates (add-lingua-spanish-forms-tables D3) — every pair
# studying Spanish, since a pair that is not the reference reads the committed lemmas and caps
# them the same way (add-lingua-pack-es-en D1). French keeps 60,000 too, measured the same way
# (add-lingua-french-forms-tables D8), for every pair studying it.
max_lemmas() {
  case "$1" in
    es-* | fr-*) echo 60000 ;;
    *) echo 40000 ;;
  esac
}

# reduce <pair> <work> <snapshot> <pack version>: the reducer over the raw sources in <work>,
# tables left in <work>. The pack version is `pack_sources.py version`'s: the snapshot for an
# update, the snapshot and the rules after a re-reduction — and, for a pair that is not its studied
# language's reference, the studied tables it reads too, in either mode (add-lingua-pack-es-en D3).
# Such a pair reads the studied tables as committed (LINGUA_STUDIED, the studied folder of this
# run's root — a dry run's scratch copy); the reference's reducer, which writes them, ignores it.
reduce() {
  local pair="$1" work="$2" snapshot="$3" version="$4"
  LINGUA_STUDIED="$studied" "$PYTHON" "$here/reduce-$pair.py" --work "$work" \
    --max-lemmas "${LINGUA_MAX_LEMMAS:-$(max_lemmas "$pair")}" \
    --built-at "${snapshot//./-}" --pack-version "$version"
}

# file_sides <work> <tables root>: the reduction's tables, filed by side.
file_sides() {
  "$PYTHON" "$here/pack_sources.py" split --work "$1" --tables "$2" --pair "$pair"
}

mode=build
case "${1:-}" in
  --testdata | --reduce | --update | --dry) mode="${1#--}"; shift ;;
esac
# A pack's parts (ship-lingua-pack-bases-once D3): the flag read after the mode's, in the two modes
# that build from tables already reduced.
parts=0
if [[ "${1:-}" == --parts ]]; then
  if [[ "$mode" != build && "$mode" != testdata ]]; then
    echo "error: --parts builds from the committed tables or the testdata; --$mode builds and records the parts itself." >&2
    exit 2
  fi
  parts=1
  shift
fi
pair="${1:?pair, e.g. en-fr}"
out="${2:?output path}"
if [[ "$parts" == 1 ]]; then
  base="$out"
  layer="${3:?usage: build.sh [--testdata] --parts <pair> <base.lingua> <layer.lingua>}"
fi
root="$here/tables"
tables="$root/$pair"
studied="$root/${pair%%-*}"
pin="$tables/pin.json"
work="$here/work/$pair"
# Fetched release assets, kept by the sha256 of their decompressed bytes (add-lingua-pack-es-en
# D2): an asset two pins name is fetched once per run, and a pair reduced again on the same machine
# fetches nothing again. Outside work/<pair>, which a reduction removes.
cache="${LINGUA_CACHE:-$here/work/cache}"
# Each Wiktionary edition's catalogue, derived from its dump at the run's first read and copied by
# the later pairs of the run (migrate-lingua-pack-sources-to-raw-dumps D4): an update reads each
# dump once however many pairs read its edition. The run's own — keyed by edition and snapshot day,
# never meant for another run: remove it when the run ends (lingua-pack-update does). Outside
# work/<pair> too. On a laptop it outlives the run, one folder per edition and day: a folder of the
# same day counts only while its dump.json records today's address and catalogue and every file of
# it is there (pack_sources.py read_edition), and work/editions can be removed at any time — the
# next update fetches the dumps again.
editions="${LINGUA_EDITIONS:-$here/work/editions}"
# The editions an update keeps (add-lingua-fr-en-translated-expressions D6), comma-separated
# (`en`, `en,es`): each one's record of pin.json is carried byte for byte and its files read from the
# release it names, its dump not read (pack_sources.py fetch-live --keep-edition). For adding an
# edition to a pair without reading its others anew — fr-en gaining the French Wiktionary's
# translations, its English dump of 2026-10-03 no longer served —, never for avoiding an update: the
# next update that keeps nothing reads every edition anew.
keep=()
IFS=, read -r -a kept_editions <<<"${LINGUA_KEEP_EDITIONS:-}"
for edition in ${kept_editions[@]+"${kept_editions[@]}"}; do
  [[ -z "$edition" ]] || keep+=(--keep-edition "$edition")
done

case "$mode" in
  testdata)
    if [[ "$parts" == 1 ]]; then
      build_parts "$here/testdata/$pair" "$here/testdata/$pair" "$base" "$layer"
    else
      build_pack "$here/testdata/$pair" "$here/testdata/$pair" "$out"
    fi
    ;;

  build)
    [[ -f "$pin" ]] || { echo "error: no committed tables for $pair ($pin)." >&2; exit 2; }
    if [[ ! -f "$studied/studied.json" ]]; then
      echo "error: $pair studies ${pair%%-*}, but tables/${pair%%-*}/ holds no studied tables ($studied/studied.json)." >&2
      echo "  Add tables/${pair%%-*}/: the reduction of its reference pair writes it (build.sh --reduce <pair>)." >&2
      exit 2
    fi
    if [[ "$parts" == 1 ]]; then
      build_parts "$studied" "$tables" "$base" "$layer"
      failed=0
      check_part "$pin" base "$base" || failed=1
      check_part "$pin" layer "$layer" || failed=1
      [[ "$failed" == 0 ]] || exit 1
      echo "Built $base and $layer from the committed $pair tables (sha256 $(sha256_of "$base"), $(sha256_of "$layer"))."
      exit 0
    fi
    build_pack "$studied" "$tables" "$out"
    want="$(recorded_pack_sha256 "$pin")"
    got="$(sha256_of "$out")"
    if [[ "$got" != "$want" ]]; then
      echo "error: $pair: $out has sha256 $got, but the committed tables build $want (tables/$pair/pin.json)." >&2
      echo "  The builder, a dependency or its studied language's tables (tables/${pair%%-*}/) changed the pack; if that" >&2
      echo "  is intended, record it in the same pull request (update pack.sha256, or reduce $pair again)." >&2
      exit 1
    fi
    echo "Built $out from the committed $pair tables (sha256 $got)."
    ;;

  reduce)
    # The reduction rules changed: the same raw bytes, reduced again. The diff is the rules alone.
    rm -rf "$work" && mkdir -p "$work"
    "$PYTHON" "$here/pack_sources.py" fetch-pinned --pin "$pin" --work "$work" --cache "$cache"
    snapshot="$("$PYTHON" "$here/pack_sources.py" get --pin "$pin" snapshot)"
    # New tables from the same sources are a new dictionary: the version names the snapshot AND
    # the rules that reduced it (add-lingua-word-grammar, design D8) — reduce-<pair>.py and the
    # shared reduce_*.py modules it loads (pack_sources.py `rules`) — and, for a pair that is not
    # its studied language's reference, the studied tables it reads (`version`).
    version="$("$PYTHON" "$here/pack_sources.py" version --pin "$pin" --reducer "$here/reduce-$pair.py")"
    reduce "$pair" "$work" "$snapshot" "$version"
    file_sides "$work" "$root"
    build_pack "$studied" "$tables" "$out"
    # The parts the package carries, from the same tables, recorded beside the whole pack.
    build_parts "$studied" "$tables" "$work/base.lingua" "$work/layer.lingua"
    "$PYTHON" "$here/pack_sources.py" record-build --pin "$pin" --pack "$out" \
      --base "$work/base.lingua" --layer "$work/layer.lingua" --reducer "$here/reduce-$pair.py"
    ;;

  update | dry)
    # Today's sources. A dry run reduces into a scratch copy and leaves the committed tables alone.
    snapshot="${LINGUA_SNAPSHOT:-$(date -u +%Y.%m.%d)}"
    if [[ "$mode" == dry ]]; then
      # A scratch root laid out as tables/: the pair's folder and its studied language's, which the
      # reference overwrites and any other pair builds on.
      root="${LINGUA_DRY_ROOT:-$here/work/dry}"
      tables="$root/$pair"
      studied="$root/${pair%%-*}"
      rm -rf "$tables" "$studied" && mkdir -p "$tables"
      [[ -f "$pin" ]] && cp "$pin" "$tables/pin.json"
      "$PYTHON" "$here/pack_sources.py" keep --from "$here/tables/${pair%%-*}" --to "$studied"
      pin="$tables/pin.json"
    fi
    rm -rf "$work" && mkdir -p "$work"
    "$PYTHON" "$here/pack_sources.py" fetch-live --pin "$pin" --work "$work" --snapshot "$snapshot" --cache "$cache" --editions "$editions" \
      ${keep[@]+"${keep[@]}"}
    version="$("$PYTHON" "$here/pack_sources.py" version --pin "$pin" --reducer "$here/reduce-$pair.py" --live)"
    reduce "$pair" "$work" "$snapshot" "$version"
    file_sides "$work" "$root"
    build_pack "$studied" "$tables" "$out"
    build_parts "$studied" "$tables" "$work/base.lingua" "$work/layer.lingua"
    "$PYTHON" "$here/pack_sources.py" record-build --pin "$pin" --pack "$out" \
      --base "$work/base.lingua" --layer "$work/layer.lingua" --reducer "$here/reduce-$pair.py"
    echo "Reduced today's $pair sources into $tables (snapshot $snapshot)."
    ;;
esac
