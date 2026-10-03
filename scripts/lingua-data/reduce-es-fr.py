#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the es-fr pack into its tables (add-lingua-spanish-forms-tables).

Inputs, in `--work`:
- `kaikki-Spanish.jsonl`: kaikki's extract of the English Wiktionary, Spanish section. A lemma's
  entry lists its inflections, each with tags; a form's own entry points at its lemma (`form_of`).
- `es_gsd-ud-train.conllu`, `es_gsd-ud-dev.conllu`: UD Spanish-GSD, read for one thing only — how
  often each form stands for each lemma — to choose one lemma for a form of several (design D2).
- wordfreq `es` (the installed, pinned package): the 60,000 commonest lemmas and which forms are
  attested at all (design D3).

Outputs, in `--work`: `forms.tsv`, `freq.tsv`, an empty `gloss.tsv` (the French glosses are
add-lingua-spanish-gloss-tables), `NOTICE` and `manifest.json`.

Every rule here is Spanish: the shared rules of `reduce_common.py` are used as they are, and that
module is not edited, so the en-fr tables' rule set does not move.
"""

import argparse
import collections
import json
import os
import re
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")

# A Spanish word, whole: letters, accents, ü, ñ, and hyphenated compounds of them.
_TOKEN = re.compile(r"[a-záéíóúüñ]+(?:-[a-záéíóúüñ]+)*")

# The bookkeeping of kaikki's inflection tables, never forms: the class, the template, the table's
# own tags. `combined-form` marks a verb with its clitic pronouns, which the analyser's enclitic
# rule resolves (add-lingua-spanish-analysis D3) — 519,030 forms the table does not need. A combined
# form's own entry says the same through its sense's tags, which name the pronoun
# (`object-third-person`, `object-plural`).
_BOOKKEEPING = frozenset({"table-tags", "inflection-template", "class", "romanization"})
_COMBINED = "combined-form"
_CLITIC_TAG = "object-"

ES = common.Studied(
    code="es",
    token=_TOKEN,
    # Not read by the forms reduction; kept for the shared rules' interface.
    form_of_target=re.compile(r"([a-záéíóúüñ]+)\s*\.?$"),
    coordinators=frozenset({"y", "e", "o", "u", "ni", "pero", "sino"}),
)

# Homographs a person decided (design D2), checked before any count: form → (lemma, reason). The
# reason is part of the rule: a row without one is not a decision. Editing this table changes the
# rules' sha256, so the tables must be reduced again, as for any rule.
#
# A row costs the other lemma its place in the pack: a lemma's own form always reads as itself, so
# sending `vino` to *venir* would take the noun *vino* out altogether, where the counts keep it and
# the card names *venir* as the other reading (add-lingua-spanish-grammar-tables). No homograph is
# worth that yet.
OVERRIDES = {}


def nfc_lower(text):
    return unicodedata.normalize("NFC", (text or "").strip().lower())


def read_kaikki(path):
    """Candidate lemmas per form, the words that are lemmas of their own, and the combined forms.

    A form's candidates: the lemma entries listing it among their inflections (bookkeeping left
    out), and the `form_of` targets of its own entry. An entry with a sense that is not a form-of is
    a lemma, and is its own candidate. A verb with its clitics is never a candidate's form: the
    lemma listing it as a combined form, or a sense of its own naming the pronouns, gives it no
    lemma. A string that is also a plain form of another word keeps that one — `principales` is
    *principar* + `les`, and the plural of *principal*.
    """
    candidates = collections.defaultdict(set)
    lemmas = set()
    combined = set()
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            read_entry(entry, candidates, lemmas, combined)
    return candidates, lemmas, combined


def read_entry(entry, candidates, lemmas, combined):
    """One kaikki entry into the three collections (see `read_kaikki`)."""
    word = nfc_lower(entry.get("word"))
    if not _TOKEN.fullmatch(word):
        return
    senses = entry.get("senses") or []
    only_forms = bool(senses) and all(_is_form_of(sense) for sense in senses)
    if not only_forms:
        lemmas.add(word)
        candidates[word].add(word)
    for inflection in entry.get("forms") or []:
        tags = set(inflection.get("tags") or ())
        form = nfc_lower(inflection.get("form"))
        if _COMBINED in tags:
            combined.add(form)
            continue
        if tags & _BOOKKEEPING or not _TOKEN.fullmatch(form):
            continue
        candidates[form].add(word)
    for sense in senses:
        clitics = _names_clitics(sense)
        for ref in sense.get("form_of") or ():
            target = nfc_lower(ref.get("word") if isinstance(ref, dict) else "")
            if not _TOKEN.fullmatch(target):
                continue
            if clitics:
                combined.add(word)
            else:
                candidates[word].add(target)


def _is_form_of(sense):
    return "form-of" in (sense.get("tags") or ()) or bool(sense.get("form_of"))


def _names_clitics(sense):
    return any(tag.startswith(_CLITIC_TAG) for tag in sense.get("tags") or ())


def read_gsd_counts(paths):
    """How often each (form, lemma) pair stands in the treebank, lowercased, in NFC."""
    counts = collections.Counter()
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for line in f:
                if not line.strip() or line.startswith("#"):
                    continue
                cols = line.rstrip("\n").split("\t")
                # Multi-word tokens (`del` = 1-2) and empty nodes (8.1) carry no lemma of their own.
                if len(cols) < 3 or "-" in cols[0] or "." in cols[0]:
                    continue
                counts[(nfc_lower(cols[1]), nfc_lower(cols[2]))] += 1
    return counts


def choose_lemma(form, options, counts, lemmas, frequency, overrides=OVERRIDES):
    """The one lemma a form maps to (design D2), among `options`.

    An override first, then the treebank's counts, then the form's own entry, then the lemma's
    frequency, then alphabetical order — so the result never depends on the source's order.
    """
    if not options:
        return None
    override = overrides.get(form)
    if override and override[0] in options:
        return override[0]
    return min(
        options,
        key=lambda lemma: (
            -counts.get((form, lemma), 0),
            not (lemma == form and form in lemmas),
            -frequency(lemma),
            lemma,
        ),
    )


def reduce_forms(candidates, lemmas, combined, counts, frequency, ranks_for, max_lemmas, attested):
    """The forms table and the ranks: `form → lemma` over the kept lemmas, and `lemma → rank`.

    A first choice among every candidate says which forms are only inflected; with the combined
    forms that are no word of their own, which the enclitic rule reads, they never become lemmas —
    `hacerlo` is as frequent as a word, and ranked, it would stand in the table for itself. The ranks
    follow; the final choice keeps a form only under a kept lemma, and only when wordfreq attests
    it — a lemma's identity form always stays.
    """
    first = {form: choose_lemma(form, opts, counts, lemmas, frequency) for form, opts in candidates.items()}
    inflected = {form for form, lemma in first.items() if lemma != form} | (combined - lemmas)
    ranks = ranks_for(inflected, max_lemmas)
    forms = {}
    for form, opts in candidates.items():
        kept = {lemma for lemma in opts if lemma in ranks}
        lemma = choose_lemma(form, kept, counts, lemmas, frequency)
        if lemma is None:
            continue
        if lemma == form or attested(form):
            forms[form] = lemma
    for lemma in ranks:
        forms.setdefault(lemma, lemma)
    return forms, ranks


def analyser_version(path=_ANALYSIS_RS):
    """Spanish's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const SPANISH_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no SPANISH_ANALYZER_VERSION in {path}")
    return m.group(1)


NOTICE = """Cymbra Lingua data pack — ES->FR attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), Spanish section: CC BY-SA 4.0 + GFDL —
the forms and their lemmas.

wordfreq (Spanish frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 — the commonest lemmas and which forms are attested.

UD Spanish-GSD, the Universal Dependencies Spanish-GSD treebank
(https://github.com/UniversalDependencies/UD_Spanish-GSD): CC BY-SA 4.0 — how often a form stands
for each of its lemmas, to choose one.
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--max-lemmas", type=int, default=60000)
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    from wordfreq import zipf_frequency

    zipf = {}

    def frequency(word):
        if word not in zipf:
            zipf[word] = zipf_frequency(word, "es")
        return zipf[word]

    candidates, lemmas, combined = read_kaikki(os.path.join(a.work, "kaikki-Spanish.jsonl"))
    counts = read_gsd_counts(
        [os.path.join(a.work, "es_gsd-ud-train.conllu"), os.path.join(a.work, "es_gsd-ud-dev.conllu")]
    )
    forms, ranks = reduce_forms(
        candidates,
        lemmas,
        combined,
        counts,
        frequency,
        lambda inflected, want: {
            unicodedata.normalize("NFC", lemma): rank
            for lemma, rank in common.canonical_ranks(inflected, want, studied=ES).items()
        },
        a.max_lemmas,
        lambda form: frequency(form) > 0,
    )

    common.write(a.work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())))
    common.write(
        a.work,
        "freq.tsv",
        "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: (kv[1], kv[0]))),
    )
    common.write(a.work, "gloss.tsv", "")
    common.write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "es",
            "native": "fr",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
            "licences": [
                "kaikki / enwiktionary (CC BY-SA 4.0 + GFDL)",
                "wordfreq (CC BY-SA 4.0)",
                "UD Spanish-GSD (CC BY-SA 4.0)",
            ],
        },
        "sources": [
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "UD Spanish-GSD", "licence": "CcBySa"},
        ],
    }
    common.write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    print(f"reduced es-fr: forms={len(forms)} lemmas={len(ranks)}", file=sys.stderr)


if __name__ == "__main__":
    main()
