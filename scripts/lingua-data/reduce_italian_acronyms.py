# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Italian's acronyms: a reviewed table, glossed through their entry's expansions or synonyms
(keep-lingua-italian-acronyms-and-names D2, D3).

Italian's analysis looks every token up lowercased, so « NATO » would read as `nato`, *nascere*'s
participle, and « USA » as `usa`, a form of *uso*. A word standing alone in capitals — its neighbours
not in capitals — is first looked up as written (the core's `french::acronym`, which Italian shares
since this change), and a row of this table is what it then finds: the acronym, written wholly in
capitals.

A row is a person's decision, its reason part of it: acronym → (its expansions, in the order the
English Wiktionary's Italian section's own entry for it points at them; the synonyms that entry
names; the part of speech it gives; the reason). A row names only expansions and synonyms that entry
gives — the table chooses among the dictionary's meanings and never writes one of its own —, and is
needed only where the acronym's lower case is another word's form in the tables (`check_lowercase`):
`ONU` and `RAI` read as their own entries with no row. The entry's English (`USA`'s « the United
States of America ») is no expansion of a row: no pair glosses it, and it is never shown.

What a row does:
- it-fr's reducer, Italian's reference, writes it into Italian's studied tables: `forms.tsv` gains the
  acronym as its own lemma, capitals kept, and `lexical.tsv` gains it, a dictionary word whatever its
  sense runs and whether glossed (D4, D5); it takes no rank and no level (`freq.tsv` gains nothing):
  wordfreq lowercases, and its frequency for the lowercase spelling is another word's. The reducer
  checks that the section's entry headed exactly as the row gives every expansion and synonym the row
  names (`check_rows`), and refuses the reduction otherwise; a row naming none is accepted when its
  entry exists (`NATO`).
- Each pair glossing Italian glosses it from its own glosses (`acronym_gloss`, D3): of the row's
  expansions first, each cut to its first sense, in the row's order, an expansion it does not gloss
  passed over; when it glosses none, of the row's synonyms the same way; no gloss when it glosses
  neither (`NATO` in it-fr). The senses are joined by « ; », one run of the row's part of speech.
  Change 16, which writes it-fr's glosses, calls it.

A rule module of its own, imported by `reduce-it-fr.py` and every pair glossing Italian, as
`reduce_italian_treebank.py` is: a rule file of each (`pack_sources.rule_files`), so editing it
re-pins them, and no other pair. It imports nothing of theirs, nor French's table.
"""

import unicodedata

ACRONYMS = {
    "NATO": (
        (),
        (),
        "PROPN",
        "NATO written in capitals is the alliance (keep-lingua-italian-acronyms-and-names D2): its "
        "lowercase nato is nascere's past participle in the tables; the section's NATO entry names no "
        "expansion and no synonym. Measured on the Italian corpora (task 5.1): 27 alone in capitals, "
        "every one the alliance, none a shouted word",
    ),
    "USA": (
        (),
        ("America", "Stati Uniti"),
        "PROPN",
        "USA written in capitals is the country (keep-lingua-italian-acronyms-and-names D2): its "
        "lowercase usa reads as uso in the tables (and is usare's third person); the section's USA "
        "entry points at the English name and names the synonyms America and Stati Uniti. Measured on "
        "the Italian corpora (task 5.1): 29 alone in capitals, every one the country, none a shouted "
        "word",
    ),
}

# The separator between a gloss's senses, which the shared rules guarantee no sense holds.
SENSE_SEPARATOR = "; "


def is_acronym(word):
    """Whether `word` is written as a row must be: wholly in capitals — no lowercase letter —, two
    letters or more."""
    letters = [c for c in word if c.isalpha()]
    return len(letters) >= 2 and not any(c.islower() for c in letters)


def _written(text):
    """A word as an entry writes it: in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip()).replace("’", "'")


def _words(refs):
    """The words of a list of the dump's references (`[{"word": …}]`)."""
    return {_written(ref.get("word")) for ref in refs or () if isinstance(ref, dict) and ref.get("word")}


def given(entry):
    """What an entry gives a row: `(expansions, synonyms)` — the words its senses point at (each
    `alt_of` and `form_of` target, whole: « initialism of prodotto interno lordo »), and the synonyms
    it names, the entry's own and its senses'."""
    expansions, synonyms = set(), _words(entry.get("synonyms"))
    for sense in entry.get("senses") or ():
        expansions |= _words(sense.get("alt_of")) | _words(sense.get("form_of"))
        synonyms |= _words(sense.get("synonyms"))
    return expansions, synonyms


class Pointers:
    """What the section's entries headed exactly as a row give: acronym → (expansions, synonyms). Fed
    entry by entry (`read`), as the reducer reads the section once."""

    def __init__(self, rows=None):
        self.rows = ACRONYMS if rows is None else rows
        self.found = {}

    def read(self, entry):
        word = _written(entry.get("word"))
        if word in self.rows:
            expansions, synonyms = self.found.setdefault(word, (set(), set()))
            more_expansions, more_synonyms = given(entry)
            expansions |= more_expansions
            synonyms |= more_synonyms


def check_rows(pointers, rows=None):
    """Refuse a row the section does not back (D2): its entry missing, or an expansion or a synonym
    that entry does not give. A row naming neither is accepted when its entry exists (`NATO`). A row is
    a person's choice among the dictionary's meanings; a source that stops giving one asks the person
    again rather than keep a meaning of the table's own."""
    rows = ACRONYMS if rows is None else rows
    for acronym, (expansions, synonyms, _, _) in sorted(rows.items()):
        if acronym not in pointers.found:
            raise SystemExit(
                f"error: the Italian acronym table's row {acronym} has no entry headed {acronym} in the "
                "Italian section: a row chooses among the dictionary's entries, and never writes one"
            )
        pointed, named = pointers.found[acronym]
        missing = [e for e in expansions if e not in pointed] + [s for s in synonyms if s not in named]
        if missing:
            raise SystemExit(
                f"error: the Italian acronym table's row {acronym} names {', '.join(missing)}, which the "
                f"section's entry {acronym} does not give"
            )


def check_lowercase(forms, rows=None):
    """Refuse a row whose lower case is no form of the tables read as another word (D2): the case a
    row repairs (`nato` → *nascere*). Read on the tables before the rows join them; a lower case read
    as its own lemma (`onu`, `rai`) needs no row, and one that is no form reads as nothing a row
    chooses against."""
    rows = ACRONYMS if rows is None else rows
    alone = [a for a in sorted(rows) if forms.get(a.lower()) in (None, a.lower())]
    if alone:
        raise SystemExit(
            f"error: the Italian acronym table's row(s) {', '.join(alone)}: the lower case is no form of "
            "the tables read as another word, which a row is only for"
        )


def acronym_forms(rows=None):
    """The rows as Italian's forms write them (D2): each acronym its own lemma."""
    rows = ACRONYMS if rows is None else rows
    return {acronym: acronym for acronym in rows}


def _first_senses(words, glosses, expressions):
    """The pair's gloss of each of `words`, in order, cut to its first sense, a word it does not gloss
    passed over. A word is looked up as the pair keys it: lowercased, in NFC; a headword holding a
    space in `expressions` when given, in `glosses` otherwise."""
    senses = []
    for word in words:
        key = _written(word).lower()
        table = expressions if expressions is not None and " " in key else glosses
        gloss = table.get(key)
        if gloss:
            senses.append(gloss.split(SENSE_SEPARATOR)[0])
    return senses


def acronym_gloss(row, glosses, expressions=None):
    """A row's gloss from a pair's own glosses (D3): `(gloss, runs)`, or None when the pair glosses
    none of the row's expansions and none of its synonyms. `row` is a value of `ACRONYMS`; `glosses`
    the pair's word glosses (lemma → gloss), `expressions` its expression table (headword → gloss),
    if any. The expansions are read first; the synonyms only when no expansion is glossed. The runs
    are one run of the row's part of speech over every sense, as `senses.tsv` writes them. Never a
    gloss of the pair's own words, a translation, or the entry's English."""
    expansions, synonyms, pos, _ = row
    senses = _first_senses(expansions, glosses, expressions) or _first_senses(synonyms, glosses, expressions)
    if not senses:
        return None
    return SENSE_SEPARATOR.join(senses), [(pos, len(senses))]
