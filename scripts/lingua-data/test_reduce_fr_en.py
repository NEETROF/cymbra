# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the FR->EN reducer's French rules (add-lingua-french-forms-tables), on entries
shaped as the English Wiktionary's dump writes them — no download, and wordfreq doubled.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
import contextlib
import importlib.util
import inspect
import io
import json
import os
import sys
import tempfile
import types
import unittest
import unittest.mock as mock

_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("reduce_fr_en", os.path.join(_HERE, "reduce-fr-en.py"))
red = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(red)


def entry(word, pos="noun", forms=(), senses=None):
    """A kaikki entry: a word, its inflections `(form, [tags])`, and senses (a meaning by default)."""
    return {
        "word": word,
        "pos": pos,
        "forms": [{"form": f, "tags": list(t)} for f, t in forms],
        "senses": senses if senses is not None else [{"glosses": ["a meaning"]}],
    }


def form_of(word, *targets, pos="verb", tags=("form-of",)):
    """An entry that is only a form of `targets`, as the dump writes one (`étés`: « plural of été »)."""
    return entry(word, pos=pos, senses=[{"tags": list(tags), "form_of": [{"word": t} for t in targets]}])


def alt_of(word, *targets, pos, gloss, tags=("alt-of",)):
    """An entry pointing at `targets` by `alt_of`, as the dump writes an elided piece, a contraction
    or a spelling (`l'`: « apocopic form of le, la »)."""
    return entry(word, pos=pos, senses=[{"glosses": [gloss], "tags": list(tags), "alt_of": [{"word": t} for t in targets]}])


PRESENT_3S = ["indicative", "present", "singular", "third-person"]
PARTICIPLE = ["participle", "past"]
# What a verb's conjugation table opens with, in the dump: its bookkeeping and the compound tenses.
TABLE_HEAD = [
    ("no-table-tags", ["table-tags"]),
    ("fr-conj-auto", ["inflection-template"]),
    ("avoir + past participle", ["infinitive", "multiword-construction"]),
    ("ayant", ["gerund", "multiword-construction", "participle", "present"]),
]

# A small French section: every rule of design D3–D7 has an entry here.
ENTRIES = [
    # D3 — a form of a form, along one part of speech.
    entry("être", pos="verb", forms=[*TABLE_HEAD, ("est", PRESENT_3S), ("été", PARTICIPLE)]),
    entry("été", forms=[("étés", ["plural"])], senses=[{"glosses": ["summer"], "tags": ["masculine"]}]),
    form_of("été", "être"),
    form_of("étés", "été", pos="noun", tags=("form-of", "masculine", "plural")),
    entry("diriger", pos="verb", forms=[*TABLE_HEAD, ("dirige", PRESENT_3S), ("dirigé", PARTICIPLE), ("dirigeassions", ["imperfect", "plural", "subjunctive", "first-person"])]),
    entry(
        "dirigé",
        pos="verb",
        forms=[("dirigée", ["feminine"]), ("dirigés", ["masculine", "plural"]), ("dirigées", ["feminine", "plural"])],
        senses=[{"glosses": ["past participle of diriger"], "tags": ["form-of", "participle", "past"], "form_of": [{"word": "diriger"}]}],
    ),
    form_of("dirigée", "dirigé", tags=("feminine", "form-of", "participle", "singular")),
    # D5 — a noun a verb's form takes (M8).
    entry("porter", pos="verb", forms=[*TABLE_HEAD, ("porte", PRESENT_3S), ("portes", ["indicative", "present", "second-person", "singular"])]),
    entry("porte", senses=[{"glosses": ["door"], "tags": ["feminine"]}], forms=[("portes", ["plural"])]),
    form_of("porte", "porter"),
    # D6 — a word no form reaches, and one another of its forms does.
    entry("tenir", pos="verb", forms=[*TABLE_HEAD, ("tient", PRESENT_3S), ("tenu", PARTICIPLE)]),
    entry("tenu", pos="verb", forms=[("tenue", ["feminine"]), ("tenues", ["feminine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "tenir"}]}]),
    entry("tenue", forms=[("tenues", ["plural"])], senses=[{"glosses": ["maintenance, upkeep"], "tags": ["feminine"]}]),
    form_of("tenue", "tenu", tags=("feminine", "form-of", "participle", "singular")),
    form_of("tenues", "tenu", tags=("feminine", "form-of", "participle", "plural")),
    entry("donner", pos="verb", forms=[*TABLE_HEAD, ("donne", PRESENT_3S), ("donné", PARTICIPLE)]),
    entry("donné", pos="verb", forms=[("donnée", ["feminine"]), ("données", ["feminine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "donner"}]}]),
    entry("donnée", forms=[("données", ["plural"])], senses=[{"glosses": ["datum"], "tags": ["feminine"]}]),
    form_of("donnée", "donné", tags=("feminine", "form-of", "participle", "singular")),
    form_of("données", "donné", tags=("feminine", "form-of", "participle", "plural")),
    form_of("données", "donnée", pos="noun", tags=("form-of", "feminine", "plural")),
    # D4 — the elided pieces, as the dump writes them: alt_of several words, or no pointer at all.
    alt_of("l'", "le", "la", pos="article", gloss="apocopic form of le, la: the", tags=("abbreviation", "alt-of", "apocopic")),
    entry("s'", pos="pron", forms=[("s’", ["canonical"])], senses=[{"glosses": ["Elision of se before a word beginning with a vowel."]}]),
    alt_of("qu'", "que", "qui", pos="conj", gloss="apocopic form of que", tags=("abbreviation", "alt-of", "apocopic")),
    alt_of("jusqu'", "jusque", pos="prep", gloss="apocopic form of jusque", tags=("abbreviation", "alt-of", "apocopic")),
    entry("le", pos="article", forms=[("la", ["feminine"]), ("les", ["plural"]), ("l'", ["alternative"])]),
    entry("se", pos="pron", forms=[("s'", ["elided"])]),
    entry("que", pos="conj"),
    entry("qui", pos="pron"),
    entry("jusque", pos="prep"),
    entry("on", pos="pron"),
    # A plain word beginning with a piece, and a hyphenated run beginning with one.
    entry("d'abord", pos="adv"),
    alt_of("l'on", "on", pos="pron", gloss="alternative form of on", tags=("alt-of", "alternative")),
    entry("c'est", pos="phrase"),
    entry("jusqu'à", pos="prep"),
    entry("c'est-à-dire", pos="phrase", forms=[("c.-à-d.", ["alternative", "abbreviation"])]),
    # A word whose inner apostrophe is no piece's.
    entry("aujourd'hui", pos="adv"),
    # D4 — the contracted articles.
    entry("à", pos="prep"),
    alt_of("au", "à", pos="contraction", gloss="contraction of à + le", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("aux", "à", pos="contraction", gloss="contraction of à + les", tags=("abbreviation", "alt-of", "contraction")),
    entry("de", pos="prep"),
    entry("un", pos="article", forms=[("une", ["feminine"]), ("des", ["plural"])]),
    entry("du", pos="article", forms=[("de la", ["feminine", "singular"]), ("des", ["plural"])]),
    alt_of("du", "de", pos="contraction", gloss="contraction of de + le", tags=("abbreviation", "alt-of", "contraction")),
    form_of("des", "un", "une", "du", "de la", pos="article", tags=("form-of", "plural")),
    # As the section writes it: the contraction carries its meaning, which fr-en reads
    # (refine-lingua-fr-en-glosses D4), so `des` is glossed, and one of French's dictionary words.
    alt_of("des", "de", pos="contraction", gloss="contraction of de + les, literally “of the, from the, some”", tags=("abbreviation", "alt-of", "contraction")),
    # D4 — words ending in a pronoun: nouns kept whole, a verb and its pronouns left to the split.
    entry("rendez-vous", senses=[{"glosses": ["appointment"], "tags": ["invariable", "masculine"]}]),
    form_of("rendez-vous", "se rendre", tags=("form-of", "imperative", "plural", "second-person")),
    entry("qu'en-dira-t-on", senses=[{"glosses": ["gossip"], "tags": ["invariable", "masculine"]}]),
    entry("malgré-nous"),
    entry("est-il", pos="phrase", senses=[{"glosses": ["is he"]}]),
    entry("allez-y", pos="phrase", senses=[{"glosses": ["go ahead; go on"]}]),
    entry("souvenir", pos="verb", forms=[*TABLE_HEAD, ("souviens", PRESENT_3S), ("souviens-toi", ["imperative", "reflexive", "second-person", "singular"]), ("souvenons-nous", ["imperative", "plural", "reflexive", "first-person"]), ("sois-t'en", ["imperative", "reflexive"])]),
    # D6 — hyphenated words, ranked by GSD's evidence.
    entry("peut-être", pos="adv"),
    entry("fait-tout"),
    # D7 — spelling variants.
    entry("cœur", forms=[("cœurs", ["plural"]), ("coeur", ["alternative"])]),
    entry("coeur", pos="noun", forms=[("coeurs", ["plural"])], senses=[{"glosses": ["nonstandard spelling of cœur"], "tags": ["alt-of", "masculine", "nonstandard"], "alt_of": [{"word": "cœur"}]}]),
    entry("connaître", pos="verb", forms=[*TABLE_HEAD, ("connaît", PRESENT_3S)]),
    entry("connaitre", pos="verb", forms=[*TABLE_HEAD, ("connait", PRESENT_3S)], senses=[{"glosses": ["post-1990 spelling of connaître"]}]),
    form_of("connait", "connaitre", tags=("form-of", "indicative", "present", "singular", "third-person")),
    # Names: the commoner reading wins.
    entry("ce", pos="det", forms=[("cette", ["feminine"])]),
    entry("cette", pos="name", senses=[{"glosses": ["Sète, a town"]}]),
    entry("pari", forms=[("paris", ["plural"])]),
    entry("paris", pos="name", senses=[{"glosses": ["Paris, the capital"]}]),
    # D3 — a phrase as a form-of target: its first word, and a first word that is no form refused.
    form_of("bel", "beau used before a masculine noun that starts with a vowel sound", pos="adj"),
    entry("beau", pos="adj"),
    form_of("dabord", "d'abord et avant tout", pos="adv"),
]

# wordfreq's Zipf frequencies, doubled. A hyphenated word's is wordfreq's estimate from its parts.
FREQ = {
    "de": 7.7, "le": 7.6, "la": 7.4, "l": 7.3, "à": 7.2, "les": 7.2, "des": 7.1, "est": 7.0, "un": 7.0,
    "du": 6.9, "que": 6.9, "au": 6.8, "qu": 6.7, "d": 6.6, "se": 6.6, "ce": 6.5, "cette": 6.5, "on": 6.5,
    "être": 6.4, "été": 6.3, "aux": 6.2, "une": 6.6, "qui": 6.8, "s": 6.0, "jusqu": 5.0, "paris": 5.7,
    "porter": 5.2, "porte": 5.1, "portes": 4.5, "donner": 5.5, "donne": 5.0, "données": 5.1, "donnée": 4.5,
    "donné": 4.6, "tenir": 5.6, "tient": 5.0, "tenu": 4.6, "tenue": 4.6, "tenues": 3.9, "diriger": 4.6,
    "dirige": 4.1, "dirigé": 4.2, "dirigée": 4.0, "dirigés": 3.6, "dirigées": 3.5, "cœur": 5.3,
    "cœurs": 4.0, "coeur": 4.0, "coeurs": 3.0, "connaître": 5.2, "connaît": 4.8, "connait": 3.5,
    "souvenir": 4.9, "souviens": 3.8, "jusque": 4.6, "aujourd'hui": 5.9, "d'abord": 5.5, "c'est": 6.0,
    "pari": 4.1, "beau": 5.3, "bel": 4.5, "étés": 3.5, "connaitre": 3.4,
    # wordfreq's estimates for hyphenated words, which its list never holds.
    "peut-être": 5.4, "fait-tout": 5.6, "c'est-à-dire": 5.0, "rendez-vous": 4.3, "malgré-nous": 2.0,
    "qu'en-dira-t-on": 2.1, "est-il": 5.9, "allez-y": 4.0, "souviens-toi": 3.5, "souvenons-nous": 3.0,
    "sois-t'en": 2.0,
}
# wordfreq's list, by frequency: no hyphenated word.
TOP = [w for w in sorted(FREQ, key=lambda w: (-FREQ[w], w)) if "-" not in w]


def frequency(word):
    return FREQ.get(word, 0.0)


def top_n(n):
    return TOP[:n]


def conllu(rows):
    """A treebank's lines: one word per (form, lemma, count), as UD writes it."""
    lines, n = ["# sent_id = 1"], 0
    for form, lemma, count in rows:
        for _ in range(count):
            n += 1
            lines.append(f"{n}\t{form}\t{lemma}\tX\t_\t_\t0\troot\t_\t_")
    return "\n".join(lines) + "\n"


# GSD's counts, as its training sections give them: `porte` 39 times *porter* and 23 times the noun.
GSD = [
    ("porte", "porter", 39),
    ("porte", "porte", 23),
    ("portes", "porte", 2),
    ("été", "être", 830),
    ("été", "été", 51),
    ("tenue", "tenir", 5),
    ("tenue", "tenue", 2),
    ("tenues", "tenir", 3),
    ("tenues", "tenue", 1),
    ("donnée", "donner", 3),
    ("donnée", "donnée", 1),
    ("données", "donnée", 40),
    ("données", "donner", 2),
    ("des", "un", 1730),
    ("c'est-à-dire", "c'est-à-dire", 1),
    ("peut-être", "peut-être", 30),
    ("rendez-vous", "rendez-vous", 3),
    ("l'", "le", 2000),
]


def lexicon(*entries):
    lex = red.Lexicon()
    for e in entries or ENTRIES:
        lex.read(e)
    return lex


def counts():
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "train.conllu")
        with open(path, "w", encoding="utf-8") as f:
            f.write(conllu(GSD))
        return red.read_gsd_counts([path])


def reduce(max_lemmas=200):
    """The fixture section reduced, as `main` reduces the real one."""
    lex = lexicon()
    gsd = counts()
    red.name_or_word(lex.candidates, lex.names, frequency)
    red.own_words(lex)
    forms, ranks = red.reduce_forms(lex, gsd, red.gsd_zipf(gsd), frequency, top_n, max_lemmas)
    return lex, forms, ranks


class ReadingKaikki(unittest.TestCase):
    def test_a_lemma_lists_its_inflections_but_the_bookkeeping_and_the_doubtful(self):
        lex = lexicon()
        self.assertIn("diriger", lex.lemmas)
        self.assertEqual(lex.candidates["dirige"], {"diriger"})
        # The table's bookkeeping, and `ayant`, which the dump tags a multi-word construction (the
        # compound tenses' auxiliary), are no forms of the verb.
        for bookkeeping in ("no-table-tags", "fr-conj-auto", "ayant"):
            self.assertNotIn(bookkeeping, lex.candidates)
        # An alternative the lemma lists is no inflection: `coeur` is cœur's by its own entry.
        self.assertEqual(lexicon(entry("cœur", forms=[("coeur", ["alternative"])])).candidates.get("coeur"), None)

    def test_a_form_of_target_is_its_first_word_and_a_multi_word_target_that_is_no_form_is_refused(self):
        lex = lexicon()
        # kaikki writes `bel`'s target as a phrase: its first word is the lemma.
        self.assertEqual(lex.candidates["bel"], {"beau"})
        # `d'abord et avant tout`: its first word begins with an elided piece, which the pre-pass
        # splits, so it is no form and no candidate.
        self.assertNotIn("dabord", lex.candidates)
        # `se rendre`: the verb's first word.
        self.assertEqual(lex.candidates["rendez-vous"], {"rendez-vous", "se"})

    def test_the_elided_pieces_by_the_table(self):
        lex = lexicon()
        # The dump points `l'` at le and la, `qu'` at que and qui, `s'` at nothing: the table decides.
        self.assertEqual(lex.candidates["l'"], {"le"})
        self.assertEqual(lex.candidates["qu'"], {"que"})
        self.assertEqual(lex.candidates["s'"], {"se"})
        self.assertEqual(lex.candidates["jusqu'"], {"jusque"})
        self.assertNotIn("l'", lex.lemmas)
        # A table listing a piece among its forms adds no reading to it.
        self.assertEqual(lexicon(entry("la", pos="article", forms=[("l'", ["elided"])])).candidates.get("l'"), None)

    def test_every_piece_maps_to_the_word_the_pre_pass_reads_outside_its_special_cases(self):
        # add-lingua-french-tokenisation D3: `s'` is `si` before il/ils alone, `m'`/`t'` are
        # `moi`/`toi` right after a hyphen alone.
        want = {
            "c'": "ce", "ç'": "ça", "d'": "de", "j'": "je", "l'": "le", "m'": "me", "n'": "ne", "qu'": "que",
            "s'": "se", "t'": "te", "jusqu'": "jusque", "lorsqu'": "lorsque", "puisqu'": "puisque",
            "quoiqu'": "quoique",
        }
        self.assertEqual({piece: lemma for piece, (lemma, _) in red.ELISIONS.items()}, want)
        for piece, (lemma, reason) in red.ELISIONS.items():
            self.assertTrue(reason, piece)

    def test_no_plain_word_beginning_with_a_piece(self):
        lex = lexicon()
        for split in ("d'abord", "l'on", "c'est", "jusqu'à"):
            self.assertNotIn(split, lex.candidates)
            self.assertFalse(red.is_form(split), split)
        # A hyphenated run is read whole first; a word whose apostrophe is no piece's stays whole.
        for whole in ("c'est-à-dire", "qu'en-dira-t-on", "aujourd'hui", "presqu'île", "main-d'œuvre"):
            self.assertTrue(red.is_form(whole), whole)
        self.assertIn("c'est-à-dire", lex.lemmas)
        self.assertNotIn("c.-à-d.", lex.candidates)

    def test_words_are_lowercased_composed_and_their_apostrophe_straight(self):
        decomposed = "été"  # `été`, decomposed
        lex = lexicon(entry("Paris", pos="name"), form_of(decomposed, "être"), entry("aujourd’hui", pos="adv"))
        self.assertIn("paris", lex.candidates)
        self.assertEqual(lex.candidates["été"], {"être"})
        self.assertIn("aujourd'hui", lex.lemmas)

    def test_a_spelling_variant_is_a_form_of_the_word_it_spells(self):
        lex = lexicon()
        self.assertEqual(lex.candidates["coeur"], {"cœur"})
        self.assertNotIn("coeur", lex.lemmas)
        self.assertEqual(lex.candidates["coeurs"], {"coeur"})
        self.assertEqual(lex.candidates["connaitre"], {"connaître"})
        self.assertEqual(lex.candidates["connait"], {"connaitre"})
        self.assertIsNone(red.spelling_target({"glosses": ["heart"]}, "cœur"))
        # `clef` also glosses the musical clef: a word of its own, not only a spelling.
        clef = entry(
            "clef",
            senses=[{"glosses": ["alternative spelling of clé"], "alt_of": [{"word": "clé"}]}, {"glosses": ["clef (music)"]}],
        )
        self.assertIn("clef", lexicon(clef).lemmas)

    def test_a_gender_or_number_marker_is_no_form(self):
        # The dump writes a name's gender among its forms, untagged (`m` under Paris), or a number
        # marker tagged plural (`p` under Saintes): no form of the name, or « M. » reads as Paris.
        lex = lexicon(
            entry("Paris", pos="name", forms=[("m", [])], senses=[{"glosses": ["Paris"]}]),
            entry("Saintes", pos="name", forms=[("p", ["plural"])], senses=[{"glosses": ["Saintes"]}]),
            entry("M", pos="character", forms=[("m", ["lowercase"])], senses=[{"glosses": ["The letter M."]}]),
            entry("Angora", pos="name", forms=[("f", [])], senses=[{"glosses": ["Ankara"]}]),
        )
        self.assertEqual(lex.candidates["m"], {"m"}, "the letter's own case pair is the word itself")
        self.assertNotIn("p", lex.candidates)
        self.assertNotIn("f", lex.candidates)
        self.assertNotIn("paris", lex.candidates["m"])

    def test_names_are_the_words_whose_only_lemma_entries_are_names(self):
        lex = lexicon()
        self.assertIn("paris", lex.names)
        self.assertIn("cette", lex.names)
        self.assertNotIn("ce", lex.names)


class ChoosingOneLemma(unittest.TestCase):
    freq = staticmethod(lambda w: {"porter": 5.2, "porte": 5.1, "être": 6.4, "été": 6.3}.get(w, 0))

    def choose(self, form, options, counts=None, lemmas=frozenset(), overrides=None):
        return red.choose_lemma(form, set(options), counts or {}, set(lemmas), self.freq, overrides or {})

    def test_the_treebank_decides_first(self):
        self.assertEqual(self.choose("porte", ["porte", "porter"], {("porte", "porter"): 39, ("porte", "porte"): 23}, {"porte"}), "porter")

    def test_an_override_comes_before_the_counts(self):
        overrides = {"porte": ("porte", "a reason")}
        self.assertEqual(self.choose("porte", ["porte", "porter"], {("porte", "porter"): 39}, {"porte"}, overrides), "porte")
        # An override naming a lemma the form cannot have is no decision.
        self.assertEqual(self.choose("porte", ["porter"], overrides={"porte": ("x", "?")}), "porter")

    def test_without_counts_the_forms_own_entry_wins(self):
        self.assertEqual(self.choose("porte", ["porte", "porter"], lemmas={"porte"}), "porte")

    def test_then_the_commoner_lemma_then_the_alphabet(self):
        self.assertEqual(self.choose("été", ["être", "été"]), "être")
        self.assertEqual(self.choose("xx", ["b", "a"]), "a")

    def test_every_override_has_its_reason(self):
        for form, (lemma, reason) in red.OVERRIDES.items():
            self.assertTrue(reason and len(reason) > 20, form)

    def test_a_copy_error_of_the_source_is_overridden(self):
        # The dump's verb entry `fatiguée` reads « feminine singular of parlé »: its form of a form
        # reaches parler, the commoner verb, as well as fatiguer. The reviewed row reads it as fatiguer.
        entries = [
            entry("fatiguer", pos="verb", forms=[*TABLE_HEAD, ("fatigue", PRESENT_3S), ("fatigué", PARTICIPLE)]),
            entry("fatigué", pos="verb", forms=[("fatiguée", ["feminine"]), ("fatiguées", ["feminine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "fatiguer"}]}]),
            entry("fatigué", pos="adj", forms=[("fatiguée", ["feminine"])], senses=[{"glosses": ["tired"]}]),
            form_of("fatiguée", "fatigué", pos="adj", tags=("feminine", "form-of", "singular")),
            entry("fatiguée", pos="verb", forms=[("fatiguées", ["plural"])], senses=[{"glosses": ["feminine singular of parlé"], "tags": ["feminine", "form-of", "participle", "singular"], "form_of": [{"word": "parlé"}]}]),
            entry("parler", pos="verb", forms=[*TABLE_HEAD, ("parle", PRESENT_3S), ("parlé", PARTICIPLE)]),
            form_of("parlé", "parler", tags=("form-of", "participle", "past")),
        ]
        lex = lexicon(*entries)
        freq = {"parler": 5.5, "fatiguer": 3.9, "fatiguée": 3.9, "fatigué": 4.2, "fatiguées": 2.5, "parle": 5.0, "fatigue": 4.0, "parlé": 4.6}.get

        # GSD reads the participle `fatigué` as fatiguer, as UD lemmatises participles.
        counts = {("fatigué", "fatiguer"): 3}

        def reduced(overrides):
            top = lambda n: ["parler", "parle", "fatiguer"][:n]  # noqa: E731
            return red.reduce_forms(lex, counts, {}, lambda w: freq(w, 0.0), top, 10, overrides)[0]

        self.assertEqual(reduced({})["fatiguée"], "parler", "the source's error, read by frequency")
        # Its plural, which the erroneous entry lists, follows it to parler too.
        self.assertEqual(reduced({})["fatiguées"], "parler")
        forms = reduced(red.OVERRIDES)
        self.assertEqual(forms["fatiguée"], "fatiguer")
        self.assertEqual(forms["fatigué"], "fatiguer")
        self.assertEqual(forms["fatiguées"], "fatiguer")
        self.assertEqual(red.OVERRIDES["fatiguée"][0], "fatiguer")
        self.assertEqual(red.OVERRIDES["bridée"][0], "bridé")
        self.assertEqual(red.OVERRIDES["quis"][0], "quérir")

    def test_a_name_and_a_commoner_word(self):
        lex = lexicon()
        red.name_or_word(lex.candidates, lex.names, frequency)
        # `cette` is ce's feminine before the town (6.5 against 6.5: the name is no commoner).
        self.assertEqual(lex.candidates["cette"], {"ce"})
        # `paris` stays the city: 5.7 against pari's 4.1.
        self.assertEqual(lex.candidates["paris"], {"paris"})


class Reducing(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.lex, cls.forms, cls.ranks = reduce()

    def test_spec_scenario_a_participle_s_agreement(self):
        # `dirigée` is only the feminine of the participle `dirigé`, itself only diriger's.
        self.assertEqual(self.forms["dirigée"], "diriger")
        self.assertEqual(self.forms["dirigées"], "diriger")
        self.assertEqual(self.forms["dirigé"], "diriger")

    def test_spec_scenario_a_participle_filed_under_a_noun_s_spelling(self):
        # The participle `cité` is spelt like the noun *cité* (city), a ranked lemma; `tu` like the
        # pronoun; `compromis` like the noun, and, invariable, lists itself among its own forms. A
        # chain follows the participle's verb entry to its verb, past the lemma of another part of
        # speech its spelling is (D3). `quis`, quérir's participle, is also « masculine plural of
        # qui » in a verb entry, and `qui` has no verb entry: the chain from `quise` does not go there.
        entries = [
            entry("citer", pos="verb", forms=[*TABLE_HEAD, ("cite", PRESENT_3S), ("cité", PARTICIPLE)]),
            entry("cité", forms=[("cités", ["plural"])], senses=[{"glosses": ["city"], "tags": ["feminine"]}]),
            entry("cité", pos="verb", forms=[("citée", ["feminine"]), ("cités", ["masculine", "plural"]), ("citées", ["feminine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "citer"}]}]),
            form_of("citée", "cité", tags=("feminine", "form-of", "participle", "singular")),
            entry("tu", pos="pron", forms=[("vous", ["plural"])], senses=[{"glosses": ["you (singular)"]}]),
            entry("tu", pos="verb", forms=[("tue", ["feminine"]), ("tus", ["masculine", "plural"]), ("tues", ["feminine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "taire"}]}]),
            entry("taire", pos="verb", forms=[*TABLE_HEAD, ("tait", PRESENT_3S), ("tu", PARTICIPLE)]),
            entry("tuer", pos="verb", forms=[*TABLE_HEAD, ("tue", PRESENT_3S), ("tues", ["indicative", "present", "second-person", "singular"])]),
            form_of("tues", "tu", tags=("feminine", "form-of", "participle", "plural")),
            form_of("tus", "tu", tags=("form-of", "masculine", "participle", "plural")),
            entry("compromettre", pos="verb", forms=[*TABLE_HEAD, ("compromet", PRESENT_3S), ("compromis", PARTICIPLE)]),
            entry("compromis", forms=[("compromis", ["plural"])], senses=[{"glosses": ["compromise"], "tags": ["masculine"]}]),
            entry("compromis", pos="verb", forms=[("compromise", ["feminine"]), ("compromis", ["masculine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "compromettre"}]}]),
            form_of("compromise", "compromis", tags=("feminine", "form-of", "participle", "singular")),
            entry("quérir", pos="verb", forms=[*TABLE_HEAD, ("quis", PARTICIPLE)]),
            entry("quis", pos="verb", forms=[("quise", ["feminine"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "quérir"}]}]),
            form_of("quis", "qui", tags=("form-of", "masculine", "participle", "plural")),
            entry("qui", pos="pron"),
            form_of("quise", "quis", tags=("feminine", "form-of", "participle", "singular")),
        ]
        lex = lexicon(*entries)
        freq = {
            "qui": 6.8, "tu": 6.6, "tuer": 4.9, "cité": 4.8, "citer": 4.5, "tue": 4.5, "compromis": 4.3,
            "taire": 4.0, "cités": 3.9, "citée": 3.8, "compromettre": 3.6, "citées": 3.6, "compromise": 3.3,
            "tues": 3.2, "tus": 2.8, "quérir": 2.5, "quis": 2.1, "quise": 1.3,
        }
        top = [w for w in sorted(freq, key=lambda w: (-freq[w], w))]

        def reduced(overrides):
            return red.reduce_forms(lex, {}, {}, lambda w: freq.get(w, 0.0), lambda n: top[:n], 20, overrides)

        forms, ranks = reduced({})
        # The noun keeps its rank and its own form; the participle's feminine reads as its verb.
        self.assertEqual(forms["cité"], "cité")
        self.assertIn("cité", ranks)
        self.assertEqual(forms["citée"], "citer")
        self.assertEqual(forms["citées"], "citer")
        # `cités` is the noun's plural and the participle's: one lemma by D5, here the commoner.
        self.assertEqual(forms["cités"], "cité")
        # `tues` is taire's participle through `tu`, and tuer's second person: no longer the pronoun.
        self.assertEqual(forms["tu"], "tu")
        self.assertEqual(forms["tues"], "tuer")
        self.assertEqual(forms["tus"], "taire")
        # A participle's own inflections are no lemma entry of the verb's part of speech.
        self.assertEqual(forms["compromis"], "compromis")
        self.assertEqual(forms["compromise"], "compromettre")
        # The chain never reaches `qui` through a verb entry; `quis` itself is the override's.
        self.assertEqual(forms["quise"], "quérir")
        self.assertEqual(forms["quis"], "qui", "the source's error, read by frequency")
        self.assertEqual(reduced(red.OVERRIDES)[0]["quis"], "quérir")
        # Every ranked lemma's own form reads as itself.
        for lemma in ranks:
            self.assertEqual(forms[lemma], lemma, lemma)

    def test_a_chain_follows_one_part_of_speech_and_stops_at_a_lemma_of_it(self):
        links = {
            "citée": {"cité": {"verb"}},
            "cité": {"cité": {"noun"}, "citer": {"verb"}},
            "citer": {"citer": {"verb"}},
            "quis": {"quérir": {"verb"}, "qui": {"verb"}},
            "qui": {"qui": {"pron"}},
            "quérir": {"quérir": {"verb"}},
        }
        poses = {"cité": {"noun"}, "citer": {"verb"}, "qui": {"pron"}, "quérir": {"verb"}}
        # The commoner word first, as D5 would without counts: `qui` before `quérir`.
        choose = lambda word, options: min(options, key=lambda w: (w != "qui", w))  # noqa: E731
        self.assertEqual(red.follow("cité", "verb", links, poses, choose, {}), "citer")
        self.assertEqual(red.follow("cité", "noun", links, poses, choose, {}), "cité")
        # `qui` has no verb entry: a chain along the verb's part of speech does not reach it.
        self.assertEqual(red.follow("quis", "verb", links, poses, choose, {}), "quérir")
        # A word with nothing of the part of speech to follow stands.
        self.assertEqual(red.follow("qui", "verb", links, poses, choose, {}), "qui")
        # A person's decision holds wherever a chain passes.
        self.assertEqual(red.follow("cité", "verb", links, poses, choose, {"cité": ("cité", "a reason")}), "cité")

        def step(word, pos):
            return red.follow(word, pos, links, poses, choose, {})

        self.assertEqual(red.reach("citée", {"cité"}, links, step), {"citer"})
        # The form's own entry, and a candidate no entry links it to, stand as they are.
        self.assertEqual(red.reach("cité", {"cité", "citer"}, links, step), {"cité", "citer"})
        self.assertEqual(red.reach("l'", {"le"}, links, step), {"le"})

    def test_spec_scenario_a_noun_s_plural_is_not_its_homograph_s_verb(self):
        # `étés`, the noun's plural, does not follow `été` to être through a verb entry.
        self.assertEqual(self.forms["été"], "être")
        self.assertNotIn("étés", self.forms)

    def test_spec_scenario_a_noun_and_a_verb_share_a_form(self):
        self.assertEqual(self.forms["porte"], "porter")
        self.assertEqual(self.forms["portes"], "porter")
        self.assertNotIn("porte", self.ranks)

    def test_spec_scenario_the_elided_pieces(self):
        for piece, lemma in (("l'", "le"), ("qu'", "que"), ("s'", "se"), ("jusqu'", "jusque")):
            self.assertEqual(self.forms[piece], lemma)
        for split in ("c'est", "d'abord", "l'on", "jusqu'à"):
            self.assertNotIn(split, self.forms)
        # GSD attests it: a hyphenated run, kept whole.
        self.assertEqual(self.forms["c'est-à-dire"], "c'est-à-dire")
        self.assertIn("c'est-à-dire", self.ranks)
        self.assertEqual(self.forms["aujourd'hui"], "aujourd'hui")

    def test_spec_scenario_the_contracted_articles(self):
        for contraction in ("au", "aux"):
            self.assertNotIn(contraction, self.forms)
            self.assertNotIn(contraction, self.ranks)
        for word in ("à", "le", "les"):
            self.assertIn(word, self.forms)
        self.assertEqual(self.forms["les"], "le")
        # GSD reads `des` as un 1,730 times: M21 keeps it a word of its own.
        self.assertEqual(self.forms["du"], "du")
        self.assertEqual(self.forms["des"], "des")
        self.assertIn("des", self.ranks)

    def test_spec_scenario_a_noun_ending_in_a_pronoun_stays_whole(self):
        self.assertEqual(self.forms["rendez-vous"], "rendez-vous")
        self.assertEqual(self.forms["qu'en-dira-t-on"], "qu'en-dira-t-on")
        self.assertNotIn("est-il", self.forms)
        self.assertNotIn("allez-y", self.forms)
        self.assertNotIn("est-il", self.ranks)

    def test_the_check_of_d4_every_noun_ending_in_a_pronoun_is_a_form(self):
        # The dictionary's hyphenated nouns, adjectives, adverbs, pronouns and prepositions whose
        # last piece is a pronoun of the inversion rule (task 3.3).
        tails = sorted(w for w in self.lex.lemmas if red.ends_in_pronoun(w, self.lex.poses))
        self.assertEqual(tails, ["malgré-nous", "qu'en-dira-t-on", "rendez-vous"])
        self.assertEqual(red.unlisted_pronoun_tails(self.lex, self.forms), [])
        for word in tails:
            self.assertIn(word, self.forms)
        # GSD never meets the two others: they take the cut's last ranks, in order.
        self.assertEqual(sorted(self.ranks.values())[-2:], [self.ranks["malgré-nous"], self.ranks["qu'en-dira-t-on"]])
        self.assertEqual(self.ranks["qu'en-dira-t-on"], len(self.ranks))
        # A verb or a phrase made of a verb and its pronouns is no such word.
        self.assertFalse(red.ends_in_pronoun("est-il", self.lex.poses))
        self.assertFalse(red.ends_in_pronoun("allez-y", self.lex.poses))

    def test_a_verb_joined_to_its_pronouns_is_no_form(self):
        for joined in ("souviens-toi", "souvenons-nous", "sois-t'en"):
            self.assertNotIn(joined, self.forms)
        self.assertTrue(red.clitic_compound("souviens-toi", "souvenir"))
        self.assertTrue(red.clitic_compound("allons-nous-en", "aller"))
        self.assertTrue(red.clitic_compound("a-t-il", "avoir"))
        # A lemma of its own with a hyphen, or a tail that is no pronoun, is no such run.
        self.assertFalse(red.clitic_compound("celle-ci", "celui-ci"))
        self.assertFalse(red.clitic_compound("arc-en-ciel", "arc-en-ciel"))
        self.assertFalse(red.clitic_compound("porte-monnaie", "porte"))

    def test_spec_scenario_a_spelling_variant(self):
        self.assertEqual(self.forms["coeur"], "cœur")
        self.assertEqual(self.forms["coeurs"], "cœur")
        self.assertEqual(self.forms["connait"], "connaître")
        self.assertNotIn("coeur", self.ranks)
        self.assertNotIn("connaitre", self.ranks)

    def test_spec_scenario_wordfreq_s_elision_stems_are_no_words(self):
        for stem in ("l", "d", "qu", "s", "jusqu"):
            self.assertIn(stem, TOP)
            self.assertNotIn(stem, self.ranks)
            self.assertNotIn(stem, self.forms)
        self.assertEqual(red.WORDFREQ_STEMS, {piece[:-1] for piece in red.ELISIONS})

    def test_spec_scenario_a_word_no_form_reaches(self):
        # The first choice keeps the noun `tenue` (GSD counts it, as the noun, before the participle
        # `tenu`); once its forms of forms are followed, `tenue` and `tenues` read as tenir.
        self.assertEqual(self.forms["tenue"], "tenir")
        self.assertEqual(self.forms["tenues"], "tenir")
        self.assertNotIn("tenue", self.ranks)

    def test_a_ranked_word_whose_own_form_reads_as_another_gives_its_rank(self):
        # `donnée` reads as donner by GSD's counts. `données` still reaches the noun, but a pack
        # finds a lemma by its own form: ranked, *donnée* would lend its rank to donner. It leaves
        # the pack, as M8's nouns do, and `données` reads as donner.
        self.assertEqual(self.forms["donnée"], "donner")
        self.assertNotIn("donnée", self.ranks)
        self.assertEqual(self.forms["données"], "donner")
        # Every ranked lemma's own form reads as itself.
        for lemma in self.ranks:
            self.assertEqual(self.forms[lemma], lemma, lemma)

    def test_a_word_no_form_reaches_gives_its_rank_to_the_next(self):
        # One word short of every rankable one: ranked by the first choice alone, `tenue` would
        # hold a rank and push the last word out; it gives that rank to the next word.
        cut = len(reduce()[2]) - 1
        lex, forms, ranks = reduce(cut)
        gsd = counts()
        first = {f: red.choose_lemma(f, o, gsd, lex.lemmas, frequency) for f, o in lex.candidates.items()}
        inflected = {f for f, lemma in first.items() if lemma != f}
        unpruned = red.ranks_for(inflected, cut, lex.lemmas, lex.poses, red.gsd_zipf(gsd), frequency, top_n)
        self.assertIn("tenue", unpruned)
        self.assertNotIn("tenue", ranks)
        self.assertEqual(len(ranks), cut)
        # One word in for tenue: here `rendez-vous`, which the cut left out while tenue held a rank.
        self.assertEqual(set(unpruned) - set(ranks), {"tenue"})
        self.assertEqual(set(ranks) - set(unpruned), {"rendez-vous"})
        # Every ranked lemma is some form's lemma.
        self.assertEqual(set(ranks) - set(forms.values()), set())

    def test_spec_scenario_a_hyphenated_word_by_evidence(self):
        self.assertIn("peut-être", self.ranks)
        self.assertEqual(self.forms["peut-être"], "peut-être")
        # wordfreq's estimate ranks `fait-tout` above peut-être, but GSD never attests it.
        self.assertNotIn("fait-tout", self.ranks)
        self.assertNotIn("fait-tout", self.forms)
        self.assertNotIn("est-il", self.ranks)

    def test_an_unattested_form_of_a_kept_lemma_is_left_out(self):
        self.assertIn("diriger", self.ranks)
        self.assertNotIn("dirigeassions", self.forms)
        # A lemma's own form always stays; an elided piece too, whatever wordfreq says of it.
        self.assertEqual(self.forms["diriger"], "diriger")
        self.assertEqual(frequency("jusqu'"), 0.0)
        self.assertIn("jusqu'", self.forms)

    def test_every_ranked_lemma_is_some_form_s_lemma_and_its_own_form(self):
        self.assertEqual(set(self.ranks) - set(self.forms.values()), set())
        self.assertEqual(sorted(self.ranks.values()), list(range(1, len(self.ranks) + 1)))


class Ranks(unittest.TestCase):
    def ranks(self, compounds, lemmas, want=10, inflected=(), poses=None, freq=frequency):
        return red.ranks_for(set(inflected), want, set(lemmas), poses or {}, compounds, freq, top_n)

    def test_a_compound_at_the_lower_of_wordfreq_s_estimate_and_gsd_s_frequency(self):
        # peut-être: wordfreq estimates 5.4. With GSD's own frequency at 3.0 it is ranked at 3.0,
        # last; with GSD's at 6.0, at wordfreq's 5.4, between the words above and below it.
        low = self.ranks({"peut-être": 3.0}, {"peut-être"}, want=200)
        high = self.ranks({"peut-être": 6.0}, {"peut-être"}, want=200)
        kept_low = sorted(low, key=low.get)
        kept_high = sorted(high, key=high.get)
        self.assertEqual(kept_low[-1], "peut-être")
        before = kept_high[kept_high.index("peut-être") - 1]
        after = kept_high[kept_high.index("peut-être") + 1]
        self.assertGreaterEqual(frequency(before), 5.4)
        self.assertLess(frequency(after), 5.4)

    def test_at_a_tie_a_compound_comes_after_wordfreq_s_words_and_compounds_alphabetically(self):
        estimates = {"zz-a": 5.0, "aa-b": 5.0}
        freq = lambda w: estimates.get(w, frequency(w))  # noqa: E731
        got = self.ranks({"zz-a": 5.0, "aa-b": 5.0}, set(estimates), want=200, freq=freq)
        tied = [w for w in sorted(got, key=got.get) if freq(w) == 5.0]
        # wordfreq's words of 5.0 in its own order (`jusqu`, a stem, skipped), then the compounds.
        self.assertEqual(tied, ["donne", "tient", "aa-b", "zz-a"])

    def test_an_unattested_compound_is_unranked_and_a_noun_ending_in_a_pronoun_ranked_last(self):
        poses = {"malgré-nous": {"noun"}, "est-il": {"phrase"}}
        got = self.ranks({}, {"fait-tout", "malgré-nous", "est-il"}, want=5, poses=poses)
        self.assertNotIn("fait-tout", got)
        self.assertNotIn("est-il", got)
        self.assertEqual(got["malgré-nous"], 5)
        self.assertEqual(len(got), 5)

    def test_the_stems_the_contractions_and_the_inflected_are_skipped(self):
        got = self.ranks({}, set(), want=5, inflected={"la", "les"})
        self.assertEqual(list(got), ["de", "le", "à", "des", "est"])


class Treebank(unittest.TestCase):
    def test_counts_skip_multi_word_tokens_and_empty_nodes(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "t.conllu")
            with open(path, "w", encoding="utf-8") as f:
                f.write(
                    "# sent_id = 1\n"
                    "1\tL’\tle\tDET\t_\t_\t2\tdet\t_\t_\n"
                    "2-3\tdu\t_\t_\t_\t_\t_\t_\t_\t_\n"
                    "2\tde\tde\tADP\t_\t_\t1\tcase\t_\t_\n"
                    "3\tle\tle\tDET\t_\t_\t1\tdet\t_\t_\n"
                    "3.1\tx\tx\tX\t_\t_\t_\t_\t_\t_\n"
                    "4\tPeut-être\tpeut-être\tADV\t_\t_\t1\tadvmod\t_\t_\n"
                )
            got = red.read_gsd_counts([path])
        self.assertEqual(got[("l'", "le")], 1)
        self.assertEqual(got[("de", "de")], 1)
        self.assertNotIn(("du", "_"), got)
        self.assertNotIn(("x", "x"), got)
        zipf = red.gsd_zipf(got)
        # One of four words: 250,000,000 per billion.
        self.assertEqual(list(zipf), ["peut-être"])
        self.assertAlmostEqual(zipf["peut-être"], 8.39794, places=4)


# — French's word grammar (add-lingua-french-grammar-tables) —
#
# Each rule on entries shaped as the English dump writes them: a lemma's conjugation or inflection
# table under `forms`, a form's own entry pointing at its lemma (`form_of`), the head templates the
# rules read (`fr-noun`'s gender, a past participle's own entry).


def headed(word, pos, name, args=None, forms=(), senses=None):
    """An entry with its head template (`fr-noun`, `fr-past participle`, …)."""
    e = entry(word, pos=pos, forms=forms, senses=senses)
    e["head_templates"] = [{"name": name, "args": dict(args or {})}]
    return e


def readings(*entries):
    r = red.Readings()
    for e in entries:
        red.read_readings(e, r)
    return r


def grammar(entries, forms, ranks=None):
    """`grammar.tsv`'s rows for `forms` (form → its one lemma), as tuples."""
    ranks = ranks if ranks is not None else {lemma: n for n, lemma in enumerate(sorted(set(forms.values())), 1)}
    rows = red.grammar_rows(readings(*entries), forms, ranks)
    return [tuple(row.rstrip("\n").split("\t")) for row in rows]


def tags_of(rows, form, lemma=None, mark=None):
    return sorted(t for f, l, t, m in rows if f == form and (lemma is None or l == lemma) and (mark is None or m == mark))


IND, SUB, IMP, CND = "VERB|Mood=Ind", "VERB|Mood=Sub", "VERB|Mood=Imp", "VERB|Mood=Cnd"
# `parle`'s five readings, in byte order — the pool's (design D7, D8).
FIVE = [
    f"{IMP}|Number=Sing|Person=2|VerbForm=Fin",
    f"{IND}|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
    f"{IND}|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin",
    f"{SUB}|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
    f"{SUB}|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin",
]
# parler's table, as the dump writes it: its bookkeeping, the compound tenses (multi-word
# constructions), and the simple forms.
PARLER = headed(
    "parler",
    "verb",
    "fr-verb",
    forms=[
        *TABLE_HEAD,
        ("parler", ["infinitive"]),
        ("parlant", ["gerund", "participle", "present"]),
        ("ayant + past participle", ["gerund", "multiword-construction", "participle", "present"]),
        ("parlé", ["participle", "past"]),
        ("parle", ["first-person", "indicative", "present", "singular"]),
        ("parle", ["indicative", "present", "singular", "third-person"]),
        ("parlez", ["indicative", "plural", "present", "second-person"]),
        ("parlerait", ["conditional", "singular", "third-person"]),
        ("present indicative of avoir + past participle", ["indicative", "multiword-construction", "perfect", "present"]),
        ("parle", ["first-person", "present", "singular", "subjunctive"]),
        ("parle", ["present", "singular", "subjunctive", "third-person"]),
        ("parle", ["imperative", "second-person", "singular"]),
        ("parlez", ["imperative", "plural", "second-person"]),
        ("simple imperative of avoir + past participle", ["imperative", "multiword-construction", "second-person", "singular"]),
    ],
)
ETRE = headed(
    "être",
    "verb",
    "fr-verb",
    {"type": "auxiliary"},
    forms=[
        ("été", ["participle", "past"]),
        ("est", ["indicative", "present", "singular", "third-person"]),
        ("fut", ["historic", "indicative", "past", "singular", "third-person"]),
        ("future of avoir + past participle", ["future", "indicative", "multiword-construction", "perfect"]),
    ],
    senses=[{"glosses": ["to be"], "tags": ["copulative"]}],
)


class VerbReadings(unittest.TestCase):
    def test_spec_scenario_a_present_of_five_readings_from_the_table(self):
        rows = grammar([PARLER], {"parle": "parler", "parler": "parler"})
        self.assertEqual(tags_of(rows, "parle", "parler", "-"), FIVE)

    def test_a_sense_merging_persons_and_moods_reads_as_each(self):
        # The form's own entry, as the dump writes `parle`'s: « first/third-person singular present
        # indicative/subjunctive », and « second-person singular imperative ».
        parle = entry(
            "parle",
            pos="verb",
            senses=[
                {"glosses": ["inflection of parler"], "tags": ["form-of"], "form_of": [{"word": "parler"}, {"word": "parler"}]},
                {
                    "glosses": ["inflection of parler:", "first/third-person singular present indicative/subjunctive"],
                    "tags": ["first-person", "form-of", "indicative", "present", "singular", "subjunctive", "third-person"],
                    "form_of": [{"word": "parler"}],
                },
                {
                    "glosses": ["inflection of parler:", "second-person singular imperative"],
                    "tags": ["form-of", "imperative", "second-person", "singular"],
                    "form_of": [{"word": "parler"}],
                },
            ],
        )
        self.assertEqual(sorted(readings(parle).senses[("parle", "parler")]), FIVE)
        # A sense naming no person gives none: the conditional of `parlerait` needs its person.
        self.assertEqual(red.verb_features({"conditional", "singular"}), [])
        # The imperative has no third person: `vive`'s « third-person singular imperative » (vive le
        # roi) is the subjunctive's, which its other senses give.
        vive = form_of("vive", "vivre", tags=("form-of", "imperative", "singular", "third-person"))
        self.assertFalse(readings(vive).senses[("vive", "vivre")])

    def test_spec_scenario_a_verb_form_says_what_it_is(self):
        rows = grammar(
            [PARLER, ETRE],
            {f: "parler" for f in ("parler", "parlant", "parlé", "parlez", "parlerait")} | {"fut": "être", "être": "être"},
        )
        # The passé simple is the indicative past.
        self.assertEqual(tags_of(rows, "fut"), [f"{IND}|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"])
        # The conditional and the imperative take no tense (D7); the indicative does.
        self.assertEqual(tags_of(rows, "parlerait"), [f"{CND}|Number=Sing|Person=3|VerbForm=Fin"])
        self.assertEqual(
            tags_of(rows, "parlez"),
            [f"{IMP}|Number=Plur|Person=2|VerbForm=Fin", f"{IND}|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin"],
        )
        # The present participle as UD French writes it; the bare past participle is the masculine
        # singular; the infinitive.
        self.assertEqual(tags_of(rows, "parlant"), ["VERB|Tense=Pres|VerbForm=Part"])
        self.assertEqual(tags_of(rows, "parlé"), ["VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part"])
        self.assertEqual(tags_of(rows, "parler"), ["VERB|VerbForm=Inf"])

    def test_a_compound_tense_and_a_reflexive_row_with_its_pronoun_give_none(self):
        souvenir = headed(
            "souvenir",
            "verb",
            "fr-verb",
            forms=[
                *TABLE_HEAD,
                ("souviens", ["indicative", "present", "singular", "third-person"]),
                ("sois-t'en", ["imperative", "reflexive"]),
                ("t'en souviens", ["indicative", "present", "reflexive", "second-person", "singular"]),
            ],
        )
        got = readings(PARLER, souvenir).pairs()
        self.assertFalse([pair for pair in got if " " in pair[0] or "+" in pair[0]], "no compound tense")
        self.assertNotIn(("ayant", "parler"), got)
        # A reflexive row the pronoun cannot be taken off reads nothing.
        self.assertNotIn(("sois-t'en", "souvenir"), got)
        self.assertNotIn(("t'en souviens", "souvenir"), got)
        self.assertNotIn(("en souviens", "souvenir"), got)

    def test_spec_scenario_a_pronominal_verb(self):
        # *s'évanouir*'s table, as the dump writes it under `évanouir`: every row with its pronoun.
        evanouir = headed(
            "évanouir",
            "verb",
            "fr-verb",
            forms=[
                *TABLE_HEAD,
                ("s'évanouir", ["infinitive"]),
                ("s'évanouissant", ["gerund", "participle", "present"]),
                ("évanoui", ["participle", "past"]),
                ("m'évanouis", ["first-person", "indicative", "present", "singular"]),
                ("s'évanouit", ["indicative", "present", "singular", "third-person"]),
                ("nous évanouissons", ["first-person", "indicative", "plural", "present"]),
                ("s'évanouissaient", ["imperfect", "indicative", "plural", "third-person"]),
                ("nous évanouissions", ["first-person", "imperfect", "indicative", "plural"]),
                ("s'évanouit", ["historic", "indicative", "past", "singular", "third-person"]),
                ("s'être + past participle", ["infinitive", "multiword-construction"]),
                ("évanouis-toi", ["imperative", "second-person", "singular"]),
                ("évanouissons-nous", ["first-person", "imperative", "plural"]),
            ],
            senses=[{"glosses": ["to lose consciousness; to faint"], "tags": ["pronominal"]}],
        )
        got = readings(evanouir).pairs()
        self.assertEqual(
            sorted(got[("évanouit", "évanouir")]),
            [f"{IND}|Number=Sing|Person=3|Tense=Past|VerbForm=Fin", f"{IND}|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"],
        )
        self.assertEqual(got[("évanouissaient", "évanouir")], {f"{IND}|Number=Plur|Person=3|Tense=Imp|VerbForm=Fin"})
        self.assertEqual(got[("évanouissions", "évanouir")], {f"{IND}|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin"})
        self.assertIn(f"{IMP}|Number=Sing|Person=2|VerbForm=Fin", got[("évanouis", "évanouir")])
        self.assertIn(f"{IMP}|Number=Plur|Person=1|VerbForm=Fin", got[("évanouissons", "évanouir")])
        self.assertEqual(got[("évanouir", "évanouir")], {"VERB|VerbForm=Inf"})
        self.assertEqual(got[("évanouissant", "évanouir")], {"VERB|Tense=Pres|VerbForm=Part"})
        self.assertFalse([pair for pair in got if "'" in pair[0] or " " in pair[0] or "-" in pair[0]])
        self.assertEqual(red.without_pronoun("vous évanouissez"), "évanouissez")
        self.assertIsNone(red.without_pronoun("s'en est"))
        self.assertIsNone(red.without_pronoun("parle"))

    def test_spec_scenario_a_participle_s_agreement_through_its_own_entry(self):
        diriger = headed("diriger", "verb", "fr-verb", forms=[("dirigé", ["participle", "past"]), ("dirige", PRESENT_3S)])
        dirige = headed(
            "dirigé",
            "verb",
            "fr-past participle",
            forms=[("dirigée", ["feminine"]), ("dirigés", ["masculine", "plural"]), ("dirigées", ["feminine", "plural"])],
            senses=[{"glosses": ["past participle of diriger"], "tags": ["form-of", "participle", "past"], "form_of": [{"word": "diriger"}]}],
        )
        dirigee = form_of("dirigée", "dirigé", tags=("feminine", "form-of", "participle", "singular"))
        forms = {f: "diriger" for f in ("diriger", "dirige", "dirigé", "dirigée", "dirigés", "dirigées")}
        rows = grammar([diriger, dirige, dirigee], forms, {"diriger": 1})
        self.assertEqual(tags_of(rows, "dirigée"), ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
        self.assertEqual(tags_of(rows, "dirigées"), ["VERB|Gender=Fem|Number=Plur|Tense=Past|VerbForm=Part"])
        self.assertEqual(tags_of(rows, "dirigés"), ["VERB|Gender=Masc|Number=Plur|Tense=Past|VerbForm=Part"])
        self.assertEqual(tags_of(rows, "dirigé"), ["VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part"])
        # A participle's own entry is no lemma entry: dirigé is no verb of its own.
        self.assertNotIn("dirigé", readings(dirige).poses)

    def test_a_form_of_a_form_is_added_only_where_the_form_does_not_already_read_so(self):
        # `les` is the plural of le, and of la, itself le's feminine: it does not also read
        # feminine through `la` (design D5).
        le = headed("le", "article", "head", {"g": "m"}, forms=[("la", ["feminine"]), ("les", ["feminine", "masculine", "plural"])])
        la = form_of("la", "le", pos="article", tags=("feminine", "form-of", "singular"))
        les = entry(
            "les",
            pos="article",
            senses=[
                {"glosses": ["plural of le: the"], "tags": ["form-of", "plural"], "form_of": [{"word": "le", "extra": "the"}]},
                {"glosses": ["plural of la: the"], "tags": ["form-of", "plural"], "form_of": [{"word": "la", "extra": "the"}]},
            ],
        )
        rows = grammar([le, la, les], {"le": "le", "la": "le", "les": "le"})
        self.assertEqual(tags_of(rows, "les"), ["DET|Number=Plur"])
        self.assertEqual(tags_of(rows, "la"), ["DET|Gender=Fem|Number=Sing"])
        self.assertEqual(red.compose("DET|Gender=Fem|Number=Sing", "DET|Number=Plur"), "DET|Gender=Fem|Number=Plur")
        # Along one part of speech, and a verb's only through a participle.
        self.assertIsNone(red.compose("ADJ|Gender=Fem|Number=Sing", "DET|Number=Plur"))
        self.assertIsNone(red.compose(f"{IND}|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin", "VERB|Number=Plur"))
        self.assertIsNone(red.compose("VERB|Tense=Past|VerbForm=Part", "VERB|VerbForm=Inf"))


class NominalReadings(unittest.TestCase):
    def test_spec_scenario_a_noun_says_its_gender(self):
        maison = headed("maison", "noun", "fr-noun", {"1": "f"}, forms=[("maisons", ["plural"]), ("maisonne", ["alternative"])])
        temps = headed(
            "temps",
            "noun",
            "fr-noun",
            {"1": "m"},
            senses=[{"glosses": ["time (in general)"], "tags": ["invariable", "masculine", "uncountable"]}],
        )
        bras = headed("bras", "noun", "fr-noun", {"1": "m", "2": "#"}, senses=[{"glosses": ["arm"], "tags": ["masculine"]}])
        enfant = headed(
            "enfant",
            "noun",
            "fr-noun",
            {"1": "mfbysense"},
            forms=[("enfants", ["plural"])],
            senses=[{"glosses": ["child"], "tags": ["by-personal-gender", "feminine", "masculine"]}],
        )
        forms = {"maison": "maison", "maisons": "maison", "enfant": "enfant", "enfants": "enfant", "temps": "temps", "bras": "bras"}
        rows = grammar([maison, temps, bras, enfant], forms)
        self.assertEqual(tags_of(rows, "maison"), ["NOUN|Gender=Fem|Number=Sing"])
        self.assertEqual(tags_of(rows, "maisons"), ["NOUN|Gender=Fem|Number=Plur"])
        # A noun the dictionary gives one form for reads in both numbers (D4).
        self.assertEqual(tags_of(rows, "temps"), ["NOUN|Gender=Masc|Number=Plur", "NOUN|Gender=Masc|Number=Sing"])
        self.assertEqual(tags_of(rows, "bras"), ["NOUN|Gender=Masc|Number=Plur", "NOUN|Gender=Masc|Number=Sing"])
        # A noun of both genders, a reading per gender.
        self.assertEqual(tags_of(rows, "enfant"), ["NOUN|Gender=Fem|Number=Sing", "NOUN|Gender=Masc|Number=Sing"])
        self.assertEqual(tags_of(rows, "enfants"), ["NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Masc|Number=Plur"])
        # Without `fr-noun`, the senses' genders.
        self.assertEqual(red.noun_genders(entry("x", senses=[{"glosses": ["x"], "tags": ["feminine"]}])), (("Fem",), False))
        self.assertEqual(red.noun_genders(headed("gens", "noun", "fr-noun", {"1": "m-p"})), (("Masc",), True))

    def test_spec_scenario_a_feminine_noun_s_masculine_is_no_form_of_it(self):
        deesse = headed(
            "déesse", "noun", "fr-noun", {"1": "f", "m": "dieu"}, forms=[("déesses", ["plural"]), ("dieu", ["masculine"])]
        )
        dieu = headed(
            "dieu", "noun", "fr-noun", {"1": "m", "2": "#x", "f": "déesse"}, forms=[("dieux", ["plural"]), ("déesse", ["feminine"])]
        )
        got = readings(deesse, dieu).pairs()
        self.assertNotIn(("dieu", "déesse"), got)
        self.assertEqual(got[("déesses", "déesse")], {"NOUN|Gender=Fem|Number=Plur"})
        # dieu's own table still names its feminine.
        self.assertEqual(got[("déesse", "dieu")], {"NOUN|Gender=Fem|Number=Sing"})

    def test_an_adjective_agrees_or_takes_one_form_for_both_genders(self):
        grand = headed(
            "grand",
            "adj",
            "fr-adj",
            forms=[("grande", ["feminine"]), ("grands", ["masculine", "plural"]), ("grandes", ["feminine", "plural"])],
        )
        rapide = headed("rapide", "adj", "fr-adj", forms=[("rapides", ["plural"])])
        beau = headed(
            "beau",
            "adj",
            "fr-adj",
            {"mv": "bel"},
            forms=[("bel", ["before-vowel", "masculine", "singular"]), ("belle", ["feminine"])],
        )
        plusieurs = headed(
            "plusieurs",
            "adj",
            "fr-adj",
            {"inv": "1", "onlyg": "p"},
            senses=[{"glosses": ["several"], "tags": ["plural", "plural-only"]}],
        )
        got = readings(grand, rapide, beau, plusieurs).pairs()
        self.assertEqual(got[("grand", "grand")], {"ADJ|Gender=Masc|Number=Sing"})
        self.assertEqual(got[("grande", "grand")], {"ADJ|Gender=Fem|Number=Sing"})
        self.assertEqual(got[("grandes", "grand")], {"ADJ|Gender=Fem|Number=Plur"})
        self.assertEqual(got[("rapide", "rapide")], {"ADJ|Number=Sing"})
        self.assertEqual(got[("rapides", "rapide")], {"ADJ|Number=Plur"})
        # The masculine before a vowel is a masculine singular.
        self.assertEqual(got[("bel", "beau")], {"ADJ|Gender=Masc|Number=Sing"})
        # A comparative or superlative row reads its degree, an adjective's alone.
        self.assertEqual(red.reading_tags("ADJ", {"comparative"}), ["ADJ|Degree=Cmp|Number=Sing"])
        self.assertEqual(red.reading_tags("ADJ", {"superlative", "feminine"}), ["ADJ|Degree=Sup|Gender=Fem|Number=Sing"])
        self.assertEqual(red.reading_tags("NOUN", {"superlative"}), [])
        # Every sense plural: a plural of its own.
        self.assertEqual(got[("plusieurs", "plusieurs")], {"ADJ|Number=Plur"})

    def test_determiners_articles_and_pronouns_read_their_agreement(self):
        le_det = headed("le", "article", "head", {"g": "m"}, forms=[("la", ["feminine"]), ("les", ["feminine", "masculine", "plural"])])
        le_pron = headed("le", "pron", "head", {"g": "m"}, forms=[("la", ["feminine"]), ("les", ["feminine", "masculine", "plural"])])
        celui = headed(
            "celui",
            "pron",
            "fr-pron",
            {"1": "m", "f": "celle"},
            forms=[("celle", ["feminine"]), ("ceux", ["masculine", "plural"]), ("çui", ["alternative", "colloquial"])],
        )
        celle = form_of("celle", "celui", pos="pron", tags=("feminine", "form-of", "singular"))
        il = headed(
            "il",
            "pron",
            "head",
            {"g": "m", "5": "plural", "6": "ils"},
            forms=[("ils", ["plural"]), ("le", ["accusative"]), ("lui", ["dative"]), ("son", ["determiner", "possessive"])],
            senses=[{"glosses": ["he"], "tags": ["masculine", "singular", "third-person"]}],
        )
        tes = headed(
            "tes",
            "det",
            "head",
            {"g": "p", "3": "masculine", "4": "ton"},
            forms=[("ton", ["masculine"]), ("ta", ["feminine"])],
            senses=[{"glosses": ["your (when referring to a plural noun)"], "tags": ["plural"]}],
        )
        # `un`'s table lists `de` as its negative (« pas de »): no agreement, no reading.
        un = headed("un", "article", "head", {"g": "m"}, forms=[("une", ["feminine"]), ("des", ["plural"]), ("de", ["negative"])])
        got = readings(le_det, le_pron, celui, celle, il, tes, un).pairs()
        self.assertEqual(got[("une", "un")], {"DET|Gender=Fem|Number=Sing"})
        self.assertEqual(got[("des", "un")], {"DET|Number=Plur"})
        self.assertFalse(got.get(("de", "un")))
        # An article is a determiner; `la` reads as le's both as a determiner and as a pronoun.
        self.assertEqual(got[("la", "le")], {"DET|Gender=Fem|Number=Sing", "PRON|Gender=Fem|Number=Sing"})
        self.assertEqual(got[("celle", "celui")], {"PRON|Gender=Fem|Number=Sing"})
        self.assertEqual(got[("ceux", "celui")], {"PRON|Gender=Masc|Number=Plur"})
        # A pronoun's other persons and cases are other words, and a row without a gender gives none.
        for other_word in ("ils", "le", "lui", "son"):
            self.assertFalse(got.get((other_word, "il")), other_word)
        # A plural-headed table gives none: `ton` is no « masculine of tes ».
        self.assertNotIn(("ton", "tes"), got)
        self.assertNotIn(("ta", "tes"), got)
        # A determiner's or pronoun's own form names nothing on its card.
        self.assertNotIn(("le", "le"), got)


class WhatGivesNoReading(unittest.TestCase):
    def test_spec_scenario_what_gives_no_reading(self):
        # A capitalised headword is another word (`CE`, « works council »).
        ce = headed("CE", "noun", "fr-noun", {"1": "m", "2": "#"}, senses=[{"glosses": ["works council"], "tags": ["invariable", "masculine"]}])
        # An archaic spelling's table: `est` would be its form.
        estre = headed(
            "estre",
            "verb",
            "fr-verb",
            forms=[("estre", ["infinitive"]), ("est", ["indicative", "present", "singular", "third-person"])],
            senses=[{"glosses": ["archaic spelling of être"], "tags": ["alt-of", "archaic"], "alt_of": [{"word": "être"}]}],
        )
        # A gender-neutral neologism: `les` would be its plural.
        lea = headed(
            "lea",
            "article",
            "head",
            {"g": "gneut", "3": "plural", "4": "les"},
            forms=[("lea gender-neutral", ["canonical"]), ("les", ["plural"])],
            senses=[{"glosses": ["the"], "tags": ["neologism"]}],
        )
        # Louisiana's past participle of aller, on `été`'s own entry.
        ete = headed(
            "été",
            "verb",
            "fr-past participle",
            {"intr": "1"},
            senses=[
                {"glosses": ["past participle of être"], "tags": ["form-of", "intransitive", "participle", "past"], "form_of": [{"word": "être"}]},
                {"glosses": ["past participle of aller"], "tags": ["Louisiana", "form-of", "intransitive", "participle", "past"], "form_of": [{"word": "aller"}]},
            ],
        )
        # The letter L's name, beside the pronoun whose plural `elles` is.
        elle_letter = headed(
            "elle",
            "noun",
            "fr-noun",
            {"1": "m"},
            forms=[("elles", ["plural"])],
            senses=[{"glosses": ["The name of the Latin script letter L/l."], "tags": ["masculine"], "categories": [{"name": "fr:Latin letter names"}]}],
        )
        elle_pron = headed(
            "elle",
            "pron",
            "head",
            {"g": "f", "5": "plural", "6": "elles"},
            forms=[("elles", ["plural"]), ("la", ["accusative"])],
            senses=[{"glosses": ["she"], "tags": ["feminine", "singular", "third-person"]}],
        )
        elles = headed(
            "elles",
            "noun",
            "head",
            {"g": "f"},
            senses=[{"glosses": ["plural of elle"], "tags": ["feminine", "form-of", "plural"], "form_of": [{"word": "elle"}]}],
        )
        aller = headed("aller", "verb", "fr-verb", forms=[("allé", ["participle", "past"])])
        got = readings(ce, estre, lea, ete, elle_letter, elle_pron, elles, ETRE, aller).pairs()
        self.assertNotIn(("ce", "ce"), got)
        self.assertNotIn(("est", "estre"), got)
        self.assertEqual(got[("est", "être")], {f"{IND}|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"})
        self.assertNotIn(("les", "lea"), got)
        self.assertNotIn(("été", "aller"), got)
        self.assertEqual(got[("été", "être")], {"VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part"})
        # The letter keeps its own form; its table gives no plural, and the pronoun's table, which
        # lists `elles` without a gender, states the pair: the plural's own entry adds nothing.
        self.assertEqual(got[("elle", "elle")], {"NOUN|Gender=Masc|Number=Sing"})
        self.assertFalse(got.get(("elles", "elle")))
        rows = grammar([elle_letter, elle_pron, elles], {"elle": "elle", "elles": "elles"}, {"elle": 1, "elles": 2})
        self.assertEqual(tags_of(rows, "elles"), [])
        # A doubtful row of a table reads nothing either (`maisonne`, an alternative).
        maison = headed("maison", "noun", "fr-noun", {"1": "f"}, forms=[("maisonne", ["alternative"])])
        self.assertNotIn(("maisonne", "maison"), readings(maison).pairs())

    def test_spec_scenario_a_spelling_variant_reads_as_the_word_it_spells(self):
        coeur = headed(
            "coeur",
            "noun",
            "fr-noun",
            {"1": "m"},
            forms=[("coeurs", ["plural"])],
            senses=[{"glosses": ["nonstandard spelling of cœur"], "tags": ["alt-of", "masculine", "nonstandard"], "alt_of": [{"word": "cœur"}]}],
        )
        coeur_ = headed("cœur", "noun", "fr-noun", {"1": "m"}, forms=[("cœurs", ["plural"]), ("coeur", ["alternative"])])
        rows = grammar([coeur, coeur_], {"coeur": "cœur", "coeurs": "cœur", "cœur": "cœur", "cœurs": "cœur"})
        self.assertEqual(tags_of(rows, "coeurs", "cœur", "-"), ["NOUN|Gender=Masc|Number=Plur"])
        self.assertEqual(tags_of(rows, "coeur", "cœur", "-"), ["NOUN|Gender=Masc|Number=Sing"])
        # The variant is no lemma entry of its own.
        self.assertNotIn("coeur", readings(coeur).poses)


class OtherMarks(unittest.TestCase):
    def test_spec_scenario_a_homograph_names_its_other_dictionary_form(self):
        fils = headed("fils", "noun", "fr-noun", {"1": "m"}, senses=[{"glosses": ["son"], "tags": ["invariable", "masculine"]}])
        fils_form = headed(
            "fils",
            "noun",
            "head",
            {"g": "m-p"},
            senses=[{"glosses": ["plural of fil"], "tags": ["form-of", "masculine", "plural"], "form_of": [{"word": "fil"}]}],
        )
        fil = headed("fil", "noun", "fr-noun", {"1": "m"}, forms=[("fils", ["plural"])])
        couver = headed(
            "couver",
            "verb",
            "fr-verb",
            forms=[
                ("couvent", ["indicative", "plural", "present", "third-person"]),
                ("couvent", ["plural", "present", "subjunctive", "third-person"]),
            ],
        )
        couvent = headed("couvent", "noun", "fr-noun", {"1": "m"}, forms=[("couvents", ["plural"])])
        rows = grammar([fils, fils_form, fil, couver, couvent], {"fils": "fils", "fil": "fil", "couvent": "couvent", "couver": "couver"})
        self.assertEqual(tags_of(rows, "fils", "fils", "-"), ["NOUN|Gender=Masc|Number=Plur", "NOUN|Gender=Masc|Number=Sing"])
        self.assertEqual(tags_of(rows, "fils", "fil", "other"), ["NOUN|Gender=Masc|Number=Plur"])
        self.assertEqual(tags_of(rows, "couvent", "couvent", "-"), ["NOUN|Gender=Masc|Number=Sing"])
        self.assertEqual(
            tags_of(rows, "couvent", "couver", "other"),
            [f"{IND}|Number=Plur|Person=3|Tense=Pres|VerbForm=Fin", f"{SUB}|Number=Plur|Person=3|Tense=Pres|VerbForm=Fin"],
        )

    def test_another_word_is_named_only_when_it_is_an_entry_not_only_regional(self):
        # `irait`'s link lists « would go » after aller; `va`'s entry lists *vader*, whose every
        # sense is a region's (Louisiana, Switzerland).
        irait = headed(
            "irait",
            "verb",
            "head",
            senses=[
                {
                    "glosses": ["third-person singular conditional of aller, would go."],
                    "tags": ["conditional", "form-of", "singular", "third-person"],
                    "form_of": [{"word": "aller"}, {"word": "would go"}],
                }
            ],
        )
        va = entry(
            "va",
            pos="verb",
            senses=[
                {"glosses": ["inflection of aller:"], "tags": ["form-of", "imperative", "second-person", "singular"], "form_of": [{"word": "aller"}]},
                {"glosses": ["inflection of vader:"], "tags": ["form-of", "imperative", "second-person", "singular"], "form_of": [{"word": "vader"}]},
            ],
        )
        aller = headed("aller", "verb", "fr-verb", senses=[{"glosses": ["to go"]}])
        vader = headed("vader", "verb", "fr-verb", {"type": "defective"}, senses=[{"glosses": ["to go"], "tags": ["Louisiana", "defective"]}])
        vader_ch = headed(
            "vader", "verb", "fr-verb", forms=[("va", ["imperative", "second-person", "singular"])], senses=[{"glosses": ["to get away"], "tags": ["Switzerland"]}]
        )
        forms = {"irait": "aller", "va": "aller", "aller": "aller", "vader": "vader", "would": "would"}
        rows = grammar([irait, va, aller, vader, vader_ch], forms, {"aller": 1, "vader": 2, "would": 3})
        self.assertEqual(tags_of(rows, "irait"), [f"{CND}|Number=Sing|Person=3|VerbForm=Fin"])
        self.assertEqual({(l, m) for f, l, _, m in rows if f in ("irait", "va")}, {("aller", "-")})

    def test_spec_scenario_a_participle_filed_under_a_noun_s_spelling(self):
        # A forms table filing `citée` under the noun *cité* (as change 43's prototype did): the
        # participle reads no verb form of the noun, and names its verb.
        citer = headed("citer", "verb", "fr-verb", forms=[("cité", ["participle", "past"]), ("cite", PRESENT_3S)])
        cite_noun = headed("cité", "noun", "fr-noun", {"1": "f"}, forms=[("cités", ["plural"])], senses=[{"glosses": ["city"], "tags": ["feminine"]}])
        cite_participle = headed(
            "cité",
            "verb",
            "fr-past participle",
            forms=[("citée", ["feminine"]), ("cités", ["masculine", "plural"]), ("citées", ["feminine", "plural"])],
            senses=[{"glosses": ["past participle of citer"], "tags": ["form-of", "participle", "past"], "form_of": [{"word": "citer"}]}],
        )
        citee = headed(
            "citée",
            "verb",
            "head",
            {"g": "f-s"},
            senses=[{"glosses": ["feminine singular of cité"], "tags": ["feminine", "form-of", "participle", "singular"], "form_of": [{"word": "cité"}]}],
        )
        entries = [citer, cite_noun, cite_participle, citee]
        rows = grammar(entries, {"citée": "cité", "cité": "cité", "citer": "citer", "cités": "cité"})
        self.assertEqual(tags_of(rows, "citée", "cité"), [])
        self.assertEqual(tags_of(rows, "citée", "citer", "other"), ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
        # The noun keeps its own readings; its plural is the noun's and names the participle's verb.
        self.assertEqual(tags_of(rows, "cité", "cité", "-"), ["NOUN|Gender=Fem|Number=Sing"])
        self.assertEqual(tags_of(rows, "cités", "cité", "-"), ["NOUN|Gender=Fem|Number=Plur"])
        self.assertEqual(tags_of(rows, "cités", "citer", "other"), ["VERB|Gender=Masc|Number=Plur|Tense=Past|VerbForm=Part"])
        # Filed under the verb, as change 43's implementation files it, the participle is its own.
        rows = grammar(entries, {"citée": "citer", "cité": "cité", "citer": "citer"})
        self.assertEqual(tags_of(rows, "citée", "citer", "-"), ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
        self.assertEqual(tags_of(rows, "citée", "cité"), [])
        # A lemma the dictionary holds as none of the parts of speech read keeps its readings.
        info = entry("infos", pos="noun", senses=[{"glosses": ["plural of info"], "tags": ["form-of", "plural"], "form_of": [{"word": "info"}]}])
        self.assertEqual(tags_of(grammar([info], {"infos": "info", "info": "info"}), "infos"), ["NOUN|Number=Plur"])

    def test_a_link_toward_a_word_the_dictionary_holds_as_no_verb_gives_no_verb_reading(self):
        venait = headed(
            "venait",
            "verb",
            "head",
            senses=[
                {
                    "glosses": ["third-person singular imperfect indicative of venir, was coming, came"],
                    "tags": ["form-of", "imperfect", "indicative", "singular", "third-person"],
                    "form_of": [{"word": "venir"}, {"word": "was coming"}, {"word": "came"}],
                }
            ],
        )
        venir = headed("venir", "verb", "fr-verb", forms=[("venait", ["imperfect", "indicative", "singular", "third-person"])])
        came = headed("came", "noun", "fr-noun", {"1": "f"}, forms=[("cames", ["plural"])])
        rows = grammar([venait, venir, came], {"venait": "venir", "venir": "venir", "came": "came"})
        self.assertEqual(tags_of(rows, "venait"), [f"{IND}|Number=Sing|Person=3|Tense=Imp|VerbForm=Fin"])
        self.assertEqual({l for f, l, _, _ in rows if f == "venait"}, {"venir"})

    def test_a_link_an_override_row_sets_aside_gives_no_reading(self):
        # `fatiguée`'s verb entry reads « feminine singular of parlé », a copy error change 43's
        # override row reads past (`fatiguée` → fatiguer): it names no *parler*.
        entries = [
            headed("fatiguer", "verb", "fr-verb", forms=[("fatigué", ["participle", "past"])]),
            headed(
                "fatigué",
                "verb",
                "fr-past participle",
                forms=[("fatiguée", ["feminine"]), ("fatiguées", ["feminine", "plural"])],
                senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "fatiguer"}]}],
            ),
            headed("fatigué", "adj", "fr-adj", forms=[("fatiguée", ["feminine"])], senses=[{"glosses": ["tired"]}]),
            form_of("fatiguée", "fatigué", pos="adj", tags=("feminine", "form-of", "singular")),
            headed(
                "fatiguée",
                "verb",
                "fr-past participle",
                forms=[("fatiguées", ["plural"])],
                senses=[{"glosses": ["feminine singular of parlé"], "tags": ["feminine", "form-of", "participle", "singular"], "form_of": [{"word": "parlé"}]}],
            ),
            PARLER,
            form_of("parlé", "parler", tags=("form-of", "participle", "past")),
        ]
        forms = {"fatiguée": "fatiguer", "fatiguer": "fatiguer", "fatigué": "fatigué", "parler": "parler", "parlé": "parler"}
        ranks = {"parler": 1, "fatiguer": 2, "fatigué": 3}
        rows = grammar(entries, forms, ranks)
        self.assertEqual({(l, m) for f, l, _, m in rows if f == "fatiguée"}, {("fatiguer", "-"), ("fatigué", "other")})
        self.assertEqual(tags_of(rows, "fatiguée", "fatiguer"), ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
        self.assertEqual(red.set_aside(readings(*entries)), {("fatiguée", "parlé")})
        # Without the row, the copy error names parler.
        rows = red.grammar_rows(readings(*entries), forms, ranks, overrides={})
        self.assertIn("fatiguée\tparler\tVERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part\tother\n", rows)

    def test_rows_hold_the_tables_forms_under_kept_lemmas_sorted(self):
        rows = red.grammar_rows(readings(PARLER, ETRE), {"parle": "parler", "fut": "être", "parlez": "parlez"}, {"parler": 1})
        # `fut` is not under a kept lemma; `parlez`'s own lemma is not *parler*, which it names.
        self.assertEqual(rows, sorted(rows))
        self.assertFalse([row for row in rows if row.startswith("fut\t")])
        self.assertTrue(all(row.endswith("\tother\n") for row in rows if row.startswith("parlez\t")))
        self.assertEqual(len([row for row in rows if row.startswith("parle\t")]), 5)



def fake_wordfreq():
    module = types.ModuleType("wordfreq")
    module.top_n_list = lambda lang, n: top_n(n)
    module.zipf_frequency = lambda word, lang: frequency(word)
    return module


class Main(unittest.TestCase):
    def run_main(self, entries, gsd=GSD, files=None):
        """`main` over a work folder holding `entries` as the section, GSD's two sections and
        `files` (name → lines) beside them; every file it leaves there, by name."""
        with tempfile.TemporaryDirectory() as work:
            with open(os.path.join(work, "kaikki-French.jsonl"), "w", encoding="utf-8") as f:
                f.write("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries))
                f.write("not json\n")
            for name, lines in (files or {}).items():
                with open(os.path.join(work, name), "w", encoding="utf-8") as f:
                    f.write("".join(json.dumps(line, ensure_ascii=False) + "\n" for line in lines))
            for name in ("fr_gsd-ud-train.conllu", "fr_gsd-ud-dev.conllu"):
                with open(os.path.join(work, name), "w", encoding="utf-8") as f:
                    f.write(conllu(gsd))
            argv = ["reduce-fr-en.py", "--work", work, "--built-at", "2026-10-08", "--pack-version", "2026.10.08+abcdef0"]
            with mock.patch.dict(sys.modules, {"wordfreq": fake_wordfreq()}), mock.patch.object(sys, "argv", argv):
                with contextlib.redirect_stderr(io.StringIO()):
                    red.main()
            out = {}
            for name in sorted(os.listdir(work)):
                if name.endswith(".jsonl"):
                    continue
                with open(os.path.join(work, name), encoding="utf-8") as f:
                    out[name] = f.read()
            return out

    def test_spec_scenario_the_reference_pair_writes_french_s_folder(self):
        out = self.run_main(ENTRIES)
        # fr-en's native side (add-lingua-pack-fr-en): the ranked lemmas the section glosses, their
        # sense runs, and the expressions — `NativeSide` below.
        self.assertIn("rendez-vous\tappointment\n", out["gloss.tsv"])
        self.assertIn("rendez-vous\tNOUN:1\n", out["senses.tsv"])
        self.assertIn("d'abord\ta meaning\n", out["mwe.tsv"])
        for name in ("forms.tsv", "freq.tsv", "NOTICE", "manifest.json"):
            self.assertTrue(out[name], name)
        self.assertIn("dirigée\tdiriger\tVERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part\t-\n", out["grammar.tsv"])
        # and French's estimated levels (add-lingua-french-levels, `EstimatedLevels` below), given to
        # French's dictionary words alone, which it writes too (refine-lingua-fr-en-glosses D2, D3).
        self.assertIn("de\tA1\n", out["level.tsv"])
        self.assertIn("rendez-vous\n", out["lexical.tsv"])
        self.assertIn("dirigée\tdiriger\n", out["forms.tsv"])
        self.assertTrue(out["freq.tsv"].startswith("de\t1\nle\t2\n"), out["freq.tsv"][:40])
        # Sorted by form, and by rank: the order never depends on the source's.
        rows = out["forms.tsv"].splitlines()
        self.assertEqual(rows, sorted(rows, key=lambda r: r.split("\t")[0]))
        manifest = json.loads(out["manifest.json"])
        self.assertEqual(manifest["meta"]["studied"], "fr")
        self.assertEqual(manifest["meta"]["native"], "en")
        self.assertEqual(manifest["meta"]["pack_version"], "2026.10.08+abcdef0")
        self.assertEqual(manifest["meta"]["analyzer_version"], red.analyser_version())
        self.assertIs(manifest["meta"]["levels_estimated"], True)
        for source in manifest["sources"]:
            self.assertIn(source["name"], out["NOTICE"])
        self.assertEqual([s["name"] for s in manifest["sources"]], ["kaikki", "wordfreq", "UD French-GSD"])

    def test_a_reduction_leaving_a_noun_ending_in_a_pronoun_out_fails(self):
        # `malgré-nous` read as the form of a verb no rank keeps: no form, so the inversion rule
        # would split it — the reduction stops, naming it.
        entries = [*ENTRIES, form_of("malgré-nous", "malgrer")]
        with self.assertRaisesRegex(SystemExit, "malgré-nous"):
            self.run_main(entries, [*GSD, ("malgré-nous", "malgrer", 4)])


class Manifest(unittest.TestCase):
    def test_the_french_analyser_version_is_read_from_the_core(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "mod.rs")
            with open(path, "w", encoding="utf-8") as f:
                f.write(
                    'pub const ANALYZER_VERSION: &str = "1.1.0";\n'
                    'pub const SPANISH_ANALYZER_VERSION: &str = "1.2.0";\n'
                    'pub const FRENCH_ANALYZER_VERSION: &str = "9.9.9";\n'
                )
            self.assertEqual(red.analyser_version(path), "9.9.9")
            with open(path, "w", encoding="utf-8") as f:
                f.write('pub const ANALYZER_VERSION: &str = "1.1.0";\n')
            with self.assertRaisesRegex(SystemExit, "no FRENCH_ANALYZER_VERSION"):
                red.analyser_version(path)
        # And the real core names one.
        self.assertRegex(red.analyser_version(), r"^\d+\.\d+\.\d+$")

    def test_the_notice_names_every_source_the_manifest_declares(self):
        for name in ["kaikki", "wordfreq", "UD French-GSD", "French section"]:
            self.assertIn(name, red.NOTICE)
        # The section is credited for fr-en's glosses too (add-lingua-pack-fr-en D1), and no other
        # source is added: no translation table is read (D3).
        self.assertIn("the English glosses of French words and expressions", red.NOTICE.replace("\n", " "))
        self.assertNotIn("frwiktionary", red.NOTICE)


# — The estimated levels (add-lingua-french-levels) —

# What design D2's rules read, shaped as the English dump writes it: a name, a letter, a letter's
# abbreviation, words that are also letters, spellings of another word (one beside a name's entry).
LEVEL_ENTRIES = [
    entry(
        "France",
        pos="name",
        senses=[
            {"glosses": ["France (a country located primarily in Western Europe)"], "tags": ["feminine"]},
            {"glosses": ["a female given name"], "tags": ["feminine"]},
        ],
    ),
    entry("b", pos="character", senses=[{"glosses": ["The second letter of the French alphabet, written in the Latin script."], "tags": ["letter", "lowercase"]}]),
    entry("e", pos="character", senses=[{"glosses": ["The fifth letter of the French alphabet, written in the Latin script."], "tags": ["letter", "lowercase"]}]),
    entry("E", pos="noun", senses=[{"glosses": ["abbreviation of est; east"], "tags": ["abbreviation", "alt-of", "masculine"], "alt_of": [{"word": "est"}]}]),
    entry("à", pos="character", senses=[{"glosses": ["A with grave accent, a letter used in French mostly to distinguish some homographs."], "tags": ["letter", "lowercase"]}]),
    entry("y", pos="character", senses=[{"glosses": ["a letter in the French alphabet, after x and before z"], "tags": ["letter", "lowercase"]}]),
    entry("y", pos="pron", senses=[{"glosses": ["there (at a place)"], "tags": ["adverbial"]}]),
    entry("etre", pos="verb", senses=[{"glosses": ["obsolete spelling of être"], "tags": ["Internet", "alt-of", "colloquial", "obsolete"], "alt_of": [{"word": "être"}]}]),
    entry("orient", senses=[{"glosses": ["alternative letter-case form of Orient"], "tags": ["alt-of", "masculine"], "alt_of": [{"word": "Orient"}]}]),
    entry("Orient", pos="name", senses=[{"glosses": ["Orient"], "tags": ["masculine"]}]),
    entry("parceque", pos="conj", senses=[{"glosses": ["obsolete form of parce que"], "tags": ["alt-of", "obsolete"], "alt_of": [{"word": "parce que"}]}]),
    entry("maison", senses=[{"glosses": ["house"], "tags": ["feminine"]}]),
]


def level_senses(*entries):
    """The fixture section's senses, as `read_level_senses` keeps them."""
    with tempfile.TemporaryDirectory() as d:
        path = os.path.join(d, "kaikki-French.jsonl")
        with open(path, "w", encoding="utf-8") as f:
            f.write("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries or [*ENTRIES, *LEVEL_ENTRIES]))
            f.write("not json\n")
        return red.read_level_senses(path)


class EstimatedLevels(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        cls.senses = level_senses()

    def test_the_section_s_senses_are_those_no_form_of_another_word(self):
        self.assertEqual({pos for pos, _, _ in self.senses["y"]}, {"character", "pron"})
        # Words are lowercased: `France` and `E` are read as `france` and `e`.
        self.assertEqual({pos for pos, _, _ in self.senses["france"]}, {"name"})
        self.assertIn(("noun", "abbreviation of est; east", frozenset({"abbreviation", "alt-of", "masculine"})), self.senses["e"])
        # `étés` is only a form of `été`; `été` is a noun of its own, besides a form of être.
        self.assertNotIn("étés", self.senses)
        self.assertEqual(self.senses["été"], [("noun", "summer", frozenset({"masculine"}))])
        self.assertNotIn("the", self.senses)

    def test_spec_scenario_what_a_cefr_list_leaves_out(self):
        forms = {}
        for lemma, why in [
            ("paris", "name"),
            ("france", "name"),
            ("the", "unknown"),
            ("b", "letter"),
            ("e", "letter"),
            ("etre", "spelling"),
            ("parceque", "spelling"),
            # A spelling whose other entry is a name's: every sense but the name's spells another word.
            ("orient", "spelling"),
            ("à", None),
            ("y", None),
            ("maison", None),
        ]:
            self.assertEqual(red.no_level(lemma, forms, self.senses), why, lemma)

    def test_spec_scenario_a_lemma_whose_own_form_reads_as_another(self):
        # `donnée` ranked while the forms table maps it to *donner*: the builder would key its level on
        # donner. It takes none; donner keeps its own rank's level, and the next lemma takes the place.
        ranks = {"de": 1, "donner": 2, "donnée": 3, "maison": 4, "y": 5}
        forms = {"de": "de", "donner": "donner", "donnée": "donner", "données": "donner", "maison": "maison", "y": "y"}
        self.assertEqual(red.no_level("donnée", forms, self.senses), "elsewhere")
        levels, left_out = red.estimated_levels(ranks, forms, self.senses, bands=(("A1", 2), ("A2", 2)))
        self.assertEqual(levels, {"de": "A1", "donner": "A1", "maison": "A2", "y": "A2"})
        self.assertEqual(left_out["elsewhere"], ["donnée"])

    def test_du_and_des_are_words_of_their_own(self):
        # M21: whatever the section's senses say — none at all here — `du` and `des` take a level.
        ranks = {"de": 1, "des": 2, "du": 3}
        levels, _ = red.estimated_levels(ranks, {}, {"de": self.senses["de"]}, bands=(("A1", 3),))
        self.assertEqual(levels, {"de": "A1", "des": "A1", "du": "A1"})

    def test_the_bands_go_in_rank_order_to_the_lemmas_a_cefr_list_would_hold(self):
        ranks = {"de": 1, "the": 2, "paris": 3, "à": 4, "b": 5, "y": 6, "etre": 7, "maison": 8, "été": 9, "e": 10, "porter": 11}
        levels, left_out = red.estimated_levels(ranks, {}, self.senses, bands=(("A1", 2), ("A2", 1), ("B1", 2)))
        self.assertEqual(levels, {"de": "A1", "à": "A1", "y": "A2", "maison": "B1", "été": "B1"})
        # Each rule's lemmas, up to the last level given: `e` and `porter` come after it.
        self.assertEqual(
            left_out,
            {"unknown": ["the"], "name": ["paris"], "elsewhere": [], "letter": ["b"], "spelling": ["etre"], "unlisted": []},
        )
        # Equal ranks go by the lemma, so the order never depends on the source's.
        tied, _ = red.estimated_levels({"maison": 1, "été": 1}, {}, self.senses, bands=(("A1", 1), ("A2", 1)))
        self.assertEqual(tied, {"maison": "A1", "été": "A2"})
        # Too few lemmas: the bands stop where they run out.
        short, _ = red.estimated_levels({"de": 1}, {}, self.senses)
        self.assertEqual(short, {"de": "A1"})

    def test_english_bands_hold_its_8302_cefr_lemmas_as_es_fr_s_do(self):
        self.assertEqual([level for level, _ in red.ENGLISH_BANDS], ["A1", "A2", "B1", "B2", "C1", "C2"])
        self.assertEqual([size for _, size in red.ENGLISH_BANDS], [1020, 1158, 2015, 2347, 886, 876])
        self.assertEqual(sum(size for _, size in red.ENGLISH_BANDS), 8302)
        spec = importlib.util.spec_from_file_location("reduce_es_fr", os.path.join(_HERE, "reduce-es-fr.py"))
        es_fr = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(es_fr)
        self.assertEqual(red.ENGLISH_BANDS, es_fr.ENGLISH_BANDS)

    def test_spec_scenario_the_table_does_not_wait_for_the_glosses(self):
        # The derivation reads the ranks, the forms, the section's senses and French's dictionary
        # words (refine-lingua-fr-en-glosses D3): a gloss table moves it only through the dictionary
        # words — a change to fr-en's glosses that adds or removes none leaves the levels as they are,
        # and another pair's glosses never reach it.
        self.assertEqual(
            list(inspect.signature(red.estimated_levels).parameters), ["ranks", "forms", "senses", "words", "bands"]
        )
        ranks = {"de": 1, "maison": 2, "y": 3}
        words = {"de", "maison", "y"}
        bands = (("A1", 2), ("A2", 1))
        self.assertEqual(
            red.estimated_levels(ranks, {}, self.senses, words, bands),
            red.estimated_levels(ranks, {}, self.senses, None, bands),
        )

    def test_the_report_names_each_level_s_span_and_each_rule(self):
        ranks = {"de": 1, "the": 2, "à": 3, "y": 4}
        levels, left_out = red.estimated_levels(ranks, {}, self.senses, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(
            red.levels_report(levels, left_out, ranks),
            "levels=3 estimated: A1 2 (1–3); A2 1 (4–4); left out: unknown 1, name 0, elsewhere 0, letter 0, spelling 0, "
            "unlisted 0",
        )


class EstimatedLevelsReduced(unittest.TestCase):
    """The reduction of the fixture section, the levels' entries added and wordfreq ranking them."""

    LEVEL_FREQ = {"y": 6.3, "the": 5.0, "france": 5.8, "b": 4.8, "e": 4.9, "etre": 3.6, "maison": 5.4}

    @classmethod
    def setUpClass(cls):
        top = [w for w in sorted({**FREQ, **cls.LEVEL_FREQ}, key=lambda w: (-{**FREQ, **cls.LEVEL_FREQ}[w], w)) if "-" not in w]
        with mock.patch.dict(FREQ, cls.LEVEL_FREQ), mock.patch.object(sys.modules[__name__], "TOP", top):
            cls.out = Main.run_main(cls, [*ENTRIES, *LEVEL_ENTRIES])
        cls.levels = dict(line.split("\t") for line in cls.out["level.tsv"].splitlines())
        cls.ranks = dict(line.split("\t") for line in cls.out["freq.tsv"].splitlines())

    def test_spec_scenario_the_commonest_words_are_a1(self):
        for word in ["de", "le", "à", "y", "être", "du", "des"]:
            self.assertEqual(self.levels.get(word), "A1", word)
        for word in ["au", "aux"]:
            self.assertNotIn(word, self.ranks)
            self.assertNotIn(word, self.levels)

    def test_what_a_cefr_list_leaves_out_takes_no_level(self):
        for word in ["paris", "france", "the", "b", "e", "etre"]:
            self.assertIn(word, self.ranks, word)
            self.assertNotIn(word, self.levels, word)

    def test_spec_scenario_english_s_sizes(self):
        # Every levelled lemma is one of French's dictionary words, which the reduction writes
        # (refine-lingua-fr-en-glosses D2, D3): fr-en's glossed lemmas less its names alone.
        words = self.out["lexical.tsv"].splitlines()
        self.assertEqual(words, sorted(words))
        glossed = dict(line.split("\t", 1) for line in self.out["gloss.tsv"].splitlines())
        runs = {line.split("\t")[0]: line.split("\t")[1:] for line in self.out["senses.tsv"].splitlines()}
        self.assertEqual(words, sorted(w for w in glossed if not all(r.startswith("PROPN:") for r in runs[w])))
        for lemma in self.levels:
            self.assertIn(lemma, words)
        self.assertEqual(sum(size for _, size in red.ENGLISH_BANDS), 8302)

    def test_every_level_is_a_ranked_lemma_s_and_its_own_form_s(self):
        forms = dict(line.split("\t") for line in self.out["forms.tsv"].splitlines())
        for lemma in self.levels:
            self.assertIn(lemma, self.ranks)
            self.assertEqual(forms[lemma], lemma)
        rows = self.out["level.tsv"].splitlines()
        self.assertEqual(rows, sorted(rows))

    def test_spec_scenario_every_french_pack_says_so(self):
        manifest = json.loads(self.out["manifest.json"])
        self.assertIs(manifest["meta"]["levels_estimated"], True)
        self.assertIn("The levels are estimated, not taken from a CEFR list", self.out["NOTICE"])
        self.assertIn("the English Wiktionary's French section saying which lemmas take", self.out["NOTICE"].replace("\n", " "))
        self.assertNotIn("FLELex", self.out["NOTICE"])



# — fr-en's native side (add-lingua-pack-fr-en) —
#
# Recorded from the section fr-en pins (the English Wiktionary's French section derived from the
# dump of 2026-10-03), as `native_fields` cuts it: word, part of speech, and each sense's glosses,
# tags and pointers. VENIR and CAVALIER keep their entry's first senses only.
CHAMBRE = {
    "word": "chambre",
    "pos": "noun",
    "senses": [
        {"glosses": ["a chamber in its various senses, including:", "a room."], "tags": ["feminine"]},
        {"glosses": ["a chamber in its various senses, including:", "a hotel room."], "tags": ["feminine"]},
        {"glosses": ["a chamber in its various senses, including:", "a bedroom."], "tags": ["feminine"]},
        {"glosses": ["a chamber in its various senses, including:", "a house of a parliament."], "tags": ["feminine"]},
    ],
}
NOUS = [
    {
        "word": "nous",
        "pos": "pron",
        "senses": [
            {"glosses": ["the plural personal pronoun in the first person:", "we"], "tags": ["first-person", "plural", "pronoun", "subjective"]},
            {"glosses": ["the plural personal pronoun in the first person:", "us, to us"], "tags": ["first-person", "plural"]},
            {"glosses": ["we (as the royal we)"], "tags": ["first-person", "historical", "plural"]},
        ],
    },
    {"word": "nous", "pos": "noun", "senses": [{"glosses": ["the nous, (divine) reason in philosophy"], "tags": ["invariable", "masculine"]}]},
]
DU = [
    {
        "word": "du",
        "pos": "contraction",
        "senses": [
            {
                "glosses": ["contraction of de + le, literally “of the”"],
                "tags": ["abbreviation", "alt-of", "contraction"],
                "alt_of": [{"word": "de", "extra": "+ le, literally “of the”"}],
            }
        ],
    },
    {"word": "du", "pos": "article", "senses": [{"glosses": ["Forms the partitive article."], "tags": ["masculine", "singular"]}]},
]
# A pronoun's letter homograph: « elle », the letter L.
ELLE = [
    {
        "word": "elle",
        "pos": "pron",
        "senses": [
            {"glosses": ["she"], "tags": ["feminine", "singular", "third-person"]},
            {"glosses": ["it (feminine gender third-person singular subject pronoun)"], "tags": ["feminine", "singular", "third-person"]},
            {
                "glosses": ["disjunctive form of elle; her, it; à elle = hers, its"],
                "tags": ["disjunctive", "feminine", "form-of", "singular", "third-person"],
                "form_of": [{"word": "elle", "extra": "her, it; à elle = hers, its"}],
            },
        ],
    },
    {"word": "elle", "pos": "noun", "senses": [{"glosses": ["The name of the Latin script letter L/l."], "tags": ["masculine"]}]},
]
# Letters' entries, and words written under a single capital letter: a stool and a chess piece.
LETTERS = [
    {"word": "x", "pos": "character", "senses": [{"glosses": ["The twenty-fourth letter of the French alphabet, written in the Latin script."], "tags": ["letter", "lowercase"]}]},
    {"word": "X", "pos": "noun", "senses": [{"glosses": ["X-frame stool"], "tags": ["feminine", "invariable", "masculine"]}]},
    {
        "word": "C",
        "pos": "noun",
        "senses": [
            {
                "glosses": ["abbreviation of cavalier (“knight”): N"],
                "tags": ["abbreviation", "alt-of"],
                "alt_of": [{"word": "cavalier", "extra": "(“knight”): N"}],
            }
        ],
    },
    {"word": "cavalier", "pos": "noun", "senses": [{"glosses": ["horseman", "knight"], "tags": ["masculine"]}]},
]
NOMBRE_D_OXYDATION = {"word": "nombre d’oxydation", "pos": "noun", "senses": [{"glosses": ["Oxidation number."], "tags": ["masculine"]}]}
VENIR = {
    "word": "venir",
    "pos": "verb",
    "senses": [{"glosses": ["to come (to move from one place to another that is nearer the speaker)"], "tags": ["intransitive"]}],
}
VENUE = [
    {"word": "venue", "pos": "noun", "senses": [{"glosses": ["coming, arrival"], "tags": ["feminine"]}]},
    {
        "word": "venue",
        "pos": "verb",
        "senses": [{"glosses": ["feminine singular of venu"], "tags": ["feminine", "form-of", "participle", "singular"], "form_of": [{"word": "venu"}]}],
    },
]
# French's expressions (D11): words the tokenisation splits, a form, a name, the senses that only point.
SPLIT = [
    {"word": "d'abord", "pos": "adv", "senses": [{"glosses": ["first, at first, right away"]}, {"glosses": ["primarily"]}, {"glosses": ["for one thing"]}]},
    {"word": "allez-y", "pos": "phrase", "senses": [{"glosses": ["go ahead; go on"], "tags": ["formal", "plural", "singular"]}]},
    {"word": "aujourd'hui", "pos": "adv", "senses": [{"glosses": ["today"]}, {"glosses": ["nowadays"]}]},
    {"word": "Jean-Pierre", "pos": "name", "senses": [{"glosses": ["a male given name, a popular combination of Jean and Pierre."], "tags": ["masculine"]}]},
    {"word": "jusqu'à", "pos": "prep", "senses": [{"glosses": ["until"]}, {"glosses": ["to (used together with depuis to indicate a time range)"]}, {"glosses": ["up to"]}]},
    {"word": "jusqu'au", "pos": "contraction", "senses": [{"glosses": ["jusque + au"], "tags": ["contraction"]}]},
    {"word": "qu'elle", "pos": "contraction", "senses": [{"glosses": ["que + elle"], "tags": ["contraction"]}]},
    {
        "word": "m'a",
        "pos": "contraction",
        "senses": [
            {"glosses": ["me + a (third-person singular indicative present form of avoir)"], "tags": ["contraction"]},
            {"glosses": ["“I'm going”"], "tags": ["contraction"]},
        ],
    },
]
POINTERS = [
    {"word": "crème fraiche", "pos": "noun", "senses": [{"glosses": ["post-1990 spelling of crème fraîche"], "tags": ["feminine"]}]},
    {
        "word": "crème fraîche",
        "pos": "noun",
        "senses": [
            {"glosses": ["crème fraîche, slightly sour thick soured cream (also called crème fraîche épaisse)"], "tags": ["feminine"]},
            {"glosses": ["fresh liquid cream (also called crème fraîche liquide or crème fleurette)"], "tags": ["feminine"]},
        ],
    },
    {
        "word": "y a-t-il",
        "pos": "verb",
        "senses": [{"glosses": ["subject-inverted form of il y a; is there? are there?, (after a modal adverb) there is, there are"]}],
    },
    {"word": "maitre-nageuse", "pos": "noun", "senses": [{"glosses": ["post-1990 spelling of maître-nageuse; female equivalent of maitre-nageur"], "tags": ["feminine"]}]},
]
A_LA = [
    {
        "word": "à la",
        "pos": "prep",
        "senses": [
            {"glosses": ["Used other than figuratively or idiomatically: see à, la."]},
            {"glosses": ["a la, in the style or manner of (with a feminine singular adjective or a proper noun)"]},
        ],
    },
    {
        "word": "à la carte",
        "pos": "adv",
        "senses": [
            {"glosses": ["à la carte (allowing selection only from a fixed list of options, typically shown on a menu)"]},
            {"glosses": ["with each dish priced"]},
        ],
    },
]
NATIVE = [CHAMBRE, *NOUS, *DU, *ELLE, *LETTERS, NOMBRE_D_OXYDATION, VENIR, *VENUE, *SPLIT, *POINTERS, *A_LA]
# The ranked lemmas, each its own form's lemma, as change 43's ranks are; `venue` reads as venir, and
# `aujourd'hui` and `jusque` are forms.
NATIVE_RANKS = {"du": 10, "elle": 28, "nous": 34, "chambre": 324, "venir": 388, "x": 434, "c": 900, "aujourd'hui": 950, "jusque": 1200}
NATIVE_FORMS = {
    **{lemma: lemma for lemma in NATIVE_RANKS},
    "venue": "venir",
    "viens": "venir",
    "jusqu'": "jusque",
    # As change 43's forms table reads them: `crème fraiche` is keyed as `crème fraîche`.
    "crème": "crème",
    "fraiche": "frais",
    "fraîche": "frais",
}


class NativeSide(unittest.TestCase):
    """fr-en's native side (add-lingua-pack-fr-en D1, D2, D11) over recorded entries."""

    def native(self, entries=NATIVE, ranks=NATIVE_RANKS, forms=NATIVE_FORMS):
        with tempfile.TemporaryDirectory() as work:
            path = os.path.join(work, "kaikki-French.jsonl")
            with open(path, "w", encoding="utf-8") as f:
                f.write("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries))
                f.write("not json\n[1, 2]\n")
            # No treebank count: the parts of speech stay in the page's order (`ReadAsFrench` below).
            return red.native_side(work, path, ranks, forms, collections.Counter())

    @classmethod
    def setUpClass(cls):
        cls.glosses, cls.runs, cls.expressions, cls.stats = cls.native(cls)

    def test_spec_scenario_a_sense_group_label(self):
        gloss = self.glosses["chambre"]
        self.assertTrue(gloss.startswith("a room; a hotel room"), gloss)
        self.assertNotIn("in its various senses", gloss)
        self.assertEqual(self.runs["chambre"], [("NOUN", 4)])

    def test_spec_scenario_a_pronoun_s_senses_under_its_description(self):
        self.assertTrue(self.glosses["nous"].startswith("we; us, to us"), self.glosses["nous"])
        self.assertEqual(self.runs["nous"], [("PRON", 3), ("NOUN", 1)])

    def test_spec_scenario_the_edition_s_description_in_lower_case(self):
        # The article's description in lower case; the contraction's pointer carries its meaning,
        # « of the » (refine-lingua-fr-en-glosses D4), a run of its own (kaikki's `contraction`), in
        # the page's order without the treebank's counts (`ReadAsFrench` reads them).
        self.assertEqual(self.glosses["du"], "of the; forms the partitive article")
        self.assertEqual(self.runs["du"], [("X", 1), ("DET", 1)])

    def test_a_letter_s_sense_and_a_single_capital_headword_gloss_nothing(self):
        # `elle`, the letter L, is no sense of the pronoun; its case form reads as its meaning (23b's D3).
        self.assertEqual(self.glosses["elle"], "she; it (feminine gender third-person singular subject pronoun); her, it, à elle = hers, its")
        self.assertNotIn("letter", self.glosses["elle"])
        # `X` « X-frame stool » and `C` « abbreviation of cavalier » gloss neither `x` nor `c`, which
        # would borrow the chess piece's gloss; a letter's entry glosses nothing.
        for letter in ("x", "c"):
            self.assertNotIn(letter, self.glosses)

    def test_a_headword_s_typographic_apostrophe_is_keyed_straight(self):
        self.assertEqual(self.expressions["nombre d'oxydation"], "Oxidation number")
        self.assertNotIn("nombre d’oxydation", self.expressions)
        self.assertFalse([w for w in [*self.glosses, *self.expressions] if "’" in w])

    def test_only_ranked_lemmas_are_glossed_each_by_its_own_entries(self):
        self.assertTrue(set(self.glosses) <= set(NATIVE_RANKS))
        # `venue` reads as venir: not ranked, its own noun's « coming, arrival » glosses nothing, and
        # venir is glossed by venir's entries alone — no gloss is lent to another word.
        self.assertNotIn("venue", self.glosses)
        self.assertEqual(self.glosses["venir"], "to come (to move from one place to another that is nearer the speaker)")
        self.assertNotIn("arrival", " ".join(self.glosses.values()))

    def test_spec_scenario_a_word_the_pre_pass_splits(self):
        self.assertEqual(self.expressions["d'abord"], "first, at first, right away; primarily; for one thing")
        self.assertEqual(self.expressions["allez-y"], "go ahead; go on")
        self.assertEqual(self.expressions["jusqu'à"], "until; to (used together with depuis to indicate; up to")
        # A form is looked up whole, and a name's entry glosses nothing.
        self.assertNotIn("aujourd'hui", self.expressions)
        self.assertNotIn("jean-pierre", self.expressions)
        self.assertEqual(self.glosses["aujourd'hui"], "today; nowadays")
        self.assertEqual(self.stats["split"], 4)

    def test_spec_scenario_an_expression_that_only_points(self):
        for pointer in ("crème fraiche", "qu'elle", "jusqu'au"):
            self.assertNotIn(pointer, self.expressions)
        self.assertTrue(self.expressions["crème fraîche"].startswith("crème fraîche, slightly sour thick soured"))
        self.assertEqual(self.expressions["y a-t-il"], "is there? are there?, (after a modal adver")
        # A sense that only points goes; the word's other senses stay.
        self.assertEqual(self.expressions["m'a"], "“I'm going”")
        # What a pointer writes after its target is read by the shared rules: another pointer stays one.
        self.assertNotIn("maitre-nageuse", self.expressions)

    def test_spec_scenario_a_la(self):
        self.assertNotIn("à la", self.expressions)
        self.assertEqual(self.expressions["à la carte"], "à la carte (allowing selection only from a; with each dish priced")
        self.assertIn("à la", red.LEFT_OUT)
        self.assertIn("in the style or manner of", red.LEFT_OUT["à la"])

    def test_the_expression_rules_leave_the_glosses_and_runs_as_they_are(self):
        # Without the pass (D11, 2 and 3) the glosses and their runs are byte for byte the same: it
        # reads no lemma's entry. The expressions alone move.
        def unchanged(src, dst, forms):
            return src

        with mock.patch.object(red, "expression_senses", unchanged):
            glosses, runs, expressions, _ = self.native()
        self.assertEqual((glosses, runs), (self.glosses, self.runs))
        self.assertIn("à la", expressions)
        self.assertIn("crème fraiche", expressions)

    def test_the_section_is_read_through_the_english_edition_s_pre_passes_in_es_en_s_order(self):
        calls = []

        def spy(name, real):
            def call(src, dst, *args, **kwargs):
                calls.append((name, os.path.basename(src), os.path.basename(dst)))
                return real(src, dst, *args, **kwargs)

            return call

        with (
            mock.patch.object(red.common, "without_letter_senses", spy("letters", red.common.without_letter_senses)),
            mock.patch.object(red.english, "without_letter_headwords", spy("headwords", red.english.without_letter_headwords)),
            mock.patch.object(red.english, "read_as_meanings", spy("meanings", red.english.read_as_meanings)),
            mock.patch.object(red, "read_as_french", spy("french", red.read_as_french)),
            mock.patch.object(red.english, "merge_same_pos_etymologies", spy("merge", red.english.merge_same_pos_etymologies)),
            mock.patch.object(red, "expression_senses", spy("expressions", red.expression_senses)),
        ):
            self.native()
        self.assertEqual(
            calls,
            [
                ("letters", "kaikki-French-senses.jsonl", "kaikki-French-words.jsonl"),
                ("headwords", "kaikki-French-words.jsonl", "kaikki-French-headwords.jsonl"),
                ("meanings", "kaikki-French-headwords.jsonl", "kaikki-French-meanings.jsonl"),
                # fr-en's own pre-pass (refine-lingua-fr-en-glosses D1), before the merging.
                ("french", "kaikki-French-meanings.jsonl", "kaikki-French-french.jsonl"),
                ("merge", "kaikki-French-french.jsonl", "kaikki-French-merged.jsonl"),
                # The merging is off as committed: it answers its source.
                ("expressions", "kaikki-French-french.jsonl", "kaikki-French-expressions.jsonl"),
            ],
        )
        self.assertEqual(self.stats["dropped"], 2, "the lines that are no JSON object")

    def test_native_fields_keeps_what_the_native_side_reads(self):
        entry = {
            "word": "Nombre d’oxydation",
            "pos": "noun",
            "forms": [{"form": "nombres d'oxydation", "tags": ["plural"]}],
            "head_templates": [{"name": "fr-noun"}],
            "senses": [{"glosses": ["Oxidation number."], "tags": ["masculine"], "examples": [{"text": "…"}], "links": []}, "no sense"],
        }
        with tempfile.TemporaryDirectory() as work:
            src, dst = os.path.join(work, "in.jsonl"), os.path.join(work, "out.jsonl")
            with open(src, "w", encoding="utf-8") as f:
                f.write(json.dumps(entry, ensure_ascii=False) + "\n")
            _, dropped = red.native_fields(src, dst)
            with open(dst, encoding="utf-8") as f:
                cut = json.loads(f.read())
        self.assertEqual(dropped, 0)
        # The case is kept; the apostrophe is read as `'`.
        self.assertEqual(cut, {"word": "Nombre d'oxydation", "pos": "noun", "senses": [{"glosses": ["Oxidation number."], "tags": ["masculine"]}]})

    def test_split_word_and_is_expression(self):
        forms = {"aujourd'hui": "aujourd'hui", "peut-être": "peut-être"}
        for word in ("d'abord", "c'est", "allez-y", "qu'elle"):
            self.assertTrue(red.split_word(word, forms), word)
        for word in ("aujourd'hui", "peut-être", "abord", "il y a", "c’est", "3-d"):
            self.assertFalse(red.split_word(word, forms), word)
        self.assertTrue(red.is_expression("coup d'œil", forms))
        self.assertFalse(red.is_expression("coup d’œil", forms), "a headword the cut has not read")
        self.assertFalse(red.is_expression("w3 c", forms))

    def test_a_meaning_after_a_pointer(self):
        self.assertIsNone(red._meaning_of_pointer({"glosses": ["que + elle"]}))
        self.assertIsNone(red._meaning_of_pointer({"glosses": ["contraction of que + il"]}))
        self.assertIsNone(red._meaning_of_pointer({"glosses": ["post-1990 spelling of crème fraîche"]}))
        self.assertEqual(
            red._meaning_of_pointer({"glosses": ["subject-inverted form of il y a: is there?"], "tags": ["x"]}),
            {"glosses": ["is there?"], "tags": ["x"]},
        )
        # A meaning, a tagged pointer (the shared rules') and a sense kaikki did not shape are kept.
        for sense in ({"glosses": ["until"]}, {"glosses": ["que + elle"], "tags": ["form-of"]}, {"glosses": []}, {"tags": []}):
            self.assertIs(red._meaning_of_pointer(sense), sense)

    def test_spec_scenario_fr_en_s_rule_files(self):
        sys.path.insert(0, _HERE)
        import pack_sources as ps

        # The treebank's rule (refine-lingua-fr-en-glosses D5) is a rule module of its own, in fr-en's
        # digest.
        self.assertEqual(
            [p.name for p in ps.rule_files(ps.Path(_HERE) / "reduce-fr-en.py")],
            ["reduce-fr-en.py", "reduce_common.py", "reduce_edition_en.py", "reduce_french_treebank.py"],
        )


class NativeSideReduced(unittest.TestCase):
    """`main` over the fixture section, with the translation files change 38's catalogue registered
    for fr-en beside it: the reducer reads neither (D3)."""

    @classmethod
    def setUpClass(cls):
        top = [w for w in sorted({**FREQ, "end": 5.0}, key=lambda w: (-{**FREQ, "end": 5.0}[w], w)) if "-" not in w]
        files = {
            # The French Wiktionary's English translations, and the English Wiktionary's French ones.
            "kaikki-fr-traductions-en.jsonl": [
                {"word": "END", "pos": "noun", "translations": [{"word": "NDE"}, {"word": "NDI"}, {"word": "NDT"}]},
                {"word": "rendez-vous", "pos": "noun", "translations": [{"word": "date"}]},
            ],
            "kaikki-en-traductions-fr.jsonl": [{"word": "he's", "pos": "contraction", "translations": [{"word": "il est"}]}],
        }
        with mock.patch.dict(FREQ, {"end": 5.0}), mock.patch.object(sys.modules[__name__], "TOP", top):
            cls.out = Main.run_main(cls, ENTRIES, files=files)
        cls.glossed = dict(line.split("\t") for line in cls.out["gloss.tsv"].splitlines())
        cls.expressions = dict(line.split("\t") for line in cls.out["mwe.tsv"].splitlines())

    def test_spec_scenario_a_word_only_a_translation_table_glosses(self):
        self.assertIn("end\t", self.out["freq.tsv"], "ranked")
        self.assertNotIn("end", self.glossed)
        # A ranked word the section glosses keeps its own gloss, whatever a table lists.
        self.assertEqual(self.glossed["rendez-vous"], "appointment")

    def test_spec_scenario_a_bigram_an_english_entry_translates(self):
        self.assertNotIn("il est", self.expressions)

    def test_every_glossed_lemma_is_ranked_and_its_own_form(self):
        forms = dict(line.split("\t") for line in self.out["forms.tsv"].splitlines())
        ranks = dict(line.split("\t") for line in self.out["freq.tsv"].splitlines())
        for lemma in self.glossed:
            self.assertIn(lemma, ranks)
            self.assertEqual(forms[lemma], lemma)
        self.assertEqual(self.out["senses.tsv"].count("\n"), len(self.glossed))
        for name in ("gloss.tsv", "senses.tsv", "mwe.tsv"):
            rows = self.out[name].splitlines()
            self.assertEqual(rows, sorted(rows), name)


# — fr-en read as French (refine-lingua-fr-en-glosses) —
#
# Recorded from the section fr-en pins (lingua-pack-sources-fr-en-2026.10.09), as `native_fields`
# cuts it, in the section's order: every entry of the words each rule is measured on, whole.
FRENCH_SECTION = [
    {"word": "on", "pos": "pron", "senses": [{"glosses": ["one, people, you, someone (an unspecified individual)"], "tags": ["feminine", "indefinite", "masculine", "plural"]}, {"glosses": ["we"], "tags": ["feminine", "informal", "masculine", "personal", "plural"]}]},
    {"word": "y", "pos": "character", "senses": [{"glosses": ["a letter in the French alphabet, after x and before z"], "tags": ["letter", "lowercase"]}]},
    {"word": "y", "pos": "pron", "senses": [{"glosses": ["there (at a place)"], "tags": ["adverbial"]}, {"glosses": ["there, thither (to there)"], "tags": ["adverbial"]}, {"glosses": ["Used as a pronoun to replace an adverbial phrase starting with à."], "tags": ["adverbial"]}, {"glosses": ["Used as a pronoun to replace an adverbial phrase starting with à.", "With verbs: see Appendix:French verbs followed by à for verbs which use this structure."], "tags": ["adverbial"]}, {"glosses": ["Used as a pronoun to replace an adverbial phrase starting with à.", "With adjectives. Only used with a handful of adjectives (the most common combination being y compris, which is a special case), mainly in legal terminology."], "tags": ["adverbial", "archaic"]}]},
    {"word": "y", "pos": "pron", "senses": [{"glosses": ["alternative form of il; he"], "tags": ["Quebec", "alt-of", "alternative", "colloquial"], "alt_of": [{"word": "il", "extra": "he"}]}, {"glosses": ["alternative form of ils; they (male)"], "tags": ["Quebec", "alt-of", "alternative", "colloquial"], "alt_of": [{"word": "ils", "extra": "they (male)"}]}, {"glosses": ["alternative form of elles; they (female)"], "tags": ["Quebec", "alt-of", "alternative", "colloquial"], "alt_of": [{"word": "elles", "extra": "they (female)"}]}]},
    {"word": "en", "pos": "prep", "senses": [{"glosses": ["in (used to indicate space, also see usage notes)"]}, {"glosses": ["to (indicates direction towards certain very large locations, see usage notes)"]}, {"glosses": ["by (used to indicate means)"]}, {"glosses": ["as"]}, {"glosses": ["at (used to describe an ability)"]}, {"glosses": ["of, made of (used to describe composition)"]}, {"glosses": ["in (during the following time (used for months and years))"]}, {"glosses": ["while"]}, {"glosses": ["by, in (describing a way of getting something)"]}, {"glosses": ["in (used to describe color)"]}, {"glosses": ["in (used to describe feelings)"]}, {"glosses": ["in (as part of something)"]}]},
    {"word": "en", "pos": "pron", "senses": [{"glosses": ["Used as the object of a verb to indicate an indefinite quantity; of it, of them. Replaces the partitive article (du, de la, etc.)"]}, {"glosses": ["Adverbial preposition indicating movement away from a place already mentioned; from there, from it. Replaces the phrase de là or d’ici."]}]},
    {"word": "son", "pos": "noun", "senses": [{"glosses": ["sound"], "tags": ["masculine"]}, {"glosses": ["A piece (of music); a (musical) work; an opus."], "tags": ["masculine", "slang"]}]},
    {"word": "son", "pos": "det", "senses": [{"glosses": ["his, her, their, its (used to qualify masculine nouns and before a vowel)"], "tags": ["masculine", "possessive"]}]},
    {"word": "son", "pos": "noun", "senses": [{"glosses": ["bran"], "tags": ["masculine"]}]},
    {"word": "NE", "pos": "noun", "senses": [{"glosses": ["abbreviation of nord-est; NE"], "tags": ["abbreviation", "alt-of"], "alt_of": [{"word": "nord-est", "extra": "NE"}]}]},
    {"word": "NE", "pos": "name", "senses": [{"glosses": ["ISO 3166-2:CH code of Neuchâtel (canton)"]}]},
    {"word": "bien", "pos": "adj", "senses": [{"glosses": ["good, all right, great"], "tags": ["invariable"]}, {"glosses": ["good looking, nice"], "tags": ["invariable"]}]},
    {"word": "bien", "pos": "adv", "senses": [{"glosses": ["well"]}, {"glosses": ["indeed; so"]}, {"glosses": ["a lot (of)"]}, {"glosses": ["very; really"]}, {"glosses": ["much (more, less, better, etc.)"]}, {"glosses": ["Used to confirm or ask for confirmation"]}]},
    {"word": "bien", "pos": "noun", "senses": [{"glosses": ["good as opposed to evil"], "tags": ["masculine"]}, {"glosses": ["a commodity, a good"], "tags": ["masculine"]}, {"glosses": ["a possession"], "tags": ["masculine"]}]},
    {"word": "du", "pos": "contraction", "senses": [{"glosses": ["contraction of de + le, literally “of the”"], "tags": ["abbreviation", "alt-of", "contraction"], "alt_of": [{"word": "de", "extra": "+ le, literally “of the”"}]}]},
    {"word": "du", "pos": "article", "senses": [{"glosses": ["Forms the partitive article."], "tags": ["masculine", "singular"]}]},
    {"word": "Paris", "pos": "name", "senses": [{"glosses": ["Paris (the capital and largest city of France)"], "tags": ["feminine", "masculine"]}, {"glosses": ["Paris (a department of Île-de-France, France)"], "tags": ["feminine", "masculine"]}]},
    {"word": "Paris", "pos": "name", "senses": [{"glosses": ["a common surname"], "tags": ["feminine", "masculine"]}]},
    {"word": "ne", "pos": "particle", "senses": [{"glosses": ["not (used alone to negate a verb, now chiefly with only a few particular verbs; see usage notes)"], "tags": ["literary"]}, {"glosses": ["not, no (used before a verb, with a coordinating negative element usually following; see usage notes)"]}, {"glosses": ["used in a subordinate clause before a subjunctive verb (especially when the main verb expresses doubt or fear) to provide extra overtones of doubt or uncertainty (but not negating its verb); the so-called \"pleonastic\" or \"expletive\" ne"]}, {"glosses": ["in comparative clauses usually translated with the positive sense of the subsequent negative"]}]},
    {"word": "nice", "pos": "adj", "senses": [{"glosses": ["candid, naive"], "tags": ["archaic"]}]},
    {"word": "Marche", "pos": "name", "senses": [{"glosses": ["Marche (a department of France)"], "tags": ["feminine"]}]},
    {"word": "ouais", "pos": "intj", "senses": [{"glosses": ["synonym of oui; yeah, yep, yup, yes, affirmative expression"], "tags": ["informal"]}, {"glosses": ["synonym of oui (used to express consent)"], "tags": ["informal"]}, {"glosses": ["wow"], "tags": ["dated"]}, {"glosses": ["whoo (expresses joy)"], "tags": ["informal"]}]},
    {"word": "il", "pos": "pron", "senses": [{"glosses": ["he (third-person singular masculine subject pronoun for human subject)"], "tags": ["masculine", "singular", "third-person"]}, {"glosses": ["it (third-person singular subject pronoun for grammatically masculine objects)"], "tags": ["masculine", "singular", "third-person"]}, {"glosses": ["Impersonal subject; it"], "tags": ["impersonal", "masculine", "pronoun", "singular", "third-person"]}]},
    {"word": "le", "pos": "article", "senses": [{"glosses": ["the (definite article)"], "tags": ["masculine"]}, {"glosses": ["the; my, your, etc."], "tags": ["masculine"]}, {"glosses": ["a, an, per"], "tags": ["masculine"]}, {"glosses": ["on"], "tags": ["masculine"]}]},
    {"word": "le", "pos": "pron", "senses": [{"glosses": ["him, her, it, them"], "tags": ["direct-object", "masculine"]}, {"glosses": ["replaces the argument of a copular verb, especially être; often not translated in English"], "tags": ["masculine"]}]},
    {"word": "coup d'œil", "pos": "noun", "senses": [{"glosses": ["glance, look"], "tags": ["masculine"]}, {"glosses": ["sight"], "tags": ["masculine"]}]},
    {"word": "des", "pos": "article", "senses": [{"glosses": ["plural of un (“some”, the plural indefinite article)"], "tags": ["feminine", "form-of", "masculine", "plural"], "form_of": [{"word": "un", "extra": "“some”, the plural indefinite article"}]}, {"glosses": ["plural of une (“some”, the plural indefinite article)"], "tags": ["feminine", "form-of", "masculine", "plural"], "form_of": [{"word": "une", "extra": "“some”, the plural indefinite article"}]}, {"glosses": ["plural of du (“some”, the plural partitive article)"], "tags": ["feminine", "form-of", "masculine", "plural"], "form_of": [{"word": "du", "extra": "“some”, the plural partitive article"}]}, {"glosses": ["plural of de la (“some”, the plural partitive article)"], "tags": ["feminine", "form-of", "masculine", "plural"], "form_of": [{"word": "de la", "extra": "“some”, the plural partitive article"}]}, {"glosses": ["plural of de l' (“some”, the plural partitive article)"], "tags": ["feminine", "form-of", "masculine", "plural"], "form_of": [{"word": "de l'", "extra": "“some”, the plural partitive article"}]}]},
    {"word": "des", "pos": "contraction", "senses": [{"glosses": ["contraction of de + les, literally “of the, from the, some”"], "tags": ["abbreviation", "alt-of", "contraction"], "alt_of": [{"word": "de", "extra": "+ les, literally “of the, from the, some”"}]}]},
    {"word": "parce que", "pos": "conj", "senses": [{"glosses": ["because"]}]},
    {"word": "mort", "pos": "verb", "senses": [{"glosses": ["past participle of mourir"], "tags": ["form-of", "participle", "past"], "form_of": [{"word": "mourir"}]}]},
    {"word": "mort", "pos": "adj", "senses": [{"glosses": ["dead"]}]},
    {"word": "mort", "pos": "noun", "senses": [{"glosses": ["dead person"], "tags": ["masculine"]}]},
    {"word": "mort", "pos": "noun", "senses": [{"glosses": ["death"], "tags": ["feminine"]}]},
    {"word": "pas", "pos": "noun", "senses": [{"glosses": ["step, pace, footstep"], "tags": ["invariable", "masculine"]}, {"glosses": ["strait, pass"], "tags": ["invariable", "masculine"]}, {"glosses": ["thread, pitch (of a screw or nut)"], "tags": ["invariable", "masculine"]}]},
    {"word": "pas", "pos": "adv", "senses": [{"glosses": ["The most common adverb of negation in French, typically translating into English as not, don't, doesn't, etc."]}, {"glosses": ["used as an intensifier in underlying rhetorical questions, mostly with voilà"], "tags": ["colloquial"]}]},
    {"word": "lot", "pos": "noun", "senses": [{"glosses": ["share (of inheritance)"], "tags": ["masculine"]}, {"glosses": ["plot (of land)"], "tags": ["masculine"]}, {"glosses": ["batch (of goods for sale)"], "tags": ["masculine"]}, {"glosses": ["lot (at auction)"], "tags": ["masculine"]}, {"glosses": ["prize (in lottery)"], "tags": ["masculine"]}, {"glosses": ["lot, fate"], "tags": ["masculine"]}, {"glosses": ["babe"], "tags": ["masculine", "slang"]}]},
    {"word": "jean", "pos": "noun", "senses": [{"glosses": ["a pair of jeans"], "tags": ["masculine"]}]},
    {"word": "mieux", "pos": "adv", "senses": [{"glosses": ["comparative degree of bien; better"], "tags": ["comparative", "form-of"], "form_of": [{"word": "bien", "extra": "better"}]}, {"glosses": ["superlative degree of bien; best"], "tags": ["form-of", "superlative", "with-definite-article"], "form_of": [{"word": "bien", "extra": "best"}]}, {"glosses": ["more, -er."]}]},
    {"word": "mieux", "pos": "noun", "senses": [{"glosses": ["the best of one's ability, one's best"], "tags": ["invariable", "masculine"]}]},
    {"word": "a priori", "pos": "adj", "senses": [{"glosses": ["intuitively known, a priori"], "tags": ["invariable"]}]},
    {"word": "a priori", "pos": "adv", "senses": [{"glosses": ["at first glance"], "tags": ["informal"]}]},
    {"word": "a priori", "pos": "noun", "senses": [{"glosses": ["preconceived idea"], "tags": ["invariable", "masculine"]}]},
    {"word": "mon", "pos": "det", "senses": [{"glosses": ["my (used to qualify masculine nouns and vowel-initial words regardless of gender)"], "tags": ["masculine", "possessive"]}, {"glosses": ["Followed by rank, obligatory way of addressing a (male) superior officer within the military. (Folk etymology: military-specific short for \"monsieur\".)"], "tags": ["masculine"]}]},
    {"word": "que", "pos": "conj", "senses": [{"glosses": ["that (introduces a subordinate noun clause and connects it to its parent clause)"]}, {"glosses": ["Substitutes for another, previously stated conjunction."]}, {"glosses": ["when, no sooner"]}, {"glosses": ["Links two noun phrases in apposition forming a clause without a (finite) verb, such that the complement acts as predicate."]}]},
    {"word": "que", "pos": "conj", "senses": [{"glosses": ["introduces a comparison", "than"]}, {"glosses": ["introduces a comparison", "as"]}, {"glosses": ["only, just; but, nothing but"]}, {"glosses": ["how (in rhetorical interjections)"]}]},
    {"word": "que", "pos": "pron", "senses": [{"glosses": ["The inanimate direct-object or predicative interrogative pronoun: what"], "tags": ["interrogative", "masculine"]}, {"glosses": ["The inanimate subject interrogative pronoun in impersonal constructions."], "tags": ["interrogative", "masculine"]}, {"glosses": ["The inanimate subject interrogative pronoun."], "tags": ["interrogative", "masculine", "nominative"]}]},
    {"word": "que", "pos": "pron", "senses": [{"glosses": ["The direct object relative pronoun."], "tags": ["accusative", "feminine", "interrogative", "masculine", "relative"]}]},
    {"word": "directeur", "pos": "noun", "senses": [{"glosses": ["director"], "tags": ["masculine"]}, {"glosses": ["school principal"], "tags": ["masculine"]}]},
    {"word": "directeur", "pos": "adj", "senses": [{"glosses": ["leading, guiding"]}]},
    {"word": "même", "pos": "adv", "senses": [{"glosses": ["even"]}]},
    {"word": "même", "pos": "adj", "senses": [{"glosses": ["same"]}, {"glosses": ["very"]}]},
    {"word": "devoir", "pos": "noun", "senses": [{"glosses": ["duty"], "tags": ["masculine"]}, {"glosses": ["exercise, assignment (set for homework)"], "tags": ["masculine"]}]},
    {"word": "devoir", "pos": "verb", "senses": [{"glosses": ["must, to have to, should (as a requirement)"]}, {"glosses": ["must, to have to, should (as a requirement)", "must"], "tags": ["present"]}, {"glosses": ["must, to have to, should (as a requirement)", "should"], "tags": ["conditional"]}, {"glosses": ["must, to do or have with certainty"]}, {"glosses": ["to owe (money, obligation and etc)"], "tags": ["transitive"]}, {"glosses": ["(even) if it is necessary (+ infinitive)"], "tags": ["intransitive", "literary"]}, {"glosses": ["to have a duty to"], "tags": ["reflexive"]}]},
    {"word": "ferme", "pos": "adj", "senses": [{"glosses": ["firm"]}]},
    {"word": "ferme", "pos": "noun", "senses": [{"glosses": ["roof truss"], "tags": ["feminine"]}]},
    {"word": "ferme", "pos": "verb", "senses": [{"glosses": ["inflection of fermer:", "first/third-person singular present indicative/subjunctive"], "tags": ["first-person", "form-of", "indicative", "present", "singular", "subjunctive", "third-person"], "form_of": [{"word": "fermer"}]}, {"glosses": ["inflection of fermer:", "second-person singular imperative"], "tags": ["form-of", "imperative", "second-person", "singular"], "form_of": [{"word": "fermer"}]}]},
    {"word": "ferme", "pos": "noun", "senses": [{"glosses": ["farm"], "tags": ["feminine"]}]},
    {"word": "leur", "pos": "pron", "senses": [{"glosses": ["(to) them"], "tags": ["feminine", "indirect", "masculine", "personal", "plural"]}]},
    {"word": "leur", "pos": "det", "senses": [{"glosses": ["their"], "tags": ["feminine", "masculine"]}]},
    {"word": "liberté", "pos": "noun", "senses": [{"glosses": ["liberty, freedom. 1688, Guy Miège, The Great French Dictionary. \"Qu'y a-t-il de plus doux dans ce monde que la liberté? What is there sweeter in this world than liberty?\""], "tags": ["countable", "feminine", "uncountable"]}]},
    {"word": "il y a", "pos": "verb", "senses": [{"glosses": ["impersonal singular present indicative of y avoir: there is, there are"], "tags": ["form-of", "impersonal", "indicative", "present", "singular"], "form_of": [{"word": "y avoir", "extra": "there is, there are"}]}]},
    {"word": "il y a", "pos": "prep", "senses": [{"glosses": ["ago"]}]},
    {"word": "quand", "pos": "adv", "senses": [{"glosses": ["when (at what time)"]}]},
    {"word": "quand", "pos": "conj", "senses": [{"glosses": ["when (at the time that)"]}, {"glosses": ["whenever"], "tags": ["Louisiana"]}]},
    {"word": "Jean", "pos": "name", "senses": [{"glosses": ["John (biblical character)."], "tags": ["masculine"]}, {"glosses": ["John (book of the Bible)."], "tags": ["masculine"]}, {"glosses": ["a male given name from Hebrew, equivalent to English John, traditionally very popular in France, also common as the first part of hyphenated given names"], "tags": ["masculine"]}, {"glosses": ["a surname originating as a patronymic"], "tags": ["masculine"]}]},
    {"word": "Lot", "pos": "name", "senses": [{"glosses": ["Lot (a department of Occitania, France)"], "tags": ["masculine"]}, {"glosses": ["Lot (a right tributary of the Garonne, in southern France, flowing through the departments of Lozère, Cantal, Aveyron, Lot and Lot-et-Garonne)"], "tags": ["masculine"]}]},
    {"word": "Nice", "pos": "name", "senses": [{"glosses": ["Nice (a coastal city, the capital of Alpes-Maritimes department in the Provence-Alpes-Côte d'Azur region in southeast France)"], "tags": ["feminine"]}]},
    {"word": "marche", "pos": "noun", "senses": [{"glosses": ["march (formal, rhythmic way of walking)"], "tags": ["feminine"]}, {"glosses": ["march (song in the genre of music written for marching)"], "tags": ["feminine"]}, {"glosses": ["walk (distance walked)"], "tags": ["feminine"]}, {"glosses": ["movement (of a vehicle)"], "tags": ["feminine"]}, {"glosses": ["functioning"], "tags": ["feminine"]}, {"glosses": ["step (step of a stair)"], "tags": ["feminine"]}, {"glosses": ["marches (region near a border)"], "tags": ["feminine"]}]},
    {"word": "marche", "pos": "verb", "senses": [{"glosses": ["inflection of marcher:", "first/third-person singular present indicative/subjunctive"], "tags": ["first-person", "form-of", "indicative", "present", "singular", "subjunctive", "third-person"], "form_of": [{"word": "marcher"}]}, {"glosses": ["inflection of marcher:", "second-person singular imperative"], "tags": ["form-of", "imperative", "second-person", "singular"], "form_of": [{"word": "marcher"}]}]},
    {"word": "paris", "pos": "noun", "senses": [{"glosses": ["plural of pari"], "tags": ["form-of", "masculine", "plural"], "form_of": [{"word": "pari"}]}]},
    {"word": "moins", "pos": "adv", "senses": [{"glosses": ["comparative degree of peu; less, fewer"], "tags": ["comparative", "form-of"], "form_of": [{"word": "peu", "extra": "less, fewer"}]}, {"glosses": ["minus; negative"]}, {"glosses": ["superlative degree of peu; the least"], "tags": ["form-of", "superlative"], "form_of": [{"word": "peu", "extra": "the least"}]}]},
    {"word": "moins", "pos": "noun", "senses": [{"glosses": ["the minus sign"], "tags": ["invariable", "masculine"]}]},
    {"word": "moins", "pos": "prep", "senses": [{"glosses": ["minus"]}]},
    {"word": "parce", "pos": "prep", "senses": [{"glosses": ["only used in parce que"]}]},
    {"word": "consul", "pos": "noun", "senses": [{"glosses": ["consul, in its various senses"], "tags": ["masculine"]}]},
    {"word": "juste", "pos": "adj", "senses": [{"glosses": ["fair, just"]}, {"glosses": ["reasonable, appropriate, grounded"]}, {"glosses": ["correct"]}, {"glosses": ["perfect"], "tags": ["perfect"]}, {"glosses": ["shorter or less than desired; insufficient"]}]},
    {"word": "juste", "pos": "noun", "senses": [{"glosses": ["a righteous person"], "tags": ["masculine"]}]},
    {"word": "juste", "pos": "adv", "senses": [{"glosses": ["exactly, precisely"]}, {"glosses": ["just, only"], "tags": ["informal"]}]},
    {"word": "phare", "pos": "adj", "senses": [{"glosses": ["leading, signature, key, flagship"]}]},
    {"word": "phare", "pos": "noun", "senses": [{"glosses": ["lighthouse"], "tags": ["masculine"]}, {"glosses": ["lantern (in a lighthouse)"], "tags": ["masculine"]}, {"glosses": ["headlight (of a vehicle)"], "tags": ["masculine"]}, {"glosses": ["headlamp (of a vehicle)"], "tags": ["masculine"]}, {"glosses": ["beacon, luminary"], "tags": ["figuratively", "masculine"]}, {"glosses": ["The set of sails on the mast."], "tags": ["masculine"]}]},
    {"word": "contrôle", "pos": "noun", "senses": [{"glosses": ["control (all senses)"], "tags": ["masculine"]}, {"glosses": ["verification, checking"], "tags": ["masculine"]}, {"glosses": ["test"], "tags": ["masculine"]}]},
    {"word": "contrôle", "pos": "verb", "senses": [{"glosses": ["inflection of contrôler:", "first/third-person singular present indicative/subjunctive"], "tags": ["first-person", "form-of", "indicative", "present", "singular", "subjunctive", "third-person"], "form_of": [{"word": "contrôler"}]}, {"glosses": ["inflection of contrôler:", "second-person singular imperative"], "tags": ["form-of", "imperative", "second-person", "singular"], "form_of": [{"word": "contrôler"}]}]},
    {"word": "Coran", "pos": "name", "senses": [{"glosses": ["Koran"], "tags": ["masculine"]}]},
    {"word": "directrice", "pos": "noun", "senses": [{"glosses": ["female equivalent of directeur: directress"], "tags": ["feminine", "form-of"], "form_of": [{"word": "directeur", "extra": "directress"}]}, {"glosses": ["ellipsis of ligne directrice: directrix"], "tags": ["abbreviation", "alt-of", "ellipsis", "feminine"], "alt_of": [{"word": "ligne directrice", "extra": "directrix"}]}]},
    {"word": "téléphonie", "pos": "noun", "senses": [{"glosses": ["telephony (2)"], "tags": ["feminine"]}, {"glosses": ["act of installing a telephone"], "tags": ["feminine"]}]},
    {"word": "coran", "pos": "noun", "senses": [{"glosses": ["alternative form of Coran"], "tags": ["alt-of", "alternative", "masculine"], "alt_of": [{"word": "Coran"}]}]},
    {"word": "sur son trente et un", "pos": "adj", "senses": [{"glosses": ["all dressed up, dolled up (to the nines)"], "tags": ["colloquial", "invariable"]}]},
    {"word": "un coup", "pos": "adv", "senses": [{"glosses": ["used to soften an order"], "tags": ["colloquial"]}, {"glosses": ["once, one time"], "tags": ["colloquial"]}]},
    {"word": "Le", "pos": "name", "senses": [{"glosses": ["a surname from Vietnamese"], "tags": ["feminine", "masculine"]}]},
    {"word": "Durand", "pos": "name", "senses": [{"glosses": ["a surname"]}]},
    {"word": "On", "pos": "name", "senses": [{"glosses": ["a village in Luxembourg, Belgium"]}]},
    {"word": "et des", "pos": "phrase", "senses": [{"glosses": ["or thereabouts, and change, and a bit over"], "tags": ["Belgium", "informal"]}]},
    {"word": "sur son trente-et-un", "pos": "adj", "senses": [{"glosses": ["post-1990 spelling of sur son trente et un"], "tags": ["colloquial", "invariable"]}]},
    {"word": "à priori", "pos": "prep_phrase", "senses": [{"glosses": ["post-1990 spelling of a priori"]}]},
]

# The ranks change 43 gives these lemmas (tables/fr/freq.tsv); each is its own form's lemma.
FRENCH_RANKS = {
    "le": 2, "en": 5, "des": 6, "que": 8, "pas": 9, "du": 10, "il": 11, "on": 19, "ne": 20, "son": 29, "y": 30,
    "bien": 37, "même": 40, "mon": 42, "leur": 46, "quand": 48, "moins": 81, "juste": 90, "parce": 103,
    "paris": 109, "mieux": 116, "mort": 132, "jean": 188, "ouais": 371, "marche": 404, "contrôle": 505,
    "directeur": 531, "liberté": 533, "devoir": 615, "ferme": 820, "nice": 1371, "lot": 2006,
    "directrice": 3075, "phare": 4098, "coran": 4276, "consul": 5728, "téléphonie": 7254, "durand": 7413,
}
FRENCH_FORMS = {
    **{lemma: lemma for lemma in FRENCH_RANKS},
    "a": "avoir", "à": "à", "priori": "priori", "trente-et-un": "trente", "trente": "trente", "et": "et",
    "un": "un", "sur": "sur", "coup": "coup",
}
# UD French-GSD's counts of these words by part of speech (`gsd_pos_counts` over the training and
# development sections fr-en pins).
TREEBANK = collections.Counter({
    ("du", "DET"): 95, ("du", "ADP"): 1, ("du", "PROPN"): 3,
    ("des", "DET"): 1730, ("des", "ADP"): 3, ("des", "X"): 1, ("des", "PROPN"): 2,
    ("en", "ADP"): 5707, ("en", "PRON"): 270,
    ("que", "SCONJ"): 1119, ("que", "ADV"): 148, ("que", "PRON"): 220,
    ("il", "PRON"): 3536, ("il", "X"): 3, ("il", "PROPN"): 1,
    ("y", "PRON"): 492, ("y", "PROPN"): 3, ("y", "SYM"): 1, ("y", "X"): 1,
    ("ne", "ADV"): 780, ("mon", "DET"): 66, ("moins", "ADV"): 102, ("moins", "NOUN"): 1,
    ("mieux", "ADV"): 45, ("mieux", "NOUN"): 1, ("directrice", "NOUN"): 4, ("directeur", "NOUN"): 62,
    ("directeur", "ADJ"): 2,
    ("pas", "ADV"): 981, ("pas", "NOUN"): 8, ("pas", "ADP"): 1,
    ("son", "DET"): 1506, ("son", "NOUN"): 19,
    ("leur", "DET"): 440, ("leur", "PRON"): 50,
    ("marche", "NOUN"): 19,
    ("ferme", "ADJ"): 7, ("ferme", "NOUN"): 17,
    ("mort", "NOUN"): 99, ("mort", "ADJ"): 12,
    ("devoir", "NOUN"): 9, ("devoir", "VERB"): 351,
    ("même", "ADJ"): 262, ("même", "ADV"): 173, ("même", "X"): 2,
    ("phare", "NOUN"): 8, ("phare", "PROPN"): 1, ("phare", "ADJ"): 1,
    ("jean", "PROPN"): 91,
    ("le", "DET"): 14910, ("le", "PRON"): 277, ("le", "PROPN"): 30, ("le", "X"): 2,
    ("on", "PRON"): 507, ("on", "X"): 6,
    ("bien", "NOUN"): 31, ("bien", "ADV"): 292, ("bien", "INTJ"): 1,
    ("quand", "SCONJ"): 88, ("quand", "ADV"): 7,
    ("juste", "ADJ"): 6, ("juste", "ADV"): 38, ("juste", "NOUN"): 1,
    ("nice", "PROPN"): 11,
    ("lot", "PROPN"): 2, ("lot", "NOUN"): 4,
})


def french_native(entries=None, counts=TREEBANK):
    """fr-en's native side over the recorded entries (`FRENCH_SECTION`)."""
    with tempfile.TemporaryDirectory() as work:
        path = os.path.join(work, "kaikki-French.jsonl")
        with open(path, "w", encoding="utf-8") as f:
            f.write("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries or FRENCH_SECTION))
        return red.native_side(work, path, FRENCH_RANKS, FRENCH_FORMS, counts)


class ReadAsFrench(unittest.TestCase):
    """fr-en's own rules (refine-lingua-fr-en-glosses D4–D8) on the recorded entries."""

    @classmethod
    def setUpClass(cls):
        cls.glosses, cls.runs, cls.expressions, cls.stats = french_native()

    # D4 — a pointer that carries its meaning.

    def test_spec_scenario_an_article_s_pointers(self):
        # Borrowed from no other word: `de la` is not even in the section here.
        self.assertEqual(self.glosses["des"], "some; of the, from the, some")
        self.assertEqual(self.runs["des"], [("DET", 1), ("X", 1)])
        # The treebank reads `du` 95 times as a determiner: the article's entry first (D5).
        self.assertEqual(self.glosses["du"], "forms the partitive article; of the")
        self.assertEqual(self.runs["du"], [("DET", 1), ("X", 1)])

    def test_spec_scenario_a_degree_of_comparison(self):
        self.assertTrue(self.glosses["mieux"].startswith("better; best; more, -er"), self.glosses["mieux"])
        self.assertTrue(self.glosses["moins"].startswith("less, fewer; minus, negative"), self.glosses["moins"])

    def test_spec_scenario_a_synonym(self):
        self.assertTrue(
            self.glosses["ouais"].startswith("yeah, yep, yup, yes, affirmative expression; wow"), self.glosses["ouais"]
        )

    def test_spec_scenario_an_expression_s_pointer(self):
        self.assertEqual(self.expressions["il y a"], "there is, there are; ago")

    def test_spec_scenario_another_pointer_of_a_word_stays_a_pointer(self):
        # `y` « alternative form of il; he »: an alternative form, no wording of D4's list.
        self.assertTrue(self.glosses["y"].startswith("there (at a place)"), self.glosses["y"])
        self.assertNotIn("he", self.glosses["y"].split("; "))
        # « female equivalent of directeur: directress » stays a pointer (the owner, Q3), and so does
        # an ellipsis: `directrice` borrows directeur's gloss, as before.
        self.assertEqual(self.glosses["directrice"], "director; school principal")
        self.assertIsNone(red.carried_meaning(FRENCH_SECTION_BY_WORD["directrice"][0]["senses"][0], "directrice"))

    def test_a_meaning_in_capitals_or_none_keeps_a_pointer(self):
        # « NE » under `ne`: an initialism's expansion, no meaning, whatever the wording.
        ne = FRENCH_SECTION_BY_WORD["NE"][0]["senses"][0]
        self.assertIsNone(red.carried_meaning({**ne, "glosses": ["synonym of nord-est; NE"]}, "ne"))
        self.assertNotIn("NE", self.glosses["ne"])
        # A pointer carrying nothing — no extra, no quotation, nothing past a colon — stays a pointer.
        self.assertIsNone(red.carried_meaning({"glosses": ["plural of pari"], "tags": ["form-of"], "form_of": [{"word": "pari"}]}, "paris"))
        # A sense that does not point is no pointer, and a sense kaikki did not shape is none either.
        for sense in ({"glosses": ["synonym of the past"]}, {"glosses": []}, {"tags": ["form-of"]}):
            self.assertIsNone(red.carried_meaning(sense, "passé"))

    def test_a_carried_meaning_by_where_the_section_writes_it(self):
        # kaikki's `extra` for the target, its quoted text first; an initialism after a colon dropped,
        # its own parentheses, a pointer's wording read past; a pointer's pieces are none.
        self.assertEqual(red._meaning_of_extra("“some”, the plural indefinite article"), "some")
        self.assertEqual(red._meaning_of_extra("better"), "better")
        self.assertEqual(red._meaning_of_extra("(knight): N"), "knight")
        self.assertEqual(red._meaning_of_extra("plural of la leur; theirs"), "theirs")
        for extra in ("", "  ", "(= de + le)", "+ le", "plural of la leur"):
            self.assertIsNone(red._meaning_of_extra(extra), extra)
        # Else the gloss's quoted text, else its text after the target past a colon or a semicolon.
        quoted = {"glosses": ["contraction of de + les, literally “of the”"], "tags": ["alt-of"], "alt_of": [{"word": "de"}]}
        self.assertEqual(red.carried_meaning(quoted, "des"), "of the")
        after = {"glosses": ["impersonal singular present indicative of y avoir: there is, there are."], "tags": ["form-of"]}
        self.assertEqual(red.carried_meaning(after, "il y a"), "there is, there are")
        # A word's pointer of another wording is none; an expression's is read whatever it names.
        self.assertIsNone(red.carried_meaning(after, "ilya"))

    # D5 — the part of speech a row opens on.

    def test_spec_scenario_a_function_word_s_commonest_part_of_speech(self):
        self.assertTrue(self.glosses["pas"].startswith("the most common adverb of negation in French"), self.glosses["pas"])
        self.assertEqual(self.runs["pas"], [("ADV", 2), ("NOUN", 3)])
        self.assertTrue(self.glosses["son"].startswith("his, her, their, its"), self.glosses["son"])
        self.assertEqual(self.glosses["leur"], "their; (to) them")
        # `ADV`, measured in: `bien` « well » before « good », `juste` « exactly » before « fair »; a
        # subordinating conjunction before an adverb: `quand`.
        self.assertTrue(self.glosses["bien"].startswith("well; indeed, so"), self.glosses["bien"])
        self.assertTrue(self.glosses["juste"].startswith("exactly, precisely"), self.glosses["juste"])
        self.assertTrue(self.glosses["quand"].startswith("when (at the time that)"), self.glosses["quand"])

    def test_spec_scenario_a_common_word_before_a_place_s_name(self):
        # The page opens `marche` on its capitalised name: every case of the headword is moved, the
        # department written after the noun's senses.
        gloss = self.glosses["marche"]
        self.assertTrue(gloss.startswith("march (formal, rhythmic way of walking)"), gloss)
        self.assertIn("Marche (a department of France)", gloss)
        self.assertEqual(self.runs["marche"][0], ("NOUN", 7))

    def test_spec_scenario_a_noun_a_verb_and_an_adjective_keep_the_page_s_order(self):
        # Read more often as a noun (17 against 7) and as a verb (351 against 9): a content word
        # ahead of another stays as the page writes it (the owner, Q2).
        self.assertTrue(self.glosses["ferme"].startswith("firm; "), self.glosses["ferme"])
        self.assertTrue(self.glosses["mort"].startswith("dead; "), self.glosses["mort"])
        self.assertTrue(self.glosses["devoir"].startswith("duty; "), self.glosses["devoir"])

    def test_spec_scenario_too_little_evidence(self):
        # `même` 173 adverbs against 262 adjectives, not twice; `phare` a noun 8 times, fewer than 10.
        self.assertTrue(self.glosses["même"].startswith("even; same"), self.glosses["même"])
        self.assertTrue(self.glosses["phare"].startswith("leading, signature, key, flagship"), self.glosses["phare"])

    def test_spec_scenario_a_name_is_never_promoted(self):
        self.assertTrue(self.glosses["jean"].startswith("a pair of jeans; John"), self.glosses["jean"])
        self.assertEqual(self.runs["jean"], [("NOUN", 1), ("PROPN", 4)])

    def test_the_treebank_s_order_and_its_thresholds(self):
        items = [(0, "noun", "pas"), (1, "adv", "pas"), (2, "noun", "Pas")]
        self.assertEqual(red._treebank_order("pas", items, TREEBANK), [1, 0, 2])
        # At the thresholds: ten times, twice as often.
        for adverbs, nouns, moved in ((10, 5, True), (9, 1, False), (19, 10, False), (20, 10, True)):
            counts = collections.Counter({("pas", "ADV"): adverbs, ("pas", "NOUN"): nouns})
            self.assertEqual(red._treebank_order("pas", items, counts) is not None, moved, (adverbs, nouns))
        self.assertEqual((red.treebank.TREEBANK_MIN, red.treebank.TREEBANK_RATIO), (10, 2))
        self.assertEqual(red.treebank.TREEBANK_FIRST, frozenset({"ADP", "DET", "PRON", "CCONJ", "SCONJ", "PART", "ADV"}))
        # The module decides on parts of speech alone, whatever dictionary names them: what fr-es
        # reads it for (refine-lingua-fr-es-glosses D8).
        self.assertEqual(red.treebank.commonest_first("pas", ["NOUN", "ADV"], "NOUN", TREEBANK), [1, 0])
        self.assertIsNone(red.treebank.commonest_first("pas", ["ADV", "NOUN"], "ADV", TREEBANK))
        self.assertIsNone(red.treebank.commonest_first("jean", ["PROPN", "PROPN"], "PROPN", TREEBANK))
        self.assertEqual(red.treebank.word_key(" L’Homme "), "l'homme")
        # One part of speech, or a proper noun alone besides the first: nothing to order.
        self.assertIsNone(red._treebank_order("pas", [(0, "adv", "pas"), (1, "adv", "pas")], TREEBANK))
        self.assertIsNone(red._treebank_order("jean", [(0, "name", "Jean"), (1, "name", "Jean")], TREEBANK))
        # An acronym's entry is not the one the page opens on.
        acronym = [(0, "noun", "ON"), (1, "pron", "on"), (2, "noun", "on")]
        self.assertIsNone(red._treebank_order("on", acronym, TREEBANK))
        # Ties go to the part of speech the page writes first.
        tied = collections.Counter({("x", "ADV"): 30, ("x", "ADP"): 30})
        self.assertEqual(red._treebank_order("x", [(0, "name", "X"), (1, "prep", "x"), (2, "adv", "x")], tied), [1, 0, 2])

    def test_the_treebank_counts_parts_of_speech(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "t.conllu")
            with open(path, "w", encoding="utf-8") as f:
                f.write(
                    "# sent_id = 1\n"
                    "1\tPar\tpar\tADP\t_\t_\t3\tcase\t_\t_\n"
                    "2\tconséquent\tconséquent\tADJ\t_\t_\t1\tfixed\t_\t_\n"
                    "3\tLeur\tson\tDET\t_\t_\t4\tdet\t_\t_\n"
                    "4\tmarche\tmarche\tNOUN\t_\t_\t5\tnsubj\t_\t_\n"
                    "5\ta\tavoir\tAUX\t_\t_\t6\taux\t_\t_\n"
                    "6\tMangé\tmanger\tVERB\t_\t_\t0\troot\t_\t_\n"
                    "7-8\tdu\t_\t_\t_\t_\t_\t_\t_\t_\n"
                    "7\tde\tde\tADP\t_\t_\t8\tcase\t_\t_\n"
                    "8\tle\tle\tDET\t_\t_\t6\tobj\t_\t_\n"
                    "8.1\tx\tx\tX\t_\t_\t_\t_\t_\t_\n"
                )
            counts = red.treebank.gsd_pos_counts([path])
        # A fixed expression's word counts for none (« par conséquent »); a closed class by its own
        # form (UD lemmatises « leur » as « son »), an open one by its lemma; the auxiliary as a verb.
        self.assertEqual(
            counts,
            collections.Counter({("par", "ADP"): 1, ("leur", "DET"): 1, ("marche", "NOUN"): 1, ("avoir", "VERB"): 1,
                                 ("manger", "VERB"): 1, ("de", "ADP"): 1, ("le", "DET"): 1}),
        )

    # D6 — no name under a function word.

    def test_spec_scenario_a_function_word_s_homograph_name(self):
        self.assertNotIn("Vietnamese", self.glosses["le"])
        self.assertEqual(self.runs["le"], [("DET", 4), ("PRON", 2)])
        self.assertEqual(self.glosses["on"], "one, people, you, someone (an unspecified individual); we")
        # A word with an adjective's entry keeps its name.
        self.assertIn("Nice (a coastal city", self.glosses["nice"])
        self.assertEqual(self.stats["names under a function word"], 2)

    # D8 — the page's notes and typography.

    def test_spec_scenario_usage_notes(self):
        self.assertTrue(self.glosses["en"].startswith("in (used to indicate space); to (indicates direction"), self.glosses["en"])
        self.assertTrue(
            self.glosses["ne"].startswith("not (used alone to negate a verb, now chiefly with only a few particular verbs); "),
            self.glosses["ne"],
        )
        self.assertNotIn("usage notes", self.glosses["en"] + self.glosses["ne"])

    def test_spec_scenario_all_senses(self):
        self.assertTrue(self.glosses["contrôle"].startswith("control; verification"), self.glosses["contrôle"])
        self.assertEqual(self.glosses["consul"], "consul")

    def test_spec_scenario_a_folk_etymology_and_a_citation(self):
        self.assertTrue(self.glosses["mon"].endswith("within the military"), self.glosses["mon"])
        self.assertEqual(self.glosses["liberté"], "liberty, freedom")

    def test_spec_scenario_a_description_in_a_capital(self):
        self.assertIn("; substitutes for another, previously stated conjunction;", self.glosses["que"])
        self.assertTrue(self.glosses["il"].endswith("; impersonal subject, it"), self.glosses["il"])

    def test_spec_scenario_a_capital_that_is_no_description(self):
        for text in ("German person", "Military rank equivalent to corporal", "Swiss (of, from or relating to Switzerland)", "Found Footage"):
            self.assertEqual(red.french_text(text), text)
        self.assertEqual(red.french_text("Names a thing"), "names a thing")
        self.assertEqual(red.french_text("Exclamation (of surprise)"), "exclamation (of surprise)")

    def test_spec_scenario_a_source_s_sense_number(self):
        self.assertTrue(self.glosses["téléphonie"].startswith("telephony; "), self.glosses["téléphonie"])
        # Two digits are no sense number.
        self.assertEqual(red.french_text("year (12)"), "year (12)")

    def test_the_notes_wherever_they_sit(self):
        for text, read in (
            ("in (used to indicate space, also see usage notes)", "in (used to indicate space)"),
            ("to (see usage notes)", "to"),
            ("to, see usage notes", "to"),
            ("on (all senses)", "on"),
            ("a chamber in all its various senses, including:", "a chamber:"),
            ("my (Folk etymology: short for “monsieur” (sir).)", "my"),
            ("freedom. 1688, Guy Miège, The Great French Dictionary.", "freedom"),
            ("(all senses)", ""),
        ):
            self.assertEqual(red.french_text(text), read, text)

    def test_spec_scenario_etc_with_its_period(self):
        self.assertIn("the, my, your, etc.;", self.glosses["le"])
        self.assertIn("not, don't, doesn't, etc.;", self.glosses["pas"])
        self.assertIn("(money, obligation and etc.)", self.glosses["devoir"])
        for text in ("etc.)", "and so on, etc.", "fetch, etcetera"):
            self.assertEqual(red.with_etc_period(text), text)
        self.assertEqual(red.with_etc_period("the, my, your, etc"), "the, my, your, etc.")

    # D7 — expressions.

    def test_spec_scenario_an_expression_whose_sense_needs_a_context(self):
        for word in ("et des", "que de", "sur ce", "et si", "un coup"):
            self.assertIn(word, red.LEFT_OUT)
            self.assertIn("UD French-GSD", red.LEFT_OUT[word])
        self.assertNotIn("et des", self.expressions)
        self.assertNotIn("un coup", self.expressions)
        # « un coup d'œil » then meets `coup d'œil` alone.
        self.assertEqual(self.expressions["coup d'œil"], "glance, look; sight")
        self.assertEqual(self.expressions["parce que"], "because")

    def test_spec_scenario_a_post_1990_spelling_keyed_apart(self):
        self.assertEqual(self.expressions["à priori"], self.expressions["a priori"])
        self.assertEqual(self.expressions["a priori"], "intuitively known, a priori; at first glance; preconceived idea")
        self.assertEqual(self.expressions["sur son trente-et-un"], "all dressed up, dolled up (to the nines)")
        self.assertEqual(self.stats["lent"], 2)

    def test_a_post_1990_spelling_keyed_alike_stays_out(self):
        # `crème fraiche` reads as `crème fraîche` (both forms of *frais*): it meets it already.
        forms = {"crème": "crème", "fraiche": "frais", "fraîche": "frais", "à": "à", "a": "avoir"}
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "section.jsonl")
            with open(path, "w", encoding="utf-8") as f:
                for entry in (
                    *POINTERS,
                    {"word": "à priori", "pos": "adv", "senses": [{"glosses": ["post-1990 spelling of a priori"]}, {"glosses": ["on the face of it"]}]},
                    {"word": "à postériori", "pos": "adv", "senses": [{"glosses": ["post-1990 spelling of a posteriori"]}]},
                    "not an entry",
                ):
                    f.write(json.dumps(entry, ensure_ascii=False) + "\n")
                f.write("not json\n")
            lent = red.traditional_spellings(path, {"crème fraîche": "crème fraîche", "a priori": "a priori"}, forms)
        # Neither a spelling keyed alike, nor one with a sense of its own, nor one whose traditional
        # spelling fr-en does not gloss.
        self.assertEqual(lent, {})

    # D2 — French's dictionary words.

    def test_spec_scenario_french_s_names(self):
        words = red.dictionary_words(self.glosses, self.runs)
        for name in ("paris", "durand", "coran"):
            self.assertIn(name, self.glosses, name)
            self.assertNotIn(name, words, name)
        for word in ("marche", "lot", "nice", "le", "des"):
            self.assertIn(word, words, word)
        self.assertEqual(words, sorted(words))
        self.assertEqual(red.dictionary_words({"a": "x", "b": "y"}, {"a": [("PROPN", 2)]}), ["b"])

    def test_the_pass_reads_each_entry_once_in_its_place(self):
        # Every rule off but the pass's plumbing: an entry it does not change keeps its bytes, a line
        # it cannot read is written as it is, and the stats count what each rule moved.
        with tempfile.TemporaryDirectory() as d:
            src, dst = os.path.join(d, "in.jsonl"), os.path.join(d, "out.jsonl")
            with open(src, "w", encoding="utf-8") as f:
                f.write('{"word":"maison","pos":"noun","senses":[{"glosses":["house"]}]}\n')
                f.write("not json\n[1]\n")
                f.write('{"word":"pas","pos":"noun","senses":[{"glosses":["step"]}]}\n')
                f.write('{"word":"pas","pos":"adv","senses":[{"glosses":["not"]}, "no sense"]}\n')
                f.write('{"word":"x","pos":"noun","senses":"none"}')
            _, stats = red.read_as_french(src, dst, TREEBANK)
            with open(dst, encoding="utf-8") as f:
                out = f.read().splitlines()
        self.assertEqual(out[0], '{"word":"maison","pos":"noun","senses":[{"glosses":["house"]}]}')
        self.assertEqual(out[1:3], ["not json", "[1]"])
        self.assertEqual([json.loads(line)["pos"] for line in out[3:5]], ["adv", "noun"])
        self.assertEqual(out[5], '{"word":"x","pos":"noun","senses":"none"}')
        self.assertEqual(stats["headwords reordered by the treebank"], 1)

    def test_the_pass_runs_after_the_meanings_and_before_the_merging(self):
        # D1: fr-en's pre-pass sits between the English edition's two last ones, and the post-passes
        # after the shared rules; the counts it reads are the ones `native_side` is handed.
        calls = []
        real = red.read_as_french

        def spy(src, dst, counts):
            calls.append((os.path.basename(src), os.path.basename(dst), counts is TREEBANK))
            return real(src, dst, counts)

        with mock.patch.object(red, "read_as_french", spy):
            french_native()
        self.assertEqual(calls, [("kaikki-French-meanings.jsonl", "kaikki-French-french.jsonl", True)])


FRENCH_SECTION_BY_WORD = collections.defaultdict(list)
for _entry in FRENCH_SECTION:
    FRENCH_SECTION_BY_WORD[_entry["word"]].append(_entry)


class FrenchLevelsAndWords(unittest.TestCase):
    """French's levels given only to its dictionary words (refine-lingua-fr-en-glosses D3)."""

    @classmethod
    def setUpClass(cls):
        cls.senses = level_senses(*FRENCH_SECTION)

    def test_spec_scenario_a_word_met_only_in_an_expression(self):
        # `parce` « only used in parce que »: a pointer, no gloss; « parce que » keeps its own.
        glosses, runs, expressions, _ = french_native()
        self.assertNotIn("parce", glosses)
        self.assertEqual(expressions["parce que"], "because")
        words = set(red.dictionary_words(glosses, runs))
        ranks = {"parce": 1, "pas": 2}
        forms = {"parce": "parce", "pas": "pas"}
        # The section gives `parce` a sense no rule of change 46 reads as a pointer: it took A1.
        senses = {"parce": self.senses["parce"], "pas": self.senses["pas"]}
        self.assertEqual(senses["parce"], [("prep", "only used in parce que", frozenset())])
        self.assertIsNone(red.no_level("parce", forms, senses))
        self.assertEqual(red.no_level("parce", forms, senses, words), "unlisted")
        levels, left_out = red.estimated_levels(ranks, forms, senses, words, bands=(("A1", 1),))
        self.assertEqual(levels, {"pas": "A1"})
        self.assertEqual(left_out["unlisted"], ["parce"])

    def test_spec_scenario_a_word_glossed_only_as_a_name(self):
        # `coran` « alternative form of Coran » borrows the name's « Koran »: a name, no word.
        glosses, runs, _, _ = french_native()
        self.assertEqual(glosses["coran"], "Koran")
        self.assertEqual(runs["coran"], [("PROPN", 1)])
        words = set(red.dictionary_words(glosses, runs))
        self.assertNotIn("coran", words)
        self.assertEqual(red.no_level("coran", {"coran": "coran"}, self.senses, words), "unlisted")
        # The rule is read last: a lemma another rule leaves out is named by that one.
        self.assertEqual(red.no_level("paris", {}, {"paris": [("name", "Paris", frozenset())]}, words), "name")
        # Without French's dictionary words the rule is not read: « alternative form of Coran » is
        # no spelling of change 46's list, so `coran` took a level.
        self.assertIsNone(red.no_level("coran", {"coran": "coran"}, self.senses))

    def test_a_lemma_left_out_gives_its_slot_to_the_next(self):
        ranks = {"de": 1, "parce": 2, "le": 3, "coran": 4, "pas": 5}
        senses = {lemma: [("noun", "a meaning", frozenset())] for lemma in ranks}
        words = {"de", "le", "pas"}
        levels, left_out = red.estimated_levels(ranks, {}, senses, words, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(levels, {"de": "A1", "le": "A1", "pas": "A2"})
        self.assertEqual(left_out["unlisted"], ["parce", "coran"])
        self.assertEqual(red.LEVEL_RULES[-1], "unlisted")


if __name__ == "__main__":
    unittest.main()
