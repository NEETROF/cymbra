# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The English Wiktionary's rules (enwiktionary), for the pairs glossed in English
(generalise-lingua-gloss-reducer).

No committed pair reads it yet: es-en will (the language matrix's change 21). Its rules come from a
census of kaikki's extract of the English Wiktionary's Spanish section, the 2026-09-28 dump es-fr
pins: 811,049 entries, 875,591 senses with a gloss.

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
- A letter's name: "The name of the Latin script letter D/d.".
- Casing: 98.3 % of the meaning senses open on a lower-case letter, the edition's convention for a
  foreign word's senses, so a gloss made of translation-table words keeps the case its words have.
- Long parentheses (M20, open): 5,724 of 147,653 meaning senses hold one of 40 characters or more
  ("a former unit of length equivalent to about 27.9 cm"). Kept, until es-en's review settles it.

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does — tuning it never
re-pins en-fr or es-fr.
"""

import re

import reduce_common as common

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

# A sense naming a letter: "The name of the Latin script letter D/d.".
_LETTER = re.compile(r"^(?:the )?name of the (?:[\w-]+ )?(?:script )?(?:letter|digraph)\b", re.IGNORECASE)

EN = common.Edition(
    code="en",
    form_of=_FORM_OF,
    letter=_LETTER,
    pointer_tags=frozenset({"form-of", "alt-of"}),
    pointer_fields=("form_of", "alt_of"),
    capitalised=False,
)
