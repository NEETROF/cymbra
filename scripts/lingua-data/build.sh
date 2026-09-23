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
#
# The pack is NEVER committed; CI rebuilds it and caches it (see the reproducibility
# test in crates/lingua-pack). Raw sources are downloaded into a git-ignored work
# dir, dated, and reduced to the four TSV tables the builder consumes.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

# fetch_and_reduce: download the dated upstream sources into $2 (work dir) and reduce
# them to forms.tsv / freq.tsv / gloss.tsv / NOTICE / manifest.json. Only sources
# cleared by scripts/lingua-data/SOURCES.md are pulled (the licence denylist is also
# enforced in the builder). Raw downloads go under work/ (git-ignored), never committed;
# an already-downloaded snapshot is reused so a rebuild does not re-fetch.
# Override the scope/version with LINGUA_MAX_LEMMAS / LINGUA_PACK_VERSION.
fetch_and_reduce() {
  local pair="$1" work="$2"
  case "$pair" in
    en-fr)
      mkdir -p "$work"
      # AGID inflection database (permissive) — form -> lemma. ~3.4 MB.
      [[ -f "$work/agid-infl.txt" ]] || curl -sSL --fail -o "$work/agid-infl.txt" \
        "https://raw.githubusercontent.com/en-wl/wordlist/master/agid/infl.txt"
      # kaikki frwiktionary "Anglais" extract (CC BY-SA + GFDL) — FR glosses of EN words. ~188 MB.
      [[ -f "$work/kaikki-Anglais.jsonl" ]] || curl -sSL --fail --compressed -o "$work/kaikki-Anglais.jsonl" \
        "https://kaikki.org/frwiktionary/Anglais/kaikki.org-dictionary-Anglais.jsonl"
      # wordfreq (CC BY-SA) — the package IS the frequency source.
      python3 -c "import wordfreq" 2>/dev/null || pip3 install --user --quiet wordfreq
      # CEFR levels: CEFR-J Wordlist v1.5 (A1-B2, commercial OK + citation) and
      # Octanove Vocabulary Profile C1/C2 v1.0 (C1-C2, CC BY-SA 4.0), both from the
      # Open Language Profiles repo. ~0.4 MB together. Absence => no level.tsv.
      [[ -f "$work/cefrj-vocabulary-profile-1.5.csv" ]] || curl -sSL --fail -o "$work/cefrj-vocabulary-profile-1.5.csv" \
        "https://raw.githubusercontent.com/openlanguageprofiles/olp-en-cefrj/master/cefrj-vocabulary-profile-1.5.csv"
      [[ -f "$work/octanove-vocabulary-profile-c1c2-1.0.csv" ]] || curl -sSL --fail -o "$work/octanove-vocabulary-profile-c1c2-1.0.csv" \
        "https://raw.githubusercontent.com/openlanguageprofiles/olp-en-cefrj/master/octanove-vocabulary-profile-c1c2-1.0.csv"
      python3 "$here/reduce-en-fr.py" --work "$work" \
        --max-lemmas "${LINGUA_MAX_LEMMAS:-40000}" \
        --built-at "$(date -u +%F)" --pack-version "${LINGUA_PACK_VERSION:-1.0.0}"
      ;;
    *)
      echo "error: real-source fetch for '$pair' is not wired (see SOURCES.md)." >&2
      exit 2 ;;
  esac
}

if [[ "${1:-}" == "--testdata" ]]; then
  pair="${2:?pair, e.g. en-fr}"; out="${3:?output path}"
  input="$here/testdata/$pair"
else
  pair="${1:?pair, e.g. en-fr}"; out="${2:?output path}"
  input="$here/work/$pair"
  fetch_and_reduce "$pair" "$input"
fi

# The builder writes the file but not its folder, which a fresh checkout may lack
# (apps/lingua-extension/assets/ holds only the git-ignored pack).
mkdir -p "$(dirname "$out")"
cargo run --quiet --release -p lingua-pack --bin lingua-pack-build -- "$input" "$out"
