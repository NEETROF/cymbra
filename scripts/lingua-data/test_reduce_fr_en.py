# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the FR->EN reducer's French rules (add-lingua-french-forms-tables), on entries
shaped as the English Wiktionary's dump writes them — no download, and wordfreq doubled.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

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
    alt_of("des", "de", pos="contraction", gloss="contraction of de + les", tags=("abbreviation", "alt-of", "contraction")),
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


def fake_wordfreq():
    module = types.ModuleType("wordfreq")
    module.top_n_list = lambda lang, n: top_n(n)
    module.zipf_frequency = lambda word, lang: frequency(word)
    return module


class Main(unittest.TestCase):
    def run_main(self, entries, gsd=GSD):
        with tempfile.TemporaryDirectory() as work:
            with open(os.path.join(work, "kaikki-French.jsonl"), "w", encoding="utf-8") as f:
                f.write("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries))
                f.write("not json\n")
            for name in ("fr_gsd-ud-train.conllu", "fr_gsd-ud-dev.conllu"):
                with open(os.path.join(work, name), "w", encoding="utf-8") as f:
                    f.write(conllu(gsd))
            argv = ["reduce-fr-en.py", "--work", work, "--built-at", "2026-10-08", "--pack-version", "2026.10.08+abcdef0"]
            with mock.patch.dict(sys.modules, {"wordfreq": fake_wordfreq()}), mock.patch.object(sys, "argv", argv):
                with contextlib.redirect_stderr(io.StringIO()):
                    red.main()
            out = {}
            for name in sorted(os.listdir(work)):
                with open(os.path.join(work, name), encoding="utf-8") as f:
                    out[name] = f.read()
            return out

    def test_spec_scenario_the_reference_pair_writes_french_s_folder(self):
        out = self.run_main(ENTRIES)
        self.assertEqual(out["gloss.tsv"], "")
        for name in ("forms.tsv", "freq.tsv", "NOTICE", "manifest.json"):
            self.assertTrue(out[name], name)
        self.assertIn("dirigée\tdiriger\tVERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part\t-\n", out["grammar.tsv"])
        # and French's estimated levels (add-lingua-french-levels, `EstimatedLevels` below).
        self.assertIn("de\tA1\n", out["level.tsv"])
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
            left_out, {"unknown": ["the"], "name": ["paris"], "elsewhere": [], "letter": ["b"], "spelling": ["etre"]}
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
        # The derivation reads the ranks, the forms and the section's senses: nothing a gloss table,
        # fr-en's (change 48) or another pair's, could move.
        self.assertEqual(list(inspect.signature(red.estimated_levels).parameters), ["ranks", "forms", "senses", "bands"])

    def test_the_report_names_each_level_s_span_and_each_rule(self):
        ranks = {"de": 1, "the": 2, "à": 3, "y": 4}
        levels, left_out = red.estimated_levels(ranks, {}, self.senses, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(
            red.levels_report(levels, left_out, ranks),
            "levels=3 estimated: A1 2 (1–3); A2 1 (4–4); left out: unknown 1, name 0, elsewhere 0, letter 0, spelling 0",
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


if __name__ == "__main__":
    unittest.main()
