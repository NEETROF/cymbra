#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Reduce the raw sources of the es-fr pack into its tables (add-lingua-spanish-forms-tables).

Inputs, in `--work`:
- `kaikki-Spanish.jsonl`: kaikki's extract of the English Wiktionary, Spanish section. A lemma's
  entry lists its inflections, each with tags; a form's own entry points at its lemma (`form_of`).
- `es_gsd-ud-train.conllu`, `es_gsd-ud-dev.conllu`: UD Spanish-GSD, read for one thing only — how
  often each form stands for each lemma — to choose one lemma for a form of several (design D2).
- wordfreq `es` (the installed, pinned package): the 60,000 commonest lemmas and which forms are
  attested at all (design D3).
- `kaikki-fr-Espagnol.jsonl`: the French Wiktionary's Spanish entries, the French glosses of words
  and expressions (add-lingua-spanish-gloss-tables).
- `kaikki-es-traductions.jsonl`, `kaikki-fr-traductions.jsonl`: the French translations the Spanish
  Wiktionary lists, and the French Wiktionary's translation tables, the glosses' fallbacks.

Outputs, in `--work`: `forms.tsv`, `freq.tsv`, `grammar.tsv` (add-lingua-spanish-grammar-tables),
`gloss.tsv`, `senses.tsv` and `mwe.tsv` (add-lingua-spanish-gloss-tables), `level.tsv`
(add-lingua-spanish-levels), `NOTICE` and `manifest.json`.

Every rule here is Spanish: the shared rules of `reduce_common.py` are used as they are, and that
module is not edited, so the en-fr tables' rule set does not move.
"""

import argparse
import collections
import json
import os
import re
import sys
import unicodedata

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
import reduce_common as common  # noqa: E402

_HERE = os.path.dirname(os.path.abspath(__file__))
_ANALYSIS_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "mod.rs")

# A Spanish word, whole: letters, accents, ü, ñ, and hyphenated compounds of them.
_TOKEN = re.compile(r"[a-záéíóúüñ]+(?:-[a-záéíóúüñ]+)*")

# The bookkeeping of kaikki's inflection tables, never forms: the class, the template, the table's
# own tags. `combined-form` marks a verb with its clitic pronouns, which the analyser's enclitic
# rule resolves (add-lingua-spanish-analysis D3) — 519,030 forms the table does not need. A combined
# form's own entry says the same through its sense's tags, which name the pronoun
# (`object-third-person`, `object-plural`).
_BOOKKEEPING = frozenset({"table-tags", "inflection-template", "class", "romanization"})
_COMBINED = "combined-form"
_CLITIC_TAG = "object-"

ES = common.Studied(
    code="es",
    token=_TOKEN,
    # Not read by the forms reduction; kept for the shared rules' interface.
    form_of_target=re.compile(r"([a-záéíóúüñ]+)\s*\.?$"),
    coordinators=frozenset({"y", "e", "o", "u", "ni", "pero", "sino"}),
)

# Homographs a person decided (design D2), checked before any count: form → (lemma, reason). The
# reason is part of the rule: a row without one is not a decision. Editing this table changes the
# rules' sha256, so the tables must be reduced again, as for any rule.
#
# A row costs the other lemma its place in the pack: a lemma's own form always reads as itself, so
# sending `vino` to *venir* would take the noun *vino* out altogether, where the counts keep it and
# the card names *venir* as the other reading (add-lingua-spanish-grammar-tables). No homograph is
# worth that yet.
OVERRIDES = {}


def nfc_lower(text):
    return unicodedata.normalize("NFC", (text or "").strip().lower())


def read_kaikki(path, readings=None):
    """Candidate lemmas per form, the words that are lemmas of their own, and the combined forms.

    A form's candidates: the lemma entries listing it among their inflections (bookkeeping left
    out), and the `form_of` targets of its own entry. An entry with a sense that is not a form-of is
    a lemma, and is its own candidate. A verb with its clitics is never a candidate's form: the
    lemma listing it as a combined form, or a sense of its own naming the pronouns, gives it no
    lemma. A string that is also a plain form of another word keeps that one — `principales` is
    *principar* + `les`, and the plural of *principal*.

    With `readings` (a `Readings`), the same pass collects each entry's grammar (`read_readings`).
    """
    candidates = collections.defaultdict(set)
    lemmas = set()
    combined = set()
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            read_entry(entry, candidates, lemmas, combined)
            if readings is not None:
                read_readings(entry, readings)
    return candidates, lemmas, combined


def read_entry(entry, candidates, lemmas, combined):
    """One kaikki entry into the three collections (see `read_kaikki`)."""
    word = nfc_lower(entry.get("word"))
    if not _TOKEN.fullmatch(word):
        return
    senses = entry.get("senses") or []
    only_forms = bool(senses) and all(_is_form_of(sense) for sense in senses)
    if not only_forms:
        lemmas.add(word)
        candidates[word].add(word)
    for inflection in entry.get("forms") or []:
        tags = set(inflection.get("tags") or ())
        form = nfc_lower(inflection.get("form"))
        if _COMBINED in tags:
            combined.add(form)
            continue
        if tags & _BOOKKEEPING or not _TOKEN.fullmatch(form):
            continue
        candidates[form].add(word)
    for sense in senses:
        clitics = _names_clitics(sense)
        for ref in sense.get("form_of") or ():
            target = nfc_lower(ref.get("word") if isinstance(ref, dict) else "")
            if not _TOKEN.fullmatch(target):
                continue
            if clitics:
                combined.add(word)
            else:
                candidates[word].add(target)


def _is_form_of(sense):
    return "form-of" in (sense.get("tags") or ()) or bool(sense.get("form_of"))


def _names_clitics(sense):
    return any(tag.startswith(_CLITIC_TAG) for tag in sense.get("tags") or ())


def read_gsd_counts(paths):
    """How often each (form, lemma) pair stands in the treebank, lowercased, in NFC."""
    counts = collections.Counter()
    for path in paths:
        with open(path, encoding="utf-8") as f:
            for line in f:
                if not line.strip() or line.startswith("#"):
                    continue
                cols = line.rstrip("\n").split("\t")
                # Multi-word tokens (`del` = 1-2) and empty nodes (8.1) carry no lemma of their own.
                if len(cols) < 3 or "-" in cols[0] or "." in cols[0]:
                    continue
                counts[(nfc_lower(cols[1]), nfc_lower(cols[2]))] += 1
    return counts


def choose_lemma(form, options, counts, lemmas, frequency, overrides=OVERRIDES):
    """The one lemma a form maps to (design D2), among `options`.

    An override first, then the treebank's counts, then the form's own entry, then the lemma's
    frequency, then alphabetical order — so the result never depends on the source's order.
    """
    if not options:
        return None
    override = overrides.get(form)
    if override and override[0] in options:
        return override[0]
    return min(
        options,
        key=lambda lemma: (
            -counts.get((form, lemma), 0),
            not (lemma == form and form in lemmas),
            -frequency(lemma),
            lemma,
        ),
    )


def reduce_forms(candidates, lemmas, combined, counts, frequency, ranks_for, max_lemmas, attested):
    """The forms table and the ranks: `form → lemma` over the kept lemmas, and `lemma → rank`.

    A first choice among every candidate says which forms are only inflected; with the combined
    forms that are no word of their own, which the enclitic rule reads, they never become lemmas —
    `hacerlo` is as frequent as a word, and ranked, it would stand in the table for itself. The ranks
    follow; the final choice keeps a form only under a kept lemma, and only when wordfreq attests
    it — a lemma's identity form always stays.
    """
    first = {form: choose_lemma(form, opts, counts, lemmas, frequency) for form, opts in candidates.items()}
    inflected = {form for form, lemma in first.items() if lemma != form} | (combined - lemmas)
    ranks = ranks_for(inflected, max_lemmas)
    forms = {}
    for form, opts in candidates.items():
        kept = {lemma for lemma in opts if lemma in ranks}
        lemma = choose_lemma(form, kept, counts, lemmas, frequency)
        if lemma is None:
            continue
        if lemma == form or attested(form):
            forms[form] = lemma
    for lemma in ranks:
        forms.setdefault(lemma, lemma)
    return forms, ranks


def analyser_version(path=_ANALYSIS_RS):
    """Spanish's analyser version, read from lingua-core so a bump cannot leave the manifest behind
    (the pack would be refused at load, `Pack::load`)."""
    with open(path, encoding="utf-8") as f:
        m = re.search(r'pub const SPANISH_ANALYZER_VERSION: &str = "([^"]+)";', f.read())
    if not m:
        raise SystemExit(f"no SPANISH_ANALYZER_VERSION in {path}")
    return m.group(1)


# — The grammar readings (add-lingua-spanish-grammar-tables) —
#
# Every form kaikki lists carries its grammar as tags (`doy`: first-person, indicative, present,
# singular). They become Universal Dependencies tags, the vocabulary the pack's grammar sections
# read (add-lingua-word-grammar D1): `VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin`.

# kaikki's parts of speech whose forms carry readings, as UPOS.
_UPOS = {"verb": "VERB", "noun": "NOUN", "adj": "ADJ", "det": "DET", "pron": "PRON", "num": "NUM"}
_PERSON = {"first-person": "1", "second-person": "2", "third-person": "3"}
_TENSE = (("present", "Pres"), ("imperfect", "Imp"), ("preterite", "Past"), ("future", "Fut"))
_GENDER = {"masculine": "Masc", "feminine": "Fem", "neuter": "Neut"}
# `es-noun`'s gender argument: the genders a noun takes, and whether its headword is a plural.
_NOUN_GENDER = {
    "m": (("Masc",), False),
    "f": (("Fem",), False),
    "mf": (("Masc", "Fem"), False),
    "mfbysense": (("Masc", "Fem"), False),
    "mfequiv": (("Masc", "Fem"), False),
    "m-p": (("Masc",), True),
    "f-p": (("Fem",), True),
    "mf-p": (("Masc", "Fem"), True),
}
# Rows of an entry's table that are no reading: the bookkeeping, the combined forms, the headword
# repeated, and what kaikki could not parse or marks as a misspelling.
_NOT_A_READING = _BOOKKEEPING | {
    _COMBINED,
    "canonical",
    "error-unrecognized-form",
    "misspelling",
    "pronunciation-spelling",
}


def ud_tag(upos, features):
    """A Universal Dependencies tag: the part of speech, then the features sorted by name."""
    return "|".join([upos, *(f"{name}={value}" for name, value in sorted(features.items()))])


def verb_features(tags):
    """A verb form's features from kaikki's tags, or None when they name no reading of their own.

    - A negative imperative (`no hables`) is the present subjunctive, which kaikki lists as such.
    - The polite imperative (`hable`, *usted*) is grammatically a third person.
    - A participle reads with its agreement; kaikki's bare `participle past` row repeats the
      masculine singular.
    - The conditional is a mood of its own, as UD Spanish writes it, though kaikki also tags it
      indicative.
    """
    if "negative" in tags:
        return None
    if "infinitive" in tags:
        return {"VerbForm": "Inf"}
    if "gerund" in tags:
        return {"VerbForm": "Ger"}
    if "participle" in tags:
        agreement = _agreement(tags)
        return {"VerbForm": "Part", "Tense": "Past", **agreement} if agreement else None
    if "conditional" in tags:
        mood = "Cnd"
    elif "imperative" in tags:
        mood = "Imp"
    elif "subjunctive" in tags:
        mood = "Sub"
    elif "indicative" in tags:
        mood = "Ind"
    else:
        return None
    features = {"VerbForm": "Fin", "Mood": mood}
    if mood in ("Ind", "Sub"):
        tense = next((ud for kaikki, ud in _TENSE if kaikki in tags), None)
        if tense is None:
            return None
        features["Tense"] = tense
    persons = [_PERSON[tag] for tag in tags if tag in _PERSON]
    if len(persons) != 1 or not ({"singular", "plural"} & tags):
        return None
    features["Person"] = persons[0]
    features["Number"] = "Plur" if "plural" in tags else "Sing"
    return features


def _agreement(tags):
    """Gender and number, as a participle's tags state them."""
    out = {}
    genders = [_GENDER[tag] for tag in tags if tag in _GENDER]
    if len(genders) == 1:
        out["Gender"] = genders[0]
    if "plural" in tags:
        out["Number"] = "Plur"
    elif "singular" in tags:
        out["Number"] = "Sing"
    return out


def nominal_features(tags, genders=()):
    """The features of a noun's, adjective's, determiner's or pronoun's form, one set per gender.

    Its number (singular unless kaikki says plural), its degree, and its gender: the form's own, or
    none when it names both (`grandes`), or — for a form kaikki gives no gender, a noun's plural —
    each of the lemma's.
    """
    base = {"Number": "Plur" if "plural" in tags else "Sing"}
    if "superlative" in tags:
        base["Degree"] = "Sup"
    own = [_GENDER[tag] for tag in tags if tag in _GENDER]
    if len(own) == 1:
        return [{**base, "Gender": own[0]}]
    if own or not genders:
        return [base]
    return [{**base, "Gender": gender} for gender in genders]


def noun_genders(entry):
    """A noun's genders, and whether its headword is a plural: `es-noun`'s argument, else the
    genders its senses are tagged with."""
    for head in entry.get("head_templates") or ():
        if head.get("name") == "es-noun":
            known = _NOUN_GENDER.get((head.get("args") or {}).get("1"))
            if known:
                return known
    tags = set()
    for sense in entry.get("senses") or ():
        tags.update(sense.get("tags") or ())
    return tuple(_GENDER[tag] for tag in ("masculine", "feminine") if tag in tags), False


def adjective_agrees(entry):
    """Whether an adjective has a feminine of its own (`rápida`), unlike `grande`."""
    for inflection in entry.get("forms") or ():
        tags = set(inflection.get("tags") or ())
        if "feminine" in tags and "masculine" not in tags:
            return True
    return False


class Readings:
    """The readings kaikki states, by (form, lemma): its lemmas' tables, and its form entries'
    senses for the pairs no table lists — a form entry gives no gender, so a table's reading of
    the same pair wins (`casas`: feminine plural, not just plural)."""

    def __init__(self):
        self.table = collections.defaultdict(set)
        self.senses = collections.defaultdict(set)

    def pairs(self):
        out = {pair: set(tags) for pair, tags in self.senses.items()}
        out.update((pair, set(tags)) for pair, tags in self.table.items())
        return out


def read_readings(entry, readings):
    """One entry's readings into `readings`, word and form lowercased and in NFC.

    A lemma's entry gives its table's forms, and a noun's or an adjective's own form, which carries
    its gender (`casa`: feminine singular). A form's entry gives each form-of sense, toward its
    target — the pronominal forms (`azotarse`, the infinitive of *azotar* with `se`) only a form's
    own entry tags. A combined form's sense, naming the pronoun, gives none.
    """
    upos = _UPOS.get(entry.get("pos"))
    word = nfc_lower(entry.get("word"))
    if upos is None or not _TOKEN.fullmatch(word):
        return
    senses = entry.get("senses") or []
    if senses and all(_is_form_of(sense) for sense in senses):
        for sense in senses:
            if _names_clitics(sense):
                continue
            tags = set(sense.get("tags") or ())
            for ref in sense.get("form_of") or ():
                target = nfc_lower(ref.get("word") if isinstance(ref, dict) else "")
                if _TOKEN.fullmatch(target):
                    readings.senses[(word, target)].update(_tags(upos, tags))
        return
    genders, plural = noun_genders(entry) if upos == "NOUN" else ((), False)
    if upos == "NOUN":
        own = {"Number": "Plur" if plural else "Sing"}
        for features in [{**own, "Gender": gender} for gender in genders] or [own]:
            readings.table[(word, word)].add(ud_tag(upos, features))
    elif upos == "ADJ":
        own = {"Number": "Sing", **({"Gender": "Masc"} if adjective_agrees(entry) else {})}
        readings.table[(word, word)].add(ud_tag(upos, own))
    for inflection in entry.get("forms") or ():
        tags = set(inflection.get("tags") or ())
        form = nfc_lower(inflection.get("form"))
        if tags & _NOT_A_READING or not _TOKEN.fullmatch(form):
            continue
        readings.table[(form, word)].update(_tags(upos, tags, genders))


def _tags(upos, tags, genders=()):
    """The UD tags kaikki's tags name for a form of `upos`; none when they name no reading."""
    if upos == "VERB":
        features = verb_features(tags)
        return [ud_tag(upos, features)] if features else []
    out = []
    for features in nominal_features(tags, genders):
        if "Degree" in features and upos != "ADJ":
            continue
        out.append(ud_tag(upos, features))
    return out


def grammar_rows(readings, forms, ranks):
    """`grammar.tsv`'s rows, sorted: the readings of the forms the table holds, under the lemmas the
    pack keeps. A reading of another lemma than the one its form maps to is marked `other`: every
    relation kaikki's tables state is believable, so the card may name it (add-lingua-word-grammar
    D2) — `fue` read as *ser* is also a form of *ir*."""
    rows = []
    for (form, lemma), tags in readings.pairs().items():
        if form not in forms or lemma not in ranks:
            continue
        mark = "-" if forms[form] == lemma else "other"
        rows.extend(f"{form}\t{lemma}\t{tag}\t{mark}\n" for tag in tags)
    return sorted(rows)


# — The French glosses (add-lingua-spanish-gloss-tables) —
#
# The French Wiktionary's Spanish entries gloss a word or an expression first, through the rules
# every <studied>->FR pair shares (`reduce_common`). Where they say nothing, the glosses fall back
# on translations people wrote: the French words the Spanish Wiktionary lists for a Spanish entry,
# then the French entries whose translation tables list it. A gloss is never English, and never a
# machine translation (the programme's decision D4).

# A word's gloss is paged on its card, as en-fr's (add-lingua-word-grammar): eight whole senses.
WORD_GLOSS = {"maxlen": 800, "per_sense": 300, "max_senses": 8}
EXPRESSION_GLOSS_LEN = 80
# French words a fallback gloss keeps per part of speech.
FALLBACK_WORDS = 3

# Verbal locutions no source glosses (« hay que »), each with its French gloss, written and
# reviewed by a person: expression → gloss. It wins over every source. A gloss here is never
# generated: the programme leaves pack data written by a model undecided, and recommends none at
# launch. Editing it is a rule change, as for OVERRIDES.
LOCUTIONS = {}


def read_translated(path, *, inverted):
    """What a translation file (`pack_sources.derive`) says of Spanish words: word → {UPOS: [French
    word, …]}, in the file's order.

    Direct (the Spanish Wiktionary): a Spanish entry's French translations. Inverted (the French
    Wiktionary): a French entry is a French word for each Spanish word its table lists. A proper
    noun's entry glosses nothing: a place or a first name translated is itself (`alcalá de la
    vega`, `celina` « Céline »).
    """
    out = collections.defaultdict(lambda: collections.defaultdict(list))
    with open(path, encoding="utf-8") as f:
        for line in f:
            try:
                entry = json.loads(line)
            except json.JSONDecodeError:
                continue
            if entry.get("pos") == "name":
                continue
            pairs = (
                [(_spanish(t.get("word")), _french(entry.get("word"))) for t in entry.get("translations") or ()]
                if inverted
                else [(_spanish(entry.get("word")), _french(t.get("word"))) for t in entry.get("translations") or ()]
            )
            for spanish, french in pairs:
                if not spanish or not french:
                    continue
                upos = common.kaikki_upos(entry.get("pos") or "", spanish, studied=ES)
                if french not in out[spanish][upos]:
                    out[spanish][upos].append(french)
    return out


def _spanish(text):
    """A Spanish headword as the pack keys it — lowercase, NFC, single spaces — or None when it is
    not Spanish words."""
    word = re.sub(r"\s+", " ", nfc_lower(text))
    return word if word and all(_TOKEN.fullmatch(part) for part in word.split(" ")) else None


def _french(text):
    """A French translation, tidied: its separators stay out of the gloss's own (`;`)."""
    return re.sub(r"\s+", " ", (text or "").replace(";", ",")).strip(" ,")


def translation_gloss(by_pos, order):
    """A gloss from translations: per part of speech, up to `FALLBACK_WORDS` French words in
    `order`, as one sense — `(gloss, runs)`, the runs `[(UPOS, 1), …]`."""
    senses, runs = [], []
    for upos, words in by_pos.items():
        kept = order(words)[:FALLBACK_WORDS]
        if kept:
            text = ", ".join(kept)
            senses.append(text[:1].upper() + text[1:])
            runs.append((upos, 1))
    return "; ".join(senses), runs


def fallback_glosses(lemmas, glossed, sources):
    """Glosses for the lemmas the French Wiktionary leaves out, from each `(source, order)` in
    turn: `lemma → (gloss, runs)`."""
    out = {}
    for lemma in sorted(set(lemmas) - set(glossed)):
        for source, order in sources:
            by_pos = source.get(lemma)
            if by_pos:
                gloss, runs = translation_gloss(by_pos, order)
                if gloss:
                    out[lemma] = (gloss, runs)
                    break
    return out


def fallback_expressions(expressions, sources):
    """Glosses for the multi-word headwords no Spanish entry of the French Wiktionary glosses, from
    each `(source, order)` in turn: `expression → gloss`. An expression's gloss has no runs, so its
    parts of speech are one."""
    out = {}
    for source, order in sources:
        for headword, by_pos in source.items():
            if " " not in headword or headword in expressions or headword in out:
                continue
            words = []
            for found in by_pos.values():
                words.extend(w for w in found if w not in words)
            gloss, _ = translation_gloss({"X": words}, order)
            if gloss:
                out[headword] = gloss
    return out


def by_french_frequency(frequency):
    """The order of an inverted table's French words: the commonest first, then alphabetical."""
    return lambda words: sorted(words, key=lambda w: (-frequency(w), w))


# — The estimated levels (add-lingua-spanish-levels) —
#
# No Spanish CEFR list can be shipped, so the levels are estimated from frequency (the programme's
# decision D1) and the pack says so. The commonest lemmas, in rank order, take the sizes of
# English's CEFR levels: measured on English, giving its 8,302 CEFR lemmas their levels this way by
# their own ranks agrees with the lists for 39.8 % of them, and within one level for 82.6 %. The
# sizes are en-fr's on its 2026-09-26 tables, kept here: an English update must not move the
# Spanish levels unannounced.
ENGLISH_BANDS = (("A1", 1020), ("A2", 1158), ("B1", 2015), ("B2", 2347), ("C1", 886), ("C2", 876))


def estimated_levels(ranks, glosses, runs, bands=ENGLISH_BANDS):
    """`lemma → level`: in rank order, each band's size to the lemmas a CEFR list would hold — those
    with a French gloss that is not only a proper noun's (`the`, `twitter`, `madrid` take none)."""
    eligible = (
        lemma
        for lemma in sorted(ranks, key=lambda lemma: (ranks[lemma], lemma))
        if lemma in glosses and {pos for pos, _ in runs.get(lemma, ())} != {"PROPN"}
    )
    out = {}
    for level, size in bands:
        for _ in range(size):
            lemma = next(eligible, None)
            if lemma is None:
                return out
            out[lemma] = level
    return out


NOTICE = """Cymbra Lingua data pack — ES->FR attributions.

kaikki.org extract of the English Wiktionary (enwiktionary), Spanish section: CC BY-SA 4.0 + GFDL —
the forms, their lemmas and their grammar.

wordfreq (Spanish frequency list), by Robyn Speer (https://github.com/rspeer/wordfreq): data under
CC BY-SA 4.0 — the commonest lemmas and which forms are attested.

UD Spanish-GSD, the Universal Dependencies Spanish-GSD treebank
(https://github.com/UniversalDependencies/UD_Spanish-GSD): CC BY-SA 4.0 — how often a form stands
for each of its lemmas, to choose one.

kaikki.org extract of the French Wiktionary (frwiktionary): CC BY-SA 4.0 + GFDL — the French
glosses of Spanish words and expressions, from its Spanish entries, and from its French entries'
translation tables where those say nothing.

kaikki.org extract of the Spanish Wiktionary (eswiktionary): CC BY-SA 4.0 + GFDL — the French
translations its Spanish entries list, where the French Wiktionary has no gloss.
"""


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("--work", required=True)
    ap.add_argument("--max-lemmas", type=int, default=60000)
    ap.add_argument("--built-at", required=True, help="yyyy-mm-dd (source snapshot date)")
    ap.add_argument("--pack-version", required=True)
    a = ap.parse_args()

    from wordfreq import zipf_frequency

    zipf = {}

    def frequency(word, lang="es"):
        if (word, lang) not in zipf:
            zipf[(word, lang)] = zipf_frequency(word, lang)
        return zipf[(word, lang)]

    readings = Readings()
    candidates, lemmas, combined = read_kaikki(os.path.join(a.work, "kaikki-Spanish.jsonl"), readings)
    counts = read_gsd_counts(
        [os.path.join(a.work, "es_gsd-ud-train.conllu"), os.path.join(a.work, "es_gsd-ud-dev.conllu")]
    )
    forms, ranks = reduce_forms(
        candidates,
        lemmas,
        combined,
        counts,
        frequency,
        lambda inflected, want: {
            unicodedata.normalize("NFC", lemma): rank
            for lemma, rank in common.canonical_ranks(inflected, want, studied=ES).items()
        },
        a.max_lemmas,
        lambda form: frequency(form) > 0,
    )

    common.write(a.work, "forms.tsv", "".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())))
    common.write(
        a.work,
        "freq.tsv",
        "".join(f"{l}\t{r}\n" for l, r in sorted(ranks.items(), key=lambda kv: (kv[1], kv[0]))),
    )
    grammar = grammar_rows(readings, forms, ranks)
    common.write(a.work, "grammar.tsv", "".join(grammar))

    spanish_entries = os.path.join(a.work, "kaikki-fr-Espagnol.jsonl")
    runs = {}
    glosses = common.reduce_gloss(spanish_entries, set(ranks), **WORD_GLOSS, runs=runs, studied=ES)
    expressions = common.reduce_expressions(spanish_entries, EXPRESSION_GLOSS_LEN, studied=ES)
    direct = read_translated(os.path.join(a.work, "kaikki-es-traductions.jsonl"), inverted=False)
    inverted = read_translated(os.path.join(a.work, "kaikki-fr-traductions.jsonl"), inverted=True)
    sources = [(direct, list), (inverted, by_french_frequency(lambda w: frequency(w, "fr")))]
    primary = len(glosses)
    for lemma, (gloss, gloss_runs) in fallback_glosses(ranks, glosses, sources).items():
        glosses[lemma], runs[lemma] = gloss, gloss_runs
    expressions.update(fallback_expressions(expressions, sources))
    expressions.update(LOCUTIONS)
    common.write(a.work, "gloss.tsv", "".join(f"{l}\t{g}\n" for l, g in sorted(glosses.items())))
    common.write(
        a.work,
        "senses.tsv",
        "".join(
            f"{w}\t" + "\t".join(f"{pos}:{n}" for pos, n in r) + "\n" for w, r in sorted(runs.items()) if w in glosses
        ),
    )
    common.write(a.work, "mwe.tsv", "".join(f"{w}\t{g}\n" for w, g in sorted(expressions.items())))
    levels = estimated_levels(ranks, glosses, runs)
    common.write(a.work, "level.tsv", "".join(f"{l}\t{lvl}\n" for l, lvl in sorted(levels.items())))
    common.write(a.work, "NOTICE", NOTICE)
    manifest = {
        "meta": {
            "studied": "es",
            "native": "fr",
            "pack_version": a.pack_version,
            "analyzer_version": analyser_version(),
            # Derived from frequency, not taken from a CEFR list: the extension says so.
            "levels_estimated": True,
            "licences": [
                "kaikki / enwiktionary, frwiktionary, eswiktionary (CC BY-SA 4.0 + GFDL)",
                "wordfreq (CC BY-SA 4.0)",
                "UD Spanish-GSD (CC BY-SA 4.0)",
            ],
        },
        "sources": [
            {"name": "kaikki", "licence": "CcBySa"},
            {"name": "wordfreq", "licence": "CcBySa"},
            {"name": "UD Spanish-GSD", "licence": "CcBySa"},
        ],
    }
    common.write(a.work, "manifest.json", json.dumps(manifest, indent=2, ensure_ascii=False) + "\n")
    print(
        f"reduced es-fr: forms={len(forms)} lemmas={len(ranks)} readings={len(grammar)} "
        f"glosses={len(glosses)} (French Wiktionary {primary}) expressions={len(expressions)} levels={len(levels)}",
        file=sys.stderr,
    )


if __name__ == "__main__":
    main()
