#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw EN->FR sources (ESDB + wordfreq + kaikki + CEFR lists) into the pack tables.

Inputs (fetched into <work>/ by build.sh, git-ignored; pinned in tables/<pair>/pin.json):
  - scowl.txt              ESDB export, the inflections (SIZE: LEMMA <POS>: forms)
  - kaikki-Anglais.jsonl   kaikki frwiktionary "Anglais" extract (FR glosses of EN words, and the
                           form links that complete ESDB)
  - agid-infl.txt          AGID, retired: read only with --inflections agid
  - wordfreq (pip)         English frequency ranks
  - cefrj-*.csv / octanove-*.csv   CEFR levels (optional)
and, from this repository, the analyser's irregular-form table (lingua-core lemmatize.rs).

Outputs (into <work>/, consumed by lingua-pack-build):
  forms.tsv  (form<TAB>lemma) / freq.tsv (lemma<TAB>rank) / gloss.tsv (lemma<TAB>gloss)
  / level.tsv (lemma<TAB>A1..C2) / mwe.tsv (expression<TAB>gloss)
  / grammar.tsv (form<TAB>lemma<TAB>tag<TAB>other|-) / senses.tsv (lemma<TAB>tag:count…)
  / NOTICE / manifest.json

Key rules:
1. Only CANONICAL LEMMAS (base forms) are ever treated as lemmas. An inflected form (e.g.
   "targets", "gives") is kept in forms.tsv so it lemmatises to its base, but is NEVER
   given a rank or a gloss of its own — otherwise it becomes a spurious pool lemma that
   (a) carries a useless "Pluriel de …" form-of gloss and (b) collides with its base
   lemma's entry.
2. The inflection source is not always right about what an inflection is — AGID, the source
   before ESDB, listed "butter" as the comparative of "but", "number" as the comparative of
   "numb", "his" as the plural of "hi". A form that is really a word of its own (`own_words`)
   stays a canonical lemma.
3. Every form maps to exactly ONE lemma (`resolve_forms`): itself when it is a kept word of
   its own, otherwise the base Wiktionary names, one with a gloss, the most frequent. The
   pack's FST keeps a single lemma per form and, left to choose, keeps the alphabetically
   first — "leaves" read as "leaf".
4. CEFR-listed words the frequency list lacks — rare C1/C2 words, hyphenated compounds
   wordfreq never ranks, inflections whose base was dropped ("boring" from "bore") — are
   added (`append_level_extras`), and a listed compound gets its inflections
   (`compound_inflections`), so the level table covers nearly all of the CEFR lists.
6. GRAMMAR (add-lingua-word-grammar) is kept, not thrown away: what each inflected form is,
   from ESDB's slots and the ending of kaikki's regular form links (`esdb_slot_tags`,
   `_regular_tags`), in Universal Dependencies tags; and the part of speech of each sense of a
   gloss, whose senses are grouped by it (`_join_senses_by_pos`).
5. MULTI-WORD entries ("give up", "starting point") are reduced on their own
   (`reduce_expressions`): a lemma is one word, so the frequency lexicon can never hold
   them and every rule above passes them by. They are emitted AS WRITTEN — keying them
   is the builder's job, which lemmatises each word against the lexicon it has just
   assembled, a cascade this script can only half mirror.

Pack is scoped to the top-N canonical lemmas (+ those CEFR words) to fit the 5 MB budget.
Output is sorted, so a rebuild from the same snapshots is byte-identical.
"""

import argparse
import bisect
import csv
import functools
import itertools
import json
import os
import re

_TOKEN = re.compile(r"[A-Za-z][A-Za-z'\-]*")

_REPO = os.path.abspath(os.path.join(os.path.dirname(os.path.abspath(__file__)), "..", ".."))
_LEMMATIZE_RS = os.path.join(_REPO, "crates", "lingua-core", "src", "analysis", "lemmatize.rs")

# French frwiktionary "form-of" gloss templates — these mark an entry that is an
# inflected form, not a word with a meaning of its own; never a useful translation.
# kaikki also tags most such senses `form-of` (see `_is_form_of`); the regex catches the
# untagged ones.
_FORM_OF = re.compile(
    r"^(pluriel|singulier|f[ée]minin|masculin|participe|pr[ée]t[ée]rit|pass[ée]|imparfait|"
    r"comparatif|superlatif|g[ée]rondif|(troisi[èe]me|deuxi[èe]me|premi[èe]re) personne|"
    r"variante|autre graphie|forme (de|du|d'|verbale|fl[ée]chie)|genre|orthographe)\b",
    re.IGNORECASE,
)

# Four more pointer wordings, applied in the MULTI-WORD path ONLY: "Présent progressif.",
# "Graphie alternative de douchebag." name a tense or a spelling, not a meaning. They are
# not in `_FORM_OF` because that regex also feeds forms.tsv, freq.tsv and gloss.tsv, which
# this pair's single-word tables must keep producing byte for byte; the eight senses it
# would cost there are not worth the risk.
_MWE_FORM_OF = re.compile(r"^(pr[ée]sent|futur|conjugaison|graphie)\b", re.IGNORECASE)

# The English word a form-of gloss points at: "Pluriel de datum.", "Prétérit du verbe to
# find". Every candidate is collected; callers only test membership.
_FORM_OF_TARGET = re.compile(
    r"\b(?:de|du|d['’])\s*(?:verbe\s+|adjectif\s+|nom\s+)?(?:to\s+)?[«“\"]?\s*"
    r"([A-Za-z][A-Za-z'\-]*)",
    re.IGNORECASE,
)

# AGID relation kinds: N = plural, V = verb form, A = comparison.
# What a relation can make of a word, in the CEFR lists' vocabulary (a participle is
# routinely taught as an adjective) ...
_INFLECTION_POS = {
    "N": {"noun"},
    "V": {"verb", "be-verb", "do-verb", "have-verb", "modal auxiliary", "adjective"},
    "A": {"adjective", "adverb"},
}
# ... and what the BASE must be for the relation to make sense, per source.
_WIKT_BASE_POS = {"N": {"noun", "name"}, "V": {"verb"}, "A": {"adj", "adv"}}
_CEFR_BASE_POS = {
    "N": {"noun"},
    "V": {"verb", "be-verb", "do-verb", "have-verb", "modal auxiliary"},
    "A": {"adjective", "adverb"},
}


# An AGID variant level after a form ("born 1", "lesser 1.1"): a lesser spelling, archaic or rarer
# form — the AGID side of `lesser_variant`.
_AGID_VARIANT = re.compile(r"\S+\s+\d+(?:\.\d+)?")


def agid_forms(inflections):
    """Cleaned inflected forms from an AGID right-hand side, lesser variants out ("born 1")."""
    cleaned = re.sub(r"\{[^}]*\}", " ", inflections)
    out = []
    for tok in re.split(r"[|,]", cleaned):
        tok = tok.strip().strip("?!~").strip()
        if _AGID_VARIANT.fullmatch(tok):
            continue
        if _TOKEN.fullmatch(tok):
            out.append(tok.lower())
    return out


def parse_agid_relations(path):
    """All (form, lemma) pairs from AGID, and each inflected form's (lemma, kind) relations.

    A `?` on the part of speech (e.g. `gif N?: gives`) marks a doubtful headword; such
    lines carry bogus inflections (AGID really does claim the noun "gif" pluralises to
    "gives"), so they are dropped. The relation kind is the flag's first letter: N, V or A.
    """
    pairs = set()
    relations = {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if ":" not in line:
                continue
            left, right = line.split(":", 1)
            head = left.split()
            if not head:
                continue
            lemma = head[0].lower()
            if not _TOKEN.fullmatch(lemma):
                continue
            if any("?" in flag for flag in head[1:]):  # questionable POS -> skip
                continue
            kind = head[1][0] if len(head) > 1 else ""
            pairs.add((lemma, lemma))
            for form in agid_forms(right):
                pairs.add((form, lemma))
                if form != lemma:
                    relations.setdefault(form, set()).add((lemma, kind))
    return pairs, relations


# — ESDB, the inflection source (switch-lingua-inflections-to-esdb) —
#
# ESDB (the English Speller Database, en-wl/wordlist, formerly SCOWLv2) is the maintained successor
# of AGID, whose last release is 2016. Its `scowl.txt` holds one group per sense, a line
#   SIZE [tags]: [VARIANT-INFO: ] LEMMA <POS[/class]> [{sense}]: DERIVED, DERIVED, …
# where a derived entry may be a set of alternatives, `(focuses | ~: focusses)`, each prefixed by
# its spellings' variant levels. `parse_esdb_relations` returns what `parse_agid_relations` does,
# so everything downstream reads it unchanged.

# The parts of speech whose derived forms are inflections, and the relation kind they make.
# `d`, a determiner, only for its comparisons ("few": "fewer", "fewest"): its other derived forms
# are words of their own ("that": "those").
_ESDB_KIND = {"n": "N", "v": "V", "m": "V", "n_v": "V", "aj": "A", "av": "A", "a": "A", "aj_av": "A", "d": "A"}
# "A valid word in current usage" per ESDB's own scale; larger sizes hold rare and obscure words.
_ESDB_MAX_SIZE = 80
_ESDB_ANNOTATIONS = "*-@~!†"
# A spelling item of a variant level that counts: a spelling (A American, B British "-ise",
# Z British "-ize", C Canadian, `_` all of them) with a primary level, alone or marked equal (`.`
# or `=`). Not D, Australian: that code carries a copyright of its own (ESDB's `Copyright`, "=== AU").
_ESDB_SPELLING = re.compile(r"[ABZC_]+[.=]?")
_ESDB_LINE = re.compile(r"\s*(\d+)")
_ESDB_LEMMA = re.compile(r"\s*(.*?)\s*<([a-z_]+)?(?:/[^>]*)?>")


def lesser_variant(levels):
    """Whether an ESDB alternative is only ever a lesser spelling — never an inflection to take.

    `levels` is what precedes the form (`"AV Bv"` in `AV Bv: focussed`, `"~"` in `~: born`). An
    alternative counts when at least one of its spellings is primary or equal (`A B: focused`,
    `B Zv: learnt`, `A B= Z: learned`); it does not when every spelling is a lesser level: a variant
    (`AV Bv: focussed`), an archaic or rarer form (`~: born`, `@: art`). Taking `born` for a form of
    `bear` would send 123 tokens of the measured sample to the wrong word.
    """
    return not any(_ESDB_SPELLING.fullmatch(item) for item in levels.split())


def esdb_forms(entries):
    """The inflected forms of an ESDB derived-entries field, lesser variants and annotations out."""
    out = []
    for entry in entries.split(", "):
        entry = entry.strip()
        alternatives = entry[1:-1].split("|") if entry.startswith("(") and entry.endswith(")") else [entry]
        for alt in alternatives:
            alt = alt.strip()
            if ": " in alt:
                levels, alt = alt.split(": ", 1)
                if lesser_variant(levels):
                    continue
            alt = alt.strip().rstrip(_ESDB_ANNOTATIONS).strip()
            if alt and alt != "-":
                out.append(alt)
    return out


def _esdb_kinds(pos, form, base):
    """The relation kinds a derived form makes: a noun-verb's `-s` form is both a plural and a verb form."""
    if pos == "n_v":
        return {"N", "V"} if form.endswith("s") else {"V"}
    if pos == "d":
        return {"A"} if form in ("more", "most", "less", "least") or regular_inflection(form, base, "A") else set()
    return {_ESDB_KIND[pos]}


# — Word grammar (add-lingua-word-grammar) —
#
# Universal Dependencies tags (design D1) for what a form is, as ESDB's slots and kaikki's regular
# form links say, and for the part of speech of each sense of a gloss. The builder checks every
# tag against the core's closed vocabulary and fails on anything else.
_PAST = "VERB|Mood=Ind|Tense=Past|VerbForm=Fin"
_PARTICIPLE = "VERB|Tense=Past|VerbForm=Part"
_ING = "VERB|VerbForm=Ger"
_THIRD = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"
_PRESENT = "VERB|Mood=Ind|Tense=Pres|VerbForm=Fin"
_PLURAL = "NOUN|Number=Plur"
# `be`, the one verb with eight slots: vd vd2 vn vg vs vs2 vs3 vs4 in ESDB's README.
_BE_SLOTS = (
    (_PAST,),  # was
    (_PAST,),  # were
    (_PARTICIPLE,),  # been
    (_ING,),  # being
    ("VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",),  # am
    (_PRESENT,),  # are
    (_THIRD,),  # is
    (_PRESENT,),  # are
)
# The parts of speech whose slots are the comparative then the superlative.
_DEGREE_UPOS = {"aj": "ADJ", "a": "ADJ", "aj_av": "ADJ", "av": "ADV", "d": "DET"}
_POSSESSIVE = ("'s", "s'", "'")


def _possessive(forms):
    return bool(forms) and all(form.endswith(_POSSESSIVE) for form in forms)


def esdb_slot_tags(pos, lemma, field):
    """What each derived form of an ESDB line is: [(form, tag)], by its slot.

    The slots are ESDB's own (its README, "The derived forms are as follows"). A verb's are past,
    past participle, -ing and -s, the participle left out when it is spelled like the past; `be`
    has eight. A noun-verb's -s slot is both the plural and the -s verb form (as `_esdb_kinds`
    says). A noun's first slot is its plural; an adjective's, adverb's or determiner's are the
    comparative then the superlative, and a determiner with one slot has only a plural ("those"),
    a word of its own. A modal ("could, -, can") has no -ing, so its past is no participle, and its
    -s slot, spelled like the modal, is no reading. Possessives, trailing, are never readings.
    """
    slots = [esdb_forms(entry) for entry in field.split(", ")] if field.strip() else []
    while slots and _possessive(slots[-1]):
        slots.pop()
    if pos in ("v", "m", "n_v"):
        if lemma == "be" and len(slots) == 8:
            tags = _BE_SLOTS
        elif len(slots) == 4:
            tags = ((_PAST,), (_PARTICIPLE,), (_ING,), (_THIRD,))
        elif len(slots) == 3:
            tags = ((_PAST, _PARTICIPLE) if slots[1] else (_PAST,), (_ING,), (_THIRD,))
        else:
            return []
        if pos == "n_v":
            tags = tags[:-1] + ((_THIRD, _PLURAL),)
    elif pos == "n":
        tags = ((_PLURAL,),)
    elif pos in _DEGREE_UPOS and not (pos == "d" and len(slots) < 2):
        upos = _DEGREE_UPOS[pos]
        tags = ((f"{upos}|Degree=Cmp",), (f"{upos}|Degree=Sup",))
    else:
        return []
    out = []
    for forms, slot_tags in zip(slots, tags):
        for form in (f.lower() for f in forms):
            if form.endswith(_POSSESSIVE) or not _TOKEN.fullmatch(form):
                continue
            for tag in slot_tags:
                if tag == _THIRD and form == lemma:
                    continue  # a modal's -s slot: "can" does not inflect
                if tag == _PLURAL and pos == "n_v" and not form.endswith("s"):
                    continue
                out.append((form, tag))
    return out


def _regular_tags(form, kind):
    """What a regular inflection of `kind` is, by its ending (kaikki's form links; design D2)."""
    if kind == "N":
        return (_PLURAL,)
    if kind == "A":
        return ("ADJ|Degree=Sup",) if form.endswith("st") else ("ADJ|Degree=Cmp",)
    if form.endswith("ing"):
        return (_ING,)
    if form.endswith("d"):
        return (_PAST, _PARTICIPLE)
    return (_THIRD,)


def _kind_of(tag):
    """The relation kind a reading's tag makes: N, V or A."""
    if tag.startswith("NOUN"):
        return "N"
    return "A" if "|Degree=" in tag else "V"


def parse_esdb_relations(path, max_size=_ESDB_MAX_SIZE, readings=None):
    """All (form, lemma) pairs from ESDB's `scowl.txt`, and each inflected form's (lemma, kind) relations.

    Kept: the parts of speech of `_ESDB_KIND`, sizes up to `max_size`, primary and equal spellings.
    Never a possessive (the tokenizer splits them; AGID lists none). Never a form ESDB also lists as
    an adjective of its own in a commoner size than the line deriving it: "renowned" (an adjective
    at 35) is no verb form of "renown" (a verb only at 80) — `own_adjectives`.

    `readings`, when given, is filled with what each kept form is: (form, lemma) -> {tag}, by
    `esdb_slot_tags`, for the pairs this returns — a form spelled like its lemma included ("put" is
    its own past), since (lemma, lemma) is always a pair.
    """
    pairs, relations = set(), {}
    derived_at = {}  # (form, lemma) -> the smallest size of a line deriving it
    adjectives = {}  # headword -> the smallest size ESDB lists it at as an adjective
    group = None  # the lemma a `-` stands for, within a group
    with open(path, encoding="utf-8") as f:
        for raw in f:
            line = raw.split("#", 1)[0].rstrip("\n")
            if not line.strip():
                group = None
                continue
            parts = line.split(": ")
            size = _ESDB_LINE.match(parts[0])
            if not size:
                continue
            at = next((i for i in range(1, len(parts)) if "<" in parts[i] or (i == 1 == len(parts) - 1)), None)
            if at is None:
                continue
            head = _ESDB_LEMMA.match(parts[at])
            word = (head.group(1) if head else parts[at]).strip()
            pos = head.group(2) if head else None
            if word != "-":
                word = word.lstrip("-@!").rstrip(_ESDB_ANNOTATIONS).strip()
            if word == "-":
                if group is None:
                    continue
                word = group
            else:
                group = word
            level = int(size.group(1))
            if level > max_size or pos not in _ESDB_KIND:
                continue
            lemma = word.lower()
            if not _TOKEN.fullmatch(lemma):
                continue
            if pos == "aj":
                adjectives[lemma] = min(level, adjectives.get(lemma, level))
            pairs.add((lemma, lemma))
            for form in (f.lower() for f in esdb_forms(": ".join(parts[at + 1 :]))):
                if form.endswith(("'s", "s'", "'")) or not _TOKEN.fullmatch(form):
                    continue
                if form == lemma:
                    continue
                kinds = _esdb_kinds(pos, form, lemma)
                if not kinds:
                    continue
                pairs.add((form, lemma))
                derived_at[(form, lemma)] = min(level, derived_at.get((form, lemma), level))
                for kind in kinds:
                    relations.setdefault(form, set()).add((lemma, kind))
            if readings is not None:
                for form, tag in esdb_slot_tags(pos, lemma, ": ".join(parts[at + 1 :])):
                    readings.setdefault((form, lemma), set()).add(tag)
    for form, lemma in own_adjectives(derived_at, adjectives):
        pairs.discard((form, lemma))
        rels = {rel for rel in relations.get(form, ()) if rel[0] != lemma}
        if rels:
            relations[form] = rels
        else:
            relations.pop(form, None)
    if readings is not None:
        for key in [key for key in readings if key not in pairs]:
            del readings[key]
    return pairs, relations


def own_adjectives(derived_at, adjectives):
    """The (form, lemma) derivations to drop: the form is an adjective of its own, commoner than the derivation.

    ESDB sizes say how common a line is (35 the commonest words, 80 the rarer valid ones). A form
    it lists as an adjective at a smaller size than the line that derives it is read as that
    adjective ("renowned", 35, over "renown <v>: renowned", 80). At an equal size the derivation
    stands: "tired" stays a form of "tire", as it always has.
    """
    return {(form, lemma) for (form, lemma), level in derived_at.items() if adjectives.get(form, level) < level}


# kaikki's form-of links, for the recent words ESDB lacks (smartphones, influencers, datasets).
_FORM_OF_KIND = (
    (re.compile(r"^pluriel", re.IGNORECASE), "N"),
    (re.compile(r"pr[ée]t[ée]rit|participe|personne du pr[ée]sent|pr[ée]sent simple", re.IGNORECASE), "V"),
    (re.compile(r"comparatif|superlatif", re.IGNORECASE), "A"),
)


def form_of_relations(path, pairs, relations, readings=None):
    """Add kaikki's form-of links to `pairs` and `relations`, regular inflections only.

    A link counts only when the form is the regular inflection of its target (`regular_inflection`):
    without that condition kaikki sends "occupied" to "nanny", "stocks" to "mot", "coats" to
    "coast" and "born" to "bear". Never to a one- or two-letter target ("des" is no plural of "de"
    in English text): the irregular forms of such words ("goes", "is") come from ESDB. Returns the
    number of pairs added.

    `readings`, when given, gets what each pair it adds is, by its ending (`_regular_tags`); a pair
    ESDB already gave keeps ESDB's slots.
    """
    added = 0
    linked = set()  # the pairs these links add, which take their readings from them
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            if '"form_of"' not in line:
                continue
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            form = (d.get("word") or "").strip().lower()
            if not _TOKEN.fullmatch(form):
                continue
            for sense in d.get("senses", []):
                gloss = " ".join(sense.get("glosses") or [])
                kind = next((k for pattern, k in _FORM_OF_KIND if pattern.search(gloss)), None)
                if kind is None:
                    continue
                for ref in sense.get("form_of") or ():
                    lemma = re.sub(r"^to ", "", (ref.get("word") or "").strip()).lower() if isinstance(ref, dict) else ""
                    if lemma == form or len(lemma) <= 2 or not _TOKEN.fullmatch(lemma):
                        continue
                    if not regular_inflection(form, lemma, kind):
                        continue
                    if (form, lemma) not in pairs:
                        added += 1
                        linked.add((form, lemma))
                    pairs.update({(form, lemma), (lemma, lemma)})
                    relations.setdefault(form, set()).add((lemma, kind))
                    if readings is not None and (form, lemma) in linked:
                        readings.setdefault((form, lemma), set()).update(_regular_tags(form, kind))
    return added


def analyser_irregulars(path=_LEMMATIZE_RS):
    """The analyser's hard-coded irregular forms (lingua-core `IRREGULARS`): form -> lemma.

    The analyser resolves these before it ever looks at the pack, so a pack lemma for one
    of them ("left", "found") could never be reached by reading.
    """
    with open(path, encoding="utf-8") as f:
        src = f.read()
    start = src.find("const IRREGULARS")
    end = src.find("];", start)
    table = dict(re.findall(r'\(\s*"([^"]+)"\s*,\s*"([^"]+)"\s*\)', src[start:end])) if start >= 0 else {}
    if not table:
        raise ValueError(f"no IRREGULARS table in {path}")
    return table


def _is_form_of(sense, gloss):
    """Whether a kaikki sense only points at another word ("Pluriel de …", "Passé de …")."""
    return "form-of" in (sense.get("tags") or ()) or bool(sense.get("form_of")) or bool(_FORM_OF.match(gloss))


def wiktionary_signals(path, words):
    """For the given words: the parts of speech they have a meaning under, and their form-of targets.

    A sense is a meaning when its gloss says what the word means ("Beurre."), not which
    word it is a form of ("Comparatif de numb."); the targets are the words such pointers
    name (kaikki's `form_of` field, or the word the gloss ends on).
    """
    meanings, targets = {}, {}
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            word = (d.get("word") or "").strip().lower()
            if word not in words:
                continue
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                gloss = gg[0].strip() if gg else ""
                if not gloss:
                    continue
                if _is_form_of(sense, gloss):
                    found = {t.lower() for t in _FORM_OF_TARGET.findall(gloss)}
                    found |= {
                        (ref.get("word") or "").strip().lower()
                        for ref in sense.get("form_of") or ()
                        if isinstance(ref, dict)
                    }
                    found.discard("")
                    targets.setdefault(word, set()).update(found)
                else:
                    meanings.setdefault(word, set()).add(d.get("pos") or "")
    return meanings, targets


def regular_inflection(form, base, kind):
    """Whether `form` is the regular plural / verb form / comparison of `base`.

    "butter" IS the regular comparative of "but" — spelling alone never clears a relation;
    it only makes a relation whose base has the right part of speech believable.
    """
    vowels = "aeiou"
    doubles = len(base) >= 3 and base[-1] not in vowels + "wxy" and base[-2] in vowels and base[-3] not in vowels
    if kind == "N":
        made = {base + "s", base + "es"}
        if base.endswith("y"):
            made.add(base[:-1] + "ies")
    elif kind == "V":
        made = {base + "s", base + "es", base + "ed", base + "d", base + "ing"}
        if base.endswith("e"):
            made.add(base[:-1] + "ing")
        if base.endswith("y"):
            made |= {base[:-1] + "ied", base[:-1] + "ies"}
        if doubles:
            made |= {base + base[-1] + "ed", base + base[-1] + "ing"}
    elif kind == "A":
        made = {base + "er", base + "est", base + "r", base + "st"}
        if base.endswith("y"):
            made |= {base[:-1] + "ier", base[:-1] + "iest"}
        if doubles:
            made |= {base + base[-1] + "er", base + base[-1] + "est"}
    else:
        return False
    return form in made


def _taught(cefr, word):
    return {pos for pos, _ in cefr.get(word, ())}


def is_believable(form, base, kind, meanings, targets, cefr):
    """Whether `form` is believably a `kind` form of `base`.

    The base has the part of speech the relation needs — a meaning under it in Wiktionary, or a CEFR
    list teaching it so — and the form is its regular inflection, or one Wiktionary names as a form
    of it. `own_words` asks it of every relation; the grammar table marks with it which dictionary
    forms may be named as another reading of a form (add-lingua-word-grammar, design D2).
    """
    return bool(
        (meanings.get(base, set()) & _WIKT_BASE_POS.get(kind, set()) or _taught(cefr, base) & _CEFR_BASE_POS.get(kind, set()))
        and (regular_inflection(form, base, kind) or base in targets.get(form, set()))
    )


def own_words(relations, meanings, targets, cefr, frequency, never=frozenset()):
    """The AGID-inflected forms that are really words of their own and must stay lemmas.

    Never one of `never` (the analyser's irregular forms). A form is its own word when a
    CEFR list teaches it as a modal ("could", "might": taught apart from "can", "may"), or
    with a part of speech its relations cannot produce while none of its bases is taught
    with the part of speech the relation needs ("customer", a noun, is no comparison of
    "custom", a noun; "feed", a noun, no verb form of "fee"; "times" stays "time", taught
    as the noun it pluralises).

    Otherwise never an -ing/-ed form (in running text those are overwhelmingly the verb:
    "used", "going"), and a form is its own word when any of these holds:
    - Wiktionary gives it a meaning, never names any of its bases as what it is a form of,
      no relation is believable — a base with the right part of speech it regularly
      inflects to ("butter": "but" is no adjective; "sales" stays "sale") — and AGID's base
      is no better candidate: a form no CEFR list teaches keeps a one- or two-letter base
      ("ros", an acronym plural of "ro") and a CEFR-taught base it is no regular spelling of
      ("yourselves" stays "yourself", while "timer" leaves "time");
    - Wiktionary gives it a meaning without naming a base, and a CEFR list teaches it with
      an impossible part of speech ("owner", a noun, next to the adjective "own");
    - it has a meaning and is over six times (0.8 on wordfreq's log10 Zipf scale) commoner
      than its commonest base: at that gap the form is the word readers meet, even when it
      is a genuine, lexicalised inflection ("number" / "numb", "data" / "datum", "gas" /
      "ga").

    How the rule was chosen, and how well it does, is in SOURCES.md ("Words AGID gets wrong").
    """

    def taught(word):
        return _taught(cefr, word)

    out = set()
    for form, rels in relations.items():
        if form in never:
            continue
        producible = set().union(*(_INFLECTION_POS.get(kind, set()) for _, kind in rels))
        mistaught = bool(taught(form) - producible)
        taught_base = any(taught(base) & _CEFR_BASE_POS.get(kind, set()) for base, kind in rels)
        if (mistaught and not taught_base) or "modal auxiliary" in taught(form):
            out.add(form)
            continue
        if form.endswith(("ing", "ed")):
            continue
        meaningful = bool(meanings.get(form))
        named = targets.get(form, set())
        unattested = meaningful and not any(base in named for base, _ in rels)
        believable = any(is_believable(form, base, kind, meanings, targets, cefr) for base, kind in rels)
        vouched = bool(taught(form)) or not any(
            len(base) <= 2 or (base in cefr and not regular_inflection(form, base, kind)) for base, kind in rels
        )
        if (
            (unattested and not believable and vouched)
            or (unattested and mistaught)
            or (meaningful and frequency(form) - max(frequency(base) for base, _ in rels) >= 0.8)
        ):
            out.add(form)
    return out


def canonical_ranks(inflected, want):
    """Dense ranks over the top canonical lemmas (wordfreq order, inflected forms skipped)."""
    from wordfreq import top_n_list

    ranks = {}
    for w in top_n_list("en", want * 4):
        w = w.strip().lower()
        if w in ranks or not _TOKEN.fullmatch(w) or w in inflected:
            continue
        ranks[w] = len(ranks) + 1
        if len(ranks) >= want:
            break
    return ranks


def orphaned_forms(inflected, pairs, ranks):
    """Inflected forms with no kept lemma behind them: read as words of their own.

    A form is left out of the lemmas because its base stands for it; when no base is kept — nor
    any base of a base — nothing does, and the word drops out of the pack. ESDB knows rare bases
    AGID did not ("grandkid", "policymaker", "uprise", "gree", "crowdfund"), and wordfreq ranks
    their forms far above them: without this, "grandkids", "policymakers", "uprising", "greed" and
    "crowdfunding" would vanish. A form whose base is itself a form of a kept lemma ("buildings",
    of "building", of "build") is not orphaned: it reads as that chain always did.
    """
    bases = {}
    for form, lemma in pairs:
        if form != lemma:
            bases.setdefault(form, set()).add(lemma)
    kept = ranks.keys()

    def behind(form):
        direct = bases.get(form, set())
        return direct | set().union(*(bases.get(base, set()) for base in direct))

    return {form for form in inflected if not behind(form) & kept}


def append_level_extras(ranks, cefr, inflected, frequency, lemma_of):
    """Ranks extended with the CEFR headwords the pack would otherwise not hold.

    A CEFR word is added when it has no rank and does not already resolve to a kept lemma
    through `lemma_of` — an inflection whose base was dropped ("boring" from "bore") would
    otherwise vanish. So a declared level and the ladder see (nearly) the whole list.

    A hyphenated compound whose parts are all ranked takes its rarest part's rank
    ("well-known" ranks as "known"): its rank-based presumption matches the analyser's
    weakest-part judgement of it when unlisted, although statuses set on its parts no
    longer reach it. Any other word ranks with the last ranked lemma at least as frequent
    (`frequency`: wordfreq's Zipf value in the real build), or after the whole list, the
    commonest first, when it is rarer than all of them.
    """
    order = sorted(ranks, key=ranks.get)
    # -Zipf in rank order, forced non-decreasing so it can be bisected.
    scale = list(itertools.accumulate((-frequency(lemma) for lemma in order), max))
    out = dict(ranks)
    rarest = []
    for w in sorted(cefr):
        if w in ranks or not _TOKEN.fullmatch(w) or (w in inflected and lemma_of(w) is not None):
            continue
        if "-" in w:
            parts = [ranks.get(lemma_of(part)) for part in w.split("-")]
            if all(parts):
                out[w] = max(parts)
                continue
        else:
            at = bisect.bisect_right(scale, -frequency(w))
            if at < len(order):
                out[w] = ranks[order[max(at, 1) - 1]]
                continue
        rarest.append(w)
    last = max(ranks.values(), default=0)
    for i, w in enumerate(sorted(rarest, key=lambda w: (-frequency(w), w)), start=1):
        out[w] = last + i
    return out


def compound_inflections(lemmas, pairs, readings=None):
    """(form, compound) pairs inflecting each hyphenated lemma: "t-shirts", "mothers-in-law".

    AGID has no hyphenated headwords and the analyser looks a compound up whole, so without
    these a listed compound's plural would still be judged part by part. The first and the
    last part each take their AGID forms; the odd nonsense combination is never read. A compound
    form is what its inflected part is: `readings`, when given, gets the part's tags.
    """
    forms_of = {}
    for form, lemma in pairs:
        if form != lemma:
            forms_of.setdefault(lemma, set()).add(form)
    out = set()
    for lemma in lemmas:
        parts = lemma.split("-")
        if len(parts) < 2:
            continue
        for i in {0, len(parts) - 1}:
            for form in forms_of.get(parts[i], ()):
                whole = "-".join(parts[:i] + [form] + parts[i + 1 :])
                out.add((whole, lemma))
                if readings is not None and (form, parts[i]) in readings:
                    readings.setdefault((whole, lemma), set()).update(readings[(form, parts[i])])
    return out


def resolve_forms(pairs, ranks, targets=None, glossed=frozenset()):
    """The single lemma of every form whose lemma is kept.

    A kept lemma always maps to itself: the pack builder finds a lemma's id through this
    table, so a lemma read as another word ("bored" as "bore") would hand its rank and level
    to that word. That is also what keeps a word of its own away from the base AGID wrongly
    gave it (one too rare to keep still resolves as before). Any other form maps to one of
    its kept bases: the one its Wiktionary form-of gloss names ("uses" is "use", not "us"),
    then one with a gloss, then the most frequent, then alphabetically.
    """
    targets = targets or {}
    candidates = {}
    for form, lemma in pairs:
        if lemma in ranks and form not in ranks:
            candidates.setdefault(form, set()).add(lemma)
    for lemma in ranks:
        candidates[lemma] = {lemma}
    return {
        form: min(
            lemmas,
            key=lambda lemma: (lemma not in targets.get(form, ()), lemma not in glossed, ranks[lemma], lemma),
        )
        for form, lemmas in candidates.items()
    }


# A sense left hanging on a coordinator: the Wiktionary line read "(Vieilli) ou Pluie" and
# the parenthetical went, so the gloss opens on "ou". LOWERCASE only — `etcetera` is glossed
# "Et cetera", where the coordinator IS the translation, and a capital is what tells them
# apart across the 15 entries the corpus holds.
_DANGLING_COORDINATOR = re.compile(r"^(?:ou|et)\s+")


def clean_gloss(text, maxlen):
    g = re.sub(r"\s+", " ", text).strip().rstrip(".").strip()
    g = _DANGLING_COORDINATOR.sub("", g)
    if len(g) > maxlen:
        g = g[:maxlen].rstrip()
    return g


def _join_senses(per_entry, maxlen, max_senses):
    """One gloss out of a headword's per-entry sense lists, cut to `maxlen`.

    Senses are taken one-per-entry, round-robin ACROSS the headword's POS entries, so a
    verb meaning appears beside the noun (run -> "Course; Courir") rather than being
    crowded out by the first entry's three.
    """
    picked = []
    depth = 0
    deepest = max(len(e) for e in per_entry)
    while depth < deepest and len(picked) < max_senses:
        for e in per_entry:
            if depth < len(e) and e[depth] not in picked:
                picked.append(e[depth])
                if len(picked) >= max_senses:
                    break
        depth += 1
    joined = "; ".join(picked)
    if len(joined) > maxlen:
        joined = joined[:maxlen].rstrip().rstrip(";").strip()
    return joined


# kaikki's `pos` in Universal Dependencies parts of speech (design D4). A conjunction is
# coordinating when it is one of the seven coordinators, subordinating otherwise; anything that is
# no part of speech of a word (an affix, a phrase, a typographic variant) is `X`.
_KAIKKI_UPOS = {
    "noun": "NOUN",
    "verb": "VERB",
    "adj": "ADJ",
    "adv": "ADV",
    "name": "PROPN",
    "pron": "PRON",
    "prep": "ADP",
    "postp": "ADP",
    "det": "DET",
    "article": "DET",
    "particle": "PART",
    "intj": "INTJ",
    "onomatopoeia": "INTJ",
    "num": "NUM",
    "character": "SYM",
    "symbol": "SYM",
}
_COORDINATORS = frozenset({"and", "or", "but", "nor", "yet", "so", "for"})
# A `;` inside a sense, with the spaces French typography puts around it ("Indigène ; qui…").
_INNER_SEPARATOR = re.compile(r"\s*;\s*")


def kaikki_upos(pos, word):
    """The UD part of speech of a kaikki entry of `word`."""
    if pos == "conj":
        return "CCONJ" if word in _COORDINATORS else "SCONJ"
    return _KAIKKI_UPOS.get(pos, "X")


def _join_senses_by_pos(per_entry, poses, maxlen, max_senses):
    """`_join_senses`, with the senses grouped by part of speech (add-lingua-word-grammar, D4).

    The same senses are picked, by the same round-robin across the headword's entries; they are
    then grouped by the part of speech of the entry each came from, stably, in the order the parts
    of speech first appear, so that the senses of one part of speech are adjacent. A `;` inside a
    sense becomes `,`, so that "; " only ever separates senses. The gloss is cut as `_join_senses`
    cuts it, and the runs — [(part of speech, senses)] — count the senses that survive the cut.
    """
    picked = []  # (sense, part of speech)
    depth = 0
    deepest = max(len(e) for e in per_entry)
    while depth < deepest and len(picked) < max_senses:
        for senses, pos in zip(per_entry, poses):
            if depth < len(senses) and senses[depth] not in [sense for sense, _ in picked]:
                picked.append((senses[depth], pos))
                if len(picked) >= max_senses:
                    break
        depth += 1
    order = list(dict.fromkeys(pos for _, pos in picked))
    grouped = [
        (_INNER_SEPARATOR.sub(", ", sense).strip(" ,"), pos) for first in order for sense, pos in picked if pos == first
    ]
    joined = "; ".join(sense for sense, _ in grouped)
    if len(joined) > maxlen:
        joined = joined[:maxlen].rstrip().rstrip(";").strip()
    kept = len(joined.split("; ")) if joined else 0
    runs = []
    for _, pos in grouped[:kept]:
        if runs and runs[-1][0] == pos:
            runs[-1][1] += 1
        else:
            runs.append([pos, 1])
    return joined, [tuple(run) for run in runs]


def reduce_gloss(path, lemmas, maxlen, per_sense=42, max_senses=3, runs=None):
    """Up to `max_senses` short French glosses per canonical lemma, joined by "; ".

    Form-of senses ("Pluriel de …") are skipped — they are not meanings. The senses that
    make the cut are picked by `_join_senses_by_pos`, which groups them by part of speech;
    `runs`, when given, gets each gloss's runs: lemma -> [(part of speech, senses)].
    """
    entries = {}  # word -> [per-entry [gloss,...]]
    poses = {}  # word -> [per-entry part of speech]
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            word = (d.get("word") or "").strip().lower()
            if not word or word not in lemmas:
                continue
            senses = []
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                if gg and not _is_form_of(sense, gg[0].strip()):
                    g = clean_gloss(gg[0], per_sense)
                    if g:
                        senses.append(g)
            if senses:
                entries.setdefault(word, []).append(senses)
                poses.setdefault(word, []).append(kaikki_upos(d.get("pos") or "", word))

    glosses = {}
    for word, per_entry in entries.items():
        joined, word_runs = _join_senses_by_pos(per_entry, poses[word], maxlen, max_senses)
        if joined:
            glosses[word] = joined
            if runs is not None:
                runs[word] = word_runs
    return glosses


def grammar_rows(readings, forms, lemmas, meanings, targets, cefr):
    """The lines of `grammar.tsv`: (form, lemma, tag, "other"|"-"), sorted.

    Kept: the readings of a dictionary form the pack keeps, for a form the pack can read — one
    `forms.tsv` maps, or a kept lemma itself. The fourth field marks a relation `is_believable`
    finds believable: only such a dictionary form may be named as another reading of a form the
    analysis reads as something else (design D2).
    """
    rows = []
    for (form, lemma), tags in readings.items():
        if lemma not in lemmas or not (form in forms or form in lemmas):
            continue
        for tag in tags:
            other = form != lemma and is_believable(form, lemma, _kind_of(tag), meanings, targets, cefr)
            rows.append((form, lemma, tag, "other" if other else "-"))
    return sorted(rows)


def reduce_expressions(path, maxlen, per_sense=42, max_senses=3):
    """Up to `max_senses` short French glosses per MULTI-WORD headword ("give up").

    `reduce_gloss`'s reduction over the entries it can never reach: it is scoped to the
    kept lemmas, a lemma is one word, so the 33 404 multi-word entries of the source were
    dropped whole. Left out here: an entry whose part of speech is `name` (a proper noun,
    which the card refuses to gloss anyway), a headword outside `_TOKEN`'s character set,
    and an entry no sense survives.

    NOT left out: an expression holding a word the pack's lexicon does not hold. That test
    needs the lexicon, and only the builder has one.
    """
    entries = {}  # headword -> [per-entry [gloss, ...]]
    with open(path, encoding="utf-8", errors="replace") as f:
        for line in f:
            try:
                d = json.loads(line)
            except json.JSONDecodeError:
                continue
            word = re.sub(r"\s+", " ", (d.get("word") or "").strip().lower())
            if " " not in word or not all(_TOKEN.fullmatch(w) for w in word.split(" ")):
                continue
            if (d.get("pos") or "") == "name":
                continue
            senses = []
            for sense in d.get("senses", []):
                gg = sense.get("glosses") or []
                gloss = gg[0].strip() if gg else ""
                if not gloss or _is_form_of(sense, gloss) or _MWE_FORM_OF.match(gloss):
                    continue
                g = clean_gloss(gloss, per_sense)
                if g:
                    senses.append(g)
            if senses:
                entries.setdefault(word, []).append(senses)

    expressions = {}
    for word, per_entry in entries.items():
        joined = _join_senses(per_entry, maxlen, max_senses)
        if joined:
            expressions[word] = joined
    return expressions


NOTICE = """\
Cymbra Lingua data pack — EN->FR attributions.

ESDB (English Speller Database, SCOWLv2, en-wl/wordlist): the inflections.
Copyright 2000-2026 by Kevin Atkinson. Permission to use, copy, modify, distribute, and
sell any part of SCOWLv2, or word lists created from it, is hereby granted without fee,
provided that the above copyright notice appears in all copies and that both the above
copyright notice and this notice appear in supporting documentation. Kevin Atkinson
makes no representations about the suitability of this database for any purpose. It is
provided "as is" without express or implied warranty.

WordNet, used by ESDB for its initial part-of-speech assignment: WordNet 1.6 Copyright
1997 by Princeton University. All rights reserved. Permission to use, copy, modify and
distribute this software and database and its documentation for any purpose and without
fee or royalty is hereby granted, provided that this copyright notice and these
statements, including the disclaimer, appear on all copies. THIS SOFTWARE AND DATABASE
IS PROVIDED "AS IS" AND PRINCETON UNIVERSITY MAKES NO REPRESENTATIONS OR WARRANTIES,
EXPRESS OR IMPLIED. The name of Princeton University or Princeton may not be used in
advertising or publicity pertaining to distribution of the software and/or database.
Title to copyright in this software, database and any associated documentation shall at
all times remain with Princeton University.

wordfreq (English frequency list): CC BY-SA 4.0 (includes SUBTLEX with Brysbaert's
permission).

kaikki.org extract of the French Wiktionary (frwiktionary): CC BY-SA 4.0 + GFDL — the
glosses, the expressions, and the form links completing ESDB's inflections.

CEFR-J: The CEFR-J Wordlist Version 1.5. Compiled by Yukio Tono, Tokyo University of
Foreign Studies. Used for research and commercial purposes with acknowledgement of the
source.

Octanove: Octanove Vocabulary Profile C1/C2 v1.0, Octanove Labs, CC BY-SA 4.0.
"""

# CEFR level order (A1 lowest). A lemma listed at several levels/POS takes the LOWEST
# (earliest-taught) level — the collapse rule from the design.
_LEVEL_RANK = {"A1": 1, "A2": 2, "B1": 3, "B2": 4, "C1": 5, "C2": 6}


def read_cefr(paths):
    """headword -> {(part of speech, level)} across the CEFR lists; missing files are skipped.

    Both CSVs share the columns `headword,pos,CEFR,...`. A headword may be a slash-joined
    set of variants (e.g. "a.m./A.M./am/AM"); each alphabetic variant is kept.
    """
    cefr = {}
    for path in paths:
        if not os.path.exists(path):
            continue
        with open(path, encoding="utf-8", errors="replace", newline="") as f:
            for row in csv.DictReader(f):
                level = (row.get("CEFR") or "").strip().upper()
                if level not in _LEVEL_RANK:
                    continue
                pos = (row.get("pos") or "").strip()
                for part in (row.get("headword") or "").split("/"):
                    w = part.strip().lower()
                    if _TOKEN.fullmatch(w):
                        cefr.setdefault(w, set()).add((pos, level))
    return cefr


def reduce_levels(cefr, lemmas):
    """Lowest CEFR level per kept lemma. Only lemmas we actually keep in the pack are emitted."""
    return {
        w: min((level for _, level in entries), key=_LEVEL_RANK.__getitem__)
        for w, entries in cefr.items()
        if w in lemmas
    }


def write(work, name, text):
    with open(os.path.join(work, name), "w", encoding="utf-8") as f:
        f.write(text)


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--max-lemmas", type=int, default=40000)
    ap.add_argument("--max-gloss-len", type=int, default=80)
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    ap.add_argument(
        "--inflections",
        choices=("esdb", "agid"),
        default="esdb",
        help="ESDB's scowl.txt with kaikki's form-of links, or AGID's infl.txt (retired)",
    )
    a = ap.parse_args()

    from wordfreq import zipf_frequency

    zipf = functools.lru_cache(maxsize=None)(lambda w: zipf_frequency(w, "en"))
    kaikki = os.path.join(a.work, "kaikki-Anglais.jsonl")
    readings = {}  # (form, lemma) -> {tag}; AGID, retired, carries none
    if a.inflections == "agid":
        pairs, relations = parse_agid_relations(os.path.join(a.work, "agid-infl.txt"))
    else:
        pairs, relations = parse_esdb_relations(os.path.join(a.work, "scowl.txt"), readings=readings)
        form_of_relations(kaikki, pairs, relations, readings)
    cefr = read_cefr(
        [
            os.path.join(a.work, "cefrj-vocabulary-profile-1.5.csv"),
            os.path.join(a.work, "octanove-vocabulary-profile-c1c2-1.0.csv"),
        ]
    )
    bases = {base for rels in relations.values() for base, _ in rels}
    meanings, targets = wiktionary_signals(kaikki, set(relations) | bases)
    own = own_words(relations, meanings, targets, cefr, zipf, never=set(analyser_irregulars()))
    inflected = {form for form, lemma in pairs if form != lemma} - own
    ranks = canonical_ranks(inflected, a.max_lemmas)
    orphans = orphaned_forms(inflected, pairs, ranks)
    if orphans:
        inflected -= orphans
        ranks = canonical_ranks(inflected, a.max_lemmas)
    ranked = len(ranks)
    if cefr:
        ranked_forms = resolve_forms(pairs, ranks, targets)
        ranks = append_level_extras(ranks, cefr, inflected, zipf, ranked_forms.get)
    lemmas = set(ranks)

    pairs |= compound_inflections(lemmas, pairs, readings)
    runs = {}
    glosses = reduce_gloss(kaikki, lemmas, a.max_gloss_len, runs=runs)
    expressions = reduce_expressions(kaikki, a.max_gloss_len)
    forms = resolve_forms(pairs, ranks, targets, set(glosses))
    levels = reduce_levels(cefr, lemmas)

    write(a.work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())))
    write(
        a.work,
        "freq.tsv",
        "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: (kv[1], kv[0]))),
    )
    write(a.work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    write(a.work, "level.tsv", "".join(f"{l}\t{lvl}\n" for l, lvl in sorted(levels.items())))
    write(a.work, "mwe.tsv", "".join(f"{w}\t{g}\n" for w, g in sorted(expressions.items())))
    grammar = grammar_rows(readings, forms, lemmas, meanings, targets, cefr)
    write(a.work, "grammar.tsv", "".join("\t".join(row) + "\n" for row in grammar))
    write(
        a.work,
        "senses.tsv",
        "".join(
            f"{w}\t" + "\t".join(f"{pos}:{n}" for pos, n in r) + "\n" for w, r in sorted(runs.items()) if w in glosses
        ),
    )
    write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "en",
            "native": "fr",
            "pack_version": a.pack_version,
            "analyzer_version": "1.1.0",
            "licences": [
                "ESDB / SCOWLv2 (permissive, commercial use allowed; WordNet notice)",
                "wordfreq (CC BY-SA 4.0)",
                "kaikki / frwiktionary (CC BY-SA 4.0 + GFDL)",
                "CEFR-J Wordlist v1.5 (commercial use allowed with attribution)",
                "Octanove Vocabulary Profile C1/C2 v1.0 (CC BY-SA 4.0)",
            ],
        },
        "sources": [
            {"name": "ESDB", "licence": "Permissive"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "CEFR-J", "licence": "Permissive"},
            {"name": "Octanove", "licence": "CcBySa"},
        ],
    }
    write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")

    print(
        f"reduced en-fr: forms={len(forms)} lemmas={len(ranks)} (ranked={ranked}, "
        f"cefr-added={len(ranks) - ranked}) own-words={len(own & lemmas)} "
        f"gloss={len(glosses)} levels={len(levels)} expressions={len(expressions)} "
        f"readings={len(grammar)} runs={sum(len(r) for r in runs.values())}"
    )


if __name__ == "__main__":
    main()
