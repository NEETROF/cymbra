#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the it-es pack into its native side (add-lingua-pack-it-es).

Italian glossed in Spanish, for Spanish speakers studying Italian: a reader pair of Italian (the Italian
programme's change 27). Its studied side — Italian's forms, ranks, readings, levels, tag pool,
dictionary words and acronym rows — is kept once, in `tables/it/`, written by it-fr's reduction alone;
this reducer reads it as committed and computes nothing of it (design D1). It writes the native side
only, from sources in this order (D2) — every gloss Spanish written by a person, the pivot's after
every direct source, each marked by its source:

1. `kaikki-es-Italiano.jsonl`: the Spanish Wiktionary's Italian section, its definitions first, read
   with the Spanish edition's rules (`reduce_edition_es.ES`) and it-es's own pre-pass
   (`reduce_italian_es.py`, D4).
2. `kaikki-it-traductions-es.jsonl`: the Spanish translations the Italian Wiktionary's Italian entries
   list (direct), in the table's order.
3. `kaikki-es-traductions-it.jsonl`: the Spanish entries of the Spanish Wiktionary whose translation
   tables list the Italian word (inverted, read backwards), the commonest Spanish word first.
4. `wikidata-lexemes-it-es.jsonl`: Wikidata's translation links between an Italian and a Spanish sense
   (P5972) and its Spanish glosses of Italian senses; then
5. the same file's lexemes of one item — an Italian and a Spanish sense naming the same item (P5137).
6. `kaikki-it-Spagnolo.jsonl`: the Italian Wiktionary's Spanish section, read backwards (change 27a's
   D3 rule), the commonest Spanish word first.
7. `kaikki-{en,fr,de,pt,ru,pl,nl}-pivot-it-es.jsonl`: the pivot (D3) — an Italian and a Spanish word a
   person wrote under the same sense of another Wiktionary's translation table, or the same entry of
   one sense —, the Spanish words of a lemma by the tables and the editions giving them, then Spanish
   frequency, then the alphabet; no expression. The exception the programme's I11 records for this pair.

At most three Spanish words per part of speech from each table-shaped source (every source but the
definitions), each read through fr-es's table rules copied into `reduce_spanish_tables.py` and it-es's
own (D4): no letter, no word with a grave accent or « ° », no Italian word given as Spanish but
`ITALIAN_ALIKE`'s, each Spanish word listed once across the Italian word's parts of speech.

Inputs: `tables/it/forms.tsv`, `freq.tsv` and `grammar.tsv` (`--studied`, the committed studied
folder); the sources above, in `--work` (`pack_sources.py DUMPS["it-es"]`). Outputs, in `--work`:
`gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json`; and `measures.json`, what the
reduction measures of its glosses for the update's report, which no table holds (D9).

The pair's rule digest is this file, `reduce_italian_es.py` and `reduce_spanish_tables.py` (D4),
`reduce_italian_acronyms.py` (change 11's rows), `reduce_common.py` and `reduce_edition_es.py`: no other
pair's reducer is loaded. A change to it-fr's rules reaches this pair through the committed tables
(`pack_sources.py check-reducer`).
"""

import argparse
import collections
import functools
import json
import os
import random
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_es as spanish  # noqa: E402 — the Spanish Wiktionary's rules: it-es's glosses are Spanish
import reduce_italian_acronyms as acronyms  # noqa: E402 — Italian's acronym rows (keep-lingua-italian-acronyms-and-names)
import reduce_italian_es as italian  # noqa: E402 — it-es's own pre-pass (D4)
import reduce_spanish_tables as tables  # noqa: E402 — fr-es's table rules, copied (D4)

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")
# The committed studied folder: build.sh names this run's (a dry run's scratch copy) in LINGUA_STUDIED.
_STUDIED = os.environ.get("LINGUA_STUDIED") or os.path.join(_HERE, "tables", "it")

# The native side's sources, in `--work` (pack_sources.py DUMPS["it-es"]).
SECTION = "kaikki-es-Italiano.jsonl"
DIRECT = "kaikki-it-traductions-es.jsonl"
INVERTED = "kaikki-es-traductions-it.jsonl"
WIKIDATA = "wikidata-lexemes-it-es.jsonl"
SPANISH_SECTION = "kaikki-it-Spagnolo.jsonl"
# The pivot's editions, in the order `DUMPS["it-es"]` names them.
PIVOT = {edition: f"kaikki-{edition}-pivot-it-es.jsonl" for edition in ("en", "fr", "de", "pt", "ru", "pl", "nl")}
# What the native side writes beside its sources, as it reads them — no table.
SECTION_STRAIGHT = "kaikki-es-Italiano-straight.jsonl"
SECTION_FUNCTION_WORDS = "kaikki-es-Italiano-function-words.jsonl"
SECTION_WORDS = "kaikki-es-Italiano-words.jsonl"
SECTION_MEANINGS = "kaikki-es-Italiano-meanings.jsonl"
SECTION_READ = "kaikki-es-Italiano-read.jsonl"
DIRECT_READ = "kaikki-it-traductions-es-read.jsonl"
INVERTED_READ = "kaikki-es-traductions-it-read.jsonl"

EDITION = spanish.ES

# Italian for the shared rules (`reduce_common.Studied`): change 9's token pattern and it-fr's
# coordinators, repeated because it-fr's reducer is not loaded (D1) — test_reduce_it_es.py holds them
# to it-fr's.
IT = common.Studied(
    code="it",
    token=italian.TOKEN,
    # Not read by the native side; kept for the shared rules' interface.
    form_of_target=re.compile(rf"([{italian.LETTERS}]+)\s*\.?$"),
    coordinators=frozenset({"e", "ed", "o", "ma", "sia", "oppure", "né", "cioè", "ovvero", "ossia", "nonché", "bensì"}),
)

# fr-es's table rules, given Italian (D4): the translation template's code kaikki reads as an Italian
# word is « it » (`it` « Manzano, maletero » through `manzano`'s and `maletero`'s tables); no
# correction of a table and no other sense named yet; the Spanish words Italian writes alike.
TABLE_RULES = tables.Rules(studied=IT, language_code="it", alike=italian.ITALIAN_ALIKE, straight=italian.straight)


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
    forms `forms.tsv` maps to themselves but the acronym rows (each its own lemma, with no rank),
    capped at `max_lemmas` by rank. The two tables must agree, or the folder is not what it-fr's
    reduction writes."""
    rows = acronyms.ACRONYMS if rows is None else rows
    forms = os.path.join(studied, "forms.tsv")
    freq = os.path.join(studied, "freq.tsv")
    for path in (forms, freq):
        if not os.path.isfile(path):
            raise SystemExit(f"error: {path} is missing: it-es reads Italian's committed tables (tables/it/, it-fr's)")
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
    """The committed forms: `form → lemma`, for the split words: a form is looked up whole."""
    return dict(read_table(os.path.join(studied, "forms.tsv")))


def straight_headwords(src, dst):
    """The section's entries with each headword written as Italian's tables write it
    (`italian.straight`): `c’è` is `c'è`."""

    def read(entry):
        word = entry.get("word")
        return {**entry, "word": italian.straight(word)} if isinstance(word, str) and italian.straight(word) != word else entry

    return spanish.rewrite_entries(src, dst, read)


# — The table rules, on every table-shaped source (D4) —


def table_rules(table, readings, zipf):
    """A table (`word → {UPOS: [Spanish word, …]}`) as it-es reads every table-shaped source: no
    one-letter Italian word (a letter); each Spanish word in the edition's typography; none with a grave
    accent or « ° »; none that is the Italian word itself but `ITALIAN_ALIKE`'s
    (`tables.is_the_studied_word`, `zipf(word, language)`); each listed once across the Italian word's
    parts of speech, under the first of them Italian's readings name, or else the first listed. Each
    list keeps its order."""
    table = {word: by_pos for word, by_pos in table.items() if len(word) > 1}
    table = tables.in_typography(table)
    table = italian.without_marked_words(table)
    if zipf is not None:
        table = tables.without_studied_words(table, zipf, TABLE_RULES)
    return tables.listed_once(table, readings)


def read_translated(path, dst, *, inverted, readings, zipf, stats=None):
    """A translation file (`pack_sources.derive`) as it-es reads it: through fr-es's file rules
    (`tables.translation_words`: letters' entries out, the Italian word written as Italian's tables
    write it, read backwards the language code, names and acronyms refused), read by the shared rules
    (`common.read_translations`), then `table_rules`."""
    words = tables.translation_words(path, dst, inverted=inverted, rules=TABLE_RULES, stats=stats)
    return table_rules(common.read_translations(words, inverted=inverted, studied=IT), readings, zipf)


# — Wikidata's lexemes (D2) —

# The lexical categories of a lexeme, by item, as the shared rules name parts of speech; a category
# not listed is `X`. A proper noun's, a letter's and an affix's lexeme gloss nothing, as a name's, a
# letter's and an affix's entry gloss nothing in a Wiktionary's table.
WIKIDATA_CATEGORIES = {
    "Q1084": "NOUN",
    "Q24905": "VERB",
    "Q34698": "ADJ",
    "Q380057": "ADV",
    "Q36224": "PRON",
    "Q468801": "PRON",
    "Q4833830": "ADP",
    "Q36484": "CCONJ",
    "Q83034": "INTJ",
    "Q63116": "NUM",
    "Q576271": "DET",
    "Q2865743": "DET",
}
WIKIDATA_NOTHING = frozenset({"Q147276", "Q9788", "Q134830", "Q102047", "Q62155"})


def _studied_key(word):
    """An Italian word as the pack keys it (`common.studied_headword`), written as Italian's tables write
    it; None when it is no Italian word."""
    return common.studied_headword(italian.straight(word), studied=IT) if isinstance(word, str) else None


def read_wikidata(path):
    """Wikidata's Italian–Spanish pairs (`pack_sources.lexeme_pairs`), as two tables, each `word →
    {UPOS: [Spanish word, …]}` with the Spanish words of a part of speech by how many senses pair them,
    then their order of first appearance: `(links, items)` — the translation links between an Italian
    and a Spanish sense (P5972) and the Spanish glosses of Italian senses; the senses of one item
    (P5137). The part of speech is the Italian lexeme's lexical category; a proper noun's, a letter's or
    an affix's lexeme glosses nothing; an Italian lemma that is no Italian word, none."""
    counts = {"links": collections.defaultdict(collections.Counter), "items": collections.defaultdict(collections.Counter)}
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                pair = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(pair, dict) or not isinstance(pair.get("it"), dict) or not isinstance(pair.get("es"), dict):
                continue
            category = pair["it"].get("category")
            if category in WIKIDATA_NOTHING or pair["es"].get("category") in WIKIDATA_NOTHING:
                continue
            key = _studied_key(pair["it"].get("lemma"))
            native = pair["es"].get("lemma") if pair.get("by") != "gloss" else pair["es"].get("gloss")
            native = common.native_words(native) if isinstance(native, str) else ""
            if not key or not native:
                continue
            upos = WIKIDATA_CATEGORIES.get(category, "X")
            if upos == "CCONJ" and key not in IT.coordinators:
                upos = "SCONJ"
            which = "items" if pair.get("by") == "item" else "links"
            counts[which][(key, upos)][native] += 1
    out = []
    for which in ("links", "items"):
        table = collections.defaultdict(dict)
        for (key, upos), natives in counts[which].items():
            order = list(natives)
            table[key][upos] = sorted(natives, key=lambda n: (-natives[n], order.index(n)))
        out.append(dict(table))
    return tuple(out)


# — The Italian Wiktionary's Spanish section, read backwards (change 27a's D3 rule) —

_PARENTHESIS = re.compile(r"\s*\([^()]*\)")


def spanish_section_words(text):
    """The Italian words a sense of the Italian Wiktionary's Spanish section gives: its notes to its
    readers out (`italian.without_spanish_section_notes`), its parentheses out, split on « , » and « ; »
    into at most three parts, each of at most two words, as the pack keys an Italian word."""
    text = italian.without_spanish_section_notes(text)
    if text is None:
        return []
    text = _PARENTHESIS.sub("", text)
    out = []
    for part in re.split(r"[,;]", text)[:3]:
        part = part.strip().rstrip(".").strip()
        if not part or len(part.split()) > 2:
            continue
        key = _studied_key(part)
        if key and key not in out:
            out.append(key)
    return out


def read_spanish_section(path, stats=None):
    """The Italian Wiktionary's Spanish section read backwards: `word → {UPOS: [Spanish word, …]}` — of
    each lowercase Spanish entry that is no proper noun's or character's, the first gloss of each sense
    that is no tagged pointer gives its Italian words (`spanish_section_words`), the Spanish headword
    listed for each, under its entry's part of speech."""
    stats = stats if stats is not None else collections.Counter()
    out = collections.defaultdict(lambda: collections.defaultdict(list))
    for entry, _ in italian.entries(path):
        if entry is None:
            continue
        headword = entry.get("word")
        if not isinstance(headword, str) or headword != headword.lower() or entry.get("pos") in ("name", "character"):
            continue
        native = common.native_words(headword)
        for sense in entry.get("senses") or ():
            if not isinstance(sense, dict) or italian._tagged(sense):
                continue
            text = italian.gloss_text(sense)
            if text and italian.without_spanish_section_notes(text) != text:
                stats["the Spanish section's notes taken out"] += 1
            for key in spanish_section_words(text):
                upos = common.kaikki_upos(entry.get("pos") or "", key, studied=IT)
                if native not in out[key][upos]:
                    out[key][upos].append(native)
    return {key: dict(by_pos) for key, by_pos in out.items()}


# — The pivot (D3) —

# The tables the French and Dutch Wiktionaries leave to sort, mixing senses, in every spelling the dumps
# write: « Traductions à trier suivant le sens », « À trier », « Traductions à trier », « à trier »,
# « A trier », « Sens à trier », « Vertalingen: na te kijken en onderverdelen per verklaring », « Na te
# kijken en uit te splitsen vertalingen ».
_TO_SORT = re.compile(r"(?:^|\W)[àaÀA] trier\b|\bna te kijken\b", re.IGNORECASE)
# The parts of speech of an entry that glosses nothing: a name, an abbreviation, a symbol, a character,
# a letter, an affix, punctuation.
PIVOT_NOTHING = frozenset(
    {"name", "abbrev", "symbol", "character", "letter", "affix", "prefix", "suffix", "infix", "interfix", "circumfix", "punct"}
)
# A translation split into its words (the implementation's addition to D3): the Polish and the Russian
# Wiktionaries' extracts list each linked word of a translation as a translation of its own (`dworzec
# autobusowy` « stazione, degli, autobus » « estación, de, autobuses », `ковровый` « di » « de »), which
# glossed `degli` « De, estación, autobuses » and `del` « …; Floristería, torrero, farero ». A group of a
# content word's entry whose Italian or Spanish words hold an article, a preposition or a conjunction of
# these closed lists — never a word a content word translates as — is such a split, and glosses nothing.
SPLIT_ITALIAN = frozenset(
    "il lo la i gli le l' un uno una un' di da con per del dello della dei degli delle dell' al allo alla ai "
    "agli alle all' dal dallo dalla dai dagli dalle dall' nel nello nella nei negli nelle nell' sul sullo sulla "
    "sui sugli sulle sull' col coi e ed che".split()
)
SPLIT_SPANISH = frozenset("el la los las lo un una unos unas de del al con para que y e".split())
# The parts of speech of a function word's or a number's entry, whose tables list such words as
# translations of their own.
FUNCTION_ENTRY = frozenset({"prep", "conj", "article", "det", "pron", "particle", "contraction", "postp", "num"})


def split_translation(record):
    """Whether a pivot group lists a translation split into its words (`SPLIT_ITALIAN`, `SPLIT_SPANISH`):
    a content word's entry whose Italian or Spanish words hold an article, a preposition or a
    conjunction."""
    if record.get("pos") in FUNCTION_ENTRY:
        return False
    return any(isinstance(w, str) and w.strip().lower() in SPLIT_ITALIAN for w in record.get("it") or ()) or any(
        isinstance(w, str) and w.strip().lower() in SPLIT_SPANISH for w in record.get("es") or ()
    )


# The longest Spanish word the pivot keeps.
PIVOT_LONGEST = 60
# The longest acronym the pivot keeps in capitals.
PIVOT_ACRONYM = 6


def _capitalised(word):
    return any(c.isupper() for c in word)


def _short_acronym(word):
    letters = [c for c in word if c.isalpha()]
    return 2 <= len(letters) <= PIVOT_ACRONYM and all(c.isupper() for c in letters)


def pivot_group_kept(record, edition):
    """Whether a pivot record's group may gloss (D3): a table named by a sense, a sense number or the
    sense it sits under — never one left to sort (`_TO_SORT`) —, or the whole entry of an entry of one
    sense; no entry of `PIVOT_NOTHING`'s parts of speech, no translation split into its words
    (`split_translation`), no one-letter headword, no capitalised headword but a German noun."""
    by = record.get("by")
    if by == "entry" and record.get("several_senses"):
        return False
    if by not in ("sense", "sense_index", "under", "entry"):
        return False
    if by == "sense" and _TO_SORT.search(record.get("sense") or ""):
        return False
    if record.get("pos") in PIVOT_NOTHING or split_translation(record):
        return False
    headword = record.get("word")
    if not isinstance(headword, str) or len(headword.strip()) <= 1:
        return False
    return not (_capitalised(headword) and not (edition == "de" and record.get("pos") == "noun"))


def pivot_word_kept(word):
    """Whether a word a pivot table lists may pair: not capitalised but an acronym of at most six
    letters."""
    return isinstance(word, str) and bool(word.strip()) and (not _capitalised(word) or _short_acronym(word))


def spanish_word_kept(word):
    """Whether a Spanish word of a pivot table may gloss: `pivot_word_kept`, no grave accent or « ° »,
    at most `PIVOT_LONGEST` characters."""
    return pivot_word_kept(word) and not italian._MARKED.search(word) and len(word.strip()) <= PIVOT_LONGEST


class Pivot:
    """The pivot read: for each Italian lemma and part of speech, each Spanish word with the tables and
    the editions that pair them, and the first table that does (its edition, entry, part of speech and
    sense), for the sample."""

    def __init__(self):
        self.tables = collections.defaultdict(lambda: collections.defaultdict(lambda: collections.defaultdict(set)))
        self.editions = collections.defaultdict(lambda: collections.defaultdict(lambda: collections.defaultdict(set)))
        self.first = {}
        self.groups = collections.Counter()

    def read(self, path, edition):
        with open(path, encoding="utf-8") as f:
            for number, line in enumerate(f, 1):
                try:
                    record = json.loads(line)
                except json.JSONDecodeError:
                    continue
                if not isinstance(record, dict):
                    continue
                if not pivot_group_kept(record, edition):
                    self.groups["groups left out"] += 1
                    continue
                self.groups["groups read"] += 1
                natives = [common.native_words(w) for w in record.get("es") or () if spanish_word_kept(w)]
                natives = [n for n in natives if n]
                keys = [_studied_key(w) for w in record.get("it") or () if pivot_word_kept(w)]
                table = (edition, number)
                for key in dict.fromkeys(k for k in keys if k and " " not in k):
                    upos = common.kaikki_upos(record.get("pos") or "", key, studied=IT)
                    for native in natives:
                        self.tables[key][upos][native].add(table)
                        self.editions[key][upos][native].add(edition)
                        self.first.setdefault((key, native), (edition, record))

    def table(self, frequency):
        """`word → {UPOS: [Spanish word, …]}`, a part of speech's words by the tables giving them, then
        the editions, then Spanish frequency, then the alphabet (D3). Measured against the two
        Wiktionaries' tables on the 8,766 lemmas they and the pivot gloss, the first word agrees 78.0 %
        so, 77.7 % editions first, 77.8 % by tables alone, 45.9 % by frequency alone."""
        out = {}
        for key, by_pos in self.tables.items():
            out[key] = {
                upos: sorted(
                    natives,
                    key=lambda n, k=key, u=upos: (
                        -len(self.tables[k][u][n]),
                        -len(self.editions[k][u][n]),
                        -frequency(n),
                        n,
                    ),
                )
                for upos, natives in by_pos.items()
            }
        return out

    def provenance(self, key, native):
        """Where a pivot word first comes from: « en buyer (noun), « person who makes purchases » »."""
        found = self.first.get((key, native))
        if found is None:
            return None
        edition, record = found
        sense = record.get("sense") or (f"sense {record['sense_index']}" if record.get("sense_index") else None)
        if sense is None and record.get("by") == "under":
            sense = f"sense {record.get('under')}"
        return f"{edition}: {record.get('word')} ({record.get('pos')}), « {sense or 'its one sense'} »"


# — The native side, step by step (D1, D2, D9) —

# The table-shaped sources in D2's order, after the definitions.
SOURCES = ("definitions", "direct", "inverted", "wikidata links", "wikidata items", "spanish section", "pivot")


def native_side(section, ranks, sources):
    """The native side step by step, so that the lemmas each source glosses are kept (D1, D2, D9): the
    section's definitions first (`common.reduce_gloss`), then each `(name, source, order)` of `sources`,
    in turn, for what the steps before left out (`common.fallback_glosses`); the expressions from the
    section, then from the sources named `direct` and `inverted` (D6). `(glosses, runs, expressions,
    steps)`, the steps `{name: lemmas}`, each the set a step glossed, and `"section expressions"`."""
    runs = {}
    glosses = common.reduce_gloss(section, set(ranks), **common.WORD_GLOSS, runs=runs, studied=IT, edition=EDITION)
    expressions = common.reduce_expressions(section, studied=IT, edition=EDITION)
    steps = {"definitions": frozenset(glosses), "section expressions": frozenset(expressions)}
    for name, source, order in sources:
        found = common.fallback_glosses(ranks, glosses, [(source, order)], edition=EDITION)
        for lemma, (gloss, gloss_runs) in found.items():
            glosses[lemma], runs[lemma] = gloss, gloss_runs
        steps[name] = frozenset(found)
    tabled = [(source, order) for name, source, order in sources if name in ("direct", "inverted")]
    expressions.update(common.fallback_expressions(expressions, tabled, edition=EDITION))
    return glosses, runs, expressions, steps


def acronym_glosses(glosses, expressions, entries):
    """Each acronym row's gloss (D7, keep-lingua-italian-acronyms-and-names D3): from it-es's own glosses
    of the row's expansions, else of its synonyms — a word's from `glosses`, a headword with a space from
    `expressions` —, a synonym the pair does not gloss read from its entry in the section whatever its
    part of speech (`entries`, `italian.section_words`). `acronym → (gloss, runs)`."""
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
    return {italian.straight(w).lower() for expansions, synonyms, _, _ in rows.values() for w in (*expansions, *synonyms)}


# D9's sample: 100 glossed lemmas of the 20,000 commonest, drawn by this seed, each marked by source —
# it-fr's and it-en's seed and size, so the pairs' samples read alike.
SAMPLE_SIZE = 100
SAMPLE_TOP = 20_000
SAMPLE_SEED = 20261024
COVERAGE_TOPS = (5_000, 10_000, 20_000)
MEASURED_TOP = 10_000


def source_of(lemma, steps):
    """The source that glossed a lemma, or None."""
    return next((name for name in SOURCES if lemma in steps.get(name, ())), None)


def coverage(ranks, glosses, tops=COVERAGE_TOPS):
    """The share of each top's commonest ranked lemmas with a gloss, in percent to one decimal — as
    `gloss_coverage.measure` reads the committed tables."""
    ranked = sorted(ranks, key=lambda lemma: (ranks[lemma], lemma))
    return [round(100 * sum(1 for lemma in ranked[:top] if lemma in glosses) / len(ranked[:top]), 1) for top in tops]


def coverage_by_source(ranks, steps, tops=COVERAGE_TOPS):
    """The coverage after each source in D2's order, cumulative: `{source: [share, …]}`."""
    out, glossed = {}, set()
    for name in SOURCES:
        glossed |= steps.get(name, frozenset())
        out[name] = coverage(ranks, glossed, tops)
    return out


def pivot_editions(steps, glosses, runs, pivot, pivot_table):
    """The pivot-glossed lemmas by edition: each lemma counted under every edition a table of which
    gives one of the Spanish words its gloss shows."""
    out = collections.Counter()
    for lemma in steps.get("pivot", ()):
        shown = set()
        for upos, natives in pivot_table.get(lemma, {}).items():
            shown |= {(upos, n) for n in natives[: common.FALLBACK_WORDS]}
        editions = set()
        for upos, native in shown:
            editions |= pivot.editions[lemma][upos].get(native, set())
        for edition in editions:
            out[edition] += 1
    return {edition: out[edition] for edition in PIVOT if out[edition]}


def sample_source(lemma, steps, pivot, pivot_table):
    """A sample row's source: its name; a pivot row's with the edition, entry and sense of its first
    Spanish word's first table."""
    name = source_of(lemma, steps)
    if name != "pivot":
        return name
    for natives in pivot_table.get(lemma, {}).values():
        if natives:
            where = pivot.provenance(lemma, natives[0])
            if where:
                return f"pivot ({where})"
    return name


def measures(ranks, glosses, steps, top=MEASURED_TOP, *, pivot=None, pivot_table=None, runs=None):
    """What the reduction measures of its glosses and stores in no pack (D9): among the glossed lemmas of
    the `top` commonest, the share a table-shaped source glossed, with the lemmas the two Wiktionaries'
    tables glossed; the coverage at each of `COVERAGE_TOPS`, and after each source; the glossed lemmas
    by source, the pivot by edition; and the sample, each lemma with its rank, its source — a pivot row
    with its edition, entry and sense — and its gloss."""
    ranked = sorted(ranks, key=lambda lemma: (ranks[lemma], lemma))
    commonest = set(ranked[:top])
    glossed = sum(len(steps.get(name, frozenset()) & commonest) for name in SOURCES)
    tabled = sum(len(steps.get(name, frozenset()) & commonest) for name in SOURCES[1:])
    share = round(100 * tabled / glossed, 1) if glossed else 0.0
    pool = [lemma for lemma in ranked[:SAMPLE_TOP] if lemma in glosses]
    drawn = random.Random(SAMPLE_SEED).sample(pool, min(SAMPLE_SIZE, len(pool)))
    sample = [
        [lemma, ranks[lemma], sample_source(lemma, steps, pivot, pivot_table or {}), glosses[lemma]]
        for lemma in sorted(drawn, key=ranks.get)
    ]
    return {
        "top": top,
        "glossed": glossed,
        "share": share,
        "direct": sorted(steps.get("direct", frozenset()) & commonest),
        "inverted": sorted(steps.get("inverted", frozenset()) & commonest),
        "coverage": dict(zip((str(t) for t in COVERAGE_TOPS), coverage(ranks, glosses))),
        "coverage by source": coverage_by_source(ranks, steps),
        "by source": {name: len(steps.get(name, frozenset())) for name in SOURCES},
        "pivot by edition": pivot_editions(steps, glosses, runs, pivot, pivot_table or {}) if pivot is not None else {},
        "sample": sample,
    }


def analyser_version(path=_ANALYSIS_RS):
    """Italian's analyser version, read from lingua-core so a bump cannot leave the manifest behind."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const ITALIAN_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no ITALIAN_ANALYZER_VERSION in {path}")
    return m.group(1)


# The credits (D9): both sides' sources. The native side: the Spanish and Italian Wiktionaries, the seven
# Wiktionaries of the pivot, Wikidata's Lexemes under CC0, wordfreq's Spanish list. The studied side is
# it-fr's reduction's, read as committed, credited as it-fr's notice credits it; the levels are
# estimated.
NOTICE = """Cymbra Lingua data pack — IT->ES attributions.

kaikki.org extract of the Spanish Wiktionary (eswiktionary), Italian section: CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/) + GFDL (https://www.gnu.org/licenses/fdl-1.3.html)
— the Spanish glosses of Italian words and expressions, from its definitions; and the Italian
translations its Spanish entries list, read backwards where no definition glosses a word.

kaikki.org extract of the Italian Wiktionary (itwiktionary): CC BY-SA 4.0
(https://creativecommons.org/licenses/by-sa/4.0/) + GFDL (https://www.gnu.org/licenses/fdl-1.3.html)
— the Spanish translations its Italian entries list, and its Spanish section, read backwards.

Wikidata Lexemes (https://www.wikidata.org/wiki/Wikidata:Lexicographical_data): CC0 1.0
(https://creativecommons.org/publicdomain/zero/1.0/) — the Spanish senses its Italian senses name as
their translations or share an item with, and its Spanish glosses of Italian senses.

kaikki.org extracts of the English (enwiktionary), French (frwiktionary), German (dewiktionary),
Portuguese (ptwiktionary), Russian (ruwiktionary), Polish (plwiktionary) and Dutch (nlwiktionary)
Wiktionaries: CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/) + GFDL
(https://www.gnu.org/licenses/fdl-1.3.html) — an Italian and a Spanish word a person wrote under the
same sense of one of their translation tables, read last, where no other source glosses a word.

wordfreq (Italian and Spanish frequency lists), by Robyn Speer (https://github.com/rspeer/wordfreq):
data under CC BY-SA 4.0 (https://creativecommons.org/licenses/by-sa/4.0/) — the commonest Italian
lemmas and which forms are attested (Italian's tables, read here as committed), and the order of the
Spanish words a table lists.

kaikki.org extract of the English Wiktionary (enwiktionary), Italian section: CC BY-SA 4.0 + GFDL —
the forms and their lemmas (Italian's tables, reduced by the it-fr pack's reduction and read here as
committed).

UD Italian-MarkIT, the Universal Dependencies Italian-MarkIT treebank by Teresa Paccosi, Alessio
Palmero Aprosio and Sara Tonelli (https://github.com/UniversalDependencies/UD_Italian-MarkIT):
CC BY 4.0 (https://creativecommons.org/licenses/by/4.0/) — how often a form stands for each of its
lemmas, to choose one, and how often a hyphenated word occurs, to rank it; its training and
development sections alone.

Morph-it!, a free morphological lexicon for the Italian language, by Marco Baroni and Eros Zanchetta
(https://docs.sslmit.unibo.it/doku.php?id=resources:morph-it), version 0.4.8: CC BY-SA 2.0
(https://creativecommons.org/licenses/by-sa/2.0/), one of its two licences, the other being the
LGPL — the forms it lists that the Wiktionary names nowhere, under the lemmas the tables rank.

kaikki.org extract of the French Wiktionary (frwiktionary), Italian section: CC BY-SA 4.0 + GFDL —
which lemmas the it-fr pack glosses: Italian's dictionary words and which take a level (Italian's
tables, read here as committed).

The levels are estimated, not taken from a CEFR list: the it-fr pack's reduction derives them from the
frequency ranks, the commonest lemmas taking the sizes of English's CEFR levels, and every pack studying
Italian reads them as committed.
"""


def manifest(pack_version):
    """it-es's manifest (D9): Italian glossed in Spanish, Italian's analyser version, `levels_estimated`,
    and every source's licence — Wikidata's CC0 a licence of its own (`Cc0`)."""
    return {
        "meta": {
            "studied": "it",
            "native": "es",
            "pack_version": pack_version,
            "analyzer_version": analyser_version(),
            "levels_estimated": True,
            "licences": [
                "kaikki / eswiktionary, itwiktionary, enwiktionary, frwiktionary, dewiktionary, ptwiktionary, "
                "ruwiktionary, plwiktionary, nlwiktionary (CC BY-SA 4.0 + GFDL)",
                "Wikidata Lexemes (CC0 1.0)",
                "wordfreq (CC BY-SA 4.0)",
                "UD Italian-MarkIT (CC BY 4.0)",
                "Morph-it! (CC BY-SA 2.0)",
            ],
        },
        "sources": [
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "Wikidata", "licence": "Cc0"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "UD Italian-MarkIT", "licence": "CcBy"},
            {"name": "Morph-it!", "licence": "CcBySa"},
        ],
    }


def reduce(work, studied, max_lemmas, pack_version, *, zipf, report=print):
    """The reduction over the raw sources in `work`, its tables written there: answers what the report
    prints, as a dict, for the measurement. `zipf(word, language)` is wordfreq's Zipf frequency."""
    needed = (SECTION, DIRECT, INVERTED, WIKIDATA, SPANISH_SECTION, *PIVOT.values())
    absent = [name for name in needed if not os.path.isfile(os.path.join(work, name))]
    if absent:
        raise SystemExit(
            f"it-es: {', '.join(os.path.join(work, name) for name in absent)} missing: it-es's glosses read the "
            "files pack_sources.py DUMPS['it-es'] names"
        )
    ranks = read_studied(studied, max_lemmas)
    forms = read_forms(studied)
    readings = tables.read_readings(os.path.join(studied, "grammar.tsv"))

    def frequency(word):
        return zipf(word, EDITION.code)

    stats = collections.Counter()
    direct = read_translated(
        os.path.join(work, DIRECT), os.path.join(work, DIRECT_READ), inverted=False, readings=readings, zipf=zipf, stats=stats
    )
    inverted = read_translated(
        os.path.join(work, INVERTED), os.path.join(work, INVERTED_READ), inverted=True, readings=readings, zipf=zipf, stats=stats
    )
    links, items = (table_rules(t, readings, zipf) for t in read_wikidata(os.path.join(work, WIKIDATA)))
    spanish_section = table_rules(read_spanish_section(os.path.join(work, SPANISH_SECTION), stats), readings, zipf)
    pivot = Pivot()
    for edition, name in PIVOT.items():
        pivot.read(os.path.join(work, name), edition)
    pivot_table = table_rules(pivot.table(frequency), readings, zipf)

    fired = set()
    section = straight_headwords(os.path.join(work, SECTION), os.path.join(work, SECTION_STRAIGHT))
    section = italian.function_words(section, os.path.join(work, SECTION_FUNCTION_WORDS), fired)
    section = common.without_letter_senses(section, os.path.join(work, SECTION_WORDS), edition=EDITION)
    section = spanish.read_as_meanings(section, os.path.join(work, SECTION_MEANINGS))
    tabled = frozenset(direct) | frozenset(inverted)
    section, read_stats = italian.read_section(section, os.path.join(work, SECTION_READ), tabled=tabled)
    stats.update(read_stats)

    by_frequency = common.by_native_frequency(frequency)
    sources = [
        ("direct", direct, list),
        ("inverted", inverted, by_frequency),
        ("wikidata links", links, list),
        ("wikidata items", items, list),
        ("spanish section", spanish_section, by_frequency),
        ("pivot", pivot_table, list),
    ]
    glosses, runs, expressions, steps = native_side(section, ranks, sources)
    # The words Italian's pre-pass splits, after the headwords with a space: the section's, then the two
    # tables' (D6); and the entries of the acronym rows' synonyms, whatever their part of speech (D7).
    split, row_entries = italian.section_words(section, forms, row_words())
    split_table = italian.table_split_words([(direct, list), (inverted, by_frequency)], forms, split)
    with_space = len(expressions)
    expressions.update(split)
    expressions.update(split_table)
    glossed_acronyms = acronym_glosses(glosses, expressions, row_entries)
    for acronym, (gloss, gloss_runs) in glossed_acronyms.items():
        glosses[acronym], runs[acronym] = gloss, gloss_runs

    common.write(work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    common.write(
        work,
        "senses.tsv",
        "".join(f"{w}\t" + "\t".join(f"{pos}:{n}" for pos, n in r) + "\n" for w, r in sorted(runs.items()) if w in glosses),
    )
    common.write(work, "mwe.tsv", "".join(f"{w}\t{g}\n" for w, g in sorted(expressions.items())))
    measured = measures(ranks, glosses, steps, pivot=pivot, pivot_table=pivot_table, runs=runs)
    common.write(work, "measures.json", json.dumps(measured, indent=2, ensure_ascii=False) + "\n")
    common.write(work, "NOTICE", NOTICE)
    common.write(work, "manifest.json", json.dumps(manifest(pack_version), indent=2, ensure_ascii=False) + "\n")

    missed = italian.unfired(fired)
    summary = {
        "lemmas": len(ranks),
        "glosses": len(glosses),
        "glossed by source": measured["by source"],
        "pivot by edition": measured["pivot by edition"],
        "coverage": measured["coverage"],
        "coverage by source": measured["coverage by source"],
        "translation share": measured["share"],
        "expressions": len(expressions),
        "expressions by source": {
            "the section, with a space": len(steps["section expressions"]),
            "the two tables, with a space": with_space - len(steps["section expressions"]),
            "split words, the section": len(split),
            "split words, the two tables": len(split_table),
        },
        "as read": dict(stats),
        "pivot groups": dict(pivot.groups),
        "acronym glosses": {acronym: gloss for acronym, (gloss, _) in glossed_acronyms.items()},
        "function words unfired": missed,
    }
    report(
        f"reduced it-es: lemmas={len(ranks)} (Italian's committed tables) glosses={len(glosses)} ("
        + ", ".join(f"{name} {n}" for name, n in measured["by source"].items())
        + f", the acronym rows {len(glossed_acronyms)}); of the 5,000 / 10,000 / 20,000 commonest lemmas, "
        + " / ".join(f"{share} %" for share in measured["coverage"].values())
        + f" glossed; of the {measured['glossed']:,} glossed among the {measured['top']:,} commonest, "
        f"{measured['share']} % from a table-shaped source"
    )
    report(
        "reduced it-es: coverage after each source: "
        + "; ".join(f"{name} " + " / ".join(map(str, shares)) for name, shares in measured["coverage by source"].items())
    )
    report("reduced it-es: the pivot by edition: " + ", ".join(f"{e} {n}" for e, n in measured["pivot by edition"].items()))
    report(
        f"reduced it-es: expressions={len(expressions)} ("
        + ", ".join(f"{what} {n}" for what, n in summary["expressions by source"].items())
        + ")"
    )
    report("reduced it-es: as read: " + ", ".join(f"{n} {what}" for what, n in sorted(stats.items())))
    report(
        "reduced it-es: the acronym rows "
        + (", ".join(f"{a} « {g} »" for a, g in summary["acronym glosses"].items()) or "none glossed")
    )
    report(
        f"reduced it-es: {len(italian.FUNCTION_WORDS) - len(missed)} of {len(italian.FUNCTION_WORDS)} function words fired"
        + (f"; rows that found nothing, to remove: {', '.join(missed)}" if missed else "")
    )
    for lemma, rank, source, gloss in measured["sample"]:
        report(f"reduced it-es: sample: {rank}\t{lemma}\t{source}\t{gloss}")
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
    # wordfreq orders the Spanish words a table read backwards lists, the commonest first, and tells the
    # Italian word given as Spanish from a word Spanish writes alike.
    from wordfreq import zipf_frequency

    zipf = functools.lru_cache(maxsize=None)(lambda w, language: zipf_frequency(w, language))
    reduce(a.work, a.studied, a.max_lemmas, a.pack_version, zipf=zipf, report=lambda line: print(line, file=sys.stderr))


if __name__ == "__main__":
    main()
