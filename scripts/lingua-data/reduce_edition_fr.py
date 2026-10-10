# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The French Wiktionary's rules (frwiktionary), for the pairs glossed in French
(generalise-lingua-gloss-reducer).

`FR` is the rules every pair glossed in French had before editions existed, unchanged: the same
regular expressions, moved here from `reduce_common.py`. Re-reducing en-fr and es-fr from their
pinned sources with it gives their tables byte for byte.

en-fr also reads the French Wiktionary for its studied side — which senses of an English entry are
meanings, and which word a form-of sense names (`reduce_common.wiktionary_signals`) — so the
form-of wording here shapes en-fr's forms too.

A rule module: a pair's reducer that imports it has its sha256 in its rule digest
(pack_sources.py `rule_files`), and no pair glossed in another language does.

It also holds the rule of English's and Spanish's dictionary words (`dictionary_words`,
leave-lingua-names-out-of-english-and-spanish-words D2): en-fr and es-fr, the two pairs that load this
module, are their studied languages' references.
"""

import re

import reduce_common as common

# French frwiktionary "form-of" gloss templates — these mark an entry that is an
# inflected form, not a word with a meaning of its own; never a useful translation.
# kaikki also tags most such senses `form-of` (see `reduce_common._is_form_of`); the regex
# catches the untagged ones.
_FORM_OF = re.compile(
    r"^(pluriel|singulier|f[ée]minin|masculin|participe|pr[ée]t[ée]rit|pass[ée]|imparfait|"
    r"comparatif|superlatif|g[ée]rondif|(troisi[èe]me|deuxi[èe]me|premi[èe]re) personne|"
    r"variante|autre graphie|forme (de|du|d'|verbale|fl[ée]chie)|genre|orthographe)\b",
    re.IGNORECASE,
)


# Four more pointer wordings, applied in the MULTI-WORD path ONLY: "Présent progressif.",
# "Graphie alternative de douchebag." name a tense or a spelling, not a meaning. They are
# not in `_FORM_OF` because that regex also feeds forms.tsv, freq.tsv and gloss.tsv, which
# en-fr's single-word tables must keep producing byte for byte; the eight senses it
# would cost there are not worth the risk.
_MWE_FORM_OF = re.compile(r"^(pr[ée]sent|futur|conjugaison|graphie)\b", re.IGNORECASE)


# A sense left hanging on a coordinator: the Wiktionary line read "(Vieilli) ou Pluie" and
# the parenthetical went, so the gloss opens on "ou". LOWERCASE only — `etcetera` is glossed
# "Et cetera", where the coordinator IS the translation, and a capital is what tells them
# apart across the 15 entries the corpus holds.
_DANGLING_COORDINATOR = re.compile(r"^(?:ou|et)\s+")


# What the Wiktionary writes for its own readers, not a translation: a pointer to another page
# ("Y avoir. → voir there be", "(→ voir bone marrow)", "(→ Comparer avec -ative)") — a link on the
# wiki, dead text on a card — and the placeholders of an unfinished page ("Définition manquante ou
# à compléter. (Ajouter)", an invitation to contributors), wherever they sit in the sense.
_WIKI_NOTES = re.compile(
    r"\s*\(→[^)]*\)"
    r"|\s*→\s*(?:voir|comparer)\b[^;]*"
    r"|\s*\(?Définition manquante ou à co.*?(?:\(Ajouter\)\)?|$)[.…]*"
    r"|\s*Étymologie manquante ou incomplète.*?(?:cliquant ici\.|$)",
    re.IGNORECASE,
)


# A sense naming a letter of the alphabet: « Nom de la lettre d. », « Bé, nom de la lettre b. »,
# « Lettre s. », « … lettre de l'alphabet espagnol » (fix-lingua-spanish-card-noise D2).
_LETTER = re.compile(r"\bnom de la lettre\b|^lettre [a-zñ]\.?$|\blettre de l[’']alphabet\b", re.IGNORECASE)


# The French Wiktionary. Its pointers are tagged `form-of` and name their word in `form_of`; its
# senses open on a capital, so a gloss made of translation-table words does too.
FR = common.Edition(
    code="fr",
    form_of=_FORM_OF,
    mwe_form_of=_MWE_FORM_OF,
    notes=_WIKI_NOTES,
    letter=_LETTER,
    dangling=_DANGLING_COORDINATOR,
)


# English's and Spanish's dictionary words (leave-lingua-names-out-of-english-and-spanish-words D1,
# D2). en-fr is English's reference pair and es-fr Spanish's: the lemmas each glosses are the words
# its studied language's vocabulary estimate counts and its names rule keeps as words, but not a
# lemma it glosses by a proper noun's senses alone (`london`, `margaret`, `madrid`, `maría`), which
# is a name, not a word to learn. « A name » is what the French Wiktionary files as a proper noun
# (« Nom propre », kaikki's `name`): every sense run of the lemma's row in `senses.tsv` is `PROPN`.
# The lemma keeps its gloss, and a word with a common sense beside a name's stays a word (`bill`,
# `mark`, `luna`); a glossed lemma with no run at all is a word too.
#
# The rule has three homes, one per reference pair's edition: this module for en-fr and es-fr, the
# only pairs that load it, and `reduce-fr-en.py` for fr-en (refine-lingua-fr-en-glosses D2). No
# shared module carries it: `reduce_common.py` is in every pair's rule digest, so es-en's, en-es's,
# fr-en's and fr-es's `pack_version` would move for nothing, and a module fr-en loaded would move
# fr-en's pin and pack.
def dictionary_words(glosses, runs):
    """English's or Spanish's dictionary words, byte-sorted: the lemmas `glosses` glosses less those
    that have sense runs (`runs`, `senses.tsv`'s: `lemma → [(pos, count), …]`) and whose every run is
    a proper noun's (`PROPN`)."""
    names = {w for w in glosses if runs.get(w) and all(pos == "PROPN" for pos, _ in runs[w])}
    # Code-point order is UTF-8 byte order.
    return sorted(set(glosses) - names)
