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
- `fr_gsd-ud-train.conllu` and `fr_gsd-ud-dev.conllu`, in `--work`: UD French-GSD's training and
  development sections, the files fr-en's pin records (`pack_sources.PINNED["fr-es"]`), read for
  the part of speech a function word's row opens on (refine-lingua-fr-es-glosses D8).
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

fr-es's own rules (refine-lingua-fr-es-glosses), each in this file and measured on the tables
reduced again from `lingua-pack-sources-fr-es-2026.10.10` (rows / of the top 10,000; together 912 /
371 rows, 28 / 17 lemmas left unglossed and none gained, 66 expressions changed and 3 left
unglossed; coverage 83.2 / 70.8 / 56.8 → 83.0 / 70.7 / 56.7 %):
- The sources' slips, corrected by name (D7, `CORRECTIONS`): `rien`, `amie`, `hall`, « il y a »,
  and `el` left with no gloss — 4 / 4 rows and 1 expression.
- A function word's row opens on the part of speech UD French-GSD reads it as (D8,
  `treebank_order`, fr-en's rule from `reduce_french_treebank.py`): 9 / 9 rows (`pas` « No; Paso »).
- An infinitive's noun after its verb, a contraction of a preposition read as a preposition, and
  the part of speech named again after the meaning taken out (D6, `french_entries`): `être`,
  `devoir`; `des`, `du`; `qui`, `quoi` — 10 / 9 rows.
- A sense's register, age and place, in the edition's words (D2, `with_labels`,
  `label_expressions`): 202 / 109 rows, and 59 expressions labelled after the shared cut, their
  meaning unshortened.
- The inverted table's names, acronyms and language code, and the named other-sense words of both
  tables (D4, `translation_words`, `OTHER_SENSES`): 20 / 14 rows, 19 / 14 lemmas left unglossed.
- The French word given as its own Spanish gloss left out, the loanwords Spanish writes alike kept
  (D5, `without_french_words`, `SPANISH_ALIKE`): 12 / 4 rows, 8 / 2 lemmas left unglossed.
- A Spanish word of the direct table listed once across the French word's parts of speech (D3,
  `listed_once_direct`): 654 / 222 rows (« parti » « Partido »).
- « etc » written « etc. » (D6, `with_etc_period`): 5 / 4 rows.

Every rule here names a source or is Spanish's: the native side every pair shares
(`reduce_common`, written out step by step in `native_side` so that the lemmas each source glosses
are kept) with the Spanish edition's rules, and fr-es's own. No other pair's reducer is loaded — a
reducer loads code by import statements alone, and nothing of fr-en, en-es or es-fr may move with
this pair — so the pair's rule digest is this file, `reduce_common.py`, `reduce_edition_es.py` and
`reduce_french_treebank.py` (UD French-GSD's parts of speech, a rule module of its own that fr-en
imports too, refine-lingua-fr-es-glosses D8): an edit of the Spanish edition re-pins en-es and fr-es,
one of the treebank's module fr-en and fr-es. A change to fr-en's rules reaches this pair through
the committed tables: when a studied table moves, fr-es's pack moves and its checks name the table
(`pack_sources.py check-reducer`, add-lingua-pack-es-en D3).
"""

import argparse
import collections
import functools
import json
import os
import re
import sys

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402 — the rules every pair shares
import reduce_edition_es as spanish  # noqa: E402 — the Spanish Wiktionary's rules: fr-es's glosses are Spanish
import reduce_french_treebank as treebank  # noqa: E402 — UD French-GSD's parts of speech (refine-lingua-fr-es-glosses D8)

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


# — The sources' slips, corrected by name (refine-lingua-fr-es-glosses D7) —
#
# A few rows the sources write wrong that no measured rule tells from a right one (the 111 / 77 rows
# holding a pointer beside a meaning are nearly all verb forms beside a noun, where the meaning is
# right). Each correction is keyed by the source's text as the pinned files write it — an entry's
# senses (`section`), or the Spanish words a French entry of the direct table lists (`direct`), in
# their order — so that it fires on nothing once the page is corrected: the update that takes the
# correction in says so on its summary line, and it is removed. A corrected text takes its words
# from a source where one has them (the French Wiktionary's « amiga », « hay ») or from the slip
# itself (« Pequeña »), and is written by a person otherwise (`hall`); `corrected` None leaves the
# word with no gloss from that source. The owner reads each, and each is reported to the Wiktionary
# that wrote it (`report`). 4 / 4 rows and 1 expression.

Correction = collections.namedtuple("Correction", "source headword pos text corrected reason report")

CORRECTIONS = (
    Correction(
        "section",
        "rien",
        "noun",
        ("Pequeño cantidad de algo.",),
        ("Pequeña cantidad de algo.",),
        "an agreement slip: « cantidad » is feminine",
        "https://es.wiktionary.org/wiki/rien",
    ),
    Correction(
        "section",
        "amie",
        "noun",
        ("Amia o lamia.",),
        ("Amiga.", "Amia o lamia."),
        "the commonest meaning is missing: the friend is only « Forma del femenino singular de ami », a pointer "
        "the shared rules skip beside a meaning; « amiga » is the French Wiktionary's own Spanish for « amie »",
        "https://es.wiktionary.org/wiki/amie",
    ),
    Correction(
        "section",
        "il y a",
        "phrase",
        ("Hace.",),
        ("Hay.", "Hace."),
        "« there is » is missing; « hay » is the French Wiktionary's own Spanish for « il y a »",
        "https://es.wiktionary.org/wiki/il_y_a",
    ),
    Correction(
        "section",
        "el",
        "pron",
        ("Ella, ello o él.",),
        None,
        "no French dictionary word: met in French text as the article of a Spanish or Arabic name (« El Niño »)",
        "https://es.wiktionary.org/wiki/el",
    ),
    Correction(
        "direct",
        "el",
        "pron",
        ("elle",),
        None,
        "no French dictionary word, and the table's « elle » is French, not Spanish",
        "https://fr.wiktionary.org/wiki/el",
    ),
    Correction(
        "direct",
        "hall",
        "noun",
        ("explanada",),
        ("vestíbulo", "recibidor"),
        "an esplanade is no entrance hall; the one correction no source supplies, its two words written here",
        "https://fr.wiktionary.org/wiki/hall",
    ),
)


def _texts_of(entry, source):
    """What a correction keys an entry by: its senses' texts (`section`) or the Spanish words it
    lists (`direct`), in their order; None when the entry is not of that shape."""
    items = entry.get("senses") if source == "section" else entry.get("translations")
    if not isinstance(items, list) or not all(isinstance(item, dict) for item in items):
        return None
    if source == "section":
        return tuple(_first_gloss(item) for item in items)
    return tuple(item.get("word") for item in items)


def _with_text(item, text, source):
    """A sense or a translation reading `text`, its other fields kept (`item` None: a new one)."""
    if source == "section":
        if item is None:
            return {"glosses": [text]}
        return {**item, "glosses": [text, *(item.get("glosses") or [])[1:]]}
    return {**(item or {}), "word": text}


def _corrected(entry, source, fired=None):
    """`entry` (of the section, or of the direct table) as `CORRECTIONS` correct it (D7), or None when
    a correction leaves the word with no gloss from that source; each correction that fires is added
    to `fired`. A correction fires on an entry of its headword and part of speech whose texts are
    exactly the source's, as the pinned files write them. Each corrected text the source writes too
    keeps that sense or translation as written; a text the source does not write takes the place of
    the one it rewords — the one at its position whose text the correction does not keep — with its
    other fields, or is a new one."""
    headword = entry.get("word").strip() if isinstance(entry.get("word"), str) else None
    for correction in CORRECTIONS:
        if correction.source != source or headword != correction.headword or entry.get("pos") != correction.pos:
            continue
        if _texts_of(entry, source) != correction.text:
            continue
        if fired is not None:
            fired.add(correction)
        if correction.corrected is None:
            return None
        field = "senses" if source == "section" else "translations"
        items = entry[field]
        reworded = {k for k, text in enumerate(correction.text) if text not in correction.corrected}
        written = []
        for k, text in enumerate(correction.corrected):
            if text in correction.text:
                written.append(items[correction.text.index(text)])
            else:
                written.append(_with_text(items[k] if k in reworded else None, text, source))
        return {**entry, field: written}
    return entry


def corrected_section(src, dst, fired=None):
    """The section's entries as `CORRECTIONS` correct them (D7), written to `dst`: a pre-pass `main`
    runs after `straight_apostrophes`. An entry a correction leaves with no gloss goes; each
    correction that fires is added to `fired`."""
    return spanish.rewrite_entries(src, dst, lambda entry: _corrected(entry, "section", fired))


def unfired(fired):
    """The corrections that fired on nothing (D7): each, as the summary line names it — the page
    was corrected, or the entry is gone, and the correction is to be removed."""
    return [f"{c.headword} ({c.source}, {c.pos})" for c in CORRECTIONS if c not in fired]


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


# — Names, acronyms and other senses (refine-lingua-fr-es-glosses D4) —
#
# Read backwards, the Spanish Wiktionary's French translations gloss the French lemma spelled like
# the word they write, whatever that word is: the translation template's language code (« fr » in
# `calabaza` and `imperial`: « fr » « Imperial, calabaza »), an acronym translated by another word
# (`EEUU`'s « US »: « us » « EEUU »), a first name translated by a saint's (`San Lucas`'s « Luc »),
# an elided article read into a word (`Oriente`'s « lOrient », kaikki's « l'Orient »). Measured: 14 /
# 8 rows, 13 / 8 lemmas left unglossed — « usa » and « éu » « EEUU » among them, right but read as
# an acronym's other word —; « ONU » « ONU », « AEC » « A. e. c. », « Pâques » « Pascua », « Sao
# Tomé-et-Principe » « Santo Tomé y Príncipe » and « mer Rouge » « Mar Rojo » stay.

# The translation template's language code, which kaikki reads as a French word.
_LANGUAGE_CODE = "fr"
# A Spanish saint's name: it glosses no one-word French name.
_SAINT = re.compile(r"^(?:San|Santa|Santo)\b")
# The words of a name with more than one: a space or a hyphen.
_NAME_PARTS = re.compile(r"[\s-]")

# The other-sense words of the 10,000 commonest lemmas no measured rule tells from a translation, in
# either table: (French word, Spanish word) → why. 6 / 6 rows, each left unglossed — neither table
# has another word. An entry fires on nothing once the source changes; the owner reads the list.
OTHER_SENSES = {
    ("rap", "secuestro"): "« secuestro » lists « rap » for « rapt »",
    ("pilote", "controlador"): "a device's driver, not the pilot",
    ("merlin", "meollar"): "a rope on a ship, not the maul or the wizard",
    ("teint", "teñido"): "the participle of « teindre », not the complexion",
    ("excité", "jarioso"): "a regional word for one sense",
    ("ds", "tiburón"): "the direct table's « DS », the car's Spanish nickname, for « ds », French shorthand for « dans »",
}


def _letters(text):
    """`text`'s letters and digits, without dots, spaces or case: an acronym's spelling."""
    return "".join(c for c in text.casefold() if c.isalnum())


def _acronym_of_letters(word):
    """Whether `word` is written as an acronym: two letters or more, all capitals."""
    letters = [c for c in word if c.isalpha()]
    return len(letters) >= 2 and all(c.isupper() for c in letters)


def read_backwards(french, native):
    """Whether the inverted table's French word, as the Spanish entry writes it (`french`), may gloss
    the lemma spelled like it through that entry's Spanish word (`native`) (D4): not the language
    code « fr »; an acronym only through a Spanish word with the same letters (`_letters`); not a
    word opening on a lower-case letter followed by a capital (« lOrient »); not a one-word name
    through a saint's name; never a pair `OTHER_SENSES` names."""
    french, native = french.strip(), native.strip()
    if french == _LANGUAGE_CODE or (french.lower(), native.lower()) in OTHER_SENSES:
        return False
    if _acronym_of_letters(french):
        return _letters(french) == _letters(native)
    if french[:1].islower() and french[1:2].isupper():
        return False
    return not (any(c.isupper() for c in french) and not _NAME_PARTS.search(french) and _SAINT.match(native))


def translation_words(src, dst, *, inverted, fired=None, stats=None):
    """A translation file as fr-es reads it, written to `dst`: its letters' entries left out — a
    `character` entry, a one-letter word's noun entry (the letter under its name, or a name borrowed
    through it) and a one-letter word every translation of which is itself — and each French word's
    typographic apostrophe read as `'` (D2): the entry's word read forwards (the French Wiktionary's
    French entries), the words it lists read backwards (the Spanish Wiktionary's Spanish entries).
    Read forwards, an entry `CORRECTIONS` names is corrected (refine-lingua-fr-es-glosses D7, each
    one that fires added to `fired`) and a pair `OTHER_SENSES` names left out; read backwards, a
    French word `read_backwards` refuses is left out (D4) — counted in `stats`. A line this pass
    cannot read is written as it is: the shared rules decide."""
    stats = collections.Counter() if stats is None else stats
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
                    read = _corrected(_with_straight_word(entry), "direct", fired)
                    if read is None:
                        continue
                    read = _without_other_senses(read, stats)
                elif isinstance(entry.get("translations"), list):
                    listed = [_with_straight_word(t) if isinstance(t, dict) else t for t in entry["translations"]]
                    if isinstance(entry.get("word"), str):
                        kept = [
                            t
                            for t in listed
                            if not isinstance(t, dict) or not isinstance(t.get("word"), str) or read_backwards(t["word"], entry["word"])
                        ]
                        stats["translations read backwards left out"] += len(listed) - len(kept)
                        listed = kept
                    read = {**entry, "translations": listed}
                else:
                    read = entry
                if read != entry:
                    line = json.dumps(read, ensure_ascii=False) + "\n"
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def _without_other_senses(entry, stats):
    """A direct table's entry without the translations `OTHER_SENSES` names (D4)."""
    word, listed = entry.get("word"), entry.get("translations")
    if not isinstance(word, str) or not isinstance(listed, list):
        return entry
    kept = [
        t
        for t in listed
        if not (isinstance(t, dict) and isinstance(t.get("word"), str) and (word.strip().lower(), t["word"].strip().lower()) in OTHER_SENSES)
    ]
    if len(kept) == len(listed):
        return entry
    stats["translations named as other senses left out"] += len(listed) - len(kept)
    return {**entry, "translations": kept}


def read_translated(path, dst, *, inverted, readings, zipf=None, fired=None, stats=None):
    """What a translation file (`pack_sources.derive`) says of French words, read through
    `translation_words` (into `dst`): word → {UPOS: [Spanish word, …]} (`common.read_translations`)
    — the French Wiktionary's Spanish translations read forwards, each word in the edition's
    typography, listed once per part of speech (`in_typography`), the French word itself left out
    (`without_french_words`, `zipf(word, language)` wordfreq's Zipf frequency; none left out
    without it), then each word listed once across the French word's parts of speech
    (`listed_once_direct`); the Spanish Wiktionary's French translations read backwards, a one-letter
    French word left out whatever lists it — the entry is the Spanish word's, so the letter test sees
    nothing of the French side —, each Spanish word listed once (`listed_once`, `readings` French's
    parts of speech by lemma)."""
    words = translation_words(path, dst, inverted=inverted, fired=fired, stats=stats)
    table = common.read_translations(words, inverted=inverted, studied=FR)
    if inverted:
        for letter in [word for word in table if len(word) == 1]:
            del table[letter]
        return listed_once(table, readings)
    table = in_typography(table)
    if zipf is not None:
        table = without_french_words(table, zipf)
    return listed_once_direct(table)


# — The French word is no Spanish gloss (refine-lingua-fr-es-glosses D5) —
#
# The French Wiktionary's translators sometimes list the French word itself as Spanish (« arnaque »,
# « gâche ») or among Spanish words (« retraite » « Retraite, jubilación, retiro »). A word of the
# direct table spelled as the French headword, up to case, that wordfreq rates `FRENCH_GAP` Zipf
# points — a hundred times — commoner in French than in Spanish is left out, unless Spanish writes
# it alike (`SPANISH_ALIKE`). Measured: 12 / 4 rows (« retraite » « Jubilación, retiro, pensión »,
# « mutuel » « Mutuo »), 8 / 2 lemmas left unglossed (« arnaque », « gâche », « adage », « crêperie »),
# 5 expressions changed and 3 left unglossed (« contrôle continu »).
FRENCH_GAP = 2.0
# The loanwords Spanish writes alike, reviewed by the owner: « diaporama », « redingote »,
# « aguerrir » are in the RAE's dictionary; « raï », « poutine », « tartiflette », « andouillette »,
# « navarin », « savate », « calanque », « joual » are the names Spanish texts give a French music,
# dish, sport, coast or dialect; « vivarium » is Latin. A candidate a later update brings is left out
# until it is listed, and that update's pull request names it.
SPANISH_ALIKE = frozenset(
    {
        "diaporama",
        "redingote",
        "aguerrir",
        "raï",
        "poutine",
        "tartiflette",
        "andouillette",
        "navarin",
        "savate",
        "calanque",
        "joual",
        "vivarium",
    }
)


def is_the_french_word(word, native, zipf):
    """Whether the direct table's Spanish word `native`, listed for `word`, is the French word itself
    (D5): spelled alike up to case, not in `SPANISH_ALIKE`, and at least `FRENCH_GAP` Zipf points
    commoner in French than in Spanish (`zipf(word, language)`)."""
    if native.strip().lower() != word.strip().lower() or word.strip().lower() in SPANISH_ALIKE:
        return False
    return zipf(word, "fr") - zipf(word, EDITION.code) >= FRENCH_GAP


def without_french_words(table, zipf):
    """The direct table without the Spanish words that are the French word itself
    (`is_the_french_word`, D5): the next word comes up, a part of speech left with none goes, and a
    word left with none takes no gloss from the table — the inverted table glosses it if it can."""
    out = {}
    for word, by_pos in table.items():
        kept = {}
        for upos, natives in by_pos.items():
            words = [native for native in natives if not is_the_french_word(word, native, zipf)]
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


# — A word of the direct table listed once (refine-lingua-fr-es-glosses D3) —


def listed_once_direct(table, shown=common.FALLBACK_WORDS):
    """The direct table with each Spanish word listed once across the French word's parts of speech
    (D3), on the `shown` first words of each run — the words its gloss shows: a run whose shown words
    another run shows too goes (of two equal runs, the later: « parti » « Partido », « russe » « Ruso,
    rusa »); then a word still shown by two runs stays in the first run that shows it and leaves the
    others, whose next words come up (« clair » « Claro, luminoso, límpido; Claramente »); repeated
    until nothing moves. A run left with no word goes. Measured: 654 / 222 rows, the heading of the
    first run changed in 113 / 44 (« jeune » « [sustantivo] Joven, chaval, muchacho »), no lemma
    gained or lost."""
    out = {}
    for word, by_pos in table.items():
        lists = {upos: list(natives) for upos, natives in by_pos.items()}
        while True:
            runs = [upos for upos, natives in lists.items() if natives]
            show = {upos: set(lists[upos][:shown]) for upos in runs}
            inside = next(
                (
                    a
                    for i, a in enumerate(runs)
                    for j, b in enumerate(runs)
                    if i != j and (show[a] < show[b] or (show[a] == show[b] and i > j))
                ),
                None,
            )
            if inside is not None:
                lists[inside] = []
                continue
            seen, moved = set(), False
            for upos in runs:
                if seen & show[upos]:
                    lists[upos] = [native for native in lists[upos] if native not in seen]
                    moved = True
                seen |= show[upos]
            if not moved:
                break
        kept = {upos: natives for upos, natives in lists.items() if natives}
        if kept:
            out[word] = kept
    return out


def in_typography(table):
    """The direct table's words in the edition's typography (`spanish.typography`, « Bromas
    aparte… »), a word listed once per part of speech. Its parts of speech are the French word's own:
    a word listed under two of them is listed once by `listed_once_direct` (refine-lingua-fr-es-glosses
    D3), not here."""
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


# D6 of refine-lingua-fr-es-glosses: a contraction of a preposition (`des`, `du`, `duquel`), and a
# note naming the part of speech again after the meaning's period (« Quién. (Pronombre
# nominativo.) »).
_CONTRACTION_OF_A_PREPOSITION = re.compile(r"^Contracción de la preposición\b")
_PART_OF_SPEECH_AGAIN = re.compile(r"^(?P<meaning>.*?[^.\s])\.\s*\(Pronombre\b[^()]*\)\.?\s*$")


def _texts(glosses):
    """Whether a sense's `glosses` is what kaikki writes: a non-empty list of strings."""
    return isinstance(glosses, list) and bool(glosses) and all(isinstance(g, str) for g in glosses)


def _rewritten_in_groups(src, dst, read, order):
    """`src`'s entries, each as `read(entry)` gives it (None leaves it out), and each one-word
    headword's entries — every case of it — in the order `order(word, entries)` gives as indexes into
    them (None keeps the file's), written where the headword's first line stood, to `dst`. An entry
    `read` returns unchanged is written as its line was, and a line that is no JSON object as it is.
    `order` sees the headwords with two entries or more."""
    lines = []
    with open(src, encoding="utf-8") as f:
        for line in f:
            text = line if line.endswith("\n") else line + "\n"
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if not isinstance(entry, dict):
                lines.append((text, None))
                continue
            rewritten = read(entry)
            if rewritten is None:
                continue
            if rewritten != entry:
                text = json.dumps(rewritten, ensure_ascii=False) + "\n"
            lines.append((text, rewritten))
    headwords = collections.defaultdict(list)
    for i, (_, entry) in enumerate(lines):
        headword = _headword(entry) if entry is not None else ""
        if headword and " " not in headword:
            headwords[headword.lower()].append(i)
    moved = {}
    for word, at in headwords.items():
        if len(at) < 2:
            continue
        ordered = order(word, [lines[i][1] for i in at])
        if ordered is not None and ordered != list(range(len(at))):
            moved[at[0]] = [at[k] for k in ordered]
    held = {i for group in moved.values() for i in group}
    with open(dst, "w", encoding="utf-8") as out:
        for i, (text, _) in enumerate(lines):
            if i in moved:
                out.write("".join(lines[k][0] for k in moved[i]))
            elif i not in held:
                out.write(text)
    return dst


def _meanings(entry):
    """An entry's senses that hold a meaning, as written up to case and their closing period."""
    senses = entry.get("senses") if isinstance(entry.get("senses"), list) else ()
    return {_bare(_first_gloss(sense)) for sense in senses if _holds_a_meaning(sense)}


def _verb_first(word, entries, readings):
    """The order of a headword's entries with an infinitive's noun after its verb (D6): a `noun`
    entry before the word's last `verb` entry, every sense of which repeats a sense of those verb
    entries (`_meanings`), written after them — when French's readings (`readings`) name the word a
    verb. None when nothing moves."""
    if "VERB" not in readings.get(word, ()):
        return None
    verbs = [k for k, entry in enumerate(entries) if entry.get("pos") == "verb"]
    if not verbs:
        return None
    meant = set().union(*(_meanings(entries[k]) for k in verbs))
    after = [
        k
        for k, entry in enumerate(entries)
        if entry.get("pos") == "noun" and k < verbs[-1] and _meanings(entry) and _meanings(entry) <= meant
    ]
    if not after:
        return None
    rest = [k for k in range(len(entries)) if k not in after]
    last = rest.index(verbs[-1]) + 1
    return rest[:last] + after + rest[last:]


def french_entries(src, dst, readings=None):
    """The Spanish Wiktionary's French entries as they gloss a French word, written to `dst` — a
    pre-pass `main` runs after `spanish.read_as_meanings` (D4, D5), en-es's pair rules transposed,
    and three readings of fr-es's own (refine-lingua-fr-es-glosses D6):

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
    - An infinitive's noun after its verb: a `noun` entry every sense of which repeats, as written up
      to case and its closing period, a sense of the word's `verb` entries is written after them when
      French's readings (`readings`, `tables/fr/grammar.tsv`) name the word a verb
      (`_verb_first`): the section enters `être`'s noun « Ser » before its verb « Ser », « Estar »,
      and the round-robin kept the noun's. 2 / 2 rows, their runs alone (`être`, `devoir`); « jaune »,
      a noun beside an adjective, keeps its order.
    - A contraction of a preposition is a preposition: a `contraction` entry whose first sense opens
      « Contracción de la preposición » is read as `prep` (`des`, `du`, `duquel`, `audit`, `ès`,
      `dudit`): 6 / 5 rows, their runs alone. « c'est » is no contraction of a preposition.
    - The part of speech named again: a parenthesis after the meaning's period opening on
      « Pronombre » — what the card's heading says already — goes with the period (`qui` « Quién;
      Que », `quoi`): 2 / 2 rows. The section's other notes after a meaning say something the heading
      does not, and stay (« (Plural exclusivo.) », `il`'s « (No tiene traducción al español…) »).

    An entry it does not change, and a line it cannot read, are written as they are."""
    readings = readings or {}
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
        if entry.get("pos") == "contraction" and senses and _CONTRACTION_OF_A_PREPOSITION.match(_first_gloss(senses[0])):
            entry = {**entry, "pos": "prep"}
        cut = [_without_part_of_speech_again(sense) for sense in senses]
        if cut != senses:
            entry = {**entry, "senses": cut}
        return entry

    return _rewritten_in_groups(src, dst, read, lambda word, group: _verb_first(word, group, readings))


def _without_part_of_speech_again(sense):
    """A sense without the parenthesis after its meaning's period that names the part of speech
    again (« Quién. (Pronombre nominativo.) » → « Quién »), the period with it (D6)."""
    if not _holds_a_meaning(sense) or not _texts(sense.get("glosses")):
        return sense
    found = _PART_OF_SPEECH_AGAIN.match(sense["glosses"][0])
    return {**sense, "glosses": [found.group("meaning"), *sense["glosses"][1:]]} if found else sense


# — A function word's row by UD French-GSD (refine-lingua-fr-es-glosses D8) —
#
# The section enters `pas`'s noun « Paso » before its adverb « No », `pendant`'s adjective « Pendiente »
# before its preposition « Durante », `autour`'s noun « Halcón » (the goshawk) before its adverb
# « Alrededor ». fr-en's rule (refine-lingua-fr-en-glosses D5), its threshold and boundary alike, read
# from the module both pairs import (`reduce_french_treebank.py`). Measured: 9 / 9 rows, the first
# sense of 7 (`un`, `pas`, `pendant`, `aucun`, `autour`, `envers`, and `toutefois` « Todavía; sin
# embargo, no obstante », the one that reads worse), the runs alone of 2 (`que`, `donc`); no row
# opening on a name moves, and no lemma gains or loses a gloss.

# UD French-GSD's training and development sections, in `--work` (pack_sources.PINNED["fr-es"]).
GSD_FILES = ("fr_gsd-ud-train.conllu", "fr_gsd-ud-dev.conllu")


def _treebank_first(word, entries, counts):
    """The order of a headword's entries the treebank asks for (`treebank.commonest_first`), as
    fr-en reads it: each entry's part of speech as the shared rules read kaikki's, the page's first
    that of its first entry that is no acronym's."""
    upos = [common.kaikki_upos(entry.get("pos") or "", word, studied=FR) for entry in entries]
    first = next((u for entry, u in zip(entries, upos) if not common._acronym(_headword(entry))), upos[0])
    return treebank.commonest_first(treebank.word_key(word), upos, first, counts)


def treebank_order(src, dst, counts):
    """The section's entries with a function word's row opening on the part of speech UD French-GSD
    reads it as (D8), written to `dst` — a pre-pass `main` runs after `french_entries`: a headword's
    entries of the treebank's commonest part of speech first when the treebank (`counts`,
    `treebank.gsd_pos_counts`) reads it at least `treebank.TREEBANK_MIN` times and
    `treebank.TREEBANK_RATIO` times as often as the part of speech of the first entry, and that part
    of speech is a function word's or the first entry a proper noun's; a proper noun never first, the
    other entries in their order. An entry it does not move, and a line it cannot read, are written as
    they are."""
    return _rewritten_in_groups(src, dst, lambda entry: entry, lambda word, group: _treebank_first(word, group, counts))


def read_treebank(work):
    """UD French-GSD's counts (`treebank.gsd_pos_counts`) from the two files in `work` (D8)."""
    paths = [os.path.join(work, name) for name in GSD_FILES]
    missing = [path for path in paths if not os.path.isfile(path)]
    if missing:
        raise SystemExit(
            f"error: {', '.join(missing)} missing: fr-es reads UD French-GSD's two sections "
            "(pack_sources.py PINNED['fr-es'], fetched by build.sh)"
        )
    return treebank.gsd_pos_counts(paths)


# — A sense's register, age and place (refine-lingua-fr-es-glosses D2) —
#
# The Spanish Wiktionary's French section files each labelled sense in its own categories, in
# Spanish — « FR:Términos coloquiales », « FR:Términos anticuados », « FR:Quebec » —, and an editor's
# template that named the wrong language files « ES:Términos anticuados » (« maîtresse », « item »).
# kaikki also renders them as English tags, which miss 19 the categories hold (« Bélgica » on
# « baiser »'s « Besar » and « avec »'s « También »). A sense opens on its labels of three kinds, in
# the edition's words, register first, then age, then place, each in its table's order; a place left
# out when a place inside it is shown (« Quebec », not « América, Canadá, Quebec »). « en sentido
# figurado » and « infrecuente » say how a meaning is used, not its register, age or place: not
# shown, nor is a sense that only points at another word. No sense moves for a label: the senses
# the edition marks outdated stay after the others of their entry, as `spanish.read_as_meanings`
# writes them. Measured: 202 / 109 rows hold a labelled sense, 85 / 42 open on one; no lemma gains
# or loses a gloss.

# The prefixes of the categories the section files a French sense in.
LABEL_PREFIXES = ("FR:", "ES:")
# The register: the category « Términos … », and the word shown.
REGISTER = {
    "Términos coloquiales": "coloquial",
    "Términos malsonantes": "malsonante",
    "Términos jergales": "jergal",
    "Términos despectivos": "despectivo",
    "Términos literarios": "literario",
    "Términos vulgares": "vulgar",
    "Términos eufemísticos": "eufemístico",
    "Términos formales": "formal",
    "Términos infantiles": "infantil",
    "Términos irónicos": "irónico",
    "Términos jocosos": "jocoso",
}
# The age.
AGE = {"Términos anticuados": "anticuado", "Términos obsoletos": "obsoleto"}
# The places: each with the place it sits in, shown as the category names it.
PLACES = {
    "Quebec": "Canadá",
    "Canadá": "América",
    "América": None,
    "Provenza": "Francia",
    "Toulouse": "Francia",
    "Francia": "Europa",
    "Suiza": "Europa",
    "Bélgica": "Europa",
    "Europa": None,
    "República Democrática del Congo": "África",
    "Ruanda": "África",
    "África": None,
}


def _around(place):
    """The places `place` sits in, up to its continent."""
    out = []
    while PLACES.get(place):
        place = PLACES[place]
        out.append(place)
    return out


def labels(sense):
    """A sense's labels, in the edition's words (D2): its register, then its age, then its place, as
    its categories (`LABEL_PREFIXES`) file it, each kind in its table's order; a place left out when a
    place inside it is shown."""
    categories = sense.get("categories") if isinstance(sense, dict) else None
    filed = {
        category[len(prefix) :]
        for category in categories or ()
        if isinstance(category, str)
        for prefix in LABEL_PREFIXES
        if category.startswith(prefix)
    }
    words = [word for category, word in REGISTER.items() if category in filed]
    words += [word for category, word in AGE.items() if category in filed]
    places = [place for place in PLACES if place in filed]
    outer = {around for place in places for around in _around(place)}
    return words + [place for place in places if place not in outer]


def _labelled(text, words):
    return f"({', '.join(words)}) {text}" if words else text


def with_labels(src, dst):
    """The section's entries with each sense of a word that holds a meaning opening on its labels
    (`labels`, D2), written to `dst` — a pre-pass `main` runs after `treebank_order`: « baiser »
    « (malsonante) Coger (sexualmente) ». An expression's senses are labelled after the shared rules
    cut them (`label_expressions`). An entry it does not change, and a line it cannot read, are
    written as they are."""

    def read(entry):
        senses = entry.get("senses")
        if " " in _headword(entry) or not isinstance(senses, list):
            return entry
        shown = [
            {**sense, "glosses": [_labelled(sense["glosses"][0], labels(sense)), *sense["glosses"][1:]]}
            if _holds_a_meaning(sense) and _texts(sense.get("glosses")) and labels(sense)
            else sense
            for sense in senses
        ]
        return {**entry, "senses": shown} if shown != senses else entry

    return spanish.rewrite_entries(src, dst, read)


# What the shared rules keep of an expression (`reduce_common.reduce_expressions`' defaults): 42
# characters of each sense and three senses.
EXPRESSION_SENSE = 42
EXPRESSION_SENSES = 3


def label_expressions(expressions, src, maxlen=common.EXPRESSION_GLOSS_LEN):
    """The section's expressions (`expressions`, as `common.reduce_expressions` gives them from
    `src`, cut to `maxlen`) with each sense opening on its labels (D2), written after the shared
    rules cut them so that the meaning keeps every character it had: the shared rules' choice of
    senses rebuilt — the same round-robin over the same entries, each sense cleaned as they clean it —,
    the joined gloss walked by those senses' lengths, each sense, or the cut remnant of the last,
    opening on its source sense's labels. Measured: 59 expressions, 70 senses — « mal aux cheveux »
    « (anticuado) Resaca, caña, chaqui, chuchaqui, cruda, go », the 42 characters it had; 13 run past
    80 characters (the longest 139), none past the card's 160-character page. An expression the
    section does not gloss, or whose rebuilt gloss is not the shared rules' (a rule of theirs changed),
    is left as it is. Answers `(expressions, labelled senses)`."""
    per_entry = collections.defaultdict(list)
    with open(src, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(entry, dict):
                continue
            word = re.sub(r"\s+", " ", (entry.get("word") or "").strip().lower())
            if word not in expressions or entry.get("pos") == "name":
                continue
            senses = []
            for sense in entry.get("senses") or ():
                gloss = ((sense.get("glosses") or [""])[0] or "").strip()
                if not gloss or common._is_form_of(sense, gloss, edition=EDITION):
                    continue
                if EDITION.mwe_form_of is not None and EDITION.mwe_form_of.match(gloss):
                    continue
                text = common.clean_gloss(gloss, EXPRESSION_SENSE, edition=EDITION)
                if text:
                    senses.append((text, labels(sense)))
            if senses:
                per_entry[word].append(senses)
    out = dict(expressions)
    labelled = 0
    for word, entries in per_entry.items():
        joined = expressions[word]
        if not any(words for senses in entries for _, words in senses):
            continue
        if common._join_senses([[text for text, _ in senses] for senses in entries], maxlen, EXPRESSION_SENSES) != joined:
            continue
        picked = []
        for depth in range(max(len(senses) for senses in entries)):
            for senses in entries:
                if len(picked) < EXPRESSION_SENSES and depth < len(senses) and senses[depth][0] not in [t for t, _ in picked]:
                    picked.append(senses[depth])
        pieces, at = [], 0
        for text, words in picked:
            if at >= len(joined):
                break
            pieces.append(_labelled(joined[at : at + len(text)], words))
            labelled += bool(words)
            at += len(text) + len("; ")
        out[word] = "; ".join(pieces)
    return out, labelled


# « etc » not followed by a period (D6, fr-es's copy of refine-lingua-fr-en-glosses D8): the shared
# cleaning takes a sense's final period off, so no pre-pass can keep it. 5 / 4 rows (« avec »,
# « adresse », « regard », « cochon », « moucher »), no expression.
_ETC = re.compile(r"\betc\b(?!\.)")


def with_etc_period(text):
    """`text` with « etc » written « etc. », as the RAE writes it (D6)."""
    return _ETC.sub("etc.", text)


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
    left out; the expressions the same way, the section's labelled after the shared rules cut them
    (`label_expressions`, refine-lingua-fr-es-glosses D2), the locutions winning. `(glosses, runs,
    expressions, steps)`, the steps `{"entries": lemmas, "direct": lemmas, "inverted": lemmas,
    "yielded": lemmas, "labelled": senses}`, each the set a step glossed (`yielded` the definitions D7
    set aside; `labelled` the count of expression senses labelled). With no such definition and no
    labelled expression, the tables `reduce_common.native_tables` gives; `reduce_common.py` is not
    edited."""
    runs = {}
    glosses = common.reduce_gloss(entries, set(ranks), **common.WORD_GLOSS, runs=runs, studied=FR, edition=EDITION)
    expressions = common.reduce_expressions(entries, common.EXPRESSION_GLOSS_LEN, studied=FR, edition=EDITION)
    expressions, labelled = label_expressions(expressions, entries)
    yielded = no_self_definition(glosses, runs, sources[0][0]) if sources else []
    steps = {"entries": frozenset(glosses), "yielded": frozenset(yielded), "labelled": labelled}
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

    # wordfreq orders the inverted table's Spanish words, the commonest first (as en-es's), and tells
    # the French word given as Spanish from a loanword (refine-lingua-fr-es-glosses D5).
    from wordfreq import zipf_frequency

    zipf = functools.lru_cache(maxsize=None)(lambda w, language: zipf_frequency(w, language))

    def frequency(word):
        return zipf(word, EDITION.code)

    ranks = read_studied(a.studied, a.max_lemmas)
    readings = read_readings(a.studied)
    counts = read_treebank(a.work)
    fired, stats = set(), collections.Counter()
    entries = straight_apostrophes(
        os.path.join(a.work, "kaikki-es-Frances.jsonl"), os.path.join(a.work, "kaikki-es-Frances-apostrophes.jsonl")
    )
    entries = corrected_section(entries, os.path.join(a.work, "kaikki-es-Frances-corrected.jsonl"), fired)
    entries = common.without_letter_senses(entries, os.path.join(a.work, "kaikki-es-Frances-words.jsonl"), edition=EDITION)
    entries = spanish.read_as_meanings(entries, os.path.join(a.work, "kaikki-es-Frances-meanings.jsonl"))
    entries = french_entries(entries, os.path.join(a.work, "kaikki-es-Frances-glossing.jsonl"), readings)
    entries = treebank_order(entries, os.path.join(a.work, "kaikki-es-Frances-treebank.jsonl"), counts)
    entries = with_labels(entries, os.path.join(a.work, "kaikki-es-Frances-labels.jsonl"))
    direct = read_translated(
        os.path.join(a.work, "kaikki-fr-traductions.jsonl"),
        os.path.join(a.work, "kaikki-fr-traductions-words.jsonl"),
        inverted=False,
        readings=readings,
        zipf=zipf,
        fired=fired,
        stats=stats,
    )
    inverted = read_translated(
        os.path.join(a.work, "kaikki-es-traductions.jsonl"),
        os.path.join(a.work, "kaikki-es-traductions-words.jsonl"),
        inverted=True,
        readings=readings,
        stats=stats,
    )
    sources = [(direct, list), (inverted, by_spanish_frequency(frequency))]
    glosses, runs, expressions, steps = native_side(entries, ranks, sources)
    # « etc. » after the shared rules, which take a sense's final period off (D6).
    glosses = {lemma: with_etc_period(gloss) for lemma, gloss in glosses.items()}
    expressions = {expression: with_etc_period(gloss) for expression, gloss in expressions.items()}
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
    missed = unfired(fired)
    print(
        f"fr-es's own rules: {steps['labelled']} expression senses labelled; "
        f"{stats['translations read backwards left out']} translations read backwards left out (names, acronyms, "
        f"the language code, other senses), {stats['translations named as other senses left out']} of the direct "
        f"table named as other senses; {len(CORRECTIONS) - len(missed)} of {len(CORRECTIONS)} corrections fired"
        + (f"; corrections that found nothing, to remove: {', '.join(missed)}" if missed else ""),
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
