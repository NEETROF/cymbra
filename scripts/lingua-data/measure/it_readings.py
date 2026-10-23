#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Italian's readings measured on held-out treebanks (add-lingua-italian-grammar-tables D9).

For each part of speech, over the words whose form the committed tables map to the treebank's
lemma: the share that carry a reading of their own (`grammar.tsv`, marked `-` — what the card
shows), and the share of those whose treebank part of speech and features are among their readings.
An auxiliary counts as a verb; the conditional's and the imperative's tense is left aside, as the
tables write neither (design D3, D7); a present participle reads its number, a past one its gender
and number, a gerund nothing more, as UD Italian-MarkIT writes them. UD Italian-PUD writes its verbs
otherwise, and is read as MarkIT is: no `VerbForm` — a finite verb has a mood, a participle, an
infinitive and a gerund are told by their `XPOS` (`VBN`, `VB`, `VBG`) —, and the imperfetto as
`Aspect=Imp|Tense=Past` (MarkIT's `Tense=Imp`), the passato remoto `Aspect=Perf|Tense=Past`, the
imperfect subjunctive `Mood=Sub|Tense=Past` (MarkIT's `Tense=Imp`: the only simple past it has); a PUD
participle often names no agreement, and is then compared on its tense alone. The reduction never
reads either treebank, and the report decides nothing: a treebank's own errors would otherwise fail
an update.

    it_readings.py --tables scripts/lingua-data/tables/it <treebank.conllu>...

Run by `it-ud.sh` after its gates, over the two files it fetched: UD Italian-PUD and MarkIT's test
section. Before Italian's readings are committed (`grammar.tsv` absent), it says so and reports
nothing.
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
    ("gerunds", "VERB ger"),
    ("nouns", "NOUN"),
    ("adjectives", "ADJ"),
    ("determiners", "DET"),
    ("pronouns", "PRON"),
)
MEASURED = frozenset({"VERB", "AUX", "NOUN", "ADJ", "DET", "PRON"})
# The features a reading names of a verb's form: one the treebank does not name may not differ.
_CORE = ("Mood", "Tense", "Person", "Number", "VerbForm")
# UD Italian-PUD's verb forms, which it writes as an `XPOS` rather than a `VerbForm` feature.
_PUD_VERB_FORMS = {"VBN": "Part", "VB": "Inf", "VBG": "Ger"}


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


def verb_form(features, xpos=None):
    """A treebank verb's form: its `VerbForm`, else — UD Italian-PUD's way — `Fin` when it has a mood,
    else what its `XPOS` says (`_PUD_VERB_FORMS`), else None."""
    if "VerbForm" in features:
        return features["VerbForm"]
    if "Mood" in features:
        return "Fin"
    return _PUD_VERB_FORMS.get(xpos)


def project(upos, features, xpos=None):
    """A treebank word's part of speech and features, as the tables write them: a verb or an
    auxiliary is `VERB`; a finite verb its mood, tense, person and number, with no tense on the
    conditional or the imperative, PUD's `Aspect=Imp|Tense=Past` the imperfect; a present participle
    its tense and number, a past one its gender and number too; an infinitive or a gerund its verb
    form alone; a nominal its gender and number. Nothing the readings' vocabulary does not hold."""
    keep = {}
    if upos in ("VERB", "AUX"):
        form = verb_form(features, xpos)
        if form == "Fin":
            keep = {k: features[k] for k in ("Mood", "Tense", "Person", "Number") if k in features}
            if keep.get("Tense") == "Past" and (features.get("Aspect") == "Imp" or keep.get("Mood") == "Sub"):
                keep["Tense"] = "Imp"
            if keep.get("Mood") in ("Cnd", "Imp"):
                keep.pop("Tense", None)
            keep["VerbForm"] = "Fin"
        elif form == "Part":
            keep = {"VerbForm": "Part", **{k: features[k] for k in ("Tense", "Number") if k in features}}
            if features.get("Tense") != "Pres" and "Gender" in features:
                keep["Gender"] = features["Gender"]
        elif form in ("Inf", "Ger"):
            keep = {"VerbForm": form}
        return "VERB", keep
    return upos, {k: features[k] for k in ("Gender", "Number") if k in features}


def group_of(upos, want):
    if upos != "VERB":
        return upos
    return {"Fin": "VERB fin", "Part": "VERB part", "Inf": "VERB inf", "Ger": "VERB ger"}.get(
        want.get("VerbForm"), "VERB other"
    )


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
        # Multi-word tokens (`della` = 1-2) and empty nodes (8.1) are no words of their own.
        if len(cols) < 6 or "-" in cols[0] or "." in cols[0]:
            continue
        form, lemma, upos, xpos, feats = cols[1], cols[2], cols[3], cols[4], cols[5]
        if upos not in MEASURED:
            continue
        word = nfc_lower(form)
        if forms.get(word) != nfc_lower(lemma):
            continue
        features = dict(f.split("=", 1) for f in feats.split("|")) if feats != "_" else {}
        pos, want = project(upos, features, xpos)
        group = group_of(pos, want)
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
    out = [f"Italian readings on {name} (reported, not gated):"]
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
    ap.add_argument("--tables", required=True, help="the studied folder, tables/it")
    ap.add_argument("treebanks", nargs="+")
    a = ap.parse_args(argv)
    if not os.path.isfile(os.path.join(a.tables, "grammar.tsv")):
        print(f"Italian readings: {a.tables} holds no grammar.tsv yet (add-lingua-italian-grammar-tables); nothing to report.")
        return 0
    forms, own = read_tables(a.tables)
    for path in a.treebanks:
        with open(path, encoding="utf-8") as f:
            print("\n".join(report(os.path.basename(path), measure(f, forms, own))))
    return 0


if __name__ == "__main__":
    sys.exit(main())
