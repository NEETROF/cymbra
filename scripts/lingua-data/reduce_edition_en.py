# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The English Wiktionary's rules (enwiktionary), for the pairs glossed in English
(generalise-lingua-gloss-reducer).

es-en reads it (add-lingua-pack-es-en): its glosses are the English Wiktionary's senses of its
Spanish entries. Its rules come from a census of kaikki's extract of the English Wiktionary's
Spanish section, the 2026-09-28 dump es-fr pins: 811,049 entries, 875,591 senses with a gloss.

- Pointers. 82.4 % of the senses are tagged `form-of` ("plural of casa", "inflection of angular:")
  and 0.7 % `alt-of` ("superseded spelling of cuórum", "abbreviation of Puebla",
  "misspelling of cachái"): kaikki names the word an `alt-of` sense points at in `alt_of`, as it
  names a form's lemma in `form_of`. Untagged, a sense still only points at a word when it opens
  « only used in » (1,332: "only used in en pos de"), « synonym of » (1,072: "synonym of pues", a
  word of the studied language, never a gloss), « see » or « used other than figuratively or
  idiomatically: see … » (86), « disused form of » (20), or names an inflection or a variant
  followed by « of » ("diminutive of figura").
- No placeholder: kaikki leaves an undefined sense without a gloss and tags it `no-gloss`; the
  shared rules skip a sense with no gloss, and a word left with no sense has no gloss.
- No dangling coordinator: the 13 senses that open on « or » or « and » are meanings (the heraldic
  « or », "and a half").
- A letter's name: "The name of the Latin script letter D/d.", and a sense that only names a letter:
  "the letter r" (`r`), or a word's place in the Spanish spelling alphabet, "the letter E in the
  Spanish spelling alphabet" (36 of es-en's glossed lemmas ended on one: `españa`, `jueves`).
- A single capital letter is no headword of a word (`without_letter_headwords`): the Spanish
  section writes a chess piece, a compass point or a title under it (`A` « bishop », `C`
  « abbreviation of caballo », `N` « abbreviation of norte »), which would gloss the letter `a`, or
  lend `c` the gloss of `caballo`.
- Casing: 98.3 % of the meaning senses open on a lower-case letter, the edition's convention for a
  foreign word's senses, so a gloss made of translation-table words keeps the case its words have.
- Long parentheses (M20): 5,724 of 147,653 meaning senses hold one of 40 characters or more
  ("a former unit of length equivalent to about 27.9 cm"). `LONG_PARENTHESIS`, below, is the
  bound a parenthesis goes at; 0 keeps them all.
- Etymologies (add-lingua-pack-es-en D5): kaikki writes one entry per etymology, so a word with two
  nouns of different origin has two noun entries, and the round-robin across a headword's entries
  (`reduce_common._join_senses_by_pos`) takes one sense of each in turn — 532 of the top 10,000
  es-en lemmas. `MERGE_SAME_POS_ETYMOLOGIES`, below, merges a word's entries of one part of speech
  before the round-robin, so the first etymology's senses come first; off, they are read as written.

The two settings are the English edition's alone: a pair glossed in French loads this module no
more than it did, so a value chosen here re-pins es-en and no other committed pair (D5; the shared
rules in `reduce_common.py` are not edited for them).

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does — tuning it never
re-pins en-fr or es-fr.
"""

import json
import re

import reduce_common as common

# M20: a parenthesis whose text runs this long or longer goes from a gloss; 0 keeps every one.
# Decided on a sample of the top 10,000 es-en lemmas reduced both ways (add-lingua-pack-es-en D5):
# 0 as committed; the owner's pick re-pins es-en alone.
LONG_PARENTHESIS = 0
# Whether a word's entries of one part of speech — its etymologies — are merged before the
# round-robin (D5, below). Decided on the same sample: off as committed.
MERGE_SAME_POS_ETYMOLOGIES = False

# An untagged sense that only points at another word.
# Measured on the census: « see » points unless it greets (« nos vemos »: "see you later!", « ven
# acá »: "see here; come on"), and an inflection's name points unless a parenthesis defines it
# (« femenino »: "feminine (of or relating to women)", « flexión »: "inflection (a change in the form
# of a word …)").
_FORM_OF = re.compile(
    r"^(?:only used in|used other than figuratively|synonym of|see\s(?!you\b|here\b)|"
    r"(?:alternative|obsolete|archaic|dated|disused|rare|nonstandard|superseded) (?:form|spelling) of|"
    r"(?:plural|inflection|feminine|masculine|female equivalent|diminutive|augmentative|gerund|"
    r"(?:past|present) participle|infinitive|(?:first|second|third)-person)\b(?!\s*\()[^.:;]*\bof\b)",
    re.IGNORECASE,
)

# A sense naming a letter: "The name of the Latin script letter D/d.", "the letter r", "the letter E
# in the Spanish spelling alphabet" — one letter, never a word ("the letter of the law").
_LETTER = re.compile(
    r"^(?:(?:the )?name of the (?:[\w-]+ )?(?:script )?(?:letter|digraph)\b|the letter \w\b)", re.IGNORECASE
)

EN = common.Edition(
    code="en",
    form_of=_FORM_OF,
    letter=_LETTER,
    pointer_tags=frozenset({"form-of", "alt-of"}),
    pointer_fields=("form_of", "alt_of"),
    capitalised=False,
    long_parenthesis=LONG_PARENTHESIS,
)


def without_letter_headwords(src, dst):
    """The edition's entries without those whose headword is a single capital letter, written to
    `dst` — a pre-pass a reducer runs before the shared rules read the file, as it runs
    `merge_same_pos_etymologies`.

    The shared rules key a headword in lower case, and spare a word an acronym's entries only when
    the acronym has two capitals or more (`reduce_common._acronym`). The English Wiktionary's
    Spanish section writes chess pieces, compass points and titles under a capital letter — `A`
    « bishop », `C` « abbreviation of caballo », `N` « abbreviation of norte », `I` « abbreviation
    of ilustre » — so `a` opened on « bishop », and `c` borrowed the gloss of `caballo`. A letter's
    own entries (`character`) go with `reduce_common.without_letter_senses`. A line this pass
    cannot read is written as it is: the shared rules decide.
    """
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                entry = None
            if isinstance(entry, dict):
                headword = (entry.get("word") or "").strip()
                if len(headword) == 1 and headword.isupper():
                    continue
            out.write(line if line.endswith("\n") else line + "\n")
    return dst


def merge_same_pos_etymologies(src, dst, *, merged=None):
    """The edition's entries with a word's entries of one part of speech merged into one, their
    senses in source order — a pre-pass a reducer runs before the shared rules read the file
    (add-lingua-pack-es-en D5). With `merged` false (`MERGE_SAME_POS_ETYMOLOGIES` by default),
    nothing is written and `src` is answered as it is: the setting is off.

    kaikki writes one entry per etymology, each with the page's `word` and its `pos`; a page's
    entries are consecutive in the extract, so a word's group is the run of lines sharing its
    `word`. The first entry of a part of speech keeps its other fields and gains the later ones'
    senses, so the round-robin across a word's entries (`reduce_common._join_senses_by_pos`) takes
    the first etymology's senses before the next one's. An entry with no senses merges into none:
    it keeps its place, as written, and when it is the first of its part of speech the later ones'
    senses merge into it. A line that is no JSON object — undecodable, or another JSON value — is
    left out. The headword's exact spelling is the key: an acronym's entries (« CASA ») never merge
    with the common word's.
    """
    if not (MERGE_SAME_POS_ETYMOLOGIES if merged is None else merged):
        return src
    with open(src, encoding="utf-8") as f, open(dst, "w", encoding="utf-8") as out:
        group_word, group = None, []  # the current word's entries, by part of speech, in order

        def flush():
            for entry in group:
                out.write(json.dumps(entry, ensure_ascii=False) + "\n")
            group.clear()

        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if not isinstance(entry, dict):
                continue
            if entry.get("word") != group_word:
                flush()
                group_word = entry.get("word")
            senses = entry.get("senses") or []
            same = next((e for e in group if e.get("pos") == entry.get("pos")), None) if senses else None
            if same is None:
                group.append(entry)
            else:
                same["senses"] = [*(same.get("senses") or []), *senses]
        flush()
    return dst
