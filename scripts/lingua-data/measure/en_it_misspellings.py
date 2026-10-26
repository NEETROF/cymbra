#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""en-it's misspelling census (refine-lingua-en-it-glosses D12, task 1.7).

Lists the words of en-it's glosses, among the rows of the commonest English lemmas, that Italian's
committed forms (`tables/it/forms.tsv`, it-fr's) do not hold and that wordfreq's Italian list rates
under a threshold: the candidates a person reads for `CORRECTIONS` in `reduce_english_it.py`. Nearly
all are real Italian — rare words, regional or technical ones, names —, which is why no rule takes
them: a reduction never reads `tables/it/`, which en-it's pin does not record, so this runs outside
it, by hand, and decides nothing.

    python3 scripts/lingua-data/measure/en_it_misspellings.py [--top 5000] [--below 1.5] [--gloss <gloss.tsv>]

Its output is one line per word, `rank<TAB>lemma<TAB>word<TAB>Zipf`, then the counts. Needs wordfreq
(`requirements-reduce.txt`).
"""

import argparse
import os
import re
import sys

_HERE = os.path.dirname(os.path.abspath(__file__))
_TABLES = os.path.join(_HERE, "..", "tables")
WORD = re.compile(r"[^\W\d_]+")


def read_column(path, column=0):
    with open(path, encoding="utf-8") as f:
        return [line.rstrip("\n").split("\t")[column] for line in f if "\t" in line]


def census(glosses, ranks, forms, zipf, top, below):
    """`[(rank, lemma, word, Zipf)]`: each word of a gloss of the `top` commonest lemmas, lowercased,
    that `forms` does not hold and `zipf` rates under `below`, by rank."""
    found = []
    for lemma, gloss in glosses.items():
        rank = ranks.get(lemma)
        if rank is None or rank > top:
            continue
        for word in sorted({w.lower() for w in WORD.findall(gloss)}):
            if word not in forms and zipf(word) < below:
                found.append((rank, lemma, word, zipf(word)))
    return sorted(found)


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.split("\n\n")[0])
    ap.add_argument("--gloss", default=os.path.join(_TABLES, "en-it", "gloss.tsv"))
    ap.add_argument("--forms", default=os.path.join(_TABLES, "it", "forms.tsv"))
    ap.add_argument("--freq", default=os.path.join(_TABLES, "en", "freq.tsv"))
    ap.add_argument("--top", type=int, default=5000)
    ap.add_argument("--below", type=float, default=1.5)
    a = ap.parse_args(argv)
    from wordfreq import zipf_frequency

    with open(a.gloss, encoding="utf-8") as f:
        glosses = dict(line.rstrip("\n").split("\t", 1) for line in f if "\t" in line)
    ranks = {lemma: int(rank) for lemma, rank in zip(read_column(a.freq), read_column(a.freq, 1))}
    forms = {form.lower() for form in read_column(a.forms)}
    found = census(glosses, ranks, forms, lambda w: zipf_frequency(w, "it"), a.top, a.below)
    for rank, lemma, word, z in found:
        print(f"{rank}\t{lemma}\t{word}\t{z:.2f}")
    print(f"{len(found)} words of {len({f[1] for f in found})} rows of the {a.top:,} commonest", file=sys.stderr)


if __name__ == "__main__":
    main()
