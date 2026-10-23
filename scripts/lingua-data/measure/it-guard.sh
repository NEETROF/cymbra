#!/usr/bin/env bash
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
#
# Italian's detection guard, measured (add-lingua-italian-detection-guard, task 1.1; D2, D3, D6):
# fetches the samples it-guard-corpus.json pins — every file checked by sha256, every source in a
# directory of its own under DATA — then reads them offline with lingua-core's own gate and prints
# the share of each neighbour's Wikipedia blocks read as Italian, with and without the guard, and
# every block of the Italian corpora the guard refuses, with its markers. The exit status is the
# recall gate's: at most one Italian block in a thousand refused, none of pages-it.txt. Nothing it
# fetches is committed, and nothing in DATA is run: `python3 -I` reads it as text.
#
#   scripts/lingua-data/measure/it-guard.sh [DATA]   # DATA: scripts/lingua-data/work/measure-it-guard
#
# `it_guard.py words` measures D3's candidate words one by one on the same samples.
set -euo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
data="${1:-$here/../work/measure-it-guard}"
mkdir -p "$data"
python3 -I "$here/it_guard.py" fetch "$here/it-guard-corpus.json" "$data"
python3 -I "$here/it_guard.py" measure "$here/it-guard-corpus.json" "$data" "$(cd "$here/../../.." && pwd)"
