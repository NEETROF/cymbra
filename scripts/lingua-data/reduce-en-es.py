#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the en-es pack into its native side (add-lingua-pack-en-es).

English glossed in Spanish, for Spanish speakers studying English: the first pair glossed in a
native language no shipped pack speaks. Its studied side — English's forms, ranks, readings, levels,
tag pool and dictionary words — is kept once, in `tables/en/`, written by en-fr's reduction alone
(split-lingua-pack-tables-by-language); this reducer reads it as committed and computes nothing of
it (design D1, as es-en's). It writes the native side only, from its own sources — three tables of
glosses written by people, in Spanish (the programme's M5):

Inputs:
- `tables/en/forms.tsv`, `freq.tsv` and `level.tsv` (`--studied`, the committed studied folder):
  the lemmas the glosses are matched against — those ranked within `--max-lemmas`, and the level
  lists' words en-fr keeps beyond them — and their ranks.
- `kaikki-es-English.jsonl`, in `--work`: the Spanish Wiktionary's English section, derived from
  kaikki's dump of the whole edition (`pack_sources.py DUMPS`), read for its senses — the
  definitions, cleaned by the Spanish Wiktionary's rules (`reduce_edition_es.ES`). A definition
  glosses first.
- `kaikki-en-traductions-es.jsonl`, in `--work`: the Spanish translations the English Wiktionary's
  English entries list, derived from kaikki's extract of that section — the direct fallback, in the
  table's order, a letter's entry left out (`read_translated`).
- `kaikki-es-traductions-en.jsonl`, in `--work`: the English translations the Spanish Wiktionary's
  Spanish entries list, derived from the same dump as the entries — read backwards, the inverted
  fallback, the commonest Spanish word first (wordfreq), as es-fr orders its inverted table by
  French frequency.
No pivot through a third language, no machine translation.

Outputs, in `--work`: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json` — and
`measures.json`, the translation-table share (D4), which `pack_sources.py split` files nowhere: it
is shown in the pull request and the tables' README, and stored in no pack.

Every rule here names a source or is Spanish's: the native side every pair shares
(`reduce_common.native_tables`, written out step by step in `native_side` so that the lemmas each
source glosses are kept) with the Spanish edition's rules. en-fr's reducer is not loaded — a reducer
loads code by import statements alone, and nothing of en-fr may move with this pair — so the pair's
rule digest is this file, `reduce_common.py` and `reduce_edition_es.py`. A change to en-fr's rules
reaches this pair through the committed tables: when a studied table moves, en-es's pack moves and
its checks name the table (`pack_sources.py check-reducer`, add-lingua-pack-es-en D3).
"""

import argparse
import functools
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_es as spanish  # noqa: E402 — the Spanish Wiktionary's rules: en-es's glosses are Spanish

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")
# The committed studied folder: build.sh names this run's (a dry run's scratch copy) in LINGUA_STUDIED.
_STUDIED = os.environ.get("LINGUA_STUDIED") or os.path.join(_HERE, "tables", "en")

# English, as en-fr's reducer has it — repeated here because that reducer is not loaded (D1): a
# word, whole; the word a form-of gloss points at (not read by the native side; kept for the shared
# rules' interface); the seven coordinators.
_TOKEN = re.compile(r"[A-Za-z][A-Za-z'\-]*")
_FORM_OF_TARGET = re.compile(
    r"\b(?:de|du|d['’])\s*(?:verbe\s+|adjectif\s+|nom\s+)?(?:to\s+)?[«“\"]?\s*"
    r"([A-Za-z][A-Za-z'\-]*)",
    re.IGNORECASE,
)
_COORDINATORS = frozenset({"and", "or", "but", "nor", "yet", "so", "for"})

EN = common.Studied(code="en", token=_TOKEN, form_of_target=_FORM_OF_TARGET, coordinators=_COORDINATORS)

EDITION = spanish.ES

# Expressions no source glosses, each with its Spanish gloss, written and reviewed by a person:
# expression → gloss. It wins over every source. A gloss here is never generated. Editing it is a
# rule change.
LOCUTIONS = {}

# Among the glossed lemmas of this many commonest, the share from a translation table (D4).
MEASURED_TOP = 10_000


def read_table(path):
    """A two-column table's rows, in file order: `[(first, second), …]`."""
    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            key, sep, value = line.rstrip("\n").partition("\t")
            if sep:
                rows.append((key, value))
    return rows


def read_studied(studied, max_lemmas):
    """The committed lemmas and their ranks (D1): `lemma → rank` from `freq.tsv`, the lemmas being
    the forms `forms.tsv` maps to themselves — those ranked within `max_lemmas`, and the words of
    `level.tsv`, which en-fr keeps whatever their rank (its level lists' words beyond the commonest
    40,000: 612 of English's 40,685 lemmas). The tables must agree — a lemma is ranked exactly when
    it is its own form — or the folder is not what en-fr's reduction writes."""
    forms = os.path.join(studied, "forms.tsv")
    freq = os.path.join(studied, "freq.tsv")
    level = os.path.join(studied, "level.tsv")
    for path in (forms, freq, level):
        if not os.path.isfile(path):
            raise SystemExit(f"error: {path} is missing: en-es reads English's committed tables (tables/en/, en-fr's)")
    lemmas = {form for form, lemma in read_table(forms) if form == lemma}
    ranks = {lemma: int(rank) for lemma, rank in read_table(freq)}
    if set(ranks) != lemmas:
        only_ranked = sorted(set(ranks) - lemmas)[:3]
        only_forms = sorted(lemmas - set(ranks))[:3]
        raise SystemExit(
            f"error: {freq} and {forms} disagree on the lemmas (ranked but no form of itself: {only_ranked}; "
            f"its own form but unranked: {only_forms}): they are not what en-fr's reduction writes"
        )
    levelled = {lemma for lemma, _ in read_table(level)}
    return {lemma: rank for lemma, rank in ranks.items() if rank <= max_lemmas or lemma in levelled}


def without_letters(src, dst):
    """The Spanish Wiktionary's English entries without their letters (`common.without_letter_senses`):
    a `character` entry, and a sense naming a letter (« Nombre de la letra Q. »)."""
    return common.without_letter_senses(src, dst, edition=EDITION)


def without_letter_translations(src, dst):
    """A translation file without its letters' entries, written to `dst`: a `character` entry lists
    the letter itself as its translation (`b` « b »), a gloss that says nothing, and so does a
    one-letter word's entry whose every translation is that letter. The same rule as es-en's, on
    both of en-es's tables: the English Wiktionary's letters read forwards, the Spanish one's read
    backwards. A line this pass cannot read is written as it is: the shared rules decide."""
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if isinstance(entry, dict) and _names_a_letter(entry):
                continue
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def _names_a_letter(entry):
    if entry.get("pos") == "character":
        return True
    word = (entry.get("word") or "").strip().lower()
    listed = [(t.get("word") or "").strip().lower() for t in entry.get("translations") or () if isinstance(t, dict)]
    return len(word) == 1 and all(native == word for native in listed)


def read_translated(path, dst, *, inverted):
    """What a translation file (`pack_sources.derive`) says of English words, its letters left out
    (`without_letter_translations`, into `dst`): word → {UPOS: [Spanish word, …]}
    (`common.read_translations`) — the English Wiktionary's Spanish translations read forwards,
    the Spanish Wiktionary's English translations read backwards."""
    return common.read_translations(without_letter_translations(path, dst), inverted=inverted, studied=EN)


by_spanish_frequency = common.by_native_frequency


def native_side(entries, ranks, sources):
    """`common.native_tables`, written out step by step so that the lemmas each source glosses are
    kept (D4): the Spanish Wiktionary's entries gloss first, then each `(source, order)` of
    `sources` in turn — the direct table, then the inverted one — glosses what the steps before
    left out; the expressions the same way, the locutions winning. `(glosses, runs, expressions,
    steps)`, the steps `{"entries": lemmas, "direct": lemmas, "inverted": lemmas}`, each the set a
    step glossed. The same tables as `native_tables` gives; `reduce_common.py` is not edited for
    the measure (lingua-data-packs, *A shared rule changes*)."""
    runs = {}
    glosses = common.reduce_gloss(entries, set(ranks), **common.WORD_GLOSS, runs=runs, studied=EN, edition=EDITION)
    expressions = common.reduce_expressions(entries, common.EXPRESSION_GLOSS_LEN, studied=EN, edition=EDITION)
    steps = {"entries": frozenset(glosses)}
    for name, source in zip(("direct", "inverted"), sources):
        found = common.fallback_glosses(ranks, glosses, [source], edition=EDITION)
        for lemma, (gloss, gloss_runs) in found.items():
            glosses[lemma], runs[lemma] = gloss, gloss_runs
        steps[name] = frozenset(found)
    expressions.update(common.fallback_expressions(expressions, sources, edition=EDITION))
    expressions.update(LOCUTIONS)
    return glosses, runs, expressions, steps


def translation_share(steps, ranks, top=MEASURED_TOP):
    """Among the glossed lemmas of the `top` commonest (D4): the share whose gloss came from a
    translation table — the direct or the inverted — rather than from an entry, in percent to one
    decimal, with the lemmas each table glossed there, so that a sample can be marked. The
    commonest are the first `top` by rank, as `gloss_coverage.py` counts them."""
    commonest = set(sorted(ranks, key=ranks.get)[:top])
    direct = sorted(steps["direct"] & commonest)
    inverted = sorted(steps["inverted"] & commonest)
    glossed = len(steps["entries"] & commonest) + len(direct) + len(inverted)
    share = round(100 * (len(direct) + len(inverted)) / glossed, 1) if glossed else 0.0
    return {"top": top, "glossed": glossed, "share": share, "direct": direct, "inverted": inverted}


def analyser_version(path=_ANALYSIS_RS):
    """English's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no ANALYZER_VERSION in {path}")
    return m.group(1)


# The credits (add-lingua-pack-es-en D4, applied here): both sides' sources. The studied side is
# en-fr's reduction's, read as committed, credited as en-fr's notice credits it: ESDB with its
# WordNet notice, the French Wiktionary's form links, wordfreq, CEFR-J and Octanove; its dictionary
# words are the lemmas en-fr glosses, so the French Wiktionary is credited for those too. The
# levels are CEFR-J's and Octanove's, not estimated.
NOTICE = """Cymbra Lingua data pack — EN->ES attributions.

ESDB (English Speller Database, SCOWLv2, en-wl/wordlist): the inflections (English's tables,
reduced by the en-fr pack's reduction and read here as committed).
Copyright 2000-2026 by Kevin Atkinson. Permission to use, copy, modify, distribute, and
sell any part of SCOWLv2, or word lists created from it, is hereby granted without fee,
provided that the above copyright notice appears in all copies and that both the above
copyright notice and this notice appear in supporting documentation. Kevin Atkinson
makes no representations about the suitability of this database for any purpose. It is
provided "as is" without express or implied warranty.

WordNet, used by ESDB for its initial part-of-speech assignment: WordNet 1.6 Copyright
1997 by Princeton University. All rights reserved. Permission to use, copy, modify and
distribute this software and database and its documentation for any purpose and without
fee or royalty is hereby granted, provided that this copyright notice and these
statements, including the disclaimer, appear on all copies. THIS SOFTWARE AND DATABASE
IS PROVIDED "AS IS" AND PRINCETON UNIVERSITY MAKES NO REPRESENTATIONS OR WARRANTIES,
EXPRESS OR IMPLIED. The name of Princeton University or Princeton may not be used in
advertising or publicity pertaining to distribution of the software and/or database.
Title to copyright in this software, database and any associated documentation shall at
all times remain with Princeton University.

wordfreq (English and Spanish frequency lists), by Robyn Speer (https://github.com/rspeer/wordfreq):
data under CC BY-SA 4.0 (includes SUBTLEX with Brysbaert's permission, and data from the
Google Books Ngram Viewer, http://books.google.com/ngrams) — the commonest English lemmas and
which forms are attested (English's tables, read here as committed), and the order of the
Spanish words a translation table lists.

kaikki.org extract of the Spanish Wiktionary (eswiktionary): CC BY-SA 4.0 + GFDL — the Spanish
glosses of English words and expressions, from the definitions of its English section, and the
English translations its Spanish entries list, read backwards where nothing else glosses a word.

kaikki.org extract of the English Wiktionary (enwiktionary): CC BY-SA 4.0 + GFDL — the Spanish
translations its English entries list, where the Spanish Wiktionary has no definition.

kaikki.org extract of the French Wiktionary (frwiktionary): CC BY-SA 4.0 + GFDL — the form
links completing ESDB's inflections, and which lemmas en-fr glosses: the dictionary words
(English's tables, read here as committed).

CEFR-J: The CEFR-J Wordlist Version 1.5. Compiled by Yukio Tono, Tokyo University of
Foreign Studies. Used for research and commercial purposes with acknowledgement of the
source.

Octanove: Octanove Vocabulary Profile C1/C2 v1.0, Octanove Labs, CC BY-SA 4.0.
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--studied", default=_STUDIED, help="the committed studied folder (tables/en)")
    ap.add_argument(
        "--max-lemmas",
        type=int,
        default=40000,
        help="the committed lemmas, capped by rank, the level lists' words kept beyond it: 10000 reduces the "
        "native side over the top 10,000 and them (a sample)",
    )
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    # wordfreq orders the inverted table's Spanish words, the commonest first (as es-fr's by French).
    from wordfreq import zipf_frequency

    frequency = functools.lru_cache(maxsize=None)(lambda w: zipf_frequency(w, EDITION.code))

    ranks = read_studied(a.studied, a.max_lemmas)
    entries = without_letters(
        os.path.join(a.work, "kaikki-es-English.jsonl"), os.path.join(a.work, "kaikki-es-English-words.jsonl")
    )
    direct = read_translated(
        os.path.join(a.work, "kaikki-en-traductions-es.jsonl"),
        os.path.join(a.work, "kaikki-en-traductions-es-words.jsonl"),
        inverted=False,
    )
    inverted = read_translated(
        os.path.join(a.work, "kaikki-es-traductions-en.jsonl"),
        os.path.join(a.work, "kaikki-es-traductions-en-words.jsonl"),
        inverted=True,
    )
    sources = [(direct, list), (inverted, by_spanish_frequency(frequency))]
    glosses, runs, expressions, steps = native_side(entries, ranks, sources)
    # The runs' parts of speech are the Spanish Wiktionary's; English's readings come from the
    # tables the builder reads in tables/en, not from here.
    common.write(a.work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    common.write(
        a.work,
        "senses.tsv",
        "".join(
            f"{w}\t" + "\t".join(f"{pos}:{n}" for pos, n in r) + "\n" for w, r in sorted(runs.items()) if w in glosses
        ),
    )
    common.write(a.work, "mwe.tsv", "".join(f"{w}\t{g}\n" for w, g in sorted(expressions.items())))
    common.write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "en",
            "native": "es",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
            "licences": [
                "ESDB / SCOWLv2 (permissive, commercial use allowed; WordNet notice)",
                "wordfreq (CC BY-SA 4.0)",
                "kaikki / eswiktionary, enwiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)",
                "CEFR-J Wordlist v1.5 (commercial use allowed with attribution)",
                "Octanove Vocabulary Profile C1/C2 v1.0 (CC BY-SA 4.0)",
            ],
        },
        "sources": [
            {"name": "ESDB", "licence": "Permissive"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "CEFR-J", "licence": "Permissive"},
            {"name": "Octanove", "licence": "CcBySa"},
        ],
    }
    common.write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    measures = translation_share(steps, ranks)
    common.write(a.work, "measures.json", json.dumps(measures, indent=2, ensure_ascii=False) + "\n")
    print(
        f"reduced en-es: lemmas={len(ranks)} (English's committed tables) glosses={len(glosses)} "
        f"(Spanish Wiktionary {len(steps['entries'])}, its English translations read backwards "
        f"{len(steps['inverted'])}, English Wiktionary's Spanish translations {len(steps['direct'])}) "
        f"expressions={len(expressions)}; of the {measures['glossed']} glossed lemmas among the "
        f"{measures['top']:,} commonest, {measures['share']} % come from a translation table "
        f"(direct {len(measures['direct'])}, inverted {len(measures['inverted'])})",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
