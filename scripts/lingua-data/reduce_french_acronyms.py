# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""French's acronyms: a reviewed table, glossed through their expansions
(keep-lingua-french-acronyms-in-capitals D2, D4).

French's analysis looks every token up lowercased, so « CA », the acronym, would read as `ça`, the
form `ca` stands for (read-lingua-french-ca-as-ca-cedilla). A word standing alone in capitals — its
neighbours not in capitals — is first looked up as written (the core's `french::acronym`), and a
row of this table is what it then finds: the acronym, written wholly in capitals.

A row is a person's decision, its reason part of it (the owner asked for `CA` alone at first,
2026-10-10): acronym → (its expansions, in the order the English Wiktionary's French section's own
entry for it names them; the part of speech that entry gives; the reason). A row names only
expansions that entry points at — the table chooses among the dictionary's meanings and never writes
one of its own —, and is needed only where the acronym's lower case is another word's reading.

What a row does:
- fr-en's reducer, French's reference, writes it into French's studied tables: `forms.tsv` gains the
  acronym as its own lemma, capitals kept — the one form holding a capital —, and `lexical.tsv` gains
  it, a dictionary word (the names rule then keeps it); it takes no rank, no level and no reading
  (`freq.tsv`, `level.tsv` and `grammar.tsv` gain nothing): wordfreq lowercases, and its frequency
  for the lowercase spelling is another word's. It checks that the section's entry headed exactly as
  the row points at every expansion the row names (`check_rows`), and refuses the reduction
  otherwise.
- Each pair glossing French glosses it from its own glosses of the expansions (`acronym_glosses`):
  for each expansion, in the row's order, the pair's gloss of it — its expression table's for a
  headword holding a space, its word gloss otherwise — cut to its first sense, an expansion it does
  not gloss passed over, the senses joined by « ; », one run of the row's part of speech; no gloss
  when it glosses none. Every sense is a gloss people wrote, in the reader's language, for what the
  acronym stands for: nothing is translated, and the French expansion is never shown.

A rule module of its own, imported by `reduce-fr-en.py` and `reduce-fr-es.py` alone, as
`reduce_french_treebank.py` is: a rule file of each (`pack_sources.rule_files`), so editing it
re-pins fr-en and fr-es, and no other pair. It imports nothing of theirs.
"""

import unicodedata

ACRONYMS = {
    "CA": (
        ("conseil d'administration", "chiffre d'affaires", "courant alternatif", "comptable agréé"),
        "NOUN",
        "CA written in capitals is the acronym (the owner's decision of 2026-10-10, option A of "
        "read-lingua-french-ca-as-ca-cedilla D2): its expansions as the English Wiktionary's French "
        "CA entry names them; « ca » and « Ca » read as ça (UNMARKED_SPELLINGS)",
    ),
}

# The separator between a gloss's senses, which the shared rules guarantee no sense holds.
SENSE_SEPARATOR = "; "


def is_acronym(word):
    """Whether `word` is written as a row must be: wholly in capitals — no lowercase letter —, two
    letters or more."""
    letters = [c for c in word if c.isalpha()]
    return len(letters) >= 2 and not any(c.islower() for c in letters)


def _expansion(text):
    """An expansion as a pointer writes it: in NFC, the typographic apostrophe read as `'`."""
    return unicodedata.normalize("NFC", (text or "").strip()).replace("’", "'")


def pointed_at(entry):
    """The words an entry's senses point at — each `alt_of` and `form_of` target, whole — as the
    section writes « initialism of conseil d'administration »."""
    out = set()
    for sense in entry.get("senses") or ():
        for field in ("alt_of", "form_of"):
            for ref in sense.get(field) or ():
                word = ref.get("word") if isinstance(ref, dict) else None
                if word:
                    out.add(_expansion(word))
    return out


class Pointers:
    """What the section's entries headed exactly as a row point at: acronym → expansions. Fed entry
    by entry (`read`), as the reducer reads the section once."""

    def __init__(self, rows=None):
        self.rows = ACRONYMS if rows is None else rows
        self.found = {}

    def read(self, entry):
        word = entry.get("word")
        if word in self.rows:
            self.found.setdefault(word, set()).update(pointed_at(entry))


def check_rows(pointers, rows=None):
    """Refuse a row the section does not back (D2): its entry missing, or an expansion that entry
    does not point at. A row is a person's choice among the dictionary's meanings; a source that
    stops naming one asks the person again rather than keep a meaning of the table's own."""
    rows = ACRONYMS if rows is None else rows
    for acronym, (expansions, _, _) in sorted(rows.items()):
        if acronym not in pointers.found:
            raise SystemExit(
                f"error: the acronym table's row {acronym} has no entry headed {acronym} in the French section"
            )
        missing = [e for e in expansions if e not in pointers.found[acronym]]
        if missing:
            raise SystemExit(
                f"error: the acronym table's row {acronym} names {', '.join(missing)}, which the section's "
                f"entry {acronym} does not point at"
            )


def acronym_forms(rows=None):
    """The rows as French's forms write them (D2): each acronym its own lemma."""
    rows = ACRONYMS if rows is None else rows
    return {acronym: acronym for acronym in rows}


def gloss_of(expansions, words, expressions):
    """A row's gloss from a pair's own glosses (D4): `(gloss, senses)`, or None when the pair glosses
    none of `expansions`. `words` are its word glosses (lemma → gloss), `expressions` its expression
    table (headword → gloss)."""
    senses = []
    for expansion in expansions:
        gloss = (expressions if " " in expansion else words).get(expansion)
        if gloss:
            senses.append(gloss.split(SENSE_SEPARATOR)[0])
    if not senses:
        return None
    return SENSE_SEPARATOR.join(senses), len(senses)


def acronym_glosses(words, expressions, rows=None):
    """Each row a pair glosses (D4): acronym → (gloss, runs), the runs one run of the row's part of
    speech over every sense, as `senses.tsv` writes them."""
    rows = ACRONYMS if rows is None else rows
    out = {}
    for acronym, (expansions, pos, _) in sorted(rows.items()):
        glossed = gloss_of(expansions, words, expressions)
        if glossed:
            gloss, count = glossed
            out[acronym] = (gloss, [(pos, count)])
    return out
