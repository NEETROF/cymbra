#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the fr-es pack into its native side (add-lingua-pack-fr-es).

French glossed in Spanish, for Spanish speakers studying French: a reader pair of French (design
D1). Its studied side — French's forms, ranks, readings, levels, tag pool and dictionary words — is
kept once, in `tables/fr/`, written by fr-en's reduction alone (split-lingua-pack-tables-by-language);
this reducer reads it as committed and computes nothing of it, as en-es reads `tables/en/`. It
writes the native side only, from three tables of glosses written by people, in Spanish (the
programme's M5) — no pivot through a third language, no machine translation (D2):

Inputs:
- `tables/fr/forms.tsv` and `freq.tsv` (`--studied`, the committed studied folder): the lemmas the
  glosses are matched against — those ranked within `--max-lemmas`, 60,000 by build.sh's `fr-*` cap,
  every lemma French commits — and their ranks; `grammar.tsv`, French's readings, which say under
  which part of speech a Spanish word read backwards is listed (D6; none when the file is absent).
- `kaikki-es-Frances.jsonl`, in `--work`: the Spanish Wiktionary's French section, derived from
  kaikki's dump of the whole edition (`pack_sources.py DUMPS`), read for its senses — the
  definitions, cleaned by the Spanish Wiktionary's rules (`reduce_edition_es.ES`). A definition
  glosses first, eight whole senses grouped by part of speech.
- `kaikki-fr-traductions.jsonl`, in `--work`: the Spanish translations the French Wiktionary's
  French entries list, derived from its dump — the direct fallback, in the table's order.
- `kaikki-es-traductions.jsonl`, in `--work`: the French translations the Spanish Wiktionary's
  Spanish entries list, derived from the same dump as the section — read backwards, the inverted
  fallback, the commonest Spanish word first (wordfreq `es`).
At most three Spanish words per part of speech, as one sense, opening on a capital as the edition's
senses do; expressions take the same three steps.

A French headword's typographic apostrophe is read as `'`, as French's forms are (change 43 D3) —
in the section's headwords, the direct table's French entries and the inverted table's French words
(`straight_apostrophes`, `translation_words`): `main-d’œuvre` is the committed lemma `main-d'œuvre`.
Measured, it gains 4 lemmas (`main-d'œuvre`, `chef-d'œuvre`, `inch'allah`, `qu'en-dira-t-on`) and 900
expressions: 6 headwords of the section, 1,317 of the direct table, 37 French words of the inverted
one.

The section reaches the shared rules through three passes, in this order (D4, D5): its letters left
out (`reduce_common.without_letter_senses`); its senses read as meanings, in their order, in one
Spanish typography (`spanish.read_as_meanings`, the Spanish edition's pre-pass, read as en-es reads
it); then which of them gloss a French word (`french_entries`: a surname's or a given name's note
left off the common word spelled like it, a possessive or demonstrative adjective — and an adjective
whose every sense is a form of one — read as a determiner). The translation tables are read through
`read_translated` (D6): a letter's entry left out of both, the inverted table's one-letter French
words dropped, each Spanish word read backwards listed once, each word of both in the edition's
typography. A definition that only repeats the French headword gives way to the direct table's
Spanish words (`no_self_definition`, D7).

Measured on the prototype's sources (the Spanish Wiktionary's dump regenerated 2026-10-02 12:12, the
French Wiktionary's 2026-10-02 00:10; rows / of the top 10,000): the edition's notes change 3 / 3
rows, the order of its outdated senses 19 / 7 (« chapelet » « Rosario; Guirnalda »), its typography 3
expressions; a name's note left off the common word 17 / 12 (« pierre » « Piedra »); possessives and
demonstratives read as determiners 7 / 7, their runs alone (`ce`, `mon`, `ma`, `ton`, `ta`, `notre`,
`mes`); a Spanish word read backwards listed once 94 / 31 (« cet » « Este »); a letter glosses no
word, 11 lemmas losing a gloss that was the letter (`h` « H », `x` « X »); the studied word is no
definition 9 / 4 (« et » « Y, e »). Together 149 / 64 rows and 4 expressions; coverage 83.4 / 70.9 /
56.9 → 83.2 / 70.8 / 56.8 % of the 5,000 / 10,000 / 20,000 commonest lemmas. en-es's translators'
notes and disused words occur in no word a fr-es gloss keeps: not ported.

Outputs, in `--work`: `gloss.tsv`, `senses.tsv`, `mwe.tsv`, `NOTICE` and `manifest.json` — and
`measures.json`, the translation-table share of the glossed top 10,000 (D9), which
`pack_sources.py split` files nowhere: it is shown in the pull request and the tables' README, and
stored in no pack. The share of all glossed lemmas a definition glosses is printed on the summary
line.

Every rule here names a source or is Spanish's: the native side every pair shares
(`reduce_common`, written out step by step in `native_side` so that the lemmas each source glosses
are kept) with the Spanish edition's rules. No other pair's reducer is loaded — a reducer loads code
by import statements alone, and nothing of fr-en, en-es or es-fr may move with this pair — so the
pair's rule digest is this file, `reduce_common.py` and `reduce_edition_es.py`: an edit of the
Spanish edition re-pins en-es and fr-es. A change to fr-en's rules reaches this pair through the
committed tables: when a studied table moves, fr-es's pack moves and its checks name the table
(`pack_sources.py check-reducer`, add-lingua-pack-es-en D3).
"""

import argparse
import functools
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_es as spanish  # noqa: E402 — the Spanish Wiktionary's rules: fr-es's glosses are Spanish

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")
# The committed studied folder: build.sh names this run's (a dry run's scratch copy) in LINGUA_STUDIED.
_STUDIED = os.environ.get("LINGUA_STUDIED") or os.path.join(_HERE, "tables", "fr")

# French, as fr-en's reducer has it — repeated here because that reducer is not loaded (D1): change
# 43's token pattern (letters with French's accents, ligatures and diaeresis, words joined by an
# inner hyphen or apostrophe, an elided piece's final apostrophe), and French's seven coordinators.
_LETTERS = "a-zàâäçéèêëîïôöùûüÿœæ"
_TOKEN = re.compile(rf"[{_LETTERS}]+(?:['-][{_LETTERS}]+)*'?")

FR = common.Studied(
    code="fr",
    token=_TOKEN,
    # Not read by the native side; kept for the shared rules' interface.
    form_of_target=re.compile(rf"([{_LETTERS}]+)\s*\.?$"),
    coordinators=frozenset({"et", "ou", "mais", "ni", "or", "car", "donc"}),
)

EDITION = spanish.ES

# Expressions no source glosses, each with its Spanish gloss, written and reviewed by a person:
# expression → gloss. It wins over every source. A gloss here is never generated. Editing it is a
# rule change.
LOCUTIONS = {}

# Among the glossed lemmas of this many commonest, the share from a translation table (D9).
MEASURED_TOP = 10_000

# The typographic apostrophe French text and the French Wiktionary's headwords write, read as the
# straight one French's forms are keyed with (D2).
_APOSTROPHE = "’"


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
    agree — a lemma is ranked exactly when it is its own form — or the folder is not what fr-en's
    reduction writes, and it is refused, naming both."""
    forms = os.path.join(studied, "forms.tsv")
    freq = os.path.join(studied, "freq.tsv")
    for path in (forms, freq):
        if not os.path.isfile(path):
            raise SystemExit(f"error: {path} is missing: fr-es reads French's committed tables (tables/fr/, fr-en's)")
    lemmas = {form for form, lemma in read_table(forms) if form == lemma}
    ranks = {lemma: int(rank) for lemma, rank in read_table(freq)}
    if set(ranks) != lemmas:
        only_ranked = sorted(set(ranks) - lemmas)[:3]
        only_forms = sorted(lemmas - set(ranks))[:3]
        raise SystemExit(
            f"error: {freq} and {forms} disagree on the lemmas (ranked but no form of itself: {only_ranked}; "
            f"its own form but unranked: {only_forms}): they are not what fr-en's reduction writes"
        )
    return {lemma: rank for lemma, rank in ranks.items() if rank <= max_lemmas}


def read_readings(studied):
    """French's parts of speech by lemma, from the committed `grammar.tsv` (form, lemma, reading):
    lemma → {UPOS}; an empty mapping when the folder holds no readings (D6: the inverted table's
    Spanish words then go under the first part of speech listed). One of the studied tables the
    pin's `studied` record holds."""
    path = os.path.join(studied, "grammar.tsv")
    readings = {}
    if not os.path.isfile(path):
        return readings
    with open(path, encoding="utf-8") as f:
        for line in f:
            columns = line.rstrip("\n").split("\t")
            if len(columns) >= 3:
                readings.setdefault(columns[1], set()).add(columns[2].split("|")[0])
    return readings


def _straight(word):
    return word.replace(_APOSTROPHE, "'") if isinstance(word, str) else word


def _with_straight_word(entry):
    """`entry` with its `word`'s typographic apostrophe read as `'`; itself when there is none."""
    word = entry.get("word")
    return {**entry, "word": _straight(word)} if isinstance(word, str) and _APOSTROPHE in word else entry


def straight_apostrophes(src, dst):
    """The section's entries with each headword's typographic apostrophe read as `'` (D2), written
    to `dst`: `main-d’œuvre` is the committed lemma `main-d'œuvre`, « aller de l’avant » an
    expression French's tokens key. An entry it does not change, and a line it cannot read, are
    written as they are."""
    return spanish.rewrite_entries(src, dst, _with_straight_word)


# — The translation tables (D6) —
#
# A single letter is glossed only by a sense that is neither the letter nor a name borrowed through
# it: the Spanish Wiktionary's letters `h` and `x` list the French letters, its noun `i` the letter
# under its name, and the French Wiktionary's `i` « i latina, i ». en-es's rule
# (add-lingua-pack-en-es D1), on both of fr-es's tables: 11 lemmas among French's 2,300 commonest
# lose a gloss that was the letter.


def _names_a_letter(entry):
    if entry.get("pos") == "character":
        return True
    word = entry.get("word")
    word = word.strip().lower() if isinstance(word, str) else ""
    if len(word) != 1:
        return False
    if entry.get("pos") == "noun":
        return True
    translations = entry.get("translations") if isinstance(entry.get("translations"), list) else ()
    listed = [t.get("word") for t in translations if isinstance(t, dict)]
    return all(isinstance(native, str) and native.strip().lower() == word for native in listed)


def translation_words(src, dst, *, inverted):
    """A translation file as fr-es reads it, written to `dst`: its letters' entries left out — a
    `character` entry, a one-letter word's noun entry (the letter under its name, or a name borrowed
    through it) and a one-letter word every translation of which is itself — and each French word's
    typographic apostrophe read as `'` (D2): the entry's word read forwards (the French Wiktionary's
    French entries), the words it lists read backwards (the Spanish Wiktionary's Spanish entries).
    A line this pass cannot read is written as it is: the shared rules decide."""
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if isinstance(entry, dict):
                if _names_a_letter(entry):
                    continue
                if not inverted:
                    read = _with_straight_word(entry)
                elif isinstance(entry.get("translations"), list):
                    listed = [_with_straight_word(t) if isinstance(t, dict) else t for t in entry["translations"]]
                    read = {**entry, "translations": listed}
                else:
                    read = entry
                if read != entry:
                    line = json.dumps(read, ensure_ascii=False) + "\n"
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def read_translated(path, dst, *, inverted, readings):
    """What a translation file (`pack_sources.derive`) says of French words, read through
    `translation_words` (into `dst`): word → {UPOS: [Spanish word, …]} (`common.read_translations`)
    — the French Wiktionary's Spanish translations read forwards, each word in the edition's
    typography, listed once per part of speech (`in_typography`); the Spanish Wiktionary's French
    translations read backwards, a one-letter French word left out whatever lists it — the entry is
    the Spanish word's, so the letter test sees nothing of the French side —, each Spanish word
    listed once (`listed_once`, `readings` French's parts of speech by lemma)."""
    table = common.read_translations(translation_words(path, dst, inverted=inverted), inverted=inverted, studied=FR)
    if inverted:
        for letter in [word for word in table if len(word) == 1]:
            del table[letter]
        return listed_once(table, readings)
    return in_typography(table)


def in_typography(table):
    """The direct table's words in the edition's typography (`spanish.typography`, « Bromas
    aparte… »), a word listed once per part of speech. Its parts of speech are the French word's own:
    a word listed under two of them stays under both (« parti » « Partido; Partido », as en-es keeps
    « israeli », Open Question 3)."""
    out = {}
    for word, by_pos in table.items():
        kept = {}
        for upos, natives in by_pos.items():
            words = [native for native in dict.fromkeys(spanish.typography(n) for n in natives) if native]
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


def listed_once(table, readings):
    """The inverted table with each Spanish word listed once per French word (D6): read backwards,
    the Spanish Wiktionary lists a French word under each part of speech of the Spanish word (`este`,
    adjective and pronoun, both list `cet`); it is listed under the first of them the French word's
    readings name (`readings`: French lemma → its parts of speech, from `tables/fr/grammar.tsv`),
    else under the first listed. A part of speech left with no word goes; each word in the edition's
    typography."""
    out = {}
    for word, by_pos in table.items():
        by_pos = {upos: list(dict.fromkeys(spanish.typography(n) for n in natives)) for upos, natives in by_pos.items()}
        named = readings.get(word, ())
        home = {}
        for natives in by_pos.values():
            for native in natives:
                if native not in home:
                    listed = [upos for upos, others in by_pos.items() if native in others]
                    home[native] = next((upos for upos in listed if upos in named), listed[0])
        kept = {}
        for upos, natives in by_pos.items():
            words = [native for native in natives if home[native] == upos]
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


# — Which senses gloss a French word (D5) —
#
# The Spanish Wiktionary writes a given name or a surname as a proper noun under its capitalised
# headword, and the shared rules read it for the lower-case lemma: « jean » opened on « Nombre de
# pila de varón, equivalente del español Juan », « pierre » ended on « … Pedro ». It heads `mon`,
# `ce`, `notre` « adjetivo posesivo » or « demostrativo », and `mes` and `ma` as forms of `mon`,
# where a French card says « déterminant ».

# A proper noun's sense that only says the word is a surname or a given name.
_NAME_NOTE = re.compile(r"^(?:Apellido|Nombre de pila|Nombre personal|Hipocorístico)\b")
# The entry tags of an adjective section that is a determiner's.
_DETERMINER_TAGS = frozenset({"possessive", "demonstrative"})


def _first_gloss(sense):
    glosses = sense.get("glosses") if isinstance(sense, dict) else None
    return glosses[0].strip() if isinstance(glosses, list) and glosses and isinstance(glosses[0], str) else ""


def _headword(entry):
    word = entry.get("word")
    return word.strip() if isinstance(word, str) else ""


def _tags(entry):
    tags = entry.get("tags")
    return {tag for tag in tags if isinstance(tag, str)} if isinstance(tags, list) else set()


def _holds_a_meaning(sense):
    """A sense with a gloss that is no pointer (`reduce_common._is_form_of`)."""
    return (
        isinstance(sense, dict)
        and bool(sense.get("glosses"))
        and not common._is_form_of(sense, _first_gloss(sense), edition=EDITION)
    )


def _read_entries(src):
    """`src`'s entries that are JSON objects, in file order."""
    entries = []
    with open(src, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if isinstance(entry, dict):
                entries.append(entry)
    return entries


def _common_words(entries):
    """The headwords written in lower case that have an entry of their own, not a proper noun's,
    holding a meaning — a sense with a gloss that is no pointer."""
    words = set()
    for entry in entries:
        if entry.get("pos") == "name":
            continue
        headword = _headword(entry)
        senses = entry.get("senses")
        if not headword or headword != headword.lower() or not isinstance(senses, list):
            continue
        if any(_holds_a_meaning(sense) for sense in senses):
            words.add(headword)
    return words


def _determiners(entries):
    """The headwords of the adjective sections the edition tags possessive or demonstrative."""
    return {_headword(e) for e in entries if e.get("pos") == "adj" and not _DETERMINER_TAGS.isdisjoint(_tags(e))}


def _forms_of(entry, words):
    """Whether every sense of `entry` is a form of one of `words`: it names them, and them alone, in
    its pointer fields (`form_of`)."""
    senses = entry.get("senses")
    if not isinstance(senses, list) or not senses:
        return False
    for sense in senses:
        if not isinstance(sense, dict):
            return False
        targets = [t.get("word") if isinstance(t, dict) else None for t in common._pointers(sense, EDITION)]
        if not targets or not all(isinstance(t, str) and t.strip() in words for t in targets):
            return False
    return True


def french_entries(src, dst):
    """The Spanish Wiktionary's French entries as they gloss a French word, written to `dst` — a
    pre-pass `main` runs after `spanish.read_as_meanings` (D4, D5), en-es's pair rules transposed:

    - A name does not gloss the common word spelled like it: a sense of a `name` entry whose
      headword opens on a capital, glossed « Apellido… », « Nombre de pila… », « Nombre personal… »
      or « Hipocorístico… », is left out when the word has an entry written in lower case, not a
      proper noun's, that holds a meaning (`_common_words`); an entry left with no sense goes.
      17 / 12 rows (« jean », « pierre » « Piedra », « rose », « romain » « Romano »). A word that
      is only a name keeps its notes (« françois » « Nombre de pila de varón, equivalente del
      español Francisco »): 313 / 167 rows are glossed by them alone, and tell the reader what the
      capitalised token is.
    - Possessives and demonstratives are determiners: an `adj` entry tagged `possessive` or
      `demonstrative` (`mon`, `ton`, `ta`, `notre`, `ce`) is read as `det`, and so is an `adj` entry
      whose every sense is a form of such a word (`mes`, `ma`, « Forma del masculino plural de
      mon »): without it `mes` lost its gloss, a form borrowing its lemma's gloss in its own part of
      speech only. 7 / 7 rows, their runs alone. The edition heads `son` and `leur` « pronombre
      posesivo »: kept as written.

    An entry it does not change, and a line it cannot read, are written as they are."""
    entries = _read_entries(src)
    common_words = _common_words(entries)
    determiners = _determiners(entries)

    def read(entry):
        senses = entry.get("senses")
        if not isinstance(senses, list):
            return entry
        headword = _headword(entry)
        if entry.get("pos") == "name" and headword[:1].isupper() and headword.lower() in common_words:
            senses = [sense for sense in senses if not _NAME_NOTE.match(_first_gloss(sense))]
            if not senses:
                return None
            entry = {**entry, "senses": senses}
        if entry.get("pos") == "adj" and (not _DETERMINER_TAGS.isdisjoint(_tags(entry)) or _forms_of(entry, determiners)):
            entry = {**entry, "pos": "det"}
        return entry

    return spanish.rewrite_entries(src, dst, read)


# — The studied word is no definition (D7, M5) —


def _bare(sense):
    """A sense as written, up to case and its closing period."""
    return sense.strip().rstrip(".").strip().lower()


def no_self_definition(glosses, runs, direct):
    """The lemmas whose definition only repeats them, taken out of `glosses` and `runs` so that the
    direct table glosses them (D7): a gloss whose every sense is the French headword itself, as
    written up to case (`et` « Et », `élite` « Élite »), when the French Wiktionary lists Spanish
    translations of the word (`direct`) and none of them is spelled as the headword — the Spanish
    Wiktionary wrote the studied word, not a Spanish one. A cognate the table also translates as
    itself keeps its definition (« venir » « Venir », « entre » « Entre »). Measured: 9 / 4 rows —
    `et` « Y, e », `élite` « Elite », `troll` « Trol », `slip`, `clochard`, `gourmet`, `azimut`,
    `yucca`, `octogonal`. Answers the lemmas taken out, sorted."""
    yielded = []
    for lemma, gloss in sorted(glosses.items()):
        if not all(_bare(sense) == lemma.lower() for sense in gloss.split("; ")):
            continue
        listed = direct.get(lemma)
        if not listed or any(native.lower() == lemma.lower() for natives in listed.values() for native in natives):
            continue
        del glosses[lemma]
        runs.pop(lemma, None)
        yielded.append(lemma)
    return yielded


by_spanish_frequency = common.by_native_frequency


def native_side(entries, ranks, sources):
    """The native side step by step, so that the lemmas each source glosses are kept (D2, D9): the
    Spanish Wiktionary's entries gloss first, a definition that only repeats its headword giving way
    (`no_self_definition`, against the first source, the direct table); then each `(source, order)`
    of `sources` in turn — the direct table, then the inverted one — glosses what the steps before
    left out; the expressions the same way, the locutions winning. `(glosses, runs, expressions,
    steps)`, the steps `{"entries": lemmas, "direct": lemmas, "inverted": lemmas, "yielded":
    lemmas}`, each the set a step glossed (`yielded` the definitions D7 set aside). With no such
    definition, the tables `reduce_common.native_tables` gives; `reduce_common.py` is not edited."""
    runs = {}
    glosses = common.reduce_gloss(entries, set(ranks), **common.WORD_GLOSS, runs=runs, studied=FR, edition=EDITION)
    expressions = common.reduce_expressions(entries, common.EXPRESSION_GLOSS_LEN, studied=FR, edition=EDITION)
    yielded = no_self_definition(glosses, runs, sources[0][0]) if sources else []
    steps = {"entries": frozenset(glosses), "yielded": frozenset(yielded)}
    for name, source in zip(("direct", "inverted"), sources):
        found = common.fallback_glosses(ranks, glosses, [source], edition=EDITION)
        for lemma, (gloss, gloss_runs) in found.items():
            glosses[lemma], runs[lemma] = gloss, gloss_runs
        steps[name] = frozenset(found)
    expressions.update(common.fallback_expressions(expressions, sources, edition=EDITION))
    expressions.update(LOCUTIONS)
    return glosses, runs, expressions, steps


def translation_share(steps, ranks, top=MEASURED_TOP):
    """Among the glossed lemmas of the `top` commonest (D9): the share whose gloss came from a
    translation table — the direct or the inverted — rather than from a definition, in percent to
    one decimal, with the lemmas each table glossed there, so that a sample can be marked; en-es's
    shape (`pack_report.py` prints it). The commonest are the first `top` by rank, as
    `gloss_coverage.py` counts them."""
    commonest = set(sorted(ranks, key=ranks.get)[:top])
    direct = sorted(steps["direct"] & commonest)
    inverted = sorted(steps["inverted"] & commonest)
    glossed = len(steps["entries"] & commonest) + len(direct) + len(inverted)
    share = round(100 * (len(direct) + len(inverted)) / glossed, 1) if glossed else 0.0
    return {"top": top, "glossed": glossed, "share": share, "direct": direct, "inverted": inverted}


def definition_share(steps):
    """The share of all glossed lemmas a definition glosses, in percent to one decimal: the
    programme's « 24 % of them definitions » (D9)."""
    glossed = len(steps["entries"]) + len(steps["direct"]) + len(steps["inverted"])
    return round(100 * len(steps["entries"]) / glossed, 1) if glossed else 0.0


def analyser_version(path=_ANALYSIS_RS):
    """French's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const FRENCH_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no FRENCH_ANALYZER_VERSION in {path}")
    return m.group(1)


# The credits (add-lingua-pack-es-en D4, applied here): both sides' sources. The studied side is
# fr-en's reduction's, read as committed, credited as fr-en's notice credits it: the English
# Wiktionary's French section, wordfreq and UD French-GSD; its dictionary words are the lemmas fr-en
# glosses from the same section. The levels are estimated.
NOTICE = """Cymbra Lingua data pack — FR->ES attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), French section: CC BY-SA 4.0 + GFDL —
the forms, their lemmas and their grammar (French's tables, reduced by the fr-en pack's reduction
and read here as committed), and which lemmas fr-en glosses: the dictionary words and which take a
level.

kaikki.org extract of the Spanish Wiktionary (eswiktionary): CC BY-SA 4.0 + GFDL — the Spanish
glosses of French words and expressions, from the definitions of its French section, and the
French translations its Spanish entries list, read backwards where nothing else glosses a word.

kaikki.org extract of the French Wiktionary (frwiktionary): CC BY-SA 4.0 + GFDL — the Spanish
translations its French entries list, where the Spanish Wiktionary has no definition.

wordfreq (French and Spanish frequency lists), by Robyn Speer (https://github.com/rspeer/wordfreq):
data under CC BY-SA 4.0 — the commonest French lemmas and which forms are attested (French's
tables, read here as committed), and the order of the Spanish words a translation table lists.

UD French-GSD, the Universal Dependencies French-GSD treebank
(https://github.com/UniversalDependencies/UD_French-GSD): CC BY-SA 4.0 — how often a form stands
for each of its lemmas, to choose one, and how often a hyphenated word occurs, to rank it (French's
tables, read here as committed).

The levels are estimated, not taken from a CEFR list: the fr-en pack's reduction derives them from
the frequency ranks and the English Wiktionary's French section, and every pack studying French
reads them as committed.
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--studied", default=_STUDIED, help="the committed studied folder (tables/fr)")
    ap.add_argument(
        "--max-lemmas",
        type=int,
        default=60000,
        help="the committed lemmas, capped by rank: 10000 reduces the native side over the top 10,000 (a sample)",
    )
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    # wordfreq orders the inverted table's Spanish words, the commonest first (as en-es's).
    from wordfreq import zipf_frequency

    frequency = functools.lru_cache(maxsize=None)(lambda w: zipf_frequency(w, EDITION.code))

    ranks = read_studied(a.studied, a.max_lemmas)
    readings = read_readings(a.studied)
    entries = straight_apostrophes(
        os.path.join(a.work, "kaikki-es-Frances.jsonl"), os.path.join(a.work, "kaikki-es-Frances-apostrophes.jsonl")
    )
    entries = common.without_letter_senses(entries, os.path.join(a.work, "kaikki-es-Frances-words.jsonl"), edition=EDITION)
    entries = spanish.read_as_meanings(entries, os.path.join(a.work, "kaikki-es-Frances-meanings.jsonl"))
    entries = french_entries(entries, os.path.join(a.work, "kaikki-es-Frances-glossing.jsonl"))
    direct = read_translated(
        os.path.join(a.work, "kaikki-fr-traductions.jsonl"),
        os.path.join(a.work, "kaikki-fr-traductions-words.jsonl"),
        inverted=False,
        readings=readings,
    )
    inverted = read_translated(
        os.path.join(a.work, "kaikki-es-traductions.jsonl"),
        os.path.join(a.work, "kaikki-es-traductions-words.jsonl"),
        inverted=True,
        readings=readings,
    )
    sources = [(direct, list), (inverted, by_spanish_frequency(frequency))]
    glosses, runs, expressions, steps = native_side(entries, ranks, sources)
    # The runs' parts of speech are the Spanish Wiktionary's; French's readings come from the
    # tables the builder reads in tables/fr, not from here.
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
            "studied": "fr",
            "native": "es",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
            # French's levels are fr-en's estimate, read as committed: the extension says so.
            "levels_estimated": True,
            "licences": [
                "kaikki / enwiktionary, eswiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)",
                "wordfreq (CC BY-SA 4.0)",
                "UD French-GSD (CC BY-SA 4.0)",
            ],
        },
        "sources": [
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "UD French-GSD", "licence": "CcBySa"},
        ],
    }
    common.write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    measures = translation_share(steps, ranks)
    common.write(a.work, "measures.json", json.dumps(measures, indent=2, ensure_ascii=False) + "\n")
    print(
        f"reduced fr-es: lemmas={len(ranks)} (French's committed tables) glosses={len(glosses)} "
        f"(Spanish Wiktionary {len(steps['entries'])}, French Wiktionary's Spanish translations "
        f"{len(steps['direct'])}, Spanish Wiktionary's French translations read backwards "
        f"{len(steps['inverted'])}; "
        f"{definition_share(steps)} % from a definition; {len(steps['yielded'])} definitions repeating "
        f"their headword set aside) expressions={len(expressions)}; of the {measures['glossed']} glossed "
        f"lemmas among the {measures['top']:,} commonest, {measures['share']} % come from a translation "
        f"table (direct {len(measures['direct'])}, inverted {len(measures['inverted'])})",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
