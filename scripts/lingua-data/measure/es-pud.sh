#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# The es-fr pack, built from the committed tables, measured on UD Spanish-PUD with the real analyser
# (add-lingua-spanish-forms-tables D5): tokens resolved, content-word and auxiliary lemmas, each held
# to the programme's gate. PUD is fetched at a pinned commit and checked by sha256; it is never
# committed (CC BY-SA 3.0), and the reduction never reads it.
#
#   scripts/lingua-data/measure/es-pud.sh
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PUD_COMMIT=818a82b8628c9cbec78750c7e83ccba34b9ce22b
PUD_SHA256=48a7b5c7f409100b24eba90c7397e01f6fecaaf3207315a9a1bda029f77e98d3
work="$here/../work/measure-es"
pud="$work/es_pud-ud-test.conllu"

sha256_of() {
  if command -v sha256sum >/dev/null; then sha256sum "$1" | cut -d' ' -f1; else shasum -a 256 "$1" | cut -d' ' -f1; fi
}

mkdir -p "$work"
if [[ ! -f "$pud" ]]; then
  curl -sSL --fail --retry 3 -o "$pud" \
    "https://raw.githubusercontent.com/UniversalDependencies/UD_Spanish-PUD/$PUD_COMMIT/es_pud-ud-test.conllu"
fi
if [[ "$(sha256_of "$pud")" != "$PUD_SHA256" ]]; then
  echo "error: $pud is not UD Spanish-PUD at $PUD_COMMIT (sha256 differs)." >&2
  exit 1
fi
bash "$here/../build.sh" es-fr "$work/es-fr.lingua"
cargo run --quiet --release -p lingua-pack --bin lingua-pack-measure -- "$work/es-fr.lingua" "$pud"
