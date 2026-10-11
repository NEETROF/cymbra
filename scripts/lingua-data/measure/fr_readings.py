#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""French's readings measured on held-out treebanks (add-lingua-french-grammar-tables D10).

For each part of speech, over the words whose form the committed tables map to the treebank's
lemma: the share that carry a reading of their own (`grammar.tsv`, marked `-` — what the card
shows), and the share of those whose treebank part of speech and features are among their readings.
An auxiliary counts as a verb; the conditional's and the imperative's tense is left aside, as the
tables write neither (design D3, D7); a participle without a tense, as UD French-GSD's recent
releases write it, is read either way. The reduction never reads either treebank, and the report
decides nothing: a treebank's own errors would otherwise fail an update.

    fr_readings.py --tables scripts/lingua-data/tables/fr <treebank.conllu>...

Run by `fr-ud.sh` after its gates, over the two files it fetched.
"""

import argparse
import collections
import os
import sys
import unicodedata

# The groups reported, in order: a verb by its form, then the nominal parts of speech.
GROUPS = (
    ("finite verbs", "VERB fin"),
    ("participles", "VERB part"),
    ("infinitives", "VERB inf"),
    ("nouns", "NOUN"),
    ("adjectives", "ADJ"),
    ("determiners", "DET"),
    ("pronouns", "PRON"),
)
MEASURED = frozenset({"VERB", "AUX", "NOUN", "ADJ", "DET", "PRON"})
# The features a reading names of a verb's form: one the treebank does not name may not differ.
_CORE = ("Mood", "Tense", "Person", "Number", "VerbForm")


def nfc_lower(text):
    """A word lowercased and in NFC, the typographic apostrophe read as `'`, as the reducer reads it."""
    return unicodedata.normalize("NFC", (text or "").strip().lower()).replace("’", "'")


def parse_tag(tag):
    upos, *features = tag.split("|")
    return upos, dict(feature.split("=", 1) for feature in features)


def read_tables(folder):
    """`form → lemma` and `form → [(upos, features)]`, the form's own readings, from the committed
    `forms.tsv` and `grammar.tsv`."""
    forms, own = {}, collections.defaultdict(list)
    with open(os.path.join(folder, "forms.tsv"), encoding="utf-8") as f:
        for line in f:
            form, lemma = line.rstrip("\n").split("\t")
            forms[form] = lemma
    with open(os.path.join(folder, "grammar.tsv"), encoding="utf-8") as f:
        for line in f:
            form, _, tag, mark = line.rstrip("\n").split("\t")
            if mark == "-":
                own[form].append(parse_tag(tag))
    return forms, own


def project(upos, features):
    """A treebank word's part of speech and features, as the tables write them: a verb or an
    auxiliary is `VERB`; a finite verb its mood, tense, person and number, with no tense on the
    conditional or the imperative; a participle its tense, and a past participle's agreement; a
    nominal its gender and number. Nothing the readings' vocabulary does not hold."""
    keep = {}
    if upos in ("VERB", "AUX"):
        verb_form = features.get("VerbForm")
        if verb_form == "Fin":
            keep = {k: features[k] for k in ("Mood", "Tense", "Person", "Number") if k in features}
            if keep.get("Mood") in ("Cnd", "Imp"):
                keep.pop("Tense", None)
            keep["VerbForm"] = "Fin"
        elif verb_form == "Part":
            keep = {"VerbForm": "Part", "Tense": features.get("Tense")}
            if features.get("Tense") in ("Past", None):
                keep.update((k, features[k]) for k in ("Gender", "Number") if k in features)
        elif verb_form == "Inf":
            keep = {"VerbForm": "Inf"}
        return "VERB", keep
    return upos, {k: features[k] for k in ("Gender", "Number") if k in features}


def group_of(upos, want):
    if upos != "VERB":
        return upos
    return {"Fin": "VERB fin", "Part": "VERB part", "Inf": "VERB inf"}.get(want.get("VerbForm"), "VERB other")


def agrees(want, readings):
    """Whether a reading names every feature the treebank gives — a gender only when the reading
    names one — and no verb feature the treebank contradicts."""
    return any(
        all(features.get(k) == v for k, v in want.items() if k != "Gender" or "Gender" in features)
        and all(want.get(k, v) == v for k, v in features.items() if k in _CORE)
        for features in readings
    )


def measure(lines, forms, own):
    """`(group, what) → count` over a treebank's lines: `words`, `read`, `agree`."""
    counts = collections.Counter()
    for line in lines:
        if not line.strip() or line.startswith("#"):
            continue
        cols = line.rstrip("\n").split("\t")
        # Multi-word tokens (`du` = 1-2) and empty nodes (8.1) are no words of their own.
        if len(cols) < 6 or "-" in cols[0] or "." in cols[0]:
            continue
        form, lemma, upos, feats = cols[1], cols[2], cols[3], cols[5]
        if upos not in MEASURED:
            continue
        word = nfc_lower(form)
        if forms.get(word) != nfc_lower(lemma):
            continue
        features = dict(f.split("=", 1) for f in feats.split("|")) if feats != "_" else {}
        pos, want = project(upos, features)
        group = group_of(pos, want)
        if group == "VERB part" and want.get("Tense", "") is None:
            del want["Tense"]
        counts[group, "words"] += 1
        readings = own.get(word, [])
        if not readings:
            continue
        counts[group, "read"] += 1
        if agrees(want, [f for p, f in readings if p == pos]):
            counts[group, "agree"] += 1
    return counts


def report(name, counts):
    """The figures, one line per group the treebank holds."""
    out = [f"French readings on {name} (reported, not gated):"]
    for label, group in GROUPS:
        words = counts[group, "words"]
        if not words:
            continue
        read = counts[group, "read"]
        out.append(
            f"  {label:<12} {words:>6,} words  {100 * read / words:6.2f} % read  "
            f"{100 * counts[group, 'agree'] / max(1, read):6.2f} % agree"
        )
    return out


def main(argv=None):
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("--tables", required=True, help="the studied folder, tables/fr")
    ap.add_argument("treebanks", nargs="+")
    a = ap.parse_args(argv)
    forms, own = read_tables(a.tables)
    for path in a.treebanks:
        with open(path, encoding="utf-8") as f:
            print("\n".join(report(os.path.basename(path), measure(f, forms, own))))
    return 0


if __name__ == "__main__":
    sys.exit(main())
