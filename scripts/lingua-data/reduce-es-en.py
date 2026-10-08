#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the es-en pack into its native side (add-lingua-pack-es-en).

Spanish glossed in English, the first pair of a studied language's second native language. Its
studied side — Spanish's forms, ranks, readings, levels, tag pool and dictionary words — is kept
once, in `tables/es/`, written by es-fr's reduction alone (split-lingua-pack-tables-by-language);
this reducer reads it as committed and computes nothing of it (design D1). It writes the native
side only, from its own sources:

Inputs:
- `tables/es/forms.tsv` and `freq.tsv` (`--studied`, the committed studied folder): the lemmas
  the glosses are matched against, and their ranks.
- `kaikki-Spanish.jsonl`, in `--work`: kaikki's extract of the English Wiktionary, Spanish section
  — the extract es-fr reads for Spanish's forms — read here for its senses: the English glosses of
  Spanish words and expressions, cleaned by the English Wiktionary's rules (`reduce_edition_en.EN`).
- `kaikki-es-traductions-en.jsonl`, in `--work`: the English translations the Spanish Wiktionary's
  Spanish entries list (derived from its dump, `pack_sources.py DUMPS`), the glosses' direct
  fallback. No inverted table (the English Wiktionary's English entries are change 22's source),
  no pivot, no machine translation.

Outputs, in `--work`: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json`.

Every rule here names a source or is English's: the native side every pair shares
(`reduce_common.native_tables`) with the English edition's rules. es-fr's reducer is not loaded —
a reducer loads code by import statements alone, and nothing of es-fr may move with this pair — so
the pair's rule digest is this file, `reduce_common.py` and `reduce_edition_en.py`. A change to
es-fr's rules reaches this pair through the committed tables: when a studied table moves, es-en's
pack moves and its checks name the table (`pack_sources.py check-reducer`, D3).
"""

import argparse
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_en as english  # noqa: E402 — the English Wiktionary's rules: es-en's glosses are English

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")
# The committed studied folder: build.sh names this run's (a dry run's scratch copy) in LINGUA_STUDIED.
_STUDIED = os.environ.get("LINGUA_STUDIED") or os.path.join(_HERE, "tables", "es")

# A Spanish word, whole: letters, accents, ü, ñ, and hyphenated compounds of them — as es-fr's
# reducer has it, repeated here because that reducer is not loaded (D1).
_TOKEN = re.compile(r"[a-záéíóúüñ]+(?:-[a-záéíóúüñ]+)*")

ES = common.Studied(
    code="es",
    token=_TOKEN,
    # Not read by the native side; kept for the shared rules' interface.
    form_of_target=re.compile(r"([a-záéíóúüñ]+)\s*\.?$"),
    coordinators=frozenset({"y", "e", "o", "u", "ni", "pero", "sino"}),
)

EDITION = english.EN

# Verbal locutions no source glosses (« hay que »), each with its English gloss, written and
# reviewed by a person: expression → gloss. It wins over every source. A gloss here is never
# generated. Editing it is a rule change.
LOCUTIONS = {}

# What the native side reads of an entry (`reduce_common._read_entries`, `reduce_expressions`,
# `without_letter_senses`): the rest — the inflection tables, most of the extract's bytes — is
# es-fr's to read.
_ENTRY_FIELDS = ("word", "pos", "senses")
_SENSE_FIELDS = ("glosses", "tags", *EDITION.pointer_fields)


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
    the forms `forms.tsv` maps to themselves, capped at `max_lemmas` by rank. The two tables must
    agree — a lemma is ranked exactly when it is its own form — or the folder is not what es-fr's
    reduction writes."""
    forms = os.path.join(studied, "forms.tsv")
    freq = os.path.join(studied, "freq.tsv")
    for path in (forms, freq):
        if not os.path.isfile(path):
            raise SystemExit(f"error: {path} is missing: es-en reads Spanish's committed tables (tables/es/, es-fr's)")
    lemmas = {form for form, lemma in read_table(forms) if form == lemma}
    ranks = {lemma: int(rank) for lemma, rank in read_table(freq)}
    if set(ranks) != lemmas:
        only_ranked = sorted(set(ranks) - lemmas)[:3]
        only_forms = sorted(lemmas - set(ranks))[:3]
        raise SystemExit(
            f"error: {freq} and {forms} disagree on the lemmas (ranked but no form of itself: {only_ranked}; "
            f"its own form but unranked: {only_forms}): they are not what es-fr's reduction writes"
        )
    return {lemma: rank for lemma, rank in ranks.items() if rank <= max_lemmas}


def native_fields(src, dst):
    """The extract cut down to what the native side reads (`_ENTRY_FIELDS`, `_SENSE_FIELDS`),
    written to `dst`: the shared rules read the file three times, and the inflection tables are
    most of its 1 GB. An entry's `word` and `pos` are kept as written."""
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(entry, dict):
                continue
            cut = {field: entry[field] for field in _ENTRY_FIELDS if field in entry}
            cut["senses"] = [
                {field: sense[field] for field in _SENSE_FIELDS if field in sense}
                for sense in entry.get("senses") or ()
                if isinstance(sense, dict)
            ]
            out.write(json.dumps(cut, ensure_ascii=False) + "\n")
    return dst


def without_letters(src, dst):
    """The English Wiktionary's Spanish entries without their letters (`common.without_letter_senses`)."""
    return common.without_letter_senses(src, dst, edition=EDITION)


def read_translated(path):
    """What the Spanish Wiktionary's translation file says of Spanish words: word → {UPOS: [English
    word, …]} (`common.read_translations`, the direct table)."""
    return common.read_translations(path, inverted=False, studied=ES)


def analyser_version(path=_ANALYSIS_RS):
    """Spanish's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const SPANISH_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no SPANISH_ANALYZER_VERSION in {path}")
    return m.group(1)


# The credits (design D4): both sides' sources. The studied side is es-fr's reduction's, read as
# committed; the levels are the ones es-fr estimated, and the manifest says so.
NOTICE = """Cymbra Lingua data pack — ES->EN attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), Spanish section: CC BY-SA 4.0 + GFDL —
the forms, their lemmas and their grammar (Spanish's tables, reduced by the es-fr pack's reduction
and read here as committed), and the English glosses of Spanish words and expressions, from its
senses.

kaikki.org extract of the Spanish Wiktionary (eswiktionary): CC BY-SA 4.0 + GFDL — the English
translations its Spanish entries list, where the English Wiktionary has no gloss.

wordfreq (Spanish frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 — the commonest lemmas and which forms are attested.

UD Spanish-GSD, the Universal Dependencies Spanish-GSD treebank
(https://github.com/UniversalDependencies/UD_Spanish-GSD): CC BY-SA 4.0 — how often a form stands
for each of its lemmas, to choose one.

The levels are estimated, not taken from a CEFR list: the es-fr pack's reduction derives them from
the frequency ranks and its French glosses, and every pack studying Spanish reads them as committed.
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--studied", default=_STUDIED, help="the committed studied folder (tables/es)")
    ap.add_argument(
        "--max-lemmas",
        type=int,
        default=60000,
        help="the committed lemmas, capped by rank: 10000 reduces the native side over the top 10,000 (a sample)",
    )
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    ranks = read_studied(a.studied, a.max_lemmas)

    entries = native_fields(
        os.path.join(a.work, "kaikki-Spanish.jsonl"), os.path.join(a.work, "kaikki-Spanish-senses.jsonl")
    )
    entries = without_letters(entries, os.path.join(a.work, "kaikki-Spanish-words.jsonl"))
    entries = english.merge_same_pos_etymologies(entries, os.path.join(a.work, "kaikki-Spanish-merged.jsonl"))
    direct = read_translated(os.path.join(a.work, "kaikki-es-traductions-en.jsonl"))
    glosses, runs, expressions, primary = common.native_tables(
        entries, ranks, studied=ES, edition=EDITION, fallbacks=[(direct, list)], locutions=LOCUTIONS
    )
    # The runs' parts of speech are the English Wiktionary's; a noun's gender comes from the
    # readings the builder reads in tables/es (add-lingua-pack-lexical-layer D5), not from here.
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
            "studied": "es",
            "native": "en",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
            # Spanish's levels are es-fr's estimate, read as committed: the extension says so.
            "levels_estimated": True,
            "licences": [
                "kaikki / enwiktionary, eswiktionary (CC BY-SA 4.0 + GFDL)",
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
    print(
        f"reduced es-en: lemmas={len(ranks)} (Spanish's committed tables) glosses={len(glosses)} "
        f"(English Wiktionary {primary}) expressions={len(expressions)}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
