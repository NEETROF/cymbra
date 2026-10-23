# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""UD Italian-MarkIT's training and development sections, read for Italian's tables
(add-lingua-italian-forms-tables D2, D5, D6).

No Italian treebank of GSD's size is licence-clean (the programme's I11): UD Italian-ISDT, VIT,
ParTUT and PoSTWITA are non-commercial, and the two clean ones are small — UD Italian-PUD (1,000
sentences, CC BY-SA 3.0) and MarkIT (≈ 1,300 sentences of students' essays, CC BY 4.0, its lemmas
automatic with corrections). PUD is the measurement's gate and MarkIT's test section its report, so
the reduction reads MarkIT's two other sections alone, as fr-en reads UD French-GSD's
(`reduce-fr-en.py`, `reduce_french_treebank.py`):

- how often each form stands for each lemma (`markit_counts`), the second of the rules that choose a
  form's one lemma (D5);
- how often each hyphenated lemma occurs (`markit_zipf`), the evidence a hyphenated word is ranked by
  (D6);
- how often each word is read under each part of speech (`markit_pos_counts`), as
  `reduce_french_treebank.gsd_pos_counts` reads GSD's: the part of speech a gloss row opens on, which
  Italian's glosses (change 16) read.

A test section is never read here: `held_out` refuses a file named as one (`…-ud-test.conllu`), which
is how both held-out files are named — PUD has no other section, and MarkIT's test section is the
measurement's. A module of its own, so that a later pair studying Italian reads the same counts
without loading it-fr's reducer; it is a rule file of every pair that imports it
(`pack_sources.rule_files`).
"""

import collections
import math
import os
import unicodedata

# The parts of speech UD and the dictionaries lemmatise alike, counted under their lemma; any other is
# counted under its own form (`reduce_french_treebank.OPEN_CLASSES`).
OPEN_CLASSES = frozenset({"NOUN", "VERB", "ADJ", "PROPN"})
# How a held-out section's file ends: UD names every test section so, and PUD is a test section alone.
HELD_OUT_SUFFIX = "-ud-test.conllu"


def word_key(text):
    """A word as the counts key it: lowercased, in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip().lower()).replace("’", "'")


def held_out(path):
    """Refuses a held-out section (D2, D8): the reduction never reads PUD or MarkIT's test section,
    which the measurement holds out. Answers the path."""
    if os.path.basename(path).endswith(HELD_OUT_SUFFIX):
        raise ValueError(f"{os.path.basename(path)} is a held-out test section: the reduction never reads it")
    return path


def syntactic_words(paths):
    """Each syntactic word of the sections, as its CoNLL-U columns: multi-word tokens (`del` = 4-5)
    and empty nodes (8.1) carry no lemma of their own and are skipped."""
    for path in paths:
        with open(held_out(path), encoding="utf-8") as f:
            for line in f:
                if not line.strip() or line.startswith("#"):
                    continue
                cols = line.rstrip("\n").split("\t")
                if len(cols) < 4 or "-" in cols[0] or "." in cols[0]:
                    continue
                yield cols


def markit_counts(paths):
    """How often each `(form, lemma)` pair stands in the sections, both as `word_key` writes them."""
    counts = collections.Counter()
    for cols in syntactic_words(paths):
        counts[(word_key(cols[1]), word_key(cols[2]))] += 1
    return counts


def markit_zipf(counts):
    """Each hyphenated lemma the sections attest, by its own frequency there, as a Zipf value (its
    count per billion words, in log10) — the evidence a hyphenated word is ranked by (D6)."""
    total = sum(counts.values())
    per_lemma = collections.Counter()
    for (_, lemma), n in counts.items():
        if "-" in lemma:
            per_lemma[lemma] += n
    return {lemma: math.log10(n / total * 1e9) for lemma, n in per_lemma.items()}


def markit_pos_counts(paths):
    """How often the sections read each word under each part of speech: `(word, UPOS) → count`, a
    noun, verb, adjective or proper noun under its lemma, any other part of speech under its own form;
    an auxiliary counted as a verb; a token inside a fixed expression (relation `fixed`) counted for
    none — `reduce_french_treebank.gsd_pos_counts`' rules, over MarkIT."""
    counts = collections.Counter()
    for cols in syntactic_words(paths):
        if len(cols) > 7 and cols[7] == "fixed":
            continue
        upos = "VERB" if cols[3] == "AUX" else cols[3]
        counts[(word_key(cols[2] if upos in OPEN_CLASSES else cols[1]), upos)] += 1
    return counts
