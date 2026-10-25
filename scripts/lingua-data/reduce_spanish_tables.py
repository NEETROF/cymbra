# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The Spanish Wiktionary's translation-table rules, as fr-es wrote them, for any pair glossed in Spanish
(add-lingua-pack-it-es D4).

A copy, not a move: fr-es's rules live in `reduce-fr-es.py` — letters, a correction keyed by the
source's text, the other senses a list names, the inverted table's names, acronyms and language code
(`read_backwards`), the studied word given as Spanish, the edition's typography, and each Spanish word
listed once —, and moving them here would move fr-es's rule digest, so its `pack_version`, manifest,
pin, golden and shipped pack. fr-es and en-es adopt this module at their next re-pin, which moves their
digests anyway (as fr-es copied en-es's `listed_once` rather than move en-es). Every global of fr-es's
the rules read is a field of `Rules` here: the studied language (`FR`) and the translation template's
code kaikki reads as one of its words (`_LANGUAGE_CODE`), the corrections (`CORRECTIONS`), the other
senses (`OTHER_SENSES`), the words Spanish writes alike (`SPANISH_ALIKE`), the gap in Zipf points
(`FRENCH_GAP`) and the language of the studied word's frequency (`zipf(word, "fr")`, the studied
language's code); and how a studied word is written (fr-es's typographic apostrophe read as `'`).
test_reduce_it_es.py holds the copy, given fr-es's globals, to what fr-es's functions give on fr-es's
own cases.

Imported by `reduce-it-es.py` alone (and its tests): a rule file of it-es's (`pack_sources.rule_files`),
so editing it re-pins it-es and no other pair.
"""

import collections
import json
import re
from dataclasses import dataclass, field
from typing import Callable

import reduce_common as common
import reduce_edition_es as spanish

# A correction of a source's slip, keyed by the source's text as the pinned files write it (fr-es's
# refine-lingua-fr-es-glosses D7): an entry's senses (`section`), or the Spanish words a studied entry
# of the direct table lists (`direct`), in their order. `corrected` None leaves the word with no gloss
# from that source. The reason and the page reported are part of it.
Correction = collections.namedtuple("Correction", "source headword pos text corrected reason report")


def _as_written(word):
    """fr-es's reading of a studied word: its typographic apostrophe read as `'`."""
    return word.replace("’", "'") if isinstance(word, str) else word


@dataclass(frozen=True, eq=False)
class Rules:
    """What fr-es's table rules read from fr-es's globals, for one pair glossed in Spanish."""

    studied: common.Studied  # the studied language (fr-es's `FR`)
    language_code: str  # the translation template's code, which kaikki reads as a studied word (`_LANGUAGE_CODE`)
    corrections: tuple = ()  # `CORRECTIONS`
    other_senses: dict = field(default_factory=dict)  # `OTHER_SENSES`: (studied word, Spanish word) → why
    alike: frozenset = frozenset()  # the studied words Spanish writes alike (`SPANISH_ALIKE`)
    gap: float = 2.0  # `FRENCH_GAP`: Zipf points commoner in the studied language than in Spanish
    straight: Callable = _as_written  # how a studied word is written, as the studied tables key it


# — The sources' slips, corrected by name —


def first_gloss(sense):
    glosses = sense.get("glosses") if isinstance(sense, dict) else None
    return glosses[0].strip() if isinstance(glosses, list) and glosses and isinstance(glosses[0], str) else ""


def _texts_of(entry, source):
    """What a correction keys an entry by: its senses' texts (`section`) or the Spanish words it
    lists (`direct`), in their order; None when the entry is not of that shape."""
    items = entry.get("senses") if source == "section" else entry.get("translations")
    if not isinstance(items, list) or not all(isinstance(item, dict) for item in items):
        return None
    if source == "section":
        return tuple(first_gloss(item) for item in items)
    return tuple(item.get("word") for item in items)


def _with_text(item, text, source):
    """A sense or a translation reading `text`, its other fields kept (`item` None: a new one)."""
    if source == "section":
        if item is None:
            return {"glosses": [text]}
        return {**item, "glosses": [text, *(item.get("glosses") or [])[1:]]}
    return {**(item or {}), "word": text}


def corrected(entry, source, rules, fired=None):
    """`entry` (of the section, or of the direct table) as `rules.corrections` correct it, or None when
    a correction leaves the word with no gloss from that source; each correction that fires is added
    to `fired`. A correction fires on an entry of its headword and part of speech whose texts are
    exactly the source's, as the pinned files write them. Each corrected text the source writes too
    keeps that sense or translation as written; a text the source does not write takes the place of
    the one it rewords — the one at its position whose text the correction does not keep — with its
    other fields, or is a new one."""
    headword = entry.get("word").strip() if isinstance(entry.get("word"), str) else None
    for correction in rules.corrections:
        if correction.source != source or headword != correction.headword or entry.get("pos") != correction.pos:
            continue
        if _texts_of(entry, source) != correction.text:
            continue
        if fired is not None:
            fired.add(correction)
        if correction.corrected is None:
            return None
        key = "senses" if source == "section" else "translations"
        items = entry[key]
        reworded = {k for k, text in enumerate(correction.text) if text not in correction.corrected}
        written = []
        for k, text in enumerate(correction.corrected):
            if text in correction.text:
                written.append(items[correction.text.index(text)])
            else:
                written.append(_with_text(items[k] if k in reworded else None, text, source))
        return {**entry, key: written}
    return entry


def corrected_section(src, dst, rules, fired=None):
    """The section's entries as `rules.corrections` correct them, written to `dst`. An entry a
    correction leaves with no gloss goes; each correction that fires is added to `fired`."""
    return spanish.rewrite_entries(src, dst, lambda entry: corrected(entry, "section", rules, fired))


def unfired(fired, rules):
    """The corrections that fired on nothing: each, as a summary line names it — the page was
    corrected, or the entry is gone, and the correction is to be removed."""
    return [f"{c.headword} ({c.source}, {c.pos})" for c in rules.corrections if c not in fired]


# — Letters —


def names_a_letter(entry):
    """A letter's entry: a `character` one, a one-letter word's noun (the letter under its name, or a
    name borrowed through it), or a one-letter word every translation of which is itself."""
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


# — Names, acronyms and other senses, read backwards —

# A Spanish saint's name: it glosses no one-word name.
_SAINT = re.compile(r"^(?:San|Santa|Santo)\b")
# The words of a name with more than one: a space or a hyphen.
_NAME_PARTS = re.compile(r"[\s-]")


def _letters(text):
    """`text`'s letters and digits, without dots, spaces or case: an acronym's spelling."""
    return "".join(c for c in text.casefold() if c.isalnum())


def _acronym_of_letters(word):
    """Whether `word` is written as an acronym: two letters or more, all capitals."""
    letters = [c for c in word if c.isalpha()]
    return len(letters) >= 2 and all(c.isupper() for c in letters)


def read_backwards(studied_word, native, rules):
    """Whether the inverted table's studied word, as the Spanish entry writes it, may gloss the lemma
    spelled like it through that entry's Spanish word (`native`): not the language code; an acronym
    only through a Spanish word with the same letters; not a word opening on a lower-case letter
    followed by a capital (« lOrient »); not a one-word name through a saint's name; never a pair
    `rules.other_senses` names."""
    studied_word, native = studied_word.strip(), native.strip()
    if studied_word == rules.language_code or (studied_word.lower(), native.lower()) in rules.other_senses:
        return False
    if _acronym_of_letters(studied_word):
        return _letters(studied_word) == _letters(native)
    if studied_word[:1].islower() and studied_word[1:2].isupper():
        return False
    return not (any(c.isupper() for c in studied_word) and not _NAME_PARTS.search(studied_word) and _SAINT.match(native))


def _with_straight_word(entry, rules):
    """`entry` with its `word` written as the studied tables key it (`rules.straight`); itself when
    that changes nothing."""
    word = entry.get("word")
    if not isinstance(word, str):
        return entry
    written = rules.straight(word)
    return {**entry, "word": written} if written != word else entry


def without_other_senses(entry, rules, stats):
    """A direct table's entry without the translations `rules.other_senses` names."""
    word, listed = entry.get("word"), entry.get("translations")
    if not isinstance(word, str) or not isinstance(listed, list):
        return entry
    kept = [
        t
        for t in listed
        if not (
            isinstance(t, dict)
            and isinstance(t.get("word"), str)
            and (word.strip().lower(), t["word"].strip().lower()) in rules.other_senses
        )
    ]
    if len(kept) == len(listed):
        return entry
    stats["translations named as other senses left out"] += len(listed) - len(kept)
    return {**entry, "translations": kept}


def translation_words(src, dst, *, inverted, rules, fired=None, stats=None):
    """A translation file as the pair reads it, written to `dst`: its letters' entries left out
    (`names_a_letter`) and each studied word written as the studied tables key it (`rules.straight`):
    the entry's word read forwards (the studied edition's studied entries), the words it lists read
    backwards (the Spanish Wiktionary's Spanish entries). Read forwards, an entry `rules.corrections`
    names is corrected (each one that fires added to `fired`) and a pair `rules.other_senses` names
    left out; read backwards, a studied word `read_backwards` refuses is left out — counted in `stats`.
    A line this pass cannot read is written as it is: the shared rules decide."""
    stats = collections.Counter() if stats is None else stats
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if isinstance(entry, dict):
                if names_a_letter(entry):
                    continue
                if not inverted:
                    read = corrected(_with_straight_word(entry, rules), "direct", rules, fired)
                    if read is None:
                        continue
                    read = without_other_senses(read, rules, stats)
                elif isinstance(entry.get("translations"), list):
                    listed = [_with_straight_word(t, rules) if isinstance(t, dict) else t for t in entry["translations"]]
                    if isinstance(entry.get("word"), str):
                        kept = [
                            t
                            for t in listed
                            if not isinstance(t, dict)
                            or not isinstance(t.get("word"), str)
                            or read_backwards(t["word"], entry["word"], rules)
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


# — The studied word is no Spanish gloss —


def is_the_studied_word(word, native, zipf, rules):
    """Whether a table's Spanish word `native`, listed for `word`, is the studied word itself: spelled
    alike up to case, not in `rules.alike`, and at least `rules.gap` Zipf points commoner in the studied
    language than in Spanish (`zipf(word, language)`)."""
    if native.strip().lower() != word.strip().lower() or word.strip().lower() in rules.alike:
        return False
    return zipf(word, rules.studied.code) - zipf(word, spanish.ES.code) >= rules.gap


def without_studied_words(table, zipf, rules):
    """A table without the Spanish words that are the studied word itself (`is_the_studied_word`): the
    next word comes up, a part of speech left with none goes, and a word left with none takes no gloss
    from the table — a later source glosses it if it can."""
    out = {}
    for word, by_pos in table.items():
        kept = {}
        for upos, natives in by_pos.items():
            words = [native for native in natives if not is_the_studied_word(word, native, zipf, rules)]
            if words:
                kept[upos] = words
        if kept:
            out[word] = kept
    return out


# — Typography, and a Spanish word listed once —


def in_typography(table):
    """A table's words in the edition's typography (`spanish.typography`), a word listed once per part
    of speech."""
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
    """A table with each Spanish word listed once per studied word: a word listed under several parts
    of speech of one studied word is listed under the first of them the studied word's readings name
    (`readings`: lemma → its parts of speech, from the studied tables' `grammar.tsv`), else under the
    first listed. A part of speech left with no word goes; each word in the edition's typography."""
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


def read_readings(path):
    """The studied language's parts of speech by lemma, from a committed `grammar.tsv` (form, lemma,
    reading): lemma → {UPOS}; an empty mapping when there is no such file."""
    readings = {}
    try:
        f = open(path, encoding="utf-8")
    except FileNotFoundError:
        return readings
    with f:
        for line in f:
            columns = line.rstrip("\n").split("\t")
            if len(columns) >= 3:
                readings.setdefault(columns[1], set()).add(columns[2].split("|")[0])
    return readings
