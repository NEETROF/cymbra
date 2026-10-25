#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the it-en pack into its native side (add-lingua-pack-it-en).

Italian glossed in English, for English speakers studying Italian: the first reader pair of Italian
(the Italian programme's change 22). Its studied side — Italian's forms, ranks, readings, levels, tag
pool, dictionary words and acronym rows — is kept once, in `tables/it/`, written by it-fr's reduction
alone; this reducer reads it as committed and computes nothing of it (design D1), as es-en reads
`tables/es/`. It writes the native side only, from its own sources:

Inputs:
- `tables/it/forms.tsv` and `freq.tsv` (`--studied`, the committed studied folder): the lemmas the
  glosses are matched against, and their ranks. The acronym rows (`NATO`, `USA`), forms of themselves
  with no rank, are glossed apart (D6).
- `kaikki-Italian.jsonl`, in `--work`: the English Wiktionary's Italian section as `pack_sources.derive`
  writes it from the English dump — the file Italian's forms come from — read here for its senses:
  the English glosses of Italian words and expressions, cleaned by the English Wiktionary's rules
  (`reduce_edition_en.EN`) and it-en's own pre-pass (`reduce_italian_en.py`, D4).
- `kaikki-it-traductions-en.jsonl`, in `--work`: the English translations the Italian Wiktionary's
  Italian entries list (derived from its dump, `pack_sources.py DUMPS`), the glosses' fallback (D2),
  at most three words per part of speech. No table read backwards, no pivot, no machine translation.

Outputs, in `--work`: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json`; and
`measures.json`, what the reduction measures of its glosses for the update's report, which no table
holds (D8).

The English Wiktionary's entries reach the shared rules through the English edition's pre-passes,
as es-en calls them — its letters left out (`common.without_letter_senses`,
`english.without_letter_headwords`), its senses read as meanings (`english.read_as_meanings`) —, then
through it-en's own (`italian.read_as_italian`, D4: a pointer's carried meaning read by a reviewed
list of wordings, acronyms kept off common words and names off function words, no word glossed by a
letter's name, the page's notes out), then the English edition's etymology merging (off).

Every rule here names a source or is Italian's: the native side every pair shares
(`reduce_common.native_tables`, step by step so that each lemma's source is kept) with the English
edition's rules. it-fr's reducer is not loaded — a reducer loads code by import statements alone, and
nothing of it-fr may move with this pair — so the pair's rule digest is this file,
`reduce_italian_en.py`, `reduce_italian_acronyms.py` (change 11's rows), `reduce_common.py` and
`reduce_edition_en.py`. A change to it-fr's rules reaches this pair through the committed tables:
when a studied table moves, it-en's pack moves and its checks name the table (`pack_sources.py
check-reducer`).
"""

import argparse
import json
import os
import random
import re
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_en as english  # noqa: E402 — the English Wiktionary's rules: it-en's glosses are English
import reduce_italian_acronyms as acronyms  # noqa: E402 — Italian's acronym rows (keep-lingua-italian-acronyms-and-names)
import reduce_italian_en as italian  # noqa: E402 — it-en's own pre-pass over the English section (D4)

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")
# The committed studied folder: build.sh names this run's (a dry run's scratch copy) in LINGUA_STUDIED.
_STUDIED = os.environ.get("LINGUA_STUDIED") or os.path.join(_HERE, "tables", "it")

# The native side's sources, in `--work` (pack_sources.py DUMPS["it-en"]).
SECTION = "kaikki-Italian.jsonl"
DIRECT = "kaikki-it-traductions-en.jsonl"
# What the native side writes beside its sources, as it reads them — no table: `pack_sources.py split`
# files none of them.
SECTION_SENSES = "kaikki-Italian-senses.jsonl"
SECTION_WORDS = "kaikki-Italian-words.jsonl"
SECTION_HEADWORDS = "kaikki-Italian-headwords.jsonl"
SECTION_MEANINGS = "kaikki-Italian-meanings.jsonl"
SECTION_ITALIAN = "kaikki-Italian-read.jsonl"
SECTION_MERGED = "kaikki-Italian-merged.jsonl"
DIRECT_READ = "kaikki-it-traductions-en-read.jsonl"

EDITION = english.EN

# Italian for the shared rules (`reduce_common.Studied`): change 9's token pattern and it-fr's
# coordinators, repeated here because it-fr's reducer is not loaded (D1) — test_reduce_it_en.py holds
# them to it-fr's. A word: Italian letters, words joined by hyphens or by an inner apostrophe, and an
# elided piece's or a truncation's final apostrophe (`po'`); the coordinators are the conjunctions UD
# Italian-MarkIT's training and development sections tag CCONJ at least twice.
_LETTERS = italian.LETTERS
_TOKEN = italian.TOKEN
IT = common.Studied(
    code="it",
    token=_TOKEN,
    # Not read by the native side; kept for the shared rules' interface.
    form_of_target=re.compile(rf"([{_LETTERS}]+)\s*\.?$"),
    coordinators=frozenset({"e", "ed", "o", "ma", "sia", "oppure", "né", "cioè", "ovvero", "ossia", "nonché", "bensì"}),
)

# What the native side reads of an entry (`reduce_common._read_entries`, `reduce_expressions`,
# `without_letter_senses`, the pre-passes): the rest — the inflection tables, most of the section's
# bytes — is it-fr's to read.
_ENTRY_FIELDS = ("word", "pos", "senses")
_SENSE_FIELDS = ("glosses", "tags", *EDITION.pointer_fields)


def straight(text):
    """An Italian word as Italian's tables write it: in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip()).replace("’", "'")


def read_table(path):
    """A two-column table's rows, in file order: `[(first, second), …]`."""
    rows = []
    with open(path, encoding="utf-8") as f:
        for line in f:
            key, sep, value = line.rstrip("\n").partition("\t")
            if sep:
                rows.append((key, value))
    return rows


def read_studied(studied, max_lemmas, rows=None):
    """The committed lemmas and their ranks (D1): `lemma → rank` from `freq.tsv`, the lemmas being the
    forms `forms.tsv` maps to themselves but the acronym rows (`reduce_italian_acronyms.ACRONYMS`,
    each its own lemma, with no rank), capped at `max_lemmas` by rank. The two tables must agree — a
    lemma is ranked exactly when it is its own form and no row — or the folder is not what it-fr's
    reduction writes."""
    rows = acronyms.ACRONYMS if rows is None else rows
    forms = os.path.join(studied, "forms.tsv")
    freq = os.path.join(studied, "freq.tsv")
    for path in (forms, freq):
        if not os.path.isfile(path):
            raise SystemExit(f"error: {path} is missing: it-en reads Italian's committed tables (tables/it/, it-fr's)")
    lemmas = {form for form, lemma in read_table(forms) if form == lemma} - set(rows)
    ranks = {lemma: int(rank) for lemma, rank in read_table(freq)}
    if set(ranks) != lemmas:
        only_ranked = sorted(set(ranks) - lemmas)[:3]
        only_forms = sorted(lemmas - set(ranks))[:3]
        raise SystemExit(
            f"error: {freq} and {forms} disagree on the lemmas (ranked but no form of itself: {only_ranked}; "
            f"its own form but unranked: {only_forms}): they are not what it-fr's reduction writes"
        )
    return {lemma: rank for lemma, rank in ranks.items() if rank <= max_lemmas}


def read_forms(studied):
    """The committed forms: `form → lemma`, for the split words (D4.5): a form is looked up whole."""
    return dict(read_table(os.path.join(studied, "forms.tsv")))


def native_fields(src, dst):
    """The section cut down to what the native side reads (`_ENTRY_FIELDS`, `_SENSE_FIELDS`), written
    to `dst`: the shared rules read the file several times, and the inflection tables are most of its
    bytes. An entry's `word` is read as Italian is written (`straight`), its `pos` kept as written. A
    line that is no JSON object — undecodable, or another JSON value — is left out, and counted:
    `(dst, dropped)`."""
    dropped = 0
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if not isinstance(entry, dict):
                dropped += 1
                continue
            cut = {field: entry[field] for field in _ENTRY_FIELDS if field in entry}
            if isinstance(cut.get("word"), str):
                cut["word"] = straight(cut["word"])
            cut["senses"] = [
                {field: sense[field] for field in _SENSE_FIELDS if field in sense}
                for sense in entry.get("senses") or ()
                if isinstance(sense, dict)
            ]
            out.write(json.dumps(cut, ensure_ascii=False) + "\n")
    return dst, dropped


def without_letters(src, dst):
    """The English Wiktionary's Italian entries without their letters (`common.without_letter_senses`):
    a `character` entry, a sense naming a Latin letter (« the name of the Latin-script letter B/b »)."""
    return common.without_letter_senses(src, dst, edition=EDITION)


def straight_translations(src, dst):
    """The Italian Wiktionary's translation file with its Italian words read as Italian is written
    (`straight`), and without its letters' entries: a `character` entry, and a one-letter word's entry
    whose every translation is that letter, say nothing (es-en's `without_letter_translations`). An
    entry it does not change is written as its line was; a line it cannot read is written as it is."""
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if not isinstance(entry, dict):
                out.write(line if line.endswith("\n") else line + "\n")
                continue
            if _names_a_letter(entry):
                continue
            word = entry.get("word")
            if isinstance(word, str) and straight(word) != word:
                out.write(json.dumps({**entry, "word": straight(word)}, ensure_ascii=False, sort_keys=True) + "\n")
                continue
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def _names_a_letter(entry):
    if entry.get("pos") == "character":
        return True
    word = (entry.get("word") or "").strip().lower()
    listed = [(t.get("word") or "").strip().lower() for t in entry.get("translations") or () if isinstance(t, dict)]
    return len(word) == 1 and all(native == word for native in listed)


def read_translated(path, dst):
    """What the Italian Wiktionary's translation file says of Italian words (`straight_translations`,
    into `dst`): word → {UPOS: [English word, …]} (`common.read_translations`, the direct table)."""
    return common.read_translations(straight_translations(path, dst), inverted=False, studied=IT)


def native_side(section, ranks, direct):
    """The native side step by step, so that the lemmas each source glosses are kept (D2, D8):
    `common.native_tables(section, ranks, studied=IT, edition=EN, fallbacks=[(direct, list)])` — the
    section's definitions first, then the Italian Wiktionary's English translations, at most three
    words per part of speech, for what they leave out; the expressions the same way —, its tables
    `(glosses, runs, expressions, steps)`, the steps `{"definitions": lemmas, "direct": lemmas,
    "section expressions": headwords}`, each the set a step glossed (test_reduce_it_en.py holds it to
    `native_tables`)."""
    runs = {}
    glosses = common.reduce_gloss(section, set(ranks), **common.WORD_GLOSS, runs=runs, studied=IT, edition=EDITION)
    expressions = common.reduce_expressions(section, studied=IT, edition=EDITION)
    steps = {"definitions": frozenset(glosses), "section expressions": frozenset(expressions)}
    found = common.fallback_glosses(ranks, glosses, [(direct, list)], edition=EDITION)
    for lemma, (gloss, gloss_runs) in found.items():
        glosses[lemma], runs[lemma] = gloss, gloss_runs
    steps["direct"] = frozenset(found)
    expressions.update(common.fallback_expressions(expressions, [(direct, list)], edition=EDITION))
    return glosses, runs, expressions, steps


def without_repeated_senses(glosses, runs):
    """A word's gloss without a sense that repeats an earlier one word for word, up to its closing
    period — `ex` « ex, former; ex, former », `beige` « beige; beige », read across the Italian
    Wiktionary's parts of speech or the section's —, its run shortened (change 16's review rule, it-en's
    own copy). Case is kept: English writes a name's sense and a common word's apart (`cancro` « Cancer;
    cancer », `inferno` « Hell; …; hell »), and both stay. Edits `glosses` and `runs` in place; answers the
    lemmas whose gloss moved, sorted."""
    moved = []
    for lemma, gloss in sorted(glosses.items()):
        senses = gloss.split(acronyms.SENSE_SEPARATOR)
        poses = [pos for pos, n in runs.get(lemma) or () for _ in range(n)]
        if len(poses) != len(senses):
            continue
        seen, kept = set(), []
        for sense, pos in zip(senses, poses):
            key = unicodedata.normalize("NFC", sense).strip().rstrip(".").strip()
            if key in seen:
                continue
            seen.add(key)
            kept.append((sense, pos))
        if len(kept) == len(senses):
            continue
        new_runs = []
        for _, pos in kept:
            if new_runs and new_runs[-1][0] == pos:
                new_runs[-1][1] += 1
            else:
                new_runs.append([pos, 1])
        glosses[lemma] = acronyms.SENSE_SEPARATOR.join(sense for sense, _ in kept)
        runs[lemma] = [tuple(run) for run in new_runs]
        moved.append(lemma)
    return moved


def acronym_glosses(glosses, expressions, entries):
    """Each acronym row's gloss (keep-lingua-italian-acronyms-and-names D3, `acronym_gloss`; D6 here):
    from it-en's own glosses of the row's expansions, else of its synonyms — a word's from `glosses`, a
    headword with a space from `expressions` —, a synonym the pair does not gloss read from its entry
    in the section whatever its part of speech (`entries`, `italian.section_words`): `Stati Uniti`, a
    proper noun's entry, which `reduce_expressions` and the translation reader skip. `acronym →
    (gloss, runs)`."""
    words = {w: g for w, g in entries.items() if " " not in w}
    phrases = {w: g for w, g in entries.items() if " " in w}
    out = {}
    for acronym, row in sorted(acronyms.ACRONYMS.items()):
        got = acronyms.acronym_gloss(row, {**words, **glosses}, {**phrases, **expressions})
        if got:
            out[acronym] = got
    return out


def row_words(rows=None):
    """The expansions and synonyms of the acronym rows, lowercased as the pair keys them."""
    rows = acronyms.ACRONYMS if rows is None else rows
    return {straight(w).lower() for expansions, synonyms, _, _ in rows.values() for w in (*expansions, *synonyms)}


# D8's sample: 100 glossed lemmas of the 20,000 commonest, drawn by this seed, each marked by source —
# it-fr's seed and size, so the two pairs' samples read alike.
SAMPLE_SIZE = 100
SAMPLE_TOP = 20_000
SAMPLE_SEED = 20261024
# The tops coverage is measured at, as `gloss_coverage.py` counts them, and the top the translation
# share is read at (`measures.json`, en-es's, fr-es's and it-fr's shape).
COVERAGE_TOPS = (5_000, 10_000, 20_000)
MEASURED_TOP = 10_000
SOURCES = ("definitions", "direct")


def source_of(lemma, steps):
    """The source that glossed a lemma: `definitions` or `direct`, or None."""
    return next((name for name in SOURCES if lemma in steps.get(name, ())), None)


def coverage(ranks, glosses, tops=COVERAGE_TOPS):
    """The share of each top's commonest ranked lemmas with a gloss, in percent to one decimal — as
    `gloss_coverage.measure` reads the committed tables."""
    ranked = sorted(ranks, key=lambda lemma: (ranks[lemma], lemma))
    return [round(100 * sum(1 for lemma in ranked[:top] if lemma in glosses) / len(ranked[:top]), 1) for top in tops]


def measures(ranks, glosses, steps, top=MEASURED_TOP):
    """What the reduction measures of its glosses and stores in no pack (D8), for the update's report
    (`pack_report.py --measures`): among the glossed lemmas of the `top` commonest, the share the
    Italian Wiktionary's translations glossed, with those lemmas; the coverage at each of
    `COVERAGE_TOPS`; the glossed lemmas by source; and the sample, each lemma with its rank, its source
    and its gloss."""
    ranked = sorted(ranks, key=lambda lemma: (ranks[lemma], lemma))
    commonest = set(ranked[:top])
    direct = sorted(steps["direct"] & commonest)
    glossed = len(steps["definitions"] & commonest) + len(direct)
    share = round(100 * len(direct) / glossed, 1) if glossed else 0.0
    pool = [lemma for lemma in ranked[:SAMPLE_TOP] if lemma in glosses]
    drawn = random.Random(SAMPLE_SEED).sample(pool, min(SAMPLE_SIZE, len(pool)))
    sample = [[lemma, ranks[lemma], source_of(lemma, steps), glosses[lemma]] for lemma in sorted(drawn, key=ranks.get)]
    return {
        "top": top,
        "glossed": glossed,
        "share": share,
        "direct": direct,
        "coverage": dict(zip((str(t) for t in COVERAGE_TOPS), coverage(ranks, glosses))),
        "by source": {name: len(steps[name]) for name in SOURCES},
        "sample": sample,
    }


def analyser_version(path=_ANALYSIS_RS):
    """Italian's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const ITALIAN_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no ITALIAN_ANALYZER_VERSION in {path}")
    return m.group(1)


# The credits (D8): both sides' sources. The studied side is it-fr's reduction's, read as committed:
# everything it-fr's notice credits for it — the English Wiktionary's Italian section, wordfreq, UD
# Italian-MarkIT, Morph-it! while it-fr keeps it — and the French Wiktionary's Italian section, whose
# glosses decide Italian's dictionary words and which lemmas take a level; the levels are the ones
# it-fr estimated, and the manifest says so. Not UD Italian-PUD, a gate only.
NOTICE = """Cymbra Lingua data pack — IT->EN attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), Italian section: CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/) + GFDL (https://www.gnu.org/licenses/fdl-1.3.html)
— the English glosses of Italian words and expressions, from its senses; and the forms and their
lemmas (Italian's tables, reduced by the it-fr pack's reduction and read here as committed).

kaikki.org extract of the Italian Wiktionary (itwiktionary): CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/) + GFDL (https://www.gnu.org/licenses/fdl-1.3.html)
— the English translations its Italian entries list, where the English Wiktionary has no gloss.

wordfreq (Italian frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/) — the commonest lemmas and which forms
are attested.

UD Italian-MarkIT, the Universal Dependencies Italian-MarkIT treebank by Teresa Paccosi, Alessio
Palmero Aprosio and Sara Tonelli (https://github.com/UniversalDependencies/UD_Italian-MarkIT):
CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/) — how often a form stands for each of its
lemmas, to choose one, and how often a hyphenated word occurs, to rank it; its training and
development sections alone.

Morph-it!, a free morphological lexicon for the Italian language, by Marco Baroni and Eros Zanchetta
(https://docs.sslmit.unibo.it/doku.php?id=resources:morph-it), version 0.4.8: CC BY-SA 2.0
(https://creativecommons.org/licenses/by-sa/2.0/), one of its two licences, the other being the
LGPL — the forms it lists that the Wiktionary names nowhere, under the lemmas the tables rank.

kaikki.org extract of the French Wiktionary (frwiktionary), Italian section: CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/) + GFDL (https://www.gnu.org/licenses/fdl-1.3.html)
— which lemmas the it-fr pack glosses: Italian's dictionary words and which take a level (Italian's
tables, read here as committed).

The levels are estimated, not taken from a CEFR list: the it-fr pack's reduction derives them from the
frequency ranks, the commonest lemmas taking the sizes of English's CEFR levels, and every pack studying
Italian reads them as committed.
"""


def manifest(pack_version):
    """it-en's manifest (D8): Italian glossed in English, Italian's analyser version, and
    `levels_estimated`: Italian's levels are it-fr's estimate, read as committed, and the extension
    says so. kaikki's licence names the three editions whose bytes reach the pack."""
    return {
        "meta": {
            "studied": "it",
            "native": "en",
            "pack_version": pack_version,
            "analyzer_version": analyser_version(),
            # Italian's levels are it-fr's estimate, read as committed: the extension says so.
            "levels_estimated": True,
            "licences": [
                "kaikki / enwiktionary, itwiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)",
                "wordfreq (CC BY-SA 4.0)",
                "UD Italian-MarkIT (CC BY 4.0)",
                "Morph-it! (CC BY-SA 2.0)",
            ],
        },
        "sources": [
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "UD Italian-MarkIT", "licence": "CcBy"},
            {"name": "Morph-it!", "licence": "CcBySa"},
        ],
    }


def reduce(work, studied, max_lemmas, pack_version, report=print):
    """The reduction over the raw sources in `work`, its tables written there: answers what the report
    prints, as a dict, for the measurement."""
    absent = [name for name in (SECTION, DIRECT) if not os.path.isfile(os.path.join(work, name))]
    if absent:
        raise SystemExit(
            f"it-en: {', '.join(os.path.join(work, name) for name in absent)} missing: it-en's glosses read the "
            "English Wiktionary's Italian section and the Italian Wiktionary's English translations "
            "(pack_sources.py DUMPS['it-en'])"
        )
    ranks = read_studied(studied, max_lemmas)
    forms = read_forms(studied)

    section, dropped = native_fields(os.path.join(work, SECTION), os.path.join(work, SECTION_SENSES))
    section = without_letters(section, os.path.join(work, SECTION_WORDS))
    section = english.without_letter_headwords(section, os.path.join(work, SECTION_HEADWORDS))
    section = english.read_as_meanings(section, os.path.join(work, SECTION_MEANINGS))
    section, stats = italian.read_as_italian(section, os.path.join(work, SECTION_ITALIAN), edition=EDITION)
    section = english.merge_same_pos_etymologies(section, os.path.join(work, SECTION_MERGED))
    direct = read_translated(os.path.join(work, DIRECT), os.path.join(work, DIRECT_READ))

    glosses, runs, expressions, steps = native_side(section, ranks, direct)
    # A sense repeating an earlier one goes, before the acronym rows read the glosses.
    repeated = without_repeated_senses(glosses, runs)
    stats["glosses repeating a sense"] = len(repeated)
    # The words Italian's pre-pass splits, after the headwords with a space: the section's, then the
    # Italian Wiktionary's (D4.5, D5); and the entries of the acronym rows' synonyms, whatever their
    # part of speech (D6).
    split, row_entries = italian.section_words(section, forms, row_words(), edition=EDITION)
    split_table = italian.table_split_words([(direct, list)], forms, split, edition=EDITION)
    with_space = len(expressions)
    expressions.update(split)
    expressions.update(split_table)
    # Each acronym row glossed from it-en's own glosses of its expansions, else of its synonyms
    # (keep-lingua-italian-acronyms-and-names D3), the only unranked lemmas it glosses.
    glossed_acronyms = acronym_glosses(glosses, expressions, row_entries)
    for acronym, (gloss, gloss_runs) in glossed_acronyms.items():
        glosses[acronym], runs[acronym] = gloss, gloss_runs

    # The runs' parts of speech are the English Wiktionary's; a noun's gender comes from the readings
    # the builder reads in tables/it, not from here.
    common.write(work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    common.write(
        work,
        "senses.tsv",
        "".join(f"{w}\t" + "\t".join(f"{pos}:{n}" for pos, n in r) + "\n" for w, r in sorted(runs.items()) if w in glosses),
    )
    common.write(work, "mwe.tsv", "".join(f"{w}\t{g}\n" for w, g in sorted(expressions.items())))
    measured = measures(ranks, glosses, steps)
    common.write(work, "measures.json", json.dumps(measured, indent=2, ensure_ascii=False) + "\n")
    common.write(work, "NOTICE", NOTICE)
    common.write(work, "manifest.json", json.dumps(manifest(pack_version), indent=2, ensure_ascii=False) + "\n")

    by_source = measured["by source"]
    summary = {
        "lemmas": len(ranks),
        "glosses": len(glosses),
        "glossed by source": by_source,
        "coverage": measured["coverage"],
        "translation share": measured["share"],
        "expressions": len(expressions),
        "expressions by source": {
            "the section, with a space": len(steps["section expressions"]),
            "the Italian Wiktionary, with a space": with_space - len(steps["section expressions"]),
            "split words, the section": len(split),
            "split words, the Italian Wiktionary": len(split_table),
        },
        "section as read": dict(stats),
        "acronym glosses": {acronym: gloss for acronym, (gloss, _) in glossed_acronyms.items()},
        "dropped": dropped,
    }
    report(
        f"reduced it-en: lemmas={len(ranks)} (Italian's committed tables) glosses={len(glosses)} (the English "
        f"Wiktionary's Italian section {by_source['definitions']}, the Italian Wiktionary's English translations "
        f"{by_source['direct']}, the acronym rows {len(glossed_acronyms)}); of the 5,000 / 10,000 / 20,000 commonest "
        "lemmas, " + " / ".join(f"{share} %" for share in measured["coverage"].values()) + " glossed; of the "
        f"{measured['glossed']:,} glossed among the {measured['top']:,} commonest, {measured['share']} % from the "
        f"Italian Wiktionary; dropped={dropped} (extract lines that are no JSON object)"
    )
    report(
        f"reduced it-en: expressions={len(expressions)} ("
        + ", ".join(f"{what} {n}" for what, n in summary["expressions by source"].items())
        + ")"
    )
    report("reduced it-en: the section as read: " + ", ".join(f"{n} {what}" for what, n in sorted(stats.items())))
    report(
        "reduced it-en: the acronym rows "
        + (", ".join(f"{a} « {g} »" for a, g in summary["acronym glosses"].items()) or "none glossed")
    )
    for lemma, rank, source, gloss in measured["sample"]:
        report(f"reduced it-en: sample: {rank}\t{lemma}\t{source}\t{gloss}")
    return summary


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--studied", default=_STUDIED, help="the committed studied folder (tables/it)")
    ap.add_argument(
        "--max-lemmas",
        type=int,
        default=60000,
        help="the committed lemmas, capped by rank: 10000 reduces the native side over the top 10,000 (a sample)",
    )
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()
    reduce(a.work, a.studied, a.max_lemmas, a.pack_version, report=lambda line: print(line, file=sys.stderr))


if __name__ == "__main__":
    main()
