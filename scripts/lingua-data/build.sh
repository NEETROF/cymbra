#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# Reproducible offline build of a Cymbra Lingua data pack.
#
#   scripts/lingua-data/build.sh <pair> <out.lingua>
#   scripts/lingua-data/build.sh en-fr dist/en-fr.lingua        # real sources
#   scripts/lingua-data/build.sh --testdata en-fr /tmp/t.lingua # tiny fixture
#   scripts/lingua-data/build.sh emit-manifest                  # refresh the committed registry
#   scripts/lingua-data/build.sh check-manifest                 # fail if it is stale
#
# The pack is NEVER committed; CI rebuilds it and caches it (see the reproducibility
# test in crates/lingua-pack). Raw sources are downloaded into a git-ignored work
# dir, dated, and reduced to the four TSV tables the builder consumes.
#
# `emit-manifest`/`check-manifest` produce/verify backend/lingua/packs-manifest.json —
# the read-only registry the Lingua ops console serves (change: add-lingua-back-office).
# It records each shippable pack's version/analyzer/pair/build-date/size/NOTICE; the
# build date is preserved when a pack's content is unchanged, so a rebuild diffs empty.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

if [[ "${1:-}" == "emit-manifest" || "${1:-}" == "check-manifest" ]]; then
  manifest="$here/../../backend/lingua/packs-manifest.json"
  # The currently shippable pack inputs. Only the en-fr testdata pack is wired in this
  # checkout; add real per-pair input dirs here as fetch_and_reduce lands them.
  pack_inputs=("$here/testdata/en-fr")
  mode_args=()
  [[ "$1" == "check-manifest" ]] && mode_args+=(--check)
  cargo run --quiet --release -p lingua-pack --bin lingua-pack-manifest -- \
    "${mode_args[@]}" --built-at "$(date -u +%F)" "$manifest" "${pack_inputs[@]}"
  exit 0
fi

# fetch_and_reduce: download the dated upstream sources into $2 and reduce them
# to forms.tsv / freq.tsv / gloss.tsv / NOTICE / manifest.json. Kept as a
# documented stub: wire the exact download URLs + reducers per pair here. It
# MUST only pull sources cleared by scripts/lingua-data/SOURCES.md (the licence
# denylist is also enforced in the builder, which fails the build on a denied
# source). Raw downloads go under work/ (git-ignored) and are never committed.
fetch_and_reduce() {
  local pair="$1"
  echo "error: real-source fetch for '$pair' is not wired in this checkout." >&2
  echo "       See scripts/lingua-data/SOURCES.md; run with --testdata to smoke-test." >&2
  exit 2
}

if [[ "${1:-}" == "--testdata" ]]; then
  pair="${2:?pair, e.g. en-fr}"; out="${3:?output path}"
  input="$here/testdata/$pair"
else
  pair="${1:?pair, e.g. en-fr}"; out="${2:?output path}"
  input="$here/work/$pair"
  fetch_and_reduce "$pair" "$input"
fi

cargo run --quiet --release -p lingua-pack --bin lingua-pack-build -- "$input" "$out"
