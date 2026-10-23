# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the IT->FR reducer's Italian rules (add-lingua-italian-forms-tables) and for
UD Italian-MarkIT's counts (`reduce_italian_treebank.py`), on entries shaped as the English
Wiktionary's dump writes them — no download, wordfreq doubled.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
import importlib.util
import io
import json
import os
import re
import tarfile
import tempfile
import unittest
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("reduce_it_fr", os.path.join(_HERE, "reduce-it-fr.py"))
red = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(red)
import reduce_italian_treebank as treebank  # noqa: E402 — beside this file

_TOKENIZE_RS = os.path.join(_HERE, "..", "..", "crates", "lingua-core", "src", "analysis", "tokenize.rs")


def entry(word, pos="noun", forms=(), senses=None):
    """A kaikki entry: a word, its inflections `(form, [tags])`, and senses (a meaning by default)."""
    return {
        "word": word,
        "pos": pos,
        "forms": [{"form": f, "tags": list(t)} for f, t in forms],
        "senses": senses if senses is not None else [{"glosses": ["a meaning"]}],
    }


def form_of(word, *targets, pos="verb", tags=("form-of",)):
    """An entry that is only a form of `targets` (`creduta`: « feminine singular of creduto »)."""
    return entry(word, pos=pos, senses=[{"tags": list(tags), "form_of": [{"word": t} for t in targets]}])


def alt_of(word, *targets, pos, tags=("alt-of",)):
    """An entry pointing at `targets` by `alt_of` (`un`: « apocopic form of uno »)."""
    return entry(word, pos=pos, senses=[{"glosses": ["a pointer"], "tags": list(tags), "alt_of": [{"word": t} for t in targets]}])


def compound(word, pos="verb"):
    """A clitic compound's entry (`dirmelo`: « compound of the infinitive dire with me and lo »)."""
    return entry(word, pos=pos, senses=[{"glosses": ["compound of something with lo"], "tags": ["compound-of"]}])


PRESENT_3S = ["indicative", "present", "singular", "third-person"]
PRESENT_1S = ["first-person", "indicative", "present", "singular"]
PARTICIPLE = ["participle", "past"]
# What a verb's conjugation table opens with, in the dump: its bookkeeping and its auxiliary.
TABLE_HEAD = [
    ("irregular", ["table-tags"]),
    ("it-conj", ["inflection-template"]),
    ("avére", ["auxiliary"]),
]

# A small Italian section: every rule of design D3–D6 has an entry here.
ENTRIES = [
    # D3 — a form of a form, along one part of speech.
    entry("credere", pos="verb", forms=[*TABLE_HEAD, ("crédo", PRESENT_1S), ("credùto", PARTICIPLE)]),
    entry(
        "creduto",
        pos="verb",
        forms=[("creduta", ["feminine"]), ("creduti", ["masculine", "plural"]), ("credute", ["feminine", "plural"])],
        senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "credere"}]}],
    ),
    form_of("creduta", "creduto", tags=("feminine", "form-of", "participle", "singular")),
    # D3 — the table's stress marks: a mark but on the last letter dropped; the last letter's kept
    # unless the cell's entry is the bare spelling.
    entry(
        "dare",
        pos="verb",
        forms=[
            ("dàre", ["canonical"]),
            *TABLE_HEAD,
            ("dàre", ["infinitive"]),
            ("dàto", PARTICIPLE),
            ("dièdi", ["first-person", "historic", "past", "singular"]),
            ("darà", ["future", "singular", "third-person"]),
            ("dò", PRESENT_1S),
            ("dà", PRESENT_3S),
            ("dài", ["imperative", "second-person", "singular"]),
            ("darèbbe", ["conditional", "singular", "third-person"]),
            ("dèttero", ["archaic", "historic", "past", "plural", "third-person"]),
        ],
    ),
    form_of("dà", "dare"),
    form_of("do", "dare"),
    entry("do", senses=[{"glosses": ["do (musical note)"], "tags": ["masculine"]}]),
    form_of("darà", "dare"),
    form_of("dai", "dare"),
    alt_of("dai", "da", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    entry("da", pos="prep"),
    entry("da", pos="verb", senses=[{"glosses": ["misspelling of dà"], "tags": ["alt-of", "misspelling"], "alt_of": [{"word": "dà"}]}]),
    entry("avere", pos="verb", forms=[*TABLE_HEAD, ("hà", PRESENT_3S), ("hò", PRESENT_1S), ("avùto", PARTICIPLE)]),
    entry("ha", pos="verb", senses=[{"tags": ["form-of"], "form_of": [{"word": "avere and", "extra": "(obsolete) havere"}]}]),
    form_of("ho", "avere"),
    # D3 — a reflexive verb's rows written with their pronoun.
    entry("lavare", pos="verb", forms=[*TABLE_HEAD, ("làvo", PRESENT_1S), ("làva", PRESENT_3S)]),
    entry(
        "lavarsi",
        pos="verb",
        forms=[
            ("lavàrsi", ["canonical"]),
            *TABLE_HEAD,
            ("lavàrsi", ["infinitive"]),
            ("lavàndosi", ["gerund"]),
            ("lavàtosi", PARTICIPLE),
            ("mi làvo", PRESENT_1S),
            ("si làva", PRESENT_3S),
            ("làvati", ["imperative", "second-person", "singular"]),
        ],
        senses=[
            {"glosses": ["reflexive of lavare"], "tags": ["form-of", "reflexive"], "form_of": [{"word": "lavare"}]},
            {"glosses": ["to wash (oneself)"]},
        ],
    ),
    form_of("lavatosi", "lavarsi", tags=("form-of", "participle", "past")),
    compound("lavandosi"),
    entry(
        "pentirsi",
        pos="verb",
        forms=[
            ("pentìrsi", ["canonical"]),
            *TABLE_HEAD,
            ("pentìrsi", ["infinitive"]),
            ("pentèndosi", ["gerund"]),
            ("mi pènto", PRESENT_1S),
            ("si pènte", PRESENT_3S),
            ("pentìtevi", ["imperative", "plural", "second-person"]),
            ("ti pentìre", ["imperative", "negative", "second-person", "singular"]),
        ],
        senses=[{"glosses": ["to repent"]}],
    ),
    form_of("pentendosi", "pentirsi", tags=("form-of", "gerund")),
    form_of("pento", "pentirsi", tags=("first-person", "form-of", "indicative", "present", "singular")),
    # D3 — a row carrying another clitic, a multi-word lemma's cells, alternatives, a bracket.
    entry(
        "andare",
        pos="verb",
        forms=[("andàre [auxiliary essere]", ["canonical"]), *TABLE_HEAD, ("vàdo", PRESENT_1S), ("vài", ["indicative", "present", "second-person", "singular"]), ("và", PRESENT_3S)],
    ),
    form_of("va", "andare"),
    form_of("vai", "andare"),
    entry("andarsene", pos="verb", forms=[*TABLE_HEAD, ("me ne vàdo", PRESENT_1S), ("andàrsene", ["infinitive"]), ("vàttene", ["imperative"])], senses=[{"glosses": ["to go away"]}]),
    entry("stare zitto", pos="verb", forms=[("stò zitto", PRESENT_1S)], senses=[{"glosses": ["to keep quiet"]}]),
    entry("potere", pos="verb", forms=[*TABLE_HEAD, ("puote, pote", ["archaic", "present", "third-person"]), ("può", PRESENT_3S)]),
    form_of("può", "potere"),
    # D3 — a derived word's row, and its own entry.
    entry("casa", forms=[("case", ["plural"]), ("casétta", ["diminutive"]), ("casóne", ["augmentative", "masculine"]), ("ca'", ["alternative", "apocopic"])], senses=[{"glosses": ["house"], "tags": ["feminine"]}]),
    entry("casetta", forms=[("casette", ["plural"])], senses=[{"glosses": ["diminutive of casa"], "tags": ["diminutive", "feminine", "form-of"], "form_of": [{"word": "casa"}]}]),
    entry("vero", pos="adj", forms=[("vera", ["feminine"]), ("veri", ["masculine", "plural"]), ("verissimo", ["superlative"]), ("veramente", ["adverb"])], senses=[{"glosses": ["true"]}]),
    entry("veramente", pos="adv"),
    entry("mare", forms=[("mari", ["plural"]), ("marittimo", ["adjective", "relational"])], senses=[{"glosses": ["sea"], "tags": ["masculine"]}]),
    # D3 — a doubtful inflection, and a gender marker.
    entry("bello", pos="adj", forms=[("bella", ["feminine"]), ("belli", ["masculine", "plural"]), ("bèllo", ["archaic"]), ("bello'", ["misspelling"]), ("m", ["masculine"])]),
    # D5 — one lemma per form: a noun a verb's form takes (M8).
    entry("portare", pos="verb", forms=[*TABLE_HEAD, ("pòrta", PRESENT_3S), ("pòrto", PRESENT_1S)]),
    entry("porta", forms=[("porte", ["plural"])], senses=[{"glosses": ["door"], "tags": ["feminine"]}]),
    form_of("porta", "portare"),
    entry("legge", forms=[("leggi", ["plural"])], senses=[{"glosses": ["law"], "tags": ["feminine"]}]),
    entry("leggere", pos="verb", forms=[*TABLE_HEAD, ("lègge", PRESENT_3S)]),
    form_of("legge", "leggere"),
    entry("vita", forms=[("vite", ["plural"])], senses=[{"glosses": ["life"], "tags": ["feminine"]}]),
    entry("vite", forms=[("viti", ["plural"])], senses=[{"glosses": ["vine"], "tags": ["feminine"]}]),
    # D5 — names.
    entry("bianco", pos="adj", forms=[("bianca", ["feminine"]), ("bianchi", ["masculine", "plural"])], senses=[{"glosses": ["white"]}]),
    entry("bianca", senses=[{"glosses": ["female equivalent of bianco"], "tags": ["feminine", "form-of"], "form_of": [{"word": "bianco"}]}, {"glosses": ["cue ball"], "tags": ["feminine"]}]),
    entry("Bianca", pos="name", senses=[{"glosses": ["a female given name"]}]),
    entry("Roma", pos="name", senses=[{"glosses": ["Rome"]}]),
    entry("roma", senses=[{"glosses": ["a type of rice"]}]),
    entry("bambino", forms=[("bambini", ["plural"])], senses=[{"glosses": ["child"], "tags": ["masculine"]}]),
    entry("Bambini", pos="name", senses=[{"glosses": ["a surname"]}]),
    # D4 — the elided pieces, as the dump writes them, and the words they stand for.
    alt_of("l'", "il/lo", "la", pos="article", tags=("abbreviation", "alt-of", "apocopic")),
    entry("il", pos="article", forms=[("i", ["plural"]), ("la", ["feminine"]), ("lo (l')", ["masculine", "singular"]), ("gli", ["masculine", "plural"]), ("le", ["feminine", "plural"])]),
    entry("lo", pos="article", forms=[("gli", ["plural"]), ("il", ["masculine", "singular"])]),
    entry("lo", pos="pron", forms=[("li", ["plural"]), ("la", ["feminine"])]),
    entry("la", pos="pron"),
    entry("uno", pos="num", forms=[("una", ["feminine"])]),
    form_of("una", "uno", pos="article", tags=("feminine", "form-of", "singular")),
    alt_of("un", "uno", pos="article", tags=("abbreviation", "alt-of", "apocopic", "masculine")),
    alt_of("un'", "una", pos="article", tags=("abbreviation", "alt-of", "apocopic")),
    entry("buono", pos="adj", forms=[("buona", ["feminine"])]),
    alt_of("buon", "buono", pos="adj", tags=("abbreviation", "alt-of", "apocopic")),
    entry("questo", pos="pron", forms=[("questa", ["feminine"])]),
    entry("anche", pos="adv"),
    alt_of("dello", "di lo", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("dell'", "di l'", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("del", "di", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("della", "di", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("nel", "in il", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("al", "a", pos="contraction", tags=("abbreviation", "alt-of", "contraction")),
    alt_of("po'", "poco", pos="noun", tags=("abbreviation", "alt-of", "apocopic", "masculine")),
    entry("poco", pos="adv"),
    # D4 — a plain word beginning with an elided piece, and a clitic compound.
    entry("c'è", pos="verb", senses=[{"glosses": ["there is"]}]),
    entry("dell'anno", pos="phrase"),
    compound("dirmelo"),
    compound("vattene"),
    entry("vattene", pos="intj", senses=[{"glosses": ["go away!"]}]),
    compound("fallo"),
    entry("fallo", senses=[{"glosses": ["foul"], "tags": ["masculine"]}]),
    compound("farlo"),
    # D5 — the owner's overrides: the auxiliary participle, read by the counts as the noun.
    entry("essere", pos="verb", forms=[*TABLE_HEAD, ("è", PRESENT_3S), ("stàto", PARTICIPLE)]),
    entry("stato", forms=[("stati", ["plural"])], senses=[{"glosses": ["state"], "tags": ["masculine"]}]),
    form_of("stato", "essere", tags=("form-of", "participle", "past")),
    # D5 — a name beside a common reading of its own: D5's, the counts deciding.
    entry("ti", pos="pron"),
    entry("Ti", pos="name", senses=[{"glosses": ["a surname"]}]),
    entry("tu", pos="pron", forms=[("ti", ["accusative"])]),
    entry("Firmato", pos="name", senses=[{"glosses": ["a male given name"]}]),
    entry("firmare", pos="verb", forms=[*TABLE_HEAD, ("firmàto", PARTICIPLE), ("fìrmi", ["indicative", "present", "second-person", "singular"])]),
    # A name's rows are no forms of a word.
    entry("Nepal", pos="name", forms=[("nepalese", ["demonym"]), ("nepàli", ["plural"])], senses=[{"glosses": ["Nepal"]}]),
    # D6 — the apocopes written without an apostrophe, by the reviewed table.
    entry("bel", pos="adj", senses=[{"glosses": ["preconsonantal masculine singular form of bello"]}]),
    entry("san", senses=[{"glosses": ["san (Greek letter)"]}]),
    alt_of("san", "santo saint", pos="noun", tags=("abbreviation", "alt-of", "apocopic")),
    entry("santo", pos="adj"),
    # D3 — a finite form ending as an infinitive carrying clitics does is no such infinitive.
    entry("imporre", pos="verb", forms=[*TABLE_HEAD]),
    entry("importare", pos="verb", forms=[*TABLE_HEAD]),
    form_of("importi", "importare", tags=("form-of", "indicative", "present", "second-person", "singular")),
    # D4 — a form carrying clitics its own entry links, and a compound kaikki does not tag.
    entry("recarsi", pos="verb", forms=[*TABLE_HEAD, ("mi rèco", PRESENT_1S)], senses=[{"glosses": ["reflexive of recare"], "tags": ["form-of", "reflexive"], "form_of": [{"word": "recare"}]}]),
    entry("recare", pos="verb", forms=[*TABLE_HEAD]),
    form_of("recatevi", "recarsi", tags=("form-of", "imperative", "plural", "second-person")),
    form_of("pentirmi", "pentirsi", tags=("first-person", "form-of", "infinitive", "singular")),
    entry("stammi", pos="verb", senses=[{"glosses": ["combined form of sta' (second person singular imperative of stare) and mi"]}]),
    # D4 — a compound whose other entry is a surname's: no rank, and a form through its participle.
    compound("donati"),
    form_of("donati", "donato", tags=("form-of", "masculine", "participle", "plural")),
    entry("Donati", pos="name", senses=[{"glosses": ["a surname"]}]),
    entry("donare", pos="verb", forms=[*TABLE_HEAD, ("donàto", PARTICIPLE)]),
    entry("donato", pos="verb", forms=[("donati", ["masculine", "plural"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "donare"}]}]),
    # D6 — hyphenated words, ranked by MarkIT's evidence.
    entry("pellerossa", senses=[{"glosses": ["redskin"]}]),
    entry("anglo-americano", pos="adj"),
    entry("nord-est", pos="noun"),
]

# wordfreq, doubled: its Italian list in order, with the elided pieces' stems, a token beginning with
# a piece, and words no rank may go to (the order wordfreq gives them is irrelevant to the rules).
TOP = [
    "di", "l", "il", "la", "lo", "un", "d", "una", "da", "po", "del", "della", "nel", "al", "dai", "dell'acqua",
    "anche", "questo", "quest", "buon", "dare", "avere", "ha", "porta", "portare", "legge", "leggere",
    "vita", "vite", "bianca", "bianco", "roma", "bambini", "bambino", "credere", "farlo", "lavarsi",
    "lavare", "casa", "vero", "veramente", "casetta", "poco", "uno", "dirmelo", "fallo", "mare",
    "bello", "pentirsi", "andare", "potere", "do", "andarsene", "pellerossa", "dello", "buono",
    "essere", "stato", "ti", "tu", "firmare", "firmato", "bel", "san", "santo", "cent", "importare", "importi",
    "recare", "stammi", "donati", "donare", "nepalese", "nepal", "pentirmi", "recatevi",
]
# Zipf frequencies: the list's order, and the forms wordfreq attests.
FREQUENCY = {word: 7.0 - i * 0.05 for i, word in enumerate(TOP)}
FREQUENCY.update(
    {
        "bianca": 4.78, "bianco": 4.99, "roma": 5.63, "bambini": 4.97, "bambino": 4.63, "porta": 5.41, "portare": 5.09,
        "legge": 5.64, "leggere": 4.9, "anglo-americano": 6.5, "nord-est": 6.5,
    }
)
FREQUENCY.update(
    {
        w: 3.0
        for w in (
            "creduta", "creduto", "creduti", "credo", "dato", "diedi", "darà", "dà", "dai", "do", "darebbe", "ho",
            "lavo", "lava", "pento", "pente", "pentire", "pentendo", "pentite", "vado", "vai", "va", "può",
            "case", "casette", "vera", "veri", "verissimo", "mari", "belli", "porto", "porte", "leggi", "viti", "bianchi",
            "i", "gli", "le", "li", "questa", "buona", "vite", "dirmi", "stati", "firmi", "nepali",
            "pento", "reco", "recatevi", "pentirmi",
        )
    }
)


def frequency(word):
    return FREQUENCY.get(word, 0.0)


def top_n(n):
    return TOP[:n]


# MarkIT's counts, doubled: `porta` read as portare, `vite` as vita, `lo` as the pronoun.
COUNTS = collections.Counter(
    {
        ("porta", "portare"): 3,
        ("porta", "porta"): 1,
        ("vite", "vita"): 2,
        ("lo", "lo"): 5,
        ("lo", "il"): 2,
        ("anglo-americano", "anglo-americano"): 1,
        ("stato", "stato"): 29,
        ("stato", "essere"): 9,
        ("ti", "ti"): 15,
    }
)


def section(entries=ENTRIES):
    survey = red.Survey()
    for e in entries:
        survey.read(e)
    lexicon = red.Lexicon(survey)
    for e in entries:
        lexicon.read(e)
    lexicon.drop_clitic_rows()
    return lexicon


def reduce(max_lemmas=200, entries=ENTRIES, counts=COUNTS):
    lexicon = section(entries)
    red.name_or_word(lexicon.candidates, lexicon.poses, frequency)
    red.own_words(lexicon)
    red.read_apocopes(lexicon)
    forms, ranks, step = red.reduce_forms(lexicon, counts, treebank.markit_zipf(counts), frequency, top_n, max_lemmas)
    red.seed_pieces(forms)
    return lexicon, forms, ranks, step


class Forms(unittest.TestCase):
    """What a form is (D3)."""

    @classmethod
    def setUpClass(cls):
        cls.lexicon, cls.forms, cls.ranks, _ = reduce()

    def test_spec_scenario_a_participle_s_agreement(self):
        # `creduta` is listed only as the feminine of the participle `creduto`, filed under credere.
        self.assertEqual(self.forms["creduta"], "credere")
        self.assertEqual(self.forms["creduti"], "credere")

    def test_spec_scenario_the_table_s_stress_marks(self):
        for form in ("dare", "dato", "diedi", "darà", "dà", "darebbe"):
            self.assertEqual(self.forms.get(form), "dare", form)
        # `ha`: avere's entry for the cell is the bare spelling.
        self.assertEqual(self.forms["ha"], "avere")
        self.assertNotIn("hà", self.forms)
        # `do` is the bare spelling of dò, whose cell `do`'s entry carries.
        self.assertNotIn("dò", self.forms)
        # `da` the preposition is no form of dare, and a marked row never reaches the table.
        self.assertEqual(self.forms["da"], "da")
        self.assertFalse(any(c in "àèéìíòóùú" for f in self.forms for c in f[:-1].rstrip("'")), "a mark not on the last letter")

    def test_the_stress_helpers(self):
        self.assertEqual(red.without_stress("dàre"), "dare")
        self.assertEqual(red.without_stress("darèbbe"), "darebbe")
        self.assertEqual(red.without_stress("darà"), "darà")
        self.assertEqual(red.without_stress("dà'"), "dà'")
        # A loan's marks are spelling, not stress.
        self.assertEqual(red.without_stress("würstel"), "würstel")
        self.assertEqual(red.bare("hà"), "ha")
        self.assertEqual(red.bare("dà'"), "da'")
        self.assertIsNone(red.bare("dare"))

    def test_spec_scenario_a_row_written_with_its_pronoun(self):
        self.assertEqual(self.forms["lavo"], "lavare")
        self.assertEqual(self.forms["lava"], "lavare")
        self.assertNotIn("lavarsi", self.forms)
        self.assertNotIn("lavarsi", self.ranks)
        self.assertNotIn("lavarsi", self.lexicon.lemmas)
        for form in ("pento", "pente", "pentire", "pentirsi", "pentendo", "pentite"):
            self.assertEqual(self.forms.get(form), "pentirsi", form)
        # Its rows carrying the pronoun are no forms: the enclitic rule reads them.
        for form in ("pentendosi", "pentitevi", "lavatosi", "lavandosi", "lavati"):
            self.assertNotIn(form, self.forms, form)
        self.assertFalse(any(" " in form for form in self.forms))

    def test_a_row_carrying_another_clitic_gives_no_form(self):
        # andarsene's rows (`me ne vàdo`, `vàttene`) give nothing, and andarsene is no lemma.
        self.assertNotIn("andarsene", self.forms)
        self.assertNotIn("andarsene", self.ranks)
        self.assertNotIn("vattene", self.forms)
        self.assertEqual(self.forms["vado"], "andare")

    def test_a_multi_word_cell_a_bracket_and_alternatives_give_no_form(self):
        self.assertNotIn("stò", self.forms)
        self.assertNotIn("sto zitto", self.forms)
        self.assertFalse(any("[" in f or "," in f for f in self.forms))
        self.assertNotIn("pote", self.forms)
        self.assertEqual(self.forms["può"], "potere")

    def test_spec_scenario_a_derived_word(self):
        # casa's diminutive and augmentative rows, vero's adverb row and mare's relational adjective
        # are no forms of them; each is read from its own entry.
        self.assertNotIn("casétta", self.forms)
        self.assertNotIn("casone", self.forms)
        self.assertEqual(self.forms["casetta"], "casetta")
        self.assertEqual(self.forms["casette"], "casetta")
        self.assertEqual(self.forms["veramente"], "veramente")
        self.assertNotIn("marittimo", self.forms)
        self.assertEqual(self.forms["verissimo"], "vero")

    def test_spec_scenario_a_doubtful_inflection(self):
        self.assertNotIn("bello'", self.forms)
        self.assertEqual(self.forms["belli"], "bello")
        self.assertNotIn("ca'", self.forms)
        self.assertNotIn("dettero", self.forms)

    def test_bookkeeping_the_auxiliary_and_a_gender_marker_are_no_forms(self):
        self.assertNotIn("irregular", self.forms)
        self.assertNotIn("it-conj", self.forms)
        # avere's table names its auxiliary: no form of dare.
        self.assertEqual(self.forms["avere"], "avere")
        self.assertNotIn("m", self.forms)

    def test_letters(self):
        for word in ("würstel", "crêpe", "c'era-una-volta", "po'", "l'"):
            self.assertTrue(red._TOKEN.fullmatch(word), word)
        for word in ("abc1", "a b", "-lo", "'ndrangheta"):
            self.assertFalse(red._TOKEN.fullmatch(word), word)
        self.assertEqual(red.nfc_lower("Città’"), "città'")


class Tokenisation(unittest.TestCase):
    """The tables serve I7, as change 6 writes it (D4)."""

    @classmethod
    def setUpClass(cls):
        cls.lexicon, cls.forms, cls.ranks, _ = reduce()

    def test_spec_scenario_the_elided_pieces_are_forms(self):
        self.assertEqual(self.forms["lo"], "lo")
        self.assertEqual(self.forms["dello"], "dello")
        self.assertEqual(self.forms["una"], "uno")
        self.assertEqual(self.forms["questo"], "questo")
        self.assertEqual(self.forms["anche"], "anche")
        self.assertEqual(self.forms["l'"], "lo")
        self.assertEqual(self.forms["un'"], "uno")
        self.assertEqual(self.forms["quest'"], "questo")
        self.assertEqual(self.forms["anch'"], "anche")
        self.assertEqual(self.forms["dell'"], "dello")

    def test_spec_scenario_articulated_prepositions(self):
        self.assertEqual(self.forms["del"], "del")
        self.assertEqual(self.forms["della"], "della")
        self.assertIn("del", self.ranks)
        for split in ("nel", "al"):
            self.assertNotIn(split, self.forms)
            self.assertNotIn(split, self.ranks)
        self.assertEqual(self.forms["dai"], "dare")
        self.assertNotIn("dai", self.ranks)

    def test_truncations_read_as_their_word_s_lemma(self):
        self.assertEqual(self.forms["po'"], "poco")
        self.assertEqual(self.forms["da'"], "dare")
        self.assertEqual(self.forms["va'"], "andare")

    def test_spec_scenario_a_form_carrying_clitics(self):
        for word in ("dirmelo", "vattene", "farlo"):
            self.assertNotIn(word, self.forms, word)
            self.assertNotIn(word, self.ranks, word)
        # A noun spelled like a compound stands.
        self.assertEqual(self.forms["fallo"], "fallo")

    def test_no_plain_word_beginning_with_an_elided_piece(self):
        for word in ("c'è", "dell'anno", "dell'acqua", "l'ho", "dov'è", "l'anglo-americano"):
            self.assertFalse(red.is_form(word), word)
            self.assertNotIn(word, self.forms)
        for word in ("po'", "l'", "coll'ago"):
            self.assertTrue(red.is_form(word), word)

    def test_every_word_the_pre_pass_writes(self):
        words = red.prepass_words()
        for word in ("lo", "una", "dissi", "dello", "del", "a", "il", "i", "gli", "poco", "dai", "dici", "vai"):
            self.assertIn(word, words)
        self.assertNotIn("nel", words)
        self.assertEqual(red.unwritten_words({w: w for w in words}), [])
        self.assertIn("dissi", red.unwritten_words({}))

    def test_the_closed_tables_are_change_6_s(self):
        # The reviewed tables are lingua-core's, as tokenize.rs writes them: a reduction cannot drift
        # from the pre-pass it serves.
        with open(_TOKENIZE_RS, encoding="utf-8") as f:
            source = f.read()

        def table(name):
            body = source[source.index(f"pub const {name}"):]
            body = body[: body.index("];")]
            return [tuple(re.findall(r'"([^"]*)"|None', row)) for row in re.findall(r"^\s*\((.*)\),\s*$", body, re.M)]

        elisions = {f"{w}'": r for w, r in table("ITALIAN_ELISIONS")}
        self.assertEqual(elisions, red.ELISIONS)
        self.assertEqual({w: (p, a) for w, p, a in table("ITALIAN_CONTRACTIONS")}, red.CONTRACTIONS)
        truncations = {f"{w}'": r for w, r in table("ITALIAN_TRUNCATIONS")}
        self.assertEqual(truncations, red.TRUNCATIONS)
        elided = {}
        for row in re.findall(r'^\s*\("(\w+)", "(\w+)", (?:Some\("(\w+)"\)|None)\),\s*$', source[source.index("pub const ITALIAN_ELIDED_PREPOSITIONS"):], re.M):
            elided[f"{row[0]}'"] = tuple(x for x in row[1:] if x)
        self.assertEqual(elided, red.ELIDED_PREPOSITIONS)
        self.assertIn('"disse" if next == "io" => "dissi"', source)
        self.assertEqual(red.CONTEXT_WORDS, ("dissi",))


class Ranks(unittest.TestCase):
    """Lemmas and ranks (D6)."""

    @classmethod
    def setUpClass(cls):
        cls.lexicon, cls.forms, cls.ranks, _ = reduce()

    def test_spec_scenario_wordfreq_s_elision_stems(self):
        for stem in ("l", "d", "po", "quest", "dell'acqua"):
            self.assertIn(stem, TOP)
            self.assertNotIn(stem, self.ranks)
            self.assertNotIn(stem, self.forms)
        self.assertEqual(self.forms["un"], "uno")
        self.assertEqual(self.forms["buon"], "buono")
        # Dense: the ranks close up over what was left out.
        self.assertEqual(sorted(self.ranks.values()), list(range(1, len(self.ranks) + 1)))

    def test_the_stems_wordfreq_counts(self):
        for stem in ("l", "d", "gl", "ch", "quest", "dov", "all", "dell", "po", "mo", "be"):
            self.assertIn(stem, red.WORDFREQ_STEMS)
        for word in ("un", "buon", "nessun", "da", "di", "fa", "va", "sta"):
            self.assertNotIn(word, red.WORDFREQ_STEMS)

    def test_a_hyphenated_word_ranked_by_markit_s_evidence(self):
        self.assertIn("anglo-americano", self.ranks)
        self.assertNotIn("nord-est", self.ranks)

    def test_spec_scenario_every_rank_on_its_own_lemma(self):
        for lemma in self.ranks:
            self.assertEqual(self.forms[lemma], lemma, lemma)

    def test_no_infinitive_or_form_carrying_clitics_is_ranked(self):
        for word in ("lavarsi", "farlo", "dirmelo", "andarsene"):
            self.assertNotIn(word, self.ranks, word)
        self.assertIn("pentirsi", self.ranks)

    def test_the_cut(self):
        _, forms, ranks, _ = reduce(max_lemmas=5)
        self.assertEqual(len(ranks), 5)
        # `l`, `un` (uno's apocope) and `d` are passed over: the cut closes up.
        self.assertEqual(set(ranks), {"di", "il", "la", "lo", "da"})
        for lemma in ranks:
            self.assertEqual(forms[lemma], lemma)


class OneLemma(unittest.TestCase):
    """One lemma per form (D5)."""

    @classmethod
    def setUpClass(cls):
        cls.lexicon, cls.forms, cls.ranks, step = reduce()
        cls.step = staticmethod(step)

    def test_spec_scenario_one_lemma_per_form(self):
        # MarkIT's counts favour portare.
        self.assertEqual(self.forms["porta"], "portare")
        self.assertNotIn("porta", self.ranks)
        # With no count, the form's own entry: the law, not leggere's form.
        self.assertEqual(self.forms["legge"], "legge")
        # And the tables name the rule that decided.
        decided = red.decisions(self.forms, self.lexicon, self.step, COUNTS, frequency)
        self.assertGreaterEqual(decided["MarkIT's counts"], 1)
        self.assertGreaterEqual(decided["its own entry"], 1)
        lemmas = self.lexicon.lemmas
        self.assertEqual(red.decided_by("porta", {"porta", "portare"}, COUNTS, lemmas, frequency), "MarkIT's counts")
        self.assertEqual(red.decided_by("legge", {"legge", "leggere"}, COUNTS, lemmas, frequency), "its own entry")
        self.assertEqual(red.decided_by("x", {"bianco", "porta"}, COUNTS, lemmas, frequency), "frequency")
        self.assertEqual(red.decided_by("x", {"a", "b"}, COUNTS, lemmas, frequency), "alphabetical order")
        self.assertEqual(red.decided_by("x", {"a", "b"}, COUNTS, lemmas, frequency, {"x": ("b", "why")}), "override")

    def test_the_owner_s_overrides_decide_first(self):
        # stato and stati read as essere (the owner, 2026-10-23), bianca as bianco; each row says why.
        self.assertEqual({form: lemma for form, (lemma, _) in red.OVERRIDES.items()}, {"stato": "essere", "stati": "essere", "bianca": "bianco"})
        self.assertTrue(all(len(reason) > 40 for _, reason in red.OVERRIDES.values()))
        self.assertEqual(self.forms["stato"], "essere")
        self.assertEqual(self.forms["stati"], "essere")
        self.assertNotIn("stato", self.ranks, "the noun leaves the pack: the row's cost")
        self.assertEqual(red.choose_lemma("porta", {"porta", "portare"}, COUNTS, {"porta"}, frequency, {"porta": ("porta", "why")}), "porta")

    def test_spec_scenario_a_name_and_a_word(self):
        # `bianca`: the name, a cue ball and bianco's feminine — the commoner reading by wordfreq.
        self.assertEqual(self.forms["bianca"], "bianco")
        # `roma`: a name and a rice, one lemma string: the city stays.
        self.assertEqual(self.forms["roma"], "roma")
        self.assertIn("roma", self.ranks)
        # A name alone, spelled like another word's form, reads as the word.
        self.assertEqual(self.forms["bambini"], "bambino")
        self.assertNotIn("bambini", self.ranks)

    def test_the_names_rule_reads_a_name_alone_and_nothing_else(self):
        # A form whose own entries are names alone reads as the other word; one with a common reading
        # of its own beside a name is D5's, whatever wordfreq says; a name no word reads stays, and a
        # name another name reads is no word's.
        poses = {"bianca": {"noun", "name"}, "bianco": {"adj"}, "bambini": {"name"}, "bambino": {"noun"}, "roma": {"name"}, "maria": {"name"}, "mario": {"name"}}
        candidates = {"bianca": {"bianca", "bianco"}, "bambini": {"bambini", "bambino"}, "roma": {"roma"}, "maria": {"maria", "mario"}}
        moved = red.name_or_word(candidates, poses, lambda w: 9.0)
        self.assertEqual(candidates, {"bianca": {"bianca", "bianco"}, "bambini": {"bambino"}, "roma": {"roma"}, "maria": {"maria", "mario"}})
        self.assertEqual(moved, {"names read as another word": 1})
        # On the section: `ti`, a surname beside the pronoun, is the pronoun by MarkIT's counts;
        # `firmato`, a given name alone, the participle of firmare.
        self.assertEqual(self.forms["ti"], "ti")
        self.assertEqual(self.forms["firmato"], "firmare")
        self.assertNotIn("firmato", self.ranks)

    def test_spec_scenario_an_apocope(self):
        # `bel`'s entry is a meaning and `san`'s own entry the Greek letter: the reviewed table reads
        # each as its word alone, and neither is a lemma; `un` and `buon` as before.
        self.assertEqual(self.forms["bel"], "bello")
        self.assertEqual(self.forms["san"], "santo")
        for form in ("bel", "san", "un", "buon"):
            self.assertNotIn(form, self.ranks, form)
            self.assertNotIn(form, self.lexicon.lemmas, form)
        self.assertEqual(self.forms["un"], "uno")
        for form, (lemma, reason) in red.APOCOPES.items():
            self.assertTrue(red.is_form(form) and red.is_form(lemma) and reason, form)
        self.assertNotIn("bei", red.APOCOPES, "bello's plural before a consonant, no apocope")
        # `cent`, the euro cent, is a word; `cent'` a piece.
        self.assertNotIn("cent", red.WORDFREQ_STEMS)
        self.assertIn("cent", self.ranks)

    def test_a_finite_form_is_no_infinitive_carrying_clitics(self):
        # `importi` ends as `impor` + `ti` would, and imporre is a verb: its entry is a finite form's.
        self.assertIsNone(self.lexicon.survey.clitic_infinitive("importi"))
        self.assertEqual(self.forms["importi"], "importare")
        self.assertEqual(self.lexicon.survey.clitic_infinitive("recarsi"), ("si", "recare"))

    def test_a_form_carrying_clitics_its_own_entry_links_is_no_form(self):
        for form in ("recatevi", "pentirmi", "stammi"):
            self.assertNotIn(form, self.forms, form)
            self.assertNotIn(form, self.ranks, form)
            self.assertIn(form, self.lexicon.unranked, form)
        self.assertGreaterEqual(self.lexicon.stats["forms carrying clitics their own entry links"], 2)
        self.assertEqual(self.forms["reco"], "recare")
        self.assertEqual(self.forms["pento"], "pentirsi")

    def test_a_compound_beside_a_surname_is_no_rank(self):
        self.assertNotIn("donati", self.ranks)
        self.assertEqual(self.forms["donati"], "donare")
        self.assertNotIn("donati", self.lexicon.survey.standing)

    def test_a_name_s_rows_are_no_forms(self):
        # Nepal's demonym and its plural are no forms of the name: `nepalese` is wordfreq's word.
        self.assertNotIn("nepali", self.forms)
        self.assertEqual(self.forms["nepalese"], "nepalese")

    def test_m8_s_cost(self):
        cost = red.m8_cost(self.forms, self.ranks, self.lexicon, top_n, tops=(25, 50))
        self.assertIn((24, "porta", "portare"), cost[25])
        self.assertEqual(cost[25], cost[50])


class MorphIt(unittest.TestCase):
    """Morph-it!'s additions (D3)."""

    @classmethod
    def setUpClass(cls):
        cls.lexicon, cls.forms, cls.ranks, _ = reduce()

    def test_spec_scenario_morph_it_completes_a_verb(self):
        pairs = {("credettero", "credere"), ("crederono", "credere")}
        added = red.morph_it_additions(pairs, self.forms, self.ranks, self.lexicon, lambda w: 2.0)
        self.assertEqual(added, {"credettero": "credere", "crederono": "credere"})

    def test_spec_scenario_morph_it_never_moves_a_reading(self):
        # `porta`, which the Wiktionary reads, and `vite`, which it lists under vita and vite.
        pairs = {("porta", "porgere"), ("vite", "vite"), ("porte", "portare")}
        self.assertEqual(red.morph_it_additions(pairs, self.forms, self.ranks, self.lexicon, frequency), {})

    def test_morph_it_adds_under_a_ranked_lemma_alone(self):
        pairs = {("xyzzi", "xyzzo"), ("credetti", "credere"), ("farlo", "fare"), ("nel", "in"), ("bianca", "bianco")}
        added = red.morph_it_additions(pairs, self.forms, self.ranks, self.lexicon, lambda w: 2.0 if w != "nonattested" else 0.0)
        self.assertEqual(added, {"credetti": "credere"})
        self.assertEqual(red.morph_it_additions({("credetti", "credere")}, self.forms, self.ranks, self.lexicon, lambda w: 0.0), {})

    def test_reading_the_archive(self):
        lines = "\n".join(
            [
                "credettero\tcredere\tVER:ind+past+3+p",
                "crederlo\tcredere\tVER:inf+pres+lo",
                "Città\tcittà\tNOUN-F:s",
                "cent'\tcento\tDET-NUM-CARD",
                "broken line",
                "più\tpiù\tADV",
            ]
        ).encode("latin-1")
        with tempfile.TemporaryDirectory() as tmp:
            path = os.path.join(tmp, red.MORPH_IT)
            with tarfile.open(path, "w:gz") as archive:
                info = tarfile.TarInfo(red.MORPH_IT_MEMBER)
                info.size = len(lines)
                archive.addfile(info, io.BytesIO(lines))
            pairs = red.read_morph_it(path)
        self.assertEqual(pairs, {("credettero", "credere"), ("città", "città"), ("cent'", "cento"), ("più", "più")})


class Treebank(unittest.TestCase):
    """UD Italian-MarkIT's training and development sections (D2, D5, D6)."""

    SAMPLE = (
        "# sent_id = 1\n"
        "1-2\tdel\t_\t_\t_\t_\t_\t_\t_\t_\n"
        "1\tdi\tdi\tADP\t_\t_\t3\tcase\t_\t_\n"
        "2\til\til\tDET\t_\t_\t3\tdet\t_\t_\n"
        "3\tPorta\tporta\tNOUN\t_\t_\t0\troot\t_\t_\n"
        "4\tporta\tportare\tVERB\t_\t_\t3\tacl\t_\t_\n"
        "5\tè\tessere\tAUX\t_\t_\t4\tcop\t_\t_\n"
        "6\tanglo-americano\tanglo-americano\tADJ\t_\t_\t3\tamod\t_\t_\n"
        "7\tconseguenza\tconseguenza\tNOUN\t_\t_\t3\tfixed\t_\t_\n"
        "7.1\tx\tx\tX\t_\t_\t_\t_\t_\t_\n"
        "8\tL’\til\tDET\t_\t_\t3\tdet\t_\t_\n"
    )

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.train = os.path.join(self._tmp.name, "it_markit-ud-train.conllu")
        with open(self.train, "w", encoding="utf-8") as f:
            f.write(self.SAMPLE)

    def tearDown(self):
        self._tmp.cleanup()

    def test_counts_of_a_form_under_each_lemma(self):
        counts = treebank.markit_counts([self.train])
        self.assertEqual(counts[("porta", "porta")], 1)
        self.assertEqual(counts[("porta", "portare")], 1)
        self.assertEqual(counts[("l'", "il")], 1)
        # A multi-word token and an empty node carry no lemma of their own.
        self.assertNotIn(("del", "_"), counts)
        self.assertNotIn(("x", "x"), counts)
        self.assertEqual(sum(counts.values()), 8)

    def test_a_hyphenated_lemma_s_own_frequency(self):
        zipf = treebank.markit_zipf(treebank.markit_counts([self.train]))
        self.assertEqual(set(zipf), {"anglo-americano"})
        self.assertAlmostEqual(zipf["anglo-americano"], 8.0969, places=3)

    def test_the_part_of_speech_of_a_word(self):
        pos = treebank.markit_pos_counts([self.train])
        self.assertEqual(pos[("portare", "VERB")], 1)
        self.assertEqual(pos[("essere", "VERB")], 1)
        self.assertEqual(pos[("il", "DET")], 1)
        self.assertEqual(pos[("l'", "DET")], 1)
        self.assertNotIn(("conseguenza", "NOUN"), pos)

    def test_spec_scenario_the_test_section_is_held_out(self):
        for name in ("it_markit-ud-test.conllu", "it_pud-ud-test.conllu"):
            path = os.path.join(self._tmp.name, name)
            with open(path, "w", encoding="utf-8") as f:
                f.write(self.SAMPLE)
            with self.assertRaises(ValueError):
                treebank.markit_counts([path])
            with self.assertRaises(ValueError):
                treebank.markit_pos_counts([path])
        self.assertEqual(red.MARKIT, ("it_markit-ud-train.conllu", "it_markit-ud-dev.conllu"))


def misspelt(word, *targets, pos="adv", tags=("alt-of", "misspelling")):
    """An entry each sense of which spells one of `targets` (`perchè`: « misspelling of perché »)."""
    return entry(
        word,
        pos=pos,
        senses=[{"glosses": [f"misspelling of {t}"], "tags": list(tags), "alt_of": [{"word": t}]} for t in targets],
    )


# The dictionary's spellings (read-lingua-italian-unmarked-spellings D2): one entry or more per case.
SPELLING_ENTRIES = [
    entry("perché", pos="conj", senses=[{"glosses": ["because, why"]}]),
    misspelt("perchè", "perché", pos="conj"),
    misspelt("perchè", "perché", pos="adv"),
    form_of("perchèe", "perchè", pos="conj"),
    entry("caffè", senses=[{"glosses": ["coffee"]}]),
    misspelt("caffé", "caffè", pos="noun"),
    entry("soprattutto", pos="adv", senses=[{"glosses": ["above all"]}]),
    misspelt("sopratutto", "soprattutto", tags=("alt-of", "alternative", "proscribed")),
    # A pointer at one of change 6's pieces is no word of the spelling's: `fà` spells `fa`.
    entry("fare", pos="verb", forms=[*TABLE_HEAD, ("fà", ["misspelling"]), ("fa", PRESENT_3S)]),
    form_of("fa", "fare"),
    misspelt("fà", "fa", "fa'", pos="verb"),
    # Two words: kept apart.
    entry("né", pos="conj"),
    entry("neh", pos="intj"),
    misspelt("nè", "né", pos="conj"),
    misspelt("nè", "neh", pos="intj"),
    # A sense of its own beside the spelling: `e'`, also the apocope of *ei*.
    entry("è", pos="verb", senses=[{"tags": ["form-of"], "form_of": [{"word": "essere"}]}]),
    entry("ei", pos="pron"),
    misspelt("e'", "è", pos="verb"),
    alt_of("e'", "ei", pos="pron", tags=("abbreviation", "alt-of", "apocopic")),
    entry("fico", pos="noun", senses=[{"glosses": ["fig"]}]),
    misspelt("figo", "fico", pos="adj", tags=("alt-of", "alternative")),
    # The word rarer than the spelling: kept apart.
    entry("ubbidire", pos="verb"),
    misspelt("obbedire", "ubbidire", pos="verb", tags=("alt-of", "alternative")),
    # A form of the spelling as common as its word: kept apart (change 43d's forms guard).
    entry("altero", pos="adj"),
    entry("altiero", pos="adj", forms=[("altiera", ["feminine"])], senses=[{"glosses": ["x"], "tags": ["alt-of", "alternative"], "alt_of": [{"word": "altero"}]}]),
    # A pointer at several words spells no word: a meaning of its own.
    entry("vabbè", pos="intj", senses=[{"glosses": ["alternative form of va bene"], "tags": ["alt-of", "alternative"], "alt_of": [{"word": "va bene"}]}]),
    # A spelling whose word is no form: kept apart.
    misspelt("daccordo", "d'accordo", pos="intj", tags=("alt-of", "alternative")),
    # A spelling's forms follow it where no other word claims them: `sguardi` stays sguardo's.
    entry("guardare", pos="verb", forms=[*TABLE_HEAD, ("guàrdi", ["indicative", "present", "second-person", "singular"])]),
    entry("sguardo", forms=[("sguardi", ["plural"])], senses=[{"glosses": ["look, glance"]}]),
    entry(
        "sguardare",
        pos="verb",
        forms=[*TABLE_HEAD, ("sguàrdi", ["indicative", "present", "second-person", "singular"]), ("sguardàvo", ["first-person", "imperfect", "indicative", "singular"])],
        senses=[{"glosses": ["alternative form of guardare"], "tags": ["alt-of", "alternative", "obsolete"], "alt_of": [{"word": "guardare"}]}],
    ),
    # A spelling the owner keeps apart: `casinò`, the gambling house.
    entry("casino", senses=[{"glosses": ["mess"]}]),
    misspelt("casinò", "casino", pos="noun", tags=("alt-of", "alternative")),
    # A row of a reviewed table is that table's: an apocope, a row of the web's list.
    misspelt("giu", "giu.", pos="noun", tags=("alt-of", "alternative")),
    misspelt("bel", "bello", pos="adj", tags=("alt-of", "alternative")),
]
SPELLING_FREQUENCY = {
    "perché": 6.18, "perchè": 5.65, "perchèe": 1.0, "caffè": 4.83, "caffé": 3.40, "soprattutto": 5.43,
    "sopratutto": 4.21, "fa": 6.09, "fà": 3.72, "fare": 6.15, "né": 5.21, "neh": 3.0, "nè": 4.55, "è": 7.10,
    "e'": 7.46, "fico": 4.0, "figo": 4.5, "ubbidire": 3.0, "obbedire": 3.2, "altero": 3.0, "altiero": 2.0,
    "altiera": 3.5, "vabbè": 4.4, "giu": 4.08, "daccordo": 3.5, "bel": 5.15, "bello": 5.3, "guardare": 5.2,
    "sguardo": 4.4, "sguardi": 3.89, "sguardare": 1.0, "sguardavo": 1.0, "casino": 4.58, "casinò": 3.77,
}


def _section_file(folder, entries):
    path = os.path.join(folder, red.KAIKKI)
    with open(path, "w", encoding="utf-8") as f:
        f.write("not json\n")
        for e in entries:
            f.write(json.dumps(e, ensure_ascii=False) + "\n")
    return path


def _spellings(entries=SPELLING_ENTRIES):
    kept = {}
    with tempfile.TemporaryDirectory() as folder:
        table = red.dictionary_spellings(_section_file(folder, entries), lambda w: SPELLING_FREQUENCY.get(w, 0.0), kept)
    return table, kept


class DictionarySpellings(unittest.TestCase):
    """read-lingua-italian-unmarked-spellings D2: a spelling the section gives only as a marked word's is
    a form of it when the word is the commoner."""

    @classmethod
    def setUpClass(cls):
        cls.table, cls.kept = _spellings()

    def test_spec_scenario_a_dictionary_spelling(self):
        # Another accent, a letter, no marks condition.
        self.assertEqual(self.table["perchè"], "perché")
        self.assertEqual(self.table["caffé"], "caffè")
        self.assertEqual(self.table["sopratutto"], "soprattutto")
        # A pointer at a piece is the piece's: `fà` spells `fa`.
        self.assertEqual(self.table["fà"], "fa")
        self.assertEqual(
            self.table,
            {"perchè": "perché", "caffé": "caffè", "sopratutto": "soprattutto", "fà": "fa", "sguardare": "guardare"},
        )
        # Read as a form of its word, nothing of its own; an entry pointing at it follows it.
        survey = red.Survey()
        for e in SPELLING_ENTRIES:
            survey.read(e)
        lexicon = red.Lexicon(survey, self.table)
        for e in SPELLING_ENTRIES:
            lexicon.read(e)
        self.assertEqual(lexicon.candidates["perchè"], {"perché"})
        self.assertNotIn("perchè", lexicon.lemmas)
        self.assertEqual(lexicon.candidates["perchèe"], {"perchè"})
        step = lambda word, pos: red.follow(word, pos, lexicon.links, lexicon.poses, lambda w, o: min(o))  # noqa: E731
        self.assertEqual(red.reach("perchèe", lexicon.candidates["perchèe"], lexicon.links, step), {"perché"})
        # Without the table, the section makes `perchè` a word of its own.
        alone = red.Lexicon(survey)
        for e in SPELLING_ENTRIES:
            alone.read(e)
        self.assertIn("perchè", alone.lemmas)

    def test_spec_scenario_a_spelling_with_a_sense_of_its_own(self):
        # `e'` is also the apocope of *ei*: no spelling, and not refused either — a word.
        self.assertNotIn("e'", self.table)
        self.assertNotIn("e'", self.kept)
        self.assertNotIn("vabbè", self.table)
        self.assertNotIn("vabbè", self.kept)
        self.assertIsNone(red.spelling_target({"tags": ["alt-of"], "alt_of": [{"word": "va bene"}]}))
        self.assertIsNone(red.spelling_target({"tags": ["alt-of", "apocopic"], "alt_of": [{"word": "ei"}]}))
        self.assertEqual(red.spelling_target({"tags": ["alt-of"], "alt_of": [{"word": "Perché"}]}), "perché")

    def test_spec_scenario_the_marked_word_is_not_the_commoner(self):
        self.assertEqual(self.kept["figo"], red.KEPT_COMMONER)
        self.assertEqual(self.kept["obbedire"], red.KEPT_COMMONER)
        self.assertEqual(self.kept["altiero"], red.KEPT_FORM)
        self.assertEqual(self.kept["nè"], red.KEPT_TWO_WORDS)
        self.assertEqual(self.kept["daccordo"], red.KEPT_NO_FORM)
        self.assertEqual(self.kept["bel"], red.KEPT_TABLE)
        self.assertEqual(self.kept["giu"], red.KEPT_TABLE)
        # The owner's three (task 4.1, 2026-10-23), each with its reason.
        self.assertEqual(self.kept["casinò"], red.KEPT_OWNER)
        self.assertNotIn("casinò", self.table)
        self.assertEqual(set(red.SPELLINGS_KEPT_APART), {"casinò", "omg", "cè"})
        self.assertTrue(all(red.SPELLINGS_KEPT_APART.values()))
        self.assertIn(f"{red.KEPT_OWNER}=1: casinò", "\n".join(red.spellings_report(self.table, self.kept)))
        # The report names each one read and each one kept apart, by its reason.
        lines = red.spellings_report(self.table, self.kept)
        self.assertTrue(lines[0].startswith("dictionary spellings read as their word=5: caffé → caffè, fà → fa"))
        self.assertIn(f"{red.KEPT_COMMONER}=2: figo, obbedire", "\n".join(lines))
        self.assertIn(f"{red.KEPT_TWO_WORDS}=1: nè", "\n".join(lines))

    def test_a_spelling_s_forms_another_word_claims_stay_its(self):
        # `sguardi` is sguardo's plural and a row of the archaic `sguardare`: it stays sguardo's — the
        # choice among its words would give it to guardare, the commoner; `sguardavo` follows.
        with tempfile.TemporaryDirectory() as folder:
            path = _section_file(folder, SPELLING_ENTRIES)
            lexicon = red.read_kaikki(path, red.survey_kaikki(path), self.table)
        self.assertEqual(lexicon.candidates["sguardi"], {"sguardo"})
        self.assertEqual(lexicon.candidates["sguardavo"], {"sguardare"})
        self.assertEqual(lexicon.candidates["sguardare"], {"guardare"})
        self.assertEqual(lexicon.stats["forms of a dictionary spelling another word claims"], 1)
        # A form two spellings claim follows them: another spelling is no other word.
        lexicon = red.Lexicon(red.Survey(), {"x": "y", "z": "y"})
        lexicon.link("x", "y", "verb")
        lexicon.link("z", "y", "verb")
        lexicon.link("w", "x", "verb")
        lexicon.link("w", "z", "verb")
        lexicon.drop_spelling_rows()
        self.assertEqual(lexicon.candidates["w"], {"x", "z"})

    def test_a_form_of_a_form_of_a_spelling_another_word_claims_stays_its(self):
        # At review: `accise` is *accisa*'s plural and a row of `acciso`, the participle of the archaic
        # `accidere` (« alternative form of uccidere »), whose one word is `accidere`: the paradigm is
        # transitive, `accise` stays accisa's, and `acciso`, which no other word claims, follows.
        entries = [
            entry("uccidere", pos="verb"),
            entry("accisa", forms=[("accise", ["plural"])], senses=[{"glosses": ["excise tax"]}]),
            entry("accidere", pos="verb", senses=[{"glosses": ["x"], "tags": ["alt-of", "alternative", "obsolete"], "alt_of": [{"word": "uccidere"}]}]),
            entry(
                "acciso",
                pos="verb",
                forms=[("accisa", ["feminine"]), ("accisi", ["masculine", "plural"]), ("accise", ["feminine", "plural"])],
                senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "accidere"}]}],
            ),
            form_of("accise", "accisa", pos="noun", tags=("feminine", "form-of", "plural")),
        ]
        freq = {"uccidere": 4.6, "accidere": 1.0, "accisa": 3.4, "accise": 3.13, "acciso": 1.0, "accisi": 1.0}
        with tempfile.TemporaryDirectory() as folder:
            path = _section_file(folder, entries)
            spellings = red.dictionary_spellings(path, lambda w: freq.get(w, 0.0))
            self.assertEqual(spellings, {"accidere": "uccidere"})
            lexicon = red.read_kaikki(path, red.survey_kaikki(path), spellings)
        self.assertEqual(lexicon.spelling_paradigms(), {"accidere", "acciso", "accisi"})
        self.assertEqual(lexicon.candidates["accise"], {"accisa"})
        self.assertEqual(lexicon.candidates["accisa"], {"accisa"})
        self.assertEqual(lexicon.candidates["accisi"], {"acciso"})
        top = ["uccidere", "accisa"]
        forms, ranks, _ = red.reduce_forms(lexicon, collections.Counter(), {}, lambda w: freq.get(w, 0.0), lambda n: top[:n], 10)
        self.assertEqual(forms["accise"], "accisa")
        self.assertEqual(forms["accisi"], "uccidere")
        self.assertEqual(forms["acciso"], "uccidere")

    def test_a_spelling_in_the_tables(self):
        # End to end, as the reduction reads it: a form of its word, no rank.
        with tempfile.TemporaryDirectory() as folder:
            path = _section_file(folder, SPELLING_ENTRIES)
            lexicon = red.read_kaikki(path, red.survey_kaikki(path), self.table)
        top = ["perché", "fare", "caffè", "soprattutto", "fa", "né"]
        freq = lambda w: SPELLING_FREQUENCY.get(w, 0.0)  # noqa: E731
        forms, ranks, _ = red.reduce_forms(lexicon, collections.Counter(), {}, freq, lambda n: top[:n], 20)
        self.assertEqual(forms["perchè"], "perché")
        self.assertEqual(forms["caffé"], "caffè")
        self.assertEqual(forms["fà"], "fare")
        for spelling in ("perchè", "caffé", "fà", "sopratutto"):
            self.assertNotIn(spelling, ranks)


# The web's spellings (read-lingua-italian-unmarked-spellings D3): the words a row reads as, the four
# exceptions' homographs, and what the boundary reads.
WEB_ENTRIES = [
    entry("più", pos="adv", senses=[{"glosses": ["more"]}]),
    entry("potere", pos="verb", forms=[*TABLE_HEAD, ("può", PRESENT_3S)]),
    form_of("può", "potere"),
    entry("già", pos="adv", senses=[{"glosses": ["already"]}]),
    entry("perché", pos="conj", senses=[{"glosses": ["because"]}], forms=[("perche", ["alternative", "obsolete"])]),
    entry("perca", forms=[("perche", ["plural"])], senses=[{"glosses": ["perch"], "tags": ["feminine"]}]),
    form_of("perche", "perca", pos="noun", tags=("form-of", "plural")),
    # An entry pointing at a row follows the row.
    form_of("puoo", "puo", pos="verb"),
]
WEB_FREQUENCY = {"più": 6.63, "piu": 5.22, "potere": 5.5, "può": 6.01, "puo": 4.61, "già": 5.95, "gia": 4.59,
                 "perché": 6.18, "perche": 4.95, "perca": 2.5, "puoo": 1.0}


def web_section(entries):
    with tempfile.TemporaryDirectory() as folder:
        path = _section_file(folder, entries)
        return red.read_kaikki(path, red.survey_kaikki(path), {})


class WebSpellings(unittest.TestCase):
    """read-lingua-italian-unmarked-spellings D3: the web's spellings without their accent, by a reviewed
    list and the owner's exceptions; a boundary at each reduction."""

    ROWS = {"piu": "più", "puo": "potere", "gia": "già"}
    EXCEPTIONS = {"perche": ("perché", "the owner's")}

    def reduced(self):
        with mock.patch.dict(red.WEB_SPELLINGS, self.ROWS, clear=True), mock.patch.dict(
            red.OWNER_EXCEPTIONS, self.EXCEPTIONS, clear=True
        ):
            lexicon = web_section(WEB_ENTRIES)
            crossed = lexicon.web_boundary()
            red.read_web_spellings(lexicon)
            freq = lambda w: WEB_FREQUENCY.get(w, 0.0)  # noqa: E731
            top = ["più", "perché", "già", "potere", "perca", "piu", "gia", "perche"]
            forms, ranks, _ = red.reduce_forms(lexicon, collections.Counter(), {}, freq, lambda n: top[:n], 20)
            unread = red.unread_web_spellings(forms)
        return lexicon, crossed, forms, ranks, unread

    def test_spec_scenario_a_web_spelling(self):
        lexicon, crossed, forms, ranks, unread = self.reduced()
        self.assertEqual(crossed, [])
        self.assertEqual(unread, [])
        self.assertEqual((forms["piu"], forms["puo"], forms["gia"]), ("più", "potere", "già"))
        # Nothing of its own: no rank — wordfreq ranks `piu` and `gia` as words of their own without
        # the list.
        for spelling in ("piu", "puo", "gia", "perche"):
            self.assertNotIn(spelling, ranks)
        self.assertEqual(forms["più"], "più")
        # An entry pointing at a row follows it.
        self.assertEqual(lexicon.candidates["puoo"], {"potere"})

    def test_spec_scenario_the_owner_s_exceptions(self):
        # `perche`, perca's plural in the section, reads as perché; perca keeps its own form.
        _, crossed, forms, _, _ = self.reduced()
        self.assertEqual(forms["perche"], "perché")
        self.assertEqual(forms["perca"], "perca")
        self.assertIsNone(red.web_spelling("perca"))
        # The owner's six (task 0.1, 2026-10-23): the four the design named, `gia` and `liberta`; and `pò`,
        # added at review on the owner's behalf.
        self.assertEqual(
            {s: w for s, (w, _) in red.OWNER_EXCEPTIONS.items()},
            {
                "perche": "perché",
                "cosi": "così",
                "citta": "città",
                "caffe": "caffè",
                "gia": "già",
                "liberta": "libertà",
                "pò": "poco",
            },
        )
        for spelling, (word, reason) in red.OWNER_EXCEPTIONS.items():
            self.assertNotIn(spelling, red.WEB_SPELLINGS)
            self.assertIn("wordfreq", reason, spelling)
            self.assertIn(word, reason, spelling)
            self.assertEqual(red.web_spelling(spelling), word)

    def test_spec_scenario_a_word_of_its_own_is_never_a_row(self):
        # The owner's list (tasks 2.2 and 4.1), sorted, one shared reason; never a word of its own, a
        # spelling commoner than its word, or one another language writes more.
        self.assertEqual(len(red.WEB_SPELLINGS), 100)
        self.assertEqual(list(red.WEB_SPELLINGS), sorted(red.WEB_SPELLINGS))
        for spelling, word in red.WEB_SPELLINGS.items():
            self.assertTrue(red.is_form(spelling) and "-" not in spelling, spelling)
            self.assertNotIn(spelling, red.OWNER_EXCEPTIONS)
            self.assertEqual(red.web_spelling(spelling), word)
            self.assertFalse(red._reviewed(spelling) and spelling not in red.WEB_SPELLINGS, spelling)
        for spelling in ("e", "da", "si", "se", "ne", "la", "li", "di", "fa", "va", "su", "po", "papa", "eta", "pero"):
            self.assertNotIn(spelling, red.WEB_SPELLINGS, spelling)
        # Spanish, French or English write it more (`cafe` 4.09 in English against 3.33; `verra`, French's).
        for spelling in ("cafe", "verra", "ando", "tribu", "dara", "potro", "carita", "arrivera", "finira", "agora"):
            self.assertNotIn(spelling, red.WEB_SPELLINGS, spelling)
        # The section's own homographs are no rows of the method: the owner's exceptions, and the ones
        # the method leaves, which the owner keeps as the tables read them (`finche`, `felicita`, `umidita`).
        for spelling in ("perche", "cosi", "citta", "caffe", "gia", "liberta", "umidita", "finche", "felicita"):
            self.assertNotIn(spelling, red.WEB_SPELLINGS, spelling)
        for spelling in ("umidita", "finche", "felicita"):
            self.assertNotIn(spelling, red.OWNER_EXCEPTIONS, spelling)

    def test_spec_scenario_a_dictionary_update_names_a_row(self):
        rows = {"cosi": "così", "gia": "già", "perche": "perché", "sara": "essere", "giu": "giù", "piu": "più", "puo": "potere"}
        entries = [
            # A form of a word with a meaning: a word of its own.
            entry("coso", forms=[("cosi", ["plural"])], senses=[{"glosses": ["thingy"]}]),
            form_of("cosi", "coso", pos="noun", tags=("form-of", "plural")),
            # An inflection a word's entry lists, its stress mark read as the tables read it.
            entry("gire", pos="verb", forms=[*TABLE_HEAD, ("gìa", ["imperfect", "indicative"])]),
            # Its word's spelling (D2's kind).
            misspelt("perche", "perché", pos="conj"),
            # A name's entry says nothing; a spelling of another word passes; a doubtful row is no row.
            entry("Sara", pos="name", senses=[{"glosses": ["a female given name"]}]),
            misspelt("giu", "giu.", pos="noun", tags=("alt-of", "alternative")),
            entry("più", pos="adv", forms=[("piu", ["misspelling"])]),
            # A form of a word without a meaning of its own passes.
            entry("potè", pos="verb", senses=[{"tags": ["form-of"], "form_of": [{"word": "potere"}]}]),
            form_of("puo", "potè"),
        ]
        with mock.patch.dict(red.WEB_SPELLINGS, rows, clear=True):
            self.assertEqual(web_section(entries).web_boundary(), ["cosi", "gia", "perche"])
            # A row D2 reads as a word crosses it too.
            survey = red.Survey()
            lexicon = red.Lexicon(survey, {"giu": "giù"})
            for e in entries:
                lexicon.read(e)
            self.assertIn("giu", lexicon.web_boundary())
        self.assertEqual(red._sense_kind({"glosses": ["x"]}), ("meaning", None))
        self.assertEqual(red._sense_kind({"tags": ["form-of"]}), ("form", ""))
        self.assertEqual(red._sense_kind({"tags": ["alt-of"], "alt_of": [{"word": "Più"}]}), ("alt", "più"))
        self.assertEqual(red._unmarked("perchè"), "perche")


def _read(work, name):
    with open(os.path.join(work, name), encoding="utf-8") as f:
        return f.read()


class Reduction(unittest.TestCase):
    """The reduction end to end, over a work folder (D9). The web's list and the owner's exceptions
    are the reviewed tables' real words, which this small section does not rank: each test reads one
    row this section ranks (read-lingua-italian-unmarked-spellings D3)."""

    WEB = {"puo": "potere"}

    def setUp(self):
        # Italian's acronym rows are the section's real headwords, which this small section has only
        # where a test writes them (keep-lingua-italian-acronyms-and-names D2).
        for table, rows in ((red.WEB_SPELLINGS, self.WEB), (red.OWNER_EXCEPTIONS, {}), (red.acronyms.ACRONYMS, {})):
            patch = mock.patch.dict(table, rows, clear=True)
            patch.start()
            self.addCleanup(patch.stop)

    def write_work(self, work, morph_it=False, extra=()):
        entries = list(ENTRIES) + list(extra)
        # Every word the pre-pass writes a form, as the real section makes it.
        listed = {e["word"].lower() for e in entries}
        entries += [entry(word, pos="pron") for word in red.prepass_words() if word not in listed]
        with open(os.path.join(work, red.KAIKKI), "w", encoding="utf-8") as f:
            f.write("not json\n")
            for e in entries:
                f.write(json.dumps(e, ensure_ascii=False) + "\n")
        for name in red.MARKIT:
            with open(os.path.join(work, name), "w", encoding="utf-8") as f:
                f.write(Treebank.SAMPLE)
        if morph_it:
            data = "credettero\tcredere\tVER:ind+past+3+p\n".encode("latin-1")
            with tarfile.open(os.path.join(work, red.MORPH_IT), "w:gz") as archive:
                info = tarfile.TarInfo(red.MORPH_IT_MEMBER)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))

    def run_reduction(self, work, morph_it, listed=True, foreign=None, extra=None):
        words = red.prepass_words()
        everything = {**{w: 3.0 for w in words}, **FREQUENCY, "credettero": 2.0, "puo": 2.0, **(extra or {})}
        listed = TOP + [w for w in words if w not in TOP] if listed else TOP
        listed += [w for w in (extra or {}) if w not in listed]
        lines = []
        summary = red.reduce(
            work,
            300,
            "2026.10.23",
            lambda w: everything.get(w, 0.0),
            lambda n: listed[:n],
            morph_it=morph_it,
            report=lines.append,
            foreign=foreign,
        )
        return summary, lines

    def test_a_form_only_a_spelling_reads_is_no_other_language_s_word_nor_a_name(self):
        # At review: `intrare` (« alternative form of entrare ») reads as entrare; its rows `intro` and
        # `intra` had no reading before. `intro`, which English writes more, and `intra`, a name to
        # Morph-it!, keep none; `intrava`, which neither says anything of, follows.
        spelling = entry(
            "intrare",
            pos="verb",
            forms=[*TABLE_HEAD, ("ìntro", PRESENT_1S), ("ìntra", PRESENT_3S), ("intràva", ["imperfect", "third-person"])],
            senses=[{"glosses": ["x"], "tags": ["alt-of", "alternative", "obsolete"], "alt_of": [{"word": "entrare"}]}],
        )
        # `intrare` itself is no word of wordfreq's: before D2 it was an unranked lemma, its rows unread.
        extra = {"entrare": 5.0, "intro": 3.25, "intra": 2.0, "intrava": 1.5}
        with tempfile.TemporaryDirectory() as work:
            self.write_work(work, morph_it=True, extra=[entry("entrare", pos="verb"), spelling])
            data = "intra\tIntra\tNPR\n".encode("latin-1")
            with tarfile.open(os.path.join(work, red.MORPH_IT), "w:gz") as archive:
                info = tarfile.TarInfo(red.MORPH_IT_MEMBER)
                info.size = len(data)
                archive.addfile(info, io.BytesIO(data))
            foreign = {"intro": 3.81}
            summary, lines = self.run_reduction(work, red.MORPH_IT, foreign=lambda w: foreign.get(w, 0.0), extra=extra)
            forms = dict(line.split("\t") for line in _read(work, "forms.tsv").splitlines())
            self.assertEqual(forms["intrava"], "entrare")
            self.assertNotIn("intro", forms)
            self.assertNotIn("intra", forms)
            self.assertEqual(summary["foreign readings"], ["intra", "intro"])
            self.assertTrue(any(line.endswith("name=2: intra, intro") for line in lines), lines)
            # Without the test, both read as entrare.
            summary, _ = self.run_reduction(work, red.MORPH_IT, extra=extra)
            forms = dict(line.split("\t") for line in _read(work, "forms.tsv").splitlines())
            self.assertEqual((forms["intro"], forms["intra"]), ("entrare", "entrare"))
            self.assertEqual(summary["foreign readings"], [])
        self.assertEqual(
            red.foreign_readings({"a": "x", "b": "x", "c": "x"}, {"a", "b"}, {"b"}, lambda w: 3.0, lambda w: 1.0, set()),
            [],
        )

    def test_a_reduction_writes_italian_s_tables_and_it_fr_s(self):
        with tempfile.TemporaryDirectory() as work:
            self.write_work(work, morph_it=True)
            summary, lines = self.run_reduction(work, red.MORPH_IT)
            read = lambda name: _read(work, name)  # noqa: E731
            forms = dict(line.split("\t") for line in read("forms.tsv").splitlines())
            self.assertEqual(forms["credettero"], "credere")
            self.assertEqual(forms["l'"], "lo")
            self.assertEqual(read("lexical.tsv"), "")
            self.assertEqual(read("gloss.tsv"), "")
            ranks = [line.split("\t") for line in read("freq.tsv").splitlines()]
            self.assertEqual([int(r) for _, r in ranks], list(range(1, len(ranks) + 1)))
            manifest = json.loads(read("manifest.json"))
            self.assertEqual(manifest["meta"]["studied"], "it")
            self.assertEqual(manifest["meta"]["native"], "fr")
            self.assertEqual(manifest["meta"]["pack_version"], "2026.10.23")
            self.assertEqual(manifest["meta"]["analyzer_version"], red.analyser_version())
            self.assertNotIn("levels_estimated", manifest["meta"])
            notice = read("NOTICE")
            for source in manifest["sources"]:
                self.assertIn(source["name"], notice)
            self.assertIn("Morph-it!", notice)
            self.assertNotIn("PUD", notice)
            self.assertEqual(summary["morph-it added"], 1)
            self.assertTrue(any("Morph-it! added 1 form" in line for line in lines))
            # The web's row, a form of its word with no rank (read-lingua-italian-unmarked-spellings D3);
            # the report names the dictionary's spellings and the list.
            self.assertEqual(forms["puo"], "potere")
            self.assertNotIn("puo", dict(ranks))
            self.assertEqual(summary["web spellings"], 1)
            self.assertTrue(any("1 row(s) of the reviewed list" in line for line in lines))
            self.assertTrue(any(line.startswith("reduced it-fr: dictionary spellings read") for line in lines))
            first = {name: read(name) for name in ("forms.tsv", "freq.tsv", "NOTICE", "manifest.json")}
            self.run_reduction(work, red.MORPH_IT)
            self.assertEqual({name: read(name) for name in first}, first, "the same inputs give the same tables")

    # keep-lingua-italian-acronyms-and-names D2, D6: the section's `NATO` and `USA` entries, `nato`
    # and `usa` read as their words, the rows its table holds.
    ACRONYM_ENTRIES = [
        entry("NATO", pos="name", senses=[{"glosses": ["NATO"], "tags": ["abbreviation", "acronym"]}]),
        {
            "word": "USA",
            "pos": "name",
            "senses": [{"glosses": ["initialism of the United States of America"], "alt_of": [{"word": "the United States of America"}]}],
            "synonyms": [{"word": "America"}, {"word": "Stati Uniti"}],
        },
        entry("nascere", pos="verb", forms=[*TABLE_HEAD, ("nàto", PARTICIPLE)]),
        entry("uso", pos="adj", forms=[("usa", ["feminine", "singular"])]),
        entry("ONU", pos="name", senses=[{"glosses": ["UN"]}]),
    ]
    ACRONYM_FREQUENCY = {"nascere": 4.0, "uso": 4.5, "onu": 3.5, "nato": 4.2, "usa": 4.4}

    def test_spec_scenario_the_rows(self):
        real = {
            "NATO": ((), (), "PROPN", "the alliance"),
            "USA": ((), ("America", "Stati Uniti"), "PROPN", "the country"),
        }
        with mock.patch.dict(red.acronyms.ACRONYMS, real, clear=True), tempfile.TemporaryDirectory() as work:
            self.write_work(work, extra=self.ACRONYM_ENTRIES)
            summary, lines = self.run_reduction(work, None, extra=self.ACRONYM_FREQUENCY)
            forms = dict(line.split("\t") for line in _read(work, "forms.tsv").splitlines())
            ranks = dict(line.split("\t") for line in _read(work, "freq.tsv").splitlines())
            # Each row a form in capitals, its own lemma, with no rank; the lower case keeps its reading.
            self.assertEqual((forms["NATO"], forms["USA"]), ("NATO", "USA"))
            self.assertEqual((forms["nato"], forms["usa"]), ("nascere", "uso"))
            self.assertEqual([f for f in forms if any(c.isupper() for c in f)], ["NATO", "USA"])
            self.assertFalse({"NATO", "USA"} & set(ranks))
            # The rows alone are Italian's dictionary words until change 16's glosses (D6).
            self.assertEqual(_read(work, "lexical.tsv"), "NATO\nUSA\n")
            self.assertEqual(_read(work, "gloss.tsv"), "")
            self.assertEqual(summary["acronyms"], ["NATO", "USA"])
            self.assertIn("reduced it-fr: acronyms=2 (NATO, USA), each a form in capitals that is its own lemma", "\n".join(lines))

    def test_a_row_the_section_or_the_tables_do_not_back_fails_the_reduction(self):
        for rows, refused in (
            # No entry headed as the row: a row never writes one.
            ({"ANSA": ((), (), "PROPN", "the press agency")}, "row ANSA has no entry headed ANSA"),
            # A synonym its entry does not give.
            ({"USA": ((), ("Stati Uniti", "Usa"), "PROPN", "x")}, "row USA names Usa"),
            # Its lower case read as its own entry: the tables read it right with no row.
            ({"ONU": ((), (), "PROPN", "x")}, "row(s) ONU"),
        ):
            with mock.patch.dict(red.acronyms.ACRONYMS, rows, clear=True), tempfile.TemporaryDirectory() as work:
                self.write_work(work, extra=self.ACRONYM_ENTRIES)
                with self.assertRaises(SystemExit) as raised:
                    self.run_reduction(work, None, extra=self.ACRONYM_FREQUENCY)
                self.assertIn(refused, str(raised.exception))

    def test_without_morph_it(self):
        with tempfile.TemporaryDirectory() as work:
            self.write_work(work)
            summary, _ = self.run_reduction(work, None)
            self.assertEqual(summary["morph-it added"], 0)
            notice = _read(work, "NOTICE")
            self.assertNotIn("Morph-it!", notice)
            # Pinned, it is read: a reduction without it fails, naming it.
            with self.assertRaises(SystemExit) as raised:
                self.run_reduction(work, red.MORPH_IT)
            self.assertIn(red.MORPH_IT, str(raised.exception))

    def test_a_word_the_pre_pass_writes_and_no_form_fails_the_reduction(self):
        with tempfile.TemporaryDirectory() as work:
            self.write_work(work)
            with open(os.path.join(work, red.KAIKKI), "w", encoding="utf-8") as f:
                for e in ENTRIES:
                    f.write(json.dumps(e, ensure_ascii=False) + "\n")
            with self.assertRaises(SystemExit) as raised:
                self.run_reduction(work, None, listed=False)
            self.assertIn("pre-pass", str(raised.exception))

    def test_spec_scenario_a_dictionary_update_names_a_row(self):
        # read-lingua-italian-unmarked-spellings D3: a dump giving a row an entry of its own stops the
        # reduction, naming the row, for a person to settle it.
        with tempfile.TemporaryDirectory() as work:
            self.write_work(work, extra=[entry("puo", senses=[{"glosses": ["a new meaning"]}])])
            with self.assertRaises(SystemExit) as raised:
                self.run_reduction(work, None)
            self.assertIn("1 spelling(s) of WEB_SPELLINGS", str(raised.exception))
            self.assertTrue(str(raised.exception).endswith(": puo"), raised.exception)

    def test_a_row_whose_word_is_no_ranked_lemma_fails_the_reduction(self):
        # A row follows the tables' choices: one whose word the tables do not rank reads as no form,
        # and the reduction refuses it, naming it.
        with mock.patch.dict(red.WEB_SPELLINGS, {"giu": "giù"}), tempfile.TemporaryDirectory() as work:
            self.write_work(work)
            with self.assertRaises(SystemExit) as raised:
                self.run_reduction(work, None)
            self.assertIn("1 row(s) of the web's list", str(raised.exception))
            self.assertTrue(str(raised.exception).endswith(": giu"), raised.exception)
        self.assertEqual(red.unread_web_spellings({"puo": "potere"}), [])
        self.assertEqual(red.unread_web_spellings({"puo": "puo"}), ["puo"])


if __name__ == "__main__":
    unittest.main()
