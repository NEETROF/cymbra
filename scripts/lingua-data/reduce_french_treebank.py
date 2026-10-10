# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The part of speech UD French-GSD reads a French word as, and the gloss row it opens
(refine-lingua-fr-en-glosses D5).

A dictionary page orders a word's entries as its editors wrote them: `pas` opens on the noun « step »
before the negation, `son` on « sound » before « his, her », and a common noun can open on the place
spelled like it (`marche`, a department of France). UD French-GSD's training and development sections
— the text fr-en's pin already records — say which part of speech French uses a word as: a function
word's row, or a row the page opens on a proper noun, opens on the treebank's commonest part of speech
when the treebank reads it at least `TREEBANK_MIN` times and at least `TREEBANK_RATIO` times as often
as the part of speech the page opens on, and that part of speech is a function word's
(`TREEBANK_FIRST`) — or the page opens on a proper noun's. A proper noun is never moved first, and a
noun, a verb or an adjective moving ahead of another stays as the page writes it (`ferme` « firm »,
`mort` « dead », `devoir` « duty »): settled by the owner on 2026-10-10 (Q2).

A module of its own, not in `reduce-fr-en.py`, so that a pair glossing French in another language
(fr-es, refine-lingua-fr-es-glosses D8) reads the same rule without loading fr-en's reducer. It is a
rule file of every pair that imports it (`pack_sources.rule_files`): editing it re-pins those pairs —
fr-en's alone while fr-en alone imports it.
"""

import collections
import unicodedata

# The thresholds and the parts of speech the treebank may open a row on, measured on fr-en's rows
# (the design's *Measured*): 13 rows move, all of the top 10,000. `ADV` is measured in — it adds `pas`
# (981 adverbs against 8 nouns), `bien` and `juste`, each a word whose adverb is its commonest use.
TREEBANK_MIN = 10
TREEBANK_RATIO = 2
TREEBANK_FIRST = frozenset({"ADP", "DET", "PRON", "CCONJ", "SCONJ", "PART", "ADV"})
# The parts of speech UD and the dictionaries lemmatise alike, counted under their lemma; any other is
# counted under its own form (UD reads « ton », « leur », « mon » as forms of « son »).
OPEN_CLASSES = frozenset({"NOUN", "VERB", "ADJ", "PROPN"})


def word_key(text):
    """A word as the counts key it: lowercased, in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip().lower()).replace("’", "'")


def gsd_pos_counts(paths):
    """How often UD French-GSD reads each word under each part of speech: `(word, UPOS) → count`, the
    word as `word_key` writes it — a noun, verb, adjective or proper noun under its lemma, any other
    part of speech under its own form; the auxiliary counted as a verb (kaikki's verbs are `VERB`); a
    token inside a fixed expression (relation `fixed`: « conséquent » in « par conséquent ») counted
    for none. Multi-word tokens and empty nodes carry no part of speech."""
    counts = collections.Counter()
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for line in f:
                if not line.strip() or line.startswith("#"):
                    continue
                cols = line.rstrip("\n").split("\t")
                if len(cols) < 4 or "-" in cols[0] or "." in cols[0]:
                    continue
                if len(cols) > 7 and cols[7] == "fixed":
                    continue
                upos = "VERB" if cols[3] == "AUX" else cols[3]
                counts[(word_key(cols[2] if upos in OPEN_CLASSES else cols[1]), upos)] += 1
    return counts


def commonest_first(word, upos, first, counts):
    """The order of a headword's entries the treebank asks for, as indexes into `upos` — each entry's
    part of speech, in the page's order —, or None to keep the page's. `first` is the part of speech
    the page opens the row on (a caller may pass over an acronym's entry); `counts` are
    `gsd_pos_counts`'. The entries of the commonest part of speech come first, never a proper noun's,
    the others in their order; ties go to the part of speech the page writes first, so the order never
    depends on a set's."""
    if len(set(upos)) < 2:
        return None
    candidates = [u for u in dict.fromkeys(upos) if u != "PROPN"]
    if not candidates:
        return None
    best = max(candidates, key=lambda u: counts.get((word, u), 0))
    if best == first:
        return None
    n_best, n_first = counts.get((word, best), 0), counts.get((word, first), 0)
    if n_best < TREEBANK_MIN or n_best < TREEBANK_RATIO * n_first:
        return None
    if first != "PROPN" and best not in TREEBANK_FIRST:
        return None
    return [k for k, u in enumerate(upos) if u == best] + [k for k, u in enumerate(upos) if u != best]
