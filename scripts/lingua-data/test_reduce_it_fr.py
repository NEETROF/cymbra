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
import inspect
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
    # refine-lingua-italian-reading D7: `prim'`, which the section files under *prima*, is read as
    # *primo*; `trent'anni`, a plain word beginning with a new piece, and the stem `trent` are none;
    # the section's `da'` entry is no form once `da'` is a truncated articulated preposition.
    entry("primo", pos="adj", forms=[("prima", ["feminine"])]),
    entry("prima", pos="adv"),
    alt_of("prim'", "prima", pos="adj", tags=("alt-of", "apocopic")),
    entry("trenta", pos="num"),
    entry("trent'anni", pos="phrase"),
    form_of("da'", "dare", tags=("form-of", "imperative", "second-person", "singular")),
    entry("degli", pos="article"),
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
    "primo", "prima", "trent'anni", "trent", "trenta", "degli",
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
    lexicon.drop_inflection_lemmas()
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
        self.assertNotIn("dettero", self.forms)
        # `casa`'s row `ca'` is doubtful (« alternative, apocopic »), and gives no form; `ca'` is a
        # form of *casa* since refine-lingua-italian-reading D3 by the truncations' table alone.
        self.assertNotIn("ca'", self.lexicon.candidates)
        self.assertEqual(self.forms["ca'"], "casa")

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
        self.assertEqual(self.forms["va'"], "andare")
        # refine-lingua-italian-reading D3: the books' `ca'` reads as *casa*; `da'` is a truncated
        # articulated preposition, `da` + `i` or the verb by its context: no form with its
        # apostrophe, whatever the section's entry says.
        self.assertEqual(self.forms["ca'"], "casa")
        self.assertNotIn("da'", self.forms)
        for piece in red.TRUNCATED_PREPOSITIONS:
            self.assertNotIn(piece, self.forms, piece)
            self.assertNotIn(piece, self.ranks, piece)

    def test_the_review_s_pieces(self):
        # refine-lingua-italian-reading D7: `prim'` is a form of *primo*, as the pre-pass reads it,
        # not of *prima*, where the section files it; `degl'` a form of `degli`; `trent'` of *trenta*.
        self.assertEqual(self.forms["prim'"], "primo")
        self.assertEqual(self.forms["degl'"], "degli")
        self.assertEqual(self.forms["trent'"], "trenta")
        # A plain word beginning with a new piece is no form, and the piece's stem no rank.
        for word in ("trent'anni", "trent"):
            self.assertNotIn(word, self.forms, word)
            self.assertNotIn(word, self.ranks, word)
        self.assertIn("trenta", self.ranks)
        # The stems: the new pieces' are wordfreq's stems, no words; the apocopes' `fin` and `qual`,
        # and the truncations' and truncated prepositions' stems, are words.
        for stem in ("trent", "tant", "coll", "degl", "null", "prim", "ogn", "gliel", "ultim", "ventiquattr"):
            self.assertIn(stem, red.WORDFREQ_STEMS, stem)
        for word in ("fin", "qual", "ca", "de", "que", "a", "co", "ne", "su", "da"):
            self.assertNotIn(word, red.WORDFREQ_STEMS, word)
        self.assertEqual(red.APOCOPES["fin"][0], "fino")

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
        for word in ("po'", "l'", "altr'ora", "oltr'alpe", "ne'"):
            self.assertTrue(red.is_form(word), word)
        # refine-lingua-italian-reading D2: the new pieces start a plain word too.
        for word in ("trent'anni", "qual'è", "coll'ago", "degl'occhi", "gliel'ho", "l'ultim'ora"):
            self.assertFalse(red.is_form(word), word)

    def test_every_word_the_pre_pass_writes(self):
        words = red.prepass_words()
        for word in ("lo", "una", "dissi", "dello", "del", "a", "il", "i", "gli", "poco", "dai", "dici", "vai"):
            self.assertIn(word, words)
        # refine-lingua-italian-reading: the new pieces' words, `mò` and `dai` by their context, the
        # truncated prepositions' pieces.
        for word in ("trenta", "quale", "glielo", "primo", "ultima", "degli", "casa", "dei", "quei", "mò", "con", "in", "su"):
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
        # refine-lingua-italian-reading D3–D4: the truncated articulated prepositions, `mo'` read `mò`
        # but before `di`, and `dai` and `da'` read as the verb by their context.
        truncated = {f"{w}'": (p, a) for w, p, a in table("ITALIAN_TRUNCATED_PREPOSITIONS")}
        self.assertEqual(truncated, red.TRUNCATED_PREPOSITIONS)
        self.assertIn('const ITALIAN_MO: (&str, &str) = ("mo", "mò");', source)
        self.assertIn('const ITALIAN_DAI: &str = "dai";', source)
        self.assertEqual(red.CONTEXT_WORDS, ("dissi", "mò", "dai"))


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
        self.assertEqual(
            {form: lemma for form, (lemma, _) in red.OVERRIDES.items()},
            # refine-lingua-it-fr-glosses D11: a clear error and the owner's three rows; no `dato` row.
            {"stato": "essere", "stati": "essere", "bianca": "bianco", "usa": "usare", "dati": "dati", "lega": "lega", "prova": "prova"},
        )
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


# — it-fr's native side (add-lingua-pack-it-fr), on entries shaped as kaikki writes them —


def fr_entry(word, pos="noun", *glosses, senses=None, tags=None):
    """An entry of the French Wiktionary's Italian section: a word, its part of speech and its French
    senses (`Maison.`), or the senses given."""
    e = {"word": word, "lang_code": "it", "lang": "Italien", "pos": pos, "senses": senses or [{"glosses": [g]} for g in glosses]}
    if tags:
        e["tags"] = list(tags)
    return e


def fr_link(word, pos, text, *targets, tags=("form-of",)):
    """A sense kaikki files as a form of `targets` (`del` « Du. », a form of « Du »)."""
    return {"glosses": [text], "tags": list(tags), "form_of": [{"word": t} for t in targets]}


def translations(word, pos, *words):
    """A line of a translation file as `pack_sources.derive` writes it: a word, its part of speech,
    the translations it lists."""
    return {"pos": pos, "translations": [{"word": w} for w in words], "word": word}


def write_native(work, section=(), direct=(), inverted=()):
    """it-fr's three native sources in `work` (pack_sources.DUMPS["it-fr"]), a line that is no JSON
    first in each."""
    for name, lines in ((red.SECTION, section), (red.DIRECT, direct), (red.INVERTED, inverted)):
        with open(os.path.join(work, name), "w", encoding="utf-8") as f:
            f.write("not json\n")
            for line in lines:
                f.write(json.dumps(line, ensure_ascii=False) + "\n")


class Reduction(unittest.TestCase):
    """The reduction end to end, over a work folder (D9). The web's list and the owner's exceptions
    are the reviewed tables' real words, which this small section does not rank: each test reads one
    row this section ranks (read-lingua-italian-unmarked-spellings D3)."""

    WEB = {"puo": "potere"}

    def setUp(self):
        # Italian's acronym rows are the section's real headwords, which this small section has only
        # where a test writes them (keep-lingua-italian-acronyms-and-names D2).
        # refine-lingua-it-fr-glosses: the reviewed table of a row's order and the readings' list name the
        # section's real words, which this small section has only where a test writes them.
        tables = (
            (red.WEB_SPELLINGS, self.WEB),
            (red.OWNER_EXCEPTIONS, {}),
            (red.acronyms.ACRONYMS, {}),
            (red.FIRST_RUN, {}),
            (red.READINGS_SET_ASIDE, {}),
        )
        for table, rows in tables:
            patch = mock.patch.dict(table, rows, clear=True)
            patch.start()
            self.addCleanup(patch.stop)

    def write_work(self, work, morph_it=False, extra=(), native=None):
        entries = list(ENTRIES) + list(extra)
        # Every word the pre-pass writes a form, as the real section makes it.
        listed = {e["word"].lower() for e in entries}
        entries += [entry(word, pos="pron") for word in red.prepass_words() if word not in listed]
        with open(os.path.join(work, red.KAIKKI), "w", encoding="utf-8") as f:
            f.write("not json\n")
            for e in entries:
                f.write(json.dumps(e, ensure_ascii=False) + "\n")
        # it-fr's native side (add-lingua-pack-it-fr): by default the French Wiktionary's Italian
        # section glosses every word of the section and of wordfreq's list as a common noun, so that
        # each is one of Italian's dictionary words, and the translation tables say nothing.
        if native is None:
            words = sorted({e["word"].lower() for e in entries} | set(TOP))
            native = ([fr_entry(w, "noun", f"Sens de {w}.") for w in words], [], [])
        write_native(work, *native)
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
            # it-fr's glosses (add-lingua-pack-it-fr): every ranked lemma the section glosses, and
            # Italian's dictionary words, those glossed by no name alone — no acronym row here.
            ranked = dict(line.split("\t") for line in read("freq.tsv").splitlines())
            glosses = dict(line.split("\t") for line in read("gloss.tsv").splitlines())
            self.assertEqual(glosses["casa"], "Sens de casa")
            self.assertEqual(set(glosses), set(ranked))
            self.assertEqual(read("lexical.tsv").splitlines(), sorted(ranked))
            ranks = [line.split("\t") for line in read("freq.tsv").splitlines()]
            self.assertEqual([int(r) for _, r in ranks], list(range(1, len(ranks) + 1)))
            manifest = json.loads(read("manifest.json"))
            self.assertEqual(manifest["meta"]["studied"], "it")
            self.assertEqual(manifest["meta"]["native"], "fr")
            self.assertEqual(manifest["meta"]["pack_version"], "2026.10.23")
            self.assertEqual(manifest["meta"]["analyzer_version"], red.analyser_version())
            # Italian's levels are estimated (add-lingua-italian-levels D3).
            self.assertIs(manifest["meta"]["levels_estimated"], True)
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
            # The readings of the forms just chosen (add-lingua-italian-grammar-tables D1, D2): each
            # row a form of forms.tsv under a ranked lemma, sorted, and counted in the report.
            grammar = read("grammar.tsv").splitlines()
            self.assertIn("creduta\tcredere\tVERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part\t-", grammar)
            self.assertEqual(grammar, sorted(grammar))
            ranked = dict(ranks)
            for row in grammar:
                form, lemma, _, mark = row.split("\t")
                self.assertIn(lemma, ranked, row)
                self.assertEqual(mark, "-" if forms[form] == lemma else "other", row)
            self.assertEqual(summary["grammar"]["readings"], len(grammar))
            self.assertTrue(any(line.startswith(f"reduced it-fr: readings={len(grammar)} in ") for line in lines), lines)
            self.assertTrue(any(line.startswith("reduced it-fr: readings by part of speech") for line in lines), lines)
            # Italian's pinned tag pool is a person's (D8): no reduction writes it.
            self.assertFalse(os.path.exists(os.path.join(work, "tags.tsv")))
            first = {
                name: read(name)
                for name in (
                    "forms.tsv", "freq.tsv", "grammar.tsv", "level.tsv", "lexical.tsv", "gloss.tsv", "senses.tsv",
                    "mwe.tsv", "NOTICE", "manifest.json",
                )
            }
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
            # No rank, so no level: the rank order never reaches them (add-lingua-italian-levels D2);
            # `nato`'s and `usa`'s words take theirs.
            levels = dict(line.split("\t") for line in _read(work, "level.tsv").splitlines())
            self.assertFalse({"NATO", "USA"} & set(levels))
            self.assertEqual((levels["nascere"], levels["uso"]), ("A1", "A1"))
            # The rows are Italian's dictionary words beside the lemmas it-fr glosses (D6, and
            # add-lingua-pack-it-fr D5), glossed or not: the section here glosses neither row's synonyms.
            words = _read(work, "lexical.tsv").splitlines()
            self.assertEqual(words[:2], ["NATO", "USA"])
            self.assertEqual(set(words[2:]), set(ranks))
            self.assertFalse({"NATO", "USA"} & {line.split("\t")[0] for line in _read(work, "gloss.tsv").splitlines()})
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


# — Italian's word grammar (add-lingua-italian-grammar-tables) —

PRESENT_2S = ["indicative", "present", "second-person", "singular"]
PRESENT_3S_TAG = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"
PRESENT_1P = ["first-person", "indicative", "plural", "present"]
SUBJUNCTIVE = lambda person, number="singular": [person, number, "present", "subjunctive"]  # noqa: E731
# What a verb's table opens with, the head's principal parts before it: its bookkeeping and auxiliary.
TABLE = [("regular", ["table-tags"]), ("it-conj", ["inflection-template"]), ("avére", ["auxiliary"])]
# kaikki's article table, which it writes identically under each of il, lo, la, gli and le.
ARTICLE_TABLE = [
    ("no-table-tags", ["table-tags"]),
    ("l", ["inflection-template"]),
    ("il", ["masculine", "singular"]),
    ("lo (l')", ["masculine", "singular"]),
    ("i", ["masculine", "plural"]),
    ("gli", ["masculine", "plural"]),
    ("la (l')", ["feminine", "singular"]),
    ("le", ["feminine", "plural"]),
]


def noun(word, gender, plural=None, forms=(), senses=None, pos="noun"):
    """A noun entry headed by `it-noun` (its gender, and its plural argument when given)."""
    e = entry(word, pos=pos, forms=forms, senses=senses if senses is not None else [{"glosses": ["a meaning"]}])
    e["head_templates"] = [{"name": "it-noun", "args": {"1": gender, **({"2": plural} if plural else {})}}]
    return e


def headed(e, **args):
    """`e` headed by the generic `head` template with these arguments (a determiner's `g`)."""
    e["head_templates"] = [{"name": "head", "args": {"1": "it", **args}}]
    return e


GRAMMAR_ENTRIES = [
    # D3 — a verb's table: the head's principal parts, then the table; the negative and formal
    # imperatives, which repeat other cells.
    entry(
        "parlare",
        pos="verb",
        forms=[
            ("parlàre", ["canonical"]),
            ("pàrlo", ["first-person", "present", "singular"]),
            ("parlàto", PARTICIPLE),
            *TABLE,
            ("parlàre", ["infinitive"]),
            ("parlàndo", ["gerund"]),
            ("parlànte", ["participle", "present"]),
            ("parlàto", PARTICIPLE),
            ("pàrlo", PRESENT_1S),
            ("pàrli", PRESENT_2S),
            ("pàrla", PRESENT_3S),
            ("parliàmo", PRESENT_1P),
            ("parlàvo", ["first-person", "imperfect", "indicative", "singular"]),
            ("parlò", ["historic", "indicative", "past", "singular", "third-person"]),
            ("parlerèbbe", ["conditional", "singular", "third-person"]),
            ("pàrli", SUBJUNCTIVE("first-person")),
            ("pàrli", SUBJUNCTIVE("second-person")),
            ("pàrli", SUBJUNCTIVE("third-person")),
            ("parliàmo", SUBJUNCTIVE("first-person", "plural")),
            ("pàrla", ["imperative", "second-person", "singular"]),
            ("pàrli", ["formal", "imperative", "second-person-semantically", "singular", "third-person"]),
            ("parliàmo", ["first-person", "imperative", "plural"]),
            ("parlàre", ["imperative", "negative", "second-person", "singular"]),
            ("pàrli", ["formal", "imperative", "negative", "second-person-semantically", "singular", "third-person"]),
        ],
        senses=[{"glosses": ["to speak"]}],
    ),
    # A form's own entry, its senses merging persons and moods: the table, which lists the pair, wins.
    entry(
        "parli",
        pos="verb",
        senses=[
            {"tags": ["form-of", "indicative", "present", "second-person", "singular"], "form_of": [{"word": "parlare"}]},
            {"tags": ["form-of", "imperative", "singular", "third-person"], "form_of": [{"word": "parlare"}]},
        ],
    ),
    # D3 — the participles' own entries: the present one's plural, the past one's agreed forms.
    entry(
        "parlante",
        pos="verb",
        forms=[("parlanti", ["plural"])],
        senses=[{"tags": ["by-personal-gender", "feminine", "form-of", "masculine", "participle", "present"], "form_of": [{"word": "parlare"}]}],
    ),
    entry("parlante", pos="adj", forms=[("parlanti", ["plural"])], senses=[{"glosses": ["talking"], "tags": ["feminine", "masculine"]}]),
    noun("parlante", "m", forms=[("parlanti", ["plural"])]),
    entry(
        "parlato",
        pos="verb",
        forms=[("parlata", ["feminine"]), ("parlati", ["masculine", "plural"]), ("parlate", ["feminine", "plural"])],
        senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "parlare"}]}],
    ),
    # A participle change 9 files under a noun's spelling: the noun's reading, naming the verb.
    noun("parlata", "f", forms=[("parlate", ["plural"])], senses=[{"glosses": ["parlance"], "tags": ["feminine"]}]),
    form_of("parlata", "parlato", tags=("feminine", "form-of", "participle", "singular")),
    # A present participle's plural whose own entry names neither tense (« plural of dormiente »).
    entry("dormire", pos="verb", forms=[*TABLE, ("dormiènte", ["participle", "present"]), ("dormiàmo", PRESENT_1P)]),
    form_of("dormienti", "dormiente", tags=("form-of", "participle", "plural")),
    entry("dormiente", pos="verb", forms=[], senses=[{"tags": ["form-of", "participle", "present"], "form_of": [{"word": "dormire"}]}]),
    # D5 — a form of a form along one part of speech.
    entry("credere", pos="verb", forms=[*TABLE, ("credùto", PARTICIPLE), ("crediàmo", PRESENT_1P)]),
    entry(
        "creduto",
        pos="verb",
        forms=[("creduta", ["feminine"]), ("creduti", ["masculine", "plural"]), ("credute", ["feminine", "plural"])],
        senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "credere"}]}],
    ),
    form_of("creduta", "creduto", tags=("feminine", "form-of", "participle", "singular")),
    # D3 — the table's stress marks; and the Roman dialect's table kaikki merges in, set aside.
    entry(
        "dare",
        pos="verb",
        forms=[
            *TABLE,
            ("dàre", ["infinitive"]),
            ("diàmo", PRESENT_1P),
            ("dièdi", ["first-person", "historic", "indicative", "past", "singular"]),
            ("diède", ["historic", "indicative", "past", "singular", "third-person"]),
            *TABLE,
            ("dàmo", PRESENT_1P),
            ("diéde", ["first-person", "historic", "indicative", "past", "singular"]),
            ("dàssi", ["historic", "indicative", "past", "second-person", "singular"]),
        ],
    ),
    # D3 — a reflexive verb whose plain verb the section has, and one that has none.
    entry("lavare", pos="verb", forms=[*TABLE, ("làvo", PRESENT_1S), ("làva", PRESENT_3S), ("lavàto", PARTICIPLE)]),
    entry(
        "lavato",
        pos="verb",
        forms=[("lavati", ["masculine", "plural"])],
        senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "lavare"}]}],
    ),
    entry(
        "lavarsi",
        pos="verb",
        forms=[
            *TABLE,
            ("lavàrsi", ["infinitive"]),
            ("mi làvo", PRESENT_1S),
            ("làvati", ["imperative", "second-person", "singular"]),
        ],
        senses=[
            {"glosses": ["reflexive of lavare"], "tags": ["form-of", "reflexive"], "form_of": [{"word": "lavare"}]},
            {"glosses": ["to wash (oneself)"]},
        ],
    ),
    entry(
        "pentirsi",
        pos="verb",
        forms=[
            *TABLE,
            ("pentìrsi", ["infinitive"]),
            ("pentèndosi", ["gerund"]),
            ("mi pènto", PRESENT_1S),
            ("pèntiti", ["imperative", "second-person", "singular"]),
            ("ti pentìre", ["imperative", "negative", "second-person", "singular"]),
            ("si pènta", ["formal", "imperative", "second-person-semantically", "singular", "third-person"]),
        ],
        senses=[{"glosses": ["to repent"]}],
    ),
    # D3 — a clitic cluster's row, and a defective verb's empty cell.
    entry("andarsene", pos="verb", forms=[*TABLE, ("me ne vàdo", PRESENT_1S)], senses=[{"glosses": ["to go away"]}]),
    entry("andare", pos="verb", forms=[*TABLE, ("vàdo", PRESENT_1S), ("vàda", SUBJUNCTIVE("third-person"))]),
    entry("solere", pos="verb", forms=[*TABLE, ("sòglio", PRESENT_1S), ("-", PARTICIPLE)]),
    # D3 — a dialectal and a hypercorrect row, a Traditional one read.
    entry(
        "fare",
        pos="verb",
        forms=[
            *TABLE,
            ("fò", ["dialectal", "first-person", "indicative", "present", "singular"]),
            ("fàccio", PRESENT_1S),
            ("fàcciano", ["hypercorrect", "plural", "present", "subjunctive", "third-person"]),
            ("fèi", ["Traditional", "first-person", "historic", "indicative", "past", "singular"]),
        ],
    ),
    # D4 — nouns: a gender and a plural, one form for both numbers, an uncountable noun, a plural of
    # another gender, a derived word's row.
    noun("casa", "f", forms=[("case", ["plural"]), ("casétta", ["diminutive"])]),
    noun(
        "casetta",
        "f",
        forms=[("casette", ["plural"])],
        senses=[{"glosses": ["diminutive of casa"], "tags": ["diminutive", "feminine", "form-of"], "form_of": [{"word": "casa"}]}],
    ),
    noun("città", "f", "#", senses=[{"glosses": ["city"], "tags": ["feminine", "invariable"]}]),
    noun("gas", "m", "-", senses=[{"glosses": ["gas"], "tags": ["masculine", "uncountable"]}]),
    noun("uovo", "m", "uova<g:f>", forms=[("uova", ["feminine", "plural"]), ("ovetto", ["diminutive"])]),
    noun("eco", "f,m<l:colloquial,or,literary><ref:<<name:dop>>>", "+<g:m>,#<g:f>", forms=[("echi", ["masculine", "plural"])]),
    # D4 — a feminine noun's masculine row; a masculine noun's feminine row, which names it.
    noun("attore", "m", forms=[("attori", ["plural"]), ("attrice", ["feminine"])]),
    noun("attrice", "f", forms=[("attrici", ["plural"]), ("attore", ["masculine"])]),
    # D4 — adjectives of four and of two forms, a comparative, an adverb's row.
    entry("rosso", pos="adj", forms=[("rossa", ["feminine"]), ("rossi", ["masculine", "plural"]), ("rosse", ["feminine", "plural"])]),
    form_of("rosse", "rosso", pos="adj", tags=("feminine", "form-of", "plural")),
    entry("grande", pos="adj", senses=[{"glosses": ["big"], "tags": ["feminine", "masculine"]}]),
    noun("grande", "mfbysense", forms=[("grandi", ["plural"])]),
    form_of("grandi", "grande", pos="adj", tags=("form-of", "plural")),
    entry(
        "piccolo",
        pos="adj",
        forms=[("piccola", ["feminine"]), ("più piccolo", ["comparative"]), ("minore", ["comparative"]), ("minimo", ["superlative"])],
    ),
    entry("minore", pos="adj", senses=[{"glosses": ["comparative degree of piccolo"]}]),
    entry("vero", pos="adj", forms=[("vera", ["feminine"]), ("veramente", ["adverb"])]),
    entry("veramente", pos="adv"),
    # D4 — the article table under each article; a plural-headed article's table; a pronoun.
    headed(entry("il", pos="article", forms=[("i", ["plural"]), ("la", ["feminine"]), *ARTICLE_TABLE]), g="m-s"),
    headed(entry("lo", pos="article", forms=[("gli", ["plural"]), *ARTICLE_TABLE]), g="m-s"),
    headed(entry("gli", pos="article", forms=[("lo", ["singular"]), *ARTICLE_TABLE]), g="m-p"),
    headed(entry("le", pos="article", forms=[("la", ["singular"]), *ARTICLE_TABLE]), g="f-p"),
    headed(entry("lo", pos="pron", forms=[("li", ["plural"]), ("la", ["feminine"])]), g="m-s"),
    headed(entry("le", pos="pron", forms=[("li", ["masculine"]), ("la", ["singular"])]), g="f-p"),
    # D5 — what never gives a reading: an archaic spelling with a table of its own, a capitalised
    # headword, an apocope.
    entry(
        "dicere",
        pos="verb",
        forms=[*TABLE, ("dìco", PRESENT_1S)],
        senses=[{"glosses": ["archaic form of dire"], "tags": ["alt-of", "archaic"], "alt_of": [{"word": "dire"}]}],
    ),
    entry("dire", pos="verb", forms=[*TABLE, ("dìre", ["infinitive"])]),
    entry("Parli", pos="noun", senses=[{"glosses": ["a surname"], "tags": ["feminine"]}]),
    entry("bel", pos="adj", senses=[{"glosses": ["apocopic form of bello"], "tags": ["alt-of", "apocopic"], "alt_of": [{"word": "bello"}]}]),
    # D6 — a homograph's other dictionary form; a regional-only word is named on no card.
    entry("portare", pos="verb", forms=[*TABLE, ("pòrta", PRESENT_3S), ("pòrti", PRESENT_2S)]),
    noun("porta", "f", forms=[("porte", ["plural"])]),
    noun("porto", "m", forms=[("porti", ["plural"])]),
    noun("vada", "f", senses=[{"glosses": ["a ford"], "tags": ["feminine", "regional"]}]),
    # Change 10's spellings: a dictionary spelling (its own head a misspelling's) and the web's row.
    noun("perché", "m", "-", senses=[{"glosses": ["the reason"], "tags": ["masculine", "uncountable"]}]),
    entry("perchè", pos="noun", senses=[{"glosses": ["misspelling of perché"], "tags": ["alt-of", "misspelling"], "alt_of": [{"word": "perché"}]}]),
    entry("potere", pos="verb", forms=[*TABLE, ("può", PRESENT_3S), ("possiàmo", PRESENT_1P)]),
    form_of("può", "potere"),
    # A spelling change 10 reads as its word that is also one of its forms: its own reading stands.
    entry(
        "dà",
        pos="verb",
        senses=[
            {"tags": ["form-of", "indicative", "present", "singular", "third-person"], "form_of": [{"word": "dare"}]},
            {"glosses": ["alternative form of dare"], "tags": ["alt-of", "alternative", "colloquial"], "alt_of": [{"word": "dare"}]},
        ],
    ),
]

# The forms table and the ranks change 9 would give this section, written by hand: the readings
# choose neither (D1, D2).
GRAMMAR_FORMS = {
    **{f: "parlare" for f in ("parlare", "parlo", "parli", "parla", "parliamo", "parlavo", "parlò", "parlerebbe", "parlando", "parlanti", "parlato", "parlate")},
    "parlante": "parlante", "parlata": "parlata",
    "dormire": "dormire", "dormienti": "dormire", "dormiente": "dormire",
    "credere": "credere", "creduto": "credere", "creduta": "credere",
    "dare": "dare", "dà": "dare", "diamo": "dare", "diedi": "dare", "diede": "dare", "damo": "dare", "dassi": "dare",
    "lavare": "lavare", "lavo": "lavare", "lava": "lavare", "lavato": "lavare", "lavati": "lavare", "lavarsi": "lavare",
    "pentirsi": "pentirsi", "pentire": "pentirsi", "pentendo": "pentirsi", "pento": "pentirsi", "penti": "pentirsi",
    "pentiti": "pentito", "pentito": "pentito",
    "andare": "andare", "vado": "andare", "vada": "andare", "solere": "solere", "soglio": "solere",
    "fare": "fare", "fo": "fare", "faccio": "fare", "facciano": "fare", "fei": "fare",
    "casa": "casa", "case": "casa", "casetta": "casetta", "casette": "casetta", "città": "città", "gas": "gas",
    "uovo": "uovo", "uova": "uovo", "eco": "eco", "echi": "eco",
    "attore": "attore", "attori": "attore", "attrice": "attrice", "attrici": "attrice",
    "rosso": "rosso", "rossa": "rosso", "rossi": "rosso", "rosse": "rosso",
    "grande": "grande", "grandi": "grande", "piccolo": "piccolo", "piccola": "piccolo", "minore": "minore", "minimo": "piccolo",
    "vero": "vero", "vera": "vero", "veramente": "veramente",
    "il": "il", "i": "il", "la": "il", "gli": "il", "le": "il", "lo": "lo", "li": "lo",
    "dire": "dire", "dico": "dire", "dicere": "dire", "bel": "bello", "bello": "bello",
    "portare": "portare", "porta": "portare", "porti": "portare", "porto": "porto",
    "perché": "perché", "perchè": "perché", "potere": "potere", "può": "potere", "puo": "potere",
}
GRAMMAR_RANKS = {lemma: rank for rank, lemma in enumerate(sorted(set(GRAMMAR_FORMS.values())), 1)}


def read_grammar(entries=GRAMMAR_ENTRIES, spellings=None):
    """The readings of `entries`, as the reduction's pass reads them."""
    survey = red.Survey()
    for e in entries:
        survey.read(e)
    readings = red.Readings()
    for e in entries:
        red.read_readings(e, readings, survey, spellings)
    return readings


def grammar(entries=GRAMMAR_ENTRIES, forms=GRAMMAR_FORMS, ranks=GRAMMAR_RANKS, spellings=None):
    """`grammar.tsv`'s rows of the section, as `(form, lemma, tag, mark)`."""
    readings = read_grammar(entries, spellings)
    rows = red.spelling_rows(red.grammar_rows(readings, forms, ranks), forms, spellings)
    return [tuple(row.rstrip("\n").split("\t")) for row in rows]


class Grammar(unittest.TestCase):
    """Italian's readings (add-lingua-italian-grammar-tables D3–D7), on entries shaped as the English
    dump writes them."""

    @classmethod
    def setUpClass(cls):
        with mock.patch.dict(red.WEB_SPELLINGS, {"puo": "potere"}, clear=True), mock.patch.dict(red.OWNER_EXCEPTIONS, {}, clear=True):
            cls.rows = grammar(spellings={"perchè": "perché", "dà": "dare"})

    def card(self, form):
        """`(own readings, {other lemma: readings})` of `form`, as the card reads them."""
        own = sorted(tag for f, _, tag, mark in self.rows if f == form and mark == "-")
        others = collections.defaultdict(list)
        for f, lemma, tag, mark in self.rows:
            if f == form and mark == "other":
                others[lemma].append(tag)
        return own, {lemma: sorted(tags) for lemma, tags in others.items()}

    def alone(self, form):
        own, others = self.card(form)
        self.assertEqual(others, {}, form)
        return own

    def test_spec_scenario_a_verb_form_says_what_it_is(self):
        self.assertEqual(self.alone("parlò"), ["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"])
        self.assertEqual(self.alone("parlavo"), ["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Imp|VerbForm=Fin"])
        # The head's principal part repeats a cell: one reading.
        self.assertEqual(self.alone("parlo"), ["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"])

    def test_spec_scenario_a_present_of_four_readings(self):
        # Each its own tag; no imperative from the formal row nor from the own entry's third person.
        self.assertEqual(
            self.alone("parli"),
            [
                "VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin",
                "VERB|Mood=Sub|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
                "VERB|Mood=Sub|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin",
                "VERB|Mood=Sub|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin",
            ],
        )
        self.assertEqual(
            self.alone("parliamo"),
            [
                "VERB|Mood=Imp|Number=Plur|Person=1|VerbForm=Fin",
                "VERB|Mood=Ind|Number=Plur|Person=1|Tense=Pres|VerbForm=Fin",
                "VERB|Mood=Sub|Number=Plur|Person=1|Tense=Pres|VerbForm=Fin",
            ],
        )

    def test_spec_scenario_the_conditional_the_imperative_the_gerund_and_the_participles(self):
        self.assertEqual(self.alone("parlerebbe"), ["VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin"])
        self.assertEqual(
            self.alone("parla"),
            ["VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin", "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"],
        )
        self.assertEqual(self.alone("parlando"), ["VERB|VerbForm=Ger"])
        # The table's bare past participle is the masculine singular.
        self.assertEqual(self.alone("parlato"), ["VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part"])
        # `parlante` is a lemma of its own: its readings, and parlare's present participle named.
        self.assertEqual(
            self.card("parlante"),
            (
                ["ADJ|Number=Sing", "NOUN|Gender=Masc|Number=Sing"],
                {"parlare": ["VERB|Number=Sing|Tense=Pres|VerbForm=Part"]},
            ),
        )
        # Its plural through the participle's own entry, with no gender.
        self.assertEqual(
            self.card("parlanti"),
            (
                ["VERB|Number=Plur|Tense=Pres|VerbForm=Part"],
                {"parlante": ["ADJ|Number=Plur", "NOUN|Gender=Masc|Number=Plur"]},
            ),
        )
        self.assertEqual(self.alone("creduta"), ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"])
        # `parlate`, parlare's feminine plural participle — and its present, which the table here
        # does not list —, names the noun parlata, whose plural it is.
        self.assertEqual(
            self.card("parlate"),
            (["VERB|Gender=Fem|Number=Plur|Tense=Past|VerbForm=Part"], {"parlata": ["NOUN|Gender=Fem|Number=Plur"]}),
        )

    def test_a_present_participle_s_plural_naming_no_tense(self):
        # « plural of dormiente », a participle sense naming neither tense: the present one.
        self.assertEqual(self.alone("dormienti"), ["VERB|Number=Plur|Tense=Pres|VerbForm=Part"])

    def test_spec_scenario_the_table_s_stress_marks_and_a_reflexive_row(self):
        self.assertEqual(self.alone("diedi"), ["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Past|VerbForm=Fin"])
        self.assertEqual(self.alone("lavo"), ["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"])
        # `làvati` is never read as `lavati`, lavare's participle: no imperative there.
        self.assertEqual(self.alone("lavati"), ["VERB|Gender=Masc|Number=Plur|Tense=Past|VerbForm=Part"])
        self.assertEqual(self.alone("lavarsi"), [])
        # A reflexive-only verb: its rows carrying the pronoun read as their bare base.
        self.assertEqual(self.alone("pentire"), ["VERB|VerbForm=Inf"])
        self.assertEqual(self.alone("pentendo"), ["VERB|VerbForm=Ger"])
        self.assertEqual(self.alone("penti"), ["VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin"])
        self.assertEqual(self.alone("pento"), ["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"])
        self.assertEqual(self.card("pentiti"), ([], {}))

    def test_the_roman_dialect_s_table_gives_no_reading(self):
        # dare's second table lists no first person plural in -iamo while its first does: the
        # dialect's, whose `diéde` is a first person and `dàssi` a passato remoto.
        self.assertEqual(self.alone("diede"), ["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"])
        self.assertEqual(self.card("dassi"), ([], {}))
        self.assertEqual(self.card("damo"), ([], {}))
        rows = [{"form": f, "tags": t} for f, t in [*TABLE, ("dàmo", PRESENT_1P), *TABLE, ("ìmo", PRESENT_1P)]]
        self.assertEqual(red._dialect_rows(rows), set(), "no table in -iamo: nothing to tell apart")
        self.assertEqual(red._dialect_rows(rows[:4]), set(), "one table")

    def test_spec_scenario_rows_that_give_no_reading(self):
        # The negative imperative (written as the infinitive) and the formal one.
        self.assertEqual(self.alone("parlare"), ["VERB|VerbForm=Inf"])
        self.assertNotIn("VERB|Mood=Imp|Number=Sing|Person=3|VerbForm=Fin", self.alone("parli"))
        # A clitic cluster, a defective verb's `-`, a dialectal and a hypercorrect row; a Traditional one.
        self.assertEqual(self.card("vado"), (["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"], {}))
        self.assertFalse([row for row in self.rows if row[0] == "-"])
        self.assertEqual(self.card("fo"), ([], {}))
        self.assertEqual(self.card("facciano"), ([], {}))
        self.assertEqual(self.alone("fei"), ["VERB|Mood=Ind|Number=Sing|Person=1|Tense=Past|VerbForm=Fin"])
        # The reflexive's negative and formal rows read as nothing either.
        self.assertNotIn("VERB|Mood=Imp|Number=Sing|Person=3|VerbForm=Fin", self.alone("penti"))

    def test_spec_scenario_a_noun_says_its_gender(self):
        self.assertEqual(self.alone("case"), ["NOUN|Gender=Fem|Number=Plur"])
        self.assertEqual(self.alone("città"), ["NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Fem|Number=Sing"])
        self.assertEqual(self.alone("gas"), ["NOUN|Gender=Masc|Number=Sing"])
        self.assertEqual(self.alone("uova"), ["NOUN|Gender=Fem|Number=Plur"])
        # A qualified gender argument reads each gender; `#` among the plurals reads both numbers.
        self.assertEqual(
            self.alone("eco"),
            ["NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Fem|Number=Sing", "NOUN|Gender=Masc|Number=Plur", "NOUN|Gender=Masc|Number=Sing"],
        )

    def test_spec_scenario_adjectives_of_four_and_of_two_forms(self):
        self.assertEqual(self.alone("rosse"), ["ADJ|Gender=Fem|Number=Plur"])
        self.assertEqual(self.alone("rosso"), ["ADJ|Gender=Masc|Number=Sing"])
        # `grande`'s noun table lists `grandi`; its adjective form's entry reads still (D5).
        self.assertEqual(
            self.alone("grandi"), ["ADJ|Number=Plur", "NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Masc|Number=Plur"]
        )
        self.assertEqual(self.card("minore"), (["ADJ|Number=Sing"], {"piccolo": ["ADJ|Degree=Cmp|Number=Sing"]}))
        # A superlative the head names is its masculine singular.
        self.assertEqual(self.alone("minimo"), ["ADJ|Degree=Sup|Gender=Masc|Number=Sing"])

    def test_spec_scenario_the_shared_article_table(self):
        self.assertEqual(self.alone("i"), ["DET|Gender=Masc|Number=Plur"])
        self.assertEqual(self.alone("le"), ["DET|Gender=Fem|Number=Plur"])
        self.assertEqual(self.alone("il"), ["DET|Gender=Masc|Number=Sing"])
        self.assertEqual(self.card("gli"), (["DET|Gender=Masc|Number=Plur"], {"lo": ["DET|Gender=Masc|Number=Plur"]}))
        # `la`: il's head names it; lo the pronoun's feminine.
        self.assertEqual(self.card("la"), (["DET|Gender=Fem|Number=Sing"], {"lo": ["PRON|Gender=Fem|Number=Sing"]}))
        # The plural-headed tables (gli, le) give nothing: `lo` reads no singular of them, nor of le.
        self.assertEqual(self.card("lo"), ([], {}))
        # A pronoun's row with no gender gives nothing (`li`, lo's plural).
        self.assertEqual(self.card("li"), ([], {}))

    def test_spec_scenario_a_homograph_names_its_other_dictionary_form(self):
        self.assertEqual(
            self.card("porti"),
            (["VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin"], {"porto": ["NOUN|Gender=Masc|Number=Plur"]}),
        )

    def test_spec_scenario_a_participle_filed_under_a_noun_s_spelling(self):
        self.assertEqual(
            self.card("parlata"),
            (["NOUN|Gender=Fem|Number=Sing"], {"parlare": ["VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"]}),
        )

    def test_spec_scenario_a_noun_the_forms_table_reads_as_a_verb(self):
        # `porta` reads as portare; the noun, no lemma of its own form, is named nowhere (M8).
        self.assertEqual(self.alone("porta"), ["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"])

    def test_spec_scenario_what_gives_no_reading(self):
        # The archaic spelling's table, the capitalised headword, the apocope, the derived words' rows.
        self.assertEqual(self.card("dico"), ([], {}))
        self.assertFalse([row for row in self.rows if row[0] == "parli" and row[2].startswith("NOUN")])
        self.assertEqual(self.card("bel"), ([], {}))
        self.assertEqual(self.alone("casetta"), ["NOUN|Gender=Fem|Number=Sing"])
        self.assertEqual(self.card("veramente"), ([], {}))
        # `attore` is no form of attrice; attrice, attore's feminine, names it.
        self.assertEqual(self.alone("attore"), ["NOUN|Gender=Masc|Number=Sing"])
        self.assertEqual(self.card("attrice"), (["NOUN|Gender=Fem|Number=Sing"], {"attore": ["NOUN|Gender=Fem|Number=Sing"]}))

    def test_other_only_toward_a_word_not_only_regional(self):
        # `vada` is andare's; the regional noun *vada* is named on no card.
        self.assertEqual(self.card("vada"), (["VERB|Mood=Sub|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"], {}))

    def test_change_10_s_spellings_carry_their_word_s_readings(self):
        # A dictionary spelling, its own head a misspelling's: its word's readings, gender included.
        self.assertEqual(self.alone("perchè"), ["NOUN|Gender=Masc|Number=Sing"])
        # A spelling that is also a form of its word keeps its own reading, not its word's infinitive.
        self.assertEqual(self.alone("dà"), ["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"])
        # The web's row without its accent: its word's form's.
        self.assertEqual(self.alone("puo"), ["VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"])

    def test_the_rows_are_sorted_and_marked(self):
        lines = ["\t".join(row) + "\n" for row in self.rows]
        self.assertEqual(lines, sorted(lines))
        for form, lemma, _, mark in self.rows:
            self.assertIn(form, GRAMMAR_FORMS)
            self.assertIn(lemma, GRAMMAR_RANKS)
            self.assertEqual(mark, "-" if GRAMMAR_FORMS[form] == lemma else "other")
        # Only ranked lemmas: an unranked word's reading is left out.
        ranks = {lemma: rank for lemma, rank in GRAMMAR_RANKS.items() if lemma != "porto"}
        self.assertNotIn("porto", {lemma for _, lemma, _, _ in grammar(ranks=ranks)})

    def test_the_tags_as_ud_writes_them(self):
        self.assertEqual(red.parse_tag("VERB|Mood=Ind|Tense=Pres"), ("VERB", {"Mood": "Ind", "Tense": "Pres"}))
        self.assertEqual(red.ud_tag("NOUN", {"Number": "Sing", "Gender": "Fem"}), "NOUN|Gender=Fem|Number=Sing")
        # The subjunctive has no future, the imperative no third person, a row no mood but a tense
        # the indicative.
        self.assertEqual(red.verb_features({"future", "subjunctive", "singular", "first-person"}), [])
        self.assertEqual(red.verb_features({"imperative", "singular", "third-person"}), [])
        self.assertEqual(
            red.verb_features({"imperfect", "singular", "first-person"}),
            [{"VerbForm": "Fin", "Mood": "Ind", "Person": "1", "Number": "Sing", "Tense": "Imp"}],
        )
        # A determiner's row naming another person or case reads nothing; a pronoun's needs a gender.
        self.assertEqual(red.reading_tags("DET", {"possessive"}), [])
        self.assertEqual(red.reading_tags("PRON", {"plural"}), [])
        self.assertEqual(red.reading_tags("DET", {"superlative"}), [])
        # A noun's gender from its senses when its head names none.
        self.assertEqual(red.noun_genders(entry("x", senses=[{"tags": ["feminine"]}])), (("Fem",), False))
        self.assertEqual(red.noun_genders(noun("x", "m-p")), (("Masc",), True))
        self.assertEqual(red.noun_genders(noun("x", "?", senses=[{"tags": ["masculine"]}])), (("Masc",), False))
        # A composed present participle takes the number alone.
        self.assertEqual(
            red.compose("VERB|Number=Sing|Tense=Pres|VerbForm=Part", "VERB|Gender=Masc|Number=Plur|Tense=Past|VerbForm=Part"),
            "VERB|Number=Plur|Tense=Pres|VerbForm=Part",
        )
        self.assertIsNone(red.compose("VERB|VerbForm=Inf", "VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part"))
        self.assertIsNone(red.compose("ADJ|Number=Sing", "NOUN|Number=Plur"))

    def test_a_link_an_override_row_sets_aside(self):
        # `stato`, overridden to essere: its own entry's link toward a word no table lists reads nothing.
        entries = [
            entry("essere", pos="verb", forms=[*TABLE, ("stàto", PARTICIPLE)]),
            form_of("stato", "ostare", tags=("form-of", "participle", "past")),
            entry("ostare", pos="verb", forms=[*TABLE]),
        ]
        forms = {"stato": "essere", "essere": "essere", "ostare": "ostare"}
        rows = grammar(entries, forms, {"essere": 1, "ostare": 2})
        self.assertEqual([row[1] for row in rows if row[0] == "stato"], ["essere"])

    def test_the_figures(self):
        figures = red.readings_figures(["\t".join(row) + "\n" for row in self.rows], GRAMMAR_FORMS)
        self.assertEqual(figures["readings"], len(self.rows))
        self.assertGreaterEqual(figures["forms reading the present indicative and subjunctive alike"], 2)
        self.assertEqual(sum(figures["by part of speech"].values()), len(self.rows))
        self.assertEqual(
            figures["forms with a reading of their own"] + figures["forms without"], len(GRAMMAR_FORMS)
        )


class GrammarAtReview(unittest.TestCase):
    """The rules the review of change 13 added (add-lingua-italian-grammar-tables, design As
    implemented), each on a small section of its own."""

    @staticmethod
    def rows(entries, forms, spellings=None):
        ranks = {lemma: rank for rank, lemma in enumerate(sorted(set(forms.values())), 1)}
        return grammar(entries, forms, ranks, spellings)

    @staticmethod
    def of(rows, form):
        return sorted((lemma, tag, mark) for f, lemma, tag, mark in rows if f == form)

    def test_no_form_of_a_form_through_a_plural(self):
        # `colle` is the hill and *colla*'s plural: `colli`, the hill's plural, is no form of *colla*.
        entries = [
            noun("colla", "f", forms=[("colle", ["plural"])]),
            noun("colle", "m", forms=[("colli", ["plural"])]),
            form_of("colle", "colla", pos="noun", tags=("feminine", "form-of", "plural")),
            form_of("colli", "colle", pos="noun", tags=("form-of", "masculine", "plural")),
        ]
        rows = self.rows(entries, {"colla": "colla", "colle": "colle", "colli": "colle"})
        self.assertEqual(self.of(rows, "colli"), [("colle", "NOUN|Gender=Masc|Number=Plur", "-")])
        self.assertEqual(
            self.of(rows, "colle"),
            [("colla", "NOUN|Gender=Fem|Number=Plur", "other"), ("colle", "NOUN|Gender=Masc|Number=Sing", "-")],
        )
        self.assertIsNone(red.compose("NOUN|Gender=Fem|Number=Plur", "NOUN|Gender=Masc|Number=Plur"))

    def test_a_spelling_s_inflection_reads_only_where_the_word_s_table_gives_nothing(self):
        # *moglia*, read as *moglie* (change 10's D2), lists `moglie` as its plural: `moglie` stays the
        # singular its own entry gives.
        entries = [
            noun("moglie", "f", forms=[("mogli", ["plural"])]),
            noun(
                "moglia",
                "f",
                forms=[("moglie", ["plural"])],
                senses=[{"glosses": ["alternative form of moglie"], "tags": ["alt-of", "alternative"], "alt_of": [{"word": "moglie"}]}],
            ),
        ]
        forms = {"moglie": "moglie", "mogli": "moglie", "moglia": "moglie"}
        rows = self.rows(entries, forms, spellings={"moglia": "moglie"})
        self.assertEqual(self.of(rows, "moglie"), [("moglie", "NOUN|Gender=Fem|Number=Sing", "-")])
        self.assertEqual(self.of(rows, "mogli"), [("moglie", "NOUN|Gender=Fem|Number=Plur", "-")])

    def test_poetry_and_letters_name_no_other_word(self):
        # `sono`'s poetic Latinism of *suonare* names nothing on essere's card; `veggo`, literary, stays
        # vedere's own; a poetic cell of essere's table (`fièro`) names essere on no card of `fiero`.
        entries = [
            entry("essere", pos="verb", forms=[*TABLE, ("sóno", PRESENT_1S), ("fièro", ["future", "indicative", "plural", "poetic", "third-person"])]),
            entry("suonare", pos="verb", forms=[*TABLE, ("suòno", PRESENT_1S)]),
            form_of("sono", "suonare", tags=("Latinism", "first-person", "form-of", "indicative", "poetic", "present", "singular")),
            entry("vedere", pos="verb", forms=[*TABLE, ("védo", PRESENT_1S)]),
            form_of("veggo", "vedere", tags=("first-person", "form-of", "indicative", "literary", "present", "singular")),
            entry("fiero", pos="adj", forms=[("fiera", ["feminine"])]),
        ]
        forms = {"essere": "essere", "sono": "essere", "suonare": "suonare", "suono": "suonare", "vedere": "vedere", "veggo": "vedere", "fiero": "fiero"}
        rows = self.rows(entries, forms)
        self.assertEqual(self.of(rows, "sono"), [("essere", "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin", "-")])
        self.assertEqual(self.of(rows, "veggo"), [("vedere", "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin", "-")])
        self.assertEqual(self.of(rows, "fiero"), [("fiero", "ADJ|Gender=Masc|Number=Sing", "-")])

    def test_a_doubtful_spelling_of_a_cell_names_no_lemma_with_that_cell(self):
        # `anno`'s verb entry is « misspelling of hanno », and avere's table lists the disused `ànno`
        # untagged: the year names no avere; `puzza`, a noun spelling *puzzo*, still names puzzare.
        entries = [
            entry("avere", pos="verb", forms=[*TABLE, ("hànno", ["indicative", "plural", "present", "third-person"]), ("ànno", ["indicative", "plural", "present", "third-person"])]),
            noun("anno", "m"),
            entry("anno", pos="verb", senses=[{"glosses": ["misspelling of hanno"], "tags": ["alt-of", "misspelling"], "alt_of": [{"word": "hanno"}]}]),
            entry("puzzare", pos="verb", forms=[*TABLE, ("pùzzo", PRESENT_1S), ("pùzza", PRESENT_3S)]),
            noun("puzza", "f", senses=[{"glosses": ["alternative form of puzzo"], "tags": ["alt-of", "alternative", "feminine"], "alt_of": [{"word": "puzzo"}]}]),
            noun("puzzo", "m"),
        ]
        forms = {"avere": "avere", "hanno": "avere", "anno": "anno", "puzzare": "puzzare", "puzza": "puzzo", "puzzo": "puzzo"}
        rows = self.rows(entries, forms)
        self.assertEqual(self.of(rows, "anno"), [("anno", "NOUN|Gender=Masc|Number=Sing", "-")])
        self.assertIn(("puzzare", PRESENT_3S_TAG, "other"), self.of(rows, "puzza"))
        self.assertEqual(
            red.Readings().misspelt_readings(), {}, "nothing is a doubtful spelling of nothing"
        )

    def test_an_archaic_or_literary_entry_is_named_on_no_card(self):
        # *diro* (« dire », literary) is named on no card of `dire`; *arrendere*, archaic, is named on
        # `arreso` through *arrendersi*, filed under it.
        entries = [
            entry("dire", pos="verb", forms=[*TABLE, ("dìre", ["infinitive"])]),
            entry("diro", pos="adj", forms=[("dira", ["feminine"]), ("dire", ["feminine", "plural"])], senses=[{"glosses": ["dire"], "tags": ["literary"]}]),
            entry("arrendere", pos="verb", forms=[*TABLE, ("arréso", PARTICIPLE)], senses=[{"glosses": ["to give back"], "tags": ["archaic"]}]),
            entry("arrendersi", pos="verb", forms=[*TABLE, ("mi arrèndo", PRESENT_1S)], senses=[{"glosses": ["to surrender"]}]),
            entry("arreso", pos="adj", forms=[("arresa", ["feminine"])]),
        ]
        forms = {"dire": "dire", "diro": "diro", "arrendere": "arrendere", "arrendo": "arrendere", "arreso": "arreso"}
        rows = self.rows(entries, forms)
        self.assertEqual(self.of(rows, "dire"), [("dire", "VERB|VerbForm=Inf", "-")])
        self.assertIn(("arrendere", "VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part", "other"), self.of(rows, "arreso"))
        # The archaic adjective keeps its own readings: no part of speech filters them.
        self.assertEqual(self.of(rows, "diro"), [("diro", "ADJ|Gender=Masc|Number=Sing", "-")])

    def test_a_feminine_noun_s_masculine_plural_and_a_superlative_s_gender(self):
        entries = [
            noun("signorina", "f", forms=[("signorine", ["plural"]), ("signorino", ["masculine"]), ("signorini", ["masculine", "plural"])]),
            entry("bello", pos="adj", forms=[("bella", ["feminine"]), ("bellissimo", ["superlative"])]),
            entry("abile", pos="adj", forms=[("abili", ["plural"]), ("abilissimo", ["superlative"])]),
        ]
        forms = {"signorina": "signorina", "signorini": "signorina", "bello": "bello", "bellissimo": "bello", "abile": "abile", "abilissimo": "abile"}
        rows = self.rows(entries, forms)
        self.assertEqual(self.of(rows, "signorini"), [])
        self.assertEqual(self.of(rows, "bellissimo"), [("bello", "ADJ|Degree=Sup|Gender=Masc|Number=Sing", "-")])
        self.assertEqual(self.of(rows, "abilissimo"), [("abile", "ADJ|Degree=Sup|Gender=Masc|Number=Sing", "-")])
        self.assertEqual(red._masculine_superlative({"superlative", "feminine"}, "bellissima"), {"superlative", "feminine"})


# — Italian's estimated levels (add-lingua-italian-levels) —
#
# The section's entries the levels read, shaped as the English edition's dump of 2026-10-21 writes them
# (`kaikki-Italian.jsonl`, its glosses shortened): a letter, a letter's name filed as a noun, another
# word's spelling, a name written with its capital, a contraction.


def letter(word, case="lowercase"):
    """A letter's own entry (`k`: « The eleventh letter of the Italian alphabet … »)."""
    return entry(
        word,
        pos="character",
        senses=[{"glosses": [f"A letter of the Italian alphabet, called {word.lower()}."], "tags": ["feminine", "invariable", "letter", case, "masculine"]}],
    )


def letter_name(word):
    """A letter's name, filed as a noun (`u`: « The name of the Latin script letter U/u. »)."""
    return entry(word, senses=[{"glosses": [f"The name of the Latin script letter {word.upper()}/{word}."], "tags": ["feminine", "invariable"]}])


def spells(word, target, pos="noun", gloss=None, tags=("alt-of", "misspelling")):
    """An entry giving its word only as `target`'s spelling (`the`: « misspelling of tè »)."""
    return entry(word, pos=pos, senses=[{"glosses": [gloss or f"misspelling of {target}"], "tags": list(tags), "alt_of": [{"word": target}]}])


LEVEL_ENTRIES = [
    # Rule 1: a name alone, written with its capital (`km` has no entry at all).
    entry(
        "Milano",
        pos="name",
        senses=[
            {"glosses": ["Milan (a city and comune, the capital of Lombardy)"], "tags": ["feminine"]},
            {"glosses": ["the letter M in the Italian spelling alphabet"], "tags": ["feminine"]},
        ],
    ),
    entry("Giovanni", pos="name", senses=[{"glosses": ["a male given name from Hebrew, equivalent to English John"], "tags": ["masculine"]}]),
    # Rule 2: single characters — letters, a letter's name, an abbreviation, another word's spelling —,
    # and the three that are words.
    letter("k"),
    letter("K", case="uppercase"),
    letter("b"),
    letter("u"),
    letter_name("u"),
    letter("r"),
    spells("r", "er", pos="article", gloss="alternative spelling of er", tags=("alt-of", "alternative", "masculine", "singular")),
    entry("R", senses=[{"glosses": ["abbreviation of re (“king”)"], "tags": ["abbreviation", "alt-of"], "alt_of": [{"word": "re"}]}]),
    letter("à"),
    spells("à", "ha", pos="verb", gloss="obsolete spelling of ha, third-person singular present indicative of avere", tags=("alt-of", "obsolete")),
    letter("e"),
    letter_name("e"),
    entry("e", pos="conj", senses=[{"glosses": ["and"]}]),
    spells("e", "è", pos="verb", gloss="misspelling of è (third-person singular present indicative of essere)"),
    letter("o"),
    letter_name("o"),
    entry("o", pos="conj", senses=[{"glosses": ["or"]}]),
    letter("a"),
    entry("a", senses=[{"glosses": ["The name of the Latin script letter A/a.; a"], "tags": ["invariable"]}]),
    entry("a", pos="prep", senses=[{"glosses": ["Indicates the indirect object. to"]}]),
    # Rule 3: only another word's spelling.
    spells("the", "tè"),
    spells("perchè", "perché", pos="conj"),
    # A contraction stays a word (`del`, « contraction of di + il »), as an apocope would.
    entry(
        "del",
        pos="contraction",
        senses=[{"glosses": ["contraction of di + il; some, of the, from the"], "tags": ["abbreviation", "alt-of", "contraction"], "alt_of": [{"word": "di"}]}],
    ),
    entry("di", pos="prep", senses=[{"glosses": ["of"]}]),
]


def level_senses(entries=LEVEL_ENTRIES):
    """`read_level_senses` over `entries` written as the section's lines."""
    with tempfile.TemporaryDirectory() as work:
        path = os.path.join(work, red.KAIKKI)
        with open(path, "w", encoding="utf-8") as f:
            f.write("not json\n")
            for e in entries:
                f.write(json.dumps(e, ensure_ascii=False) + "\n")
        return red.read_level_senses(path)


class EstimatedLevels(unittest.TestCase):
    """What a CEFR list would leave out, read from the section (design D2), and the bands (D1)."""

    @classmethod
    def setUpClass(cls):
        cls.senses = level_senses()

    def test_the_section_s_senses_are_those_no_form_of_another_word(self):
        # Words are lowercased: `Milano` is read as `milano`, `K` as `k`.
        self.assertEqual({pos for pos, _, _ in self.senses["milano"]}, {"name"})
        self.assertEqual({pos for pos, _, _ in self.senses["k"]}, {"character"})
        self.assertNotIn("km", self.senses)
        # A form of another word gives no sense (`creduta`, `un`'s apocope); a contraction does.
        senses = level_senses([*ENTRIES, *LEVEL_ENTRIES])
        self.assertNotIn("creduta", senses)
        self.assertNotIn("un", senses)
        self.assertIn(("contraction", "contraction of di + il; some, of the, from the", frozenset({"abbreviation", "alt-of", "contraction"})), senses["del"])

    def test_spec_scenario_what_a_cefr_list_leaves_out(self):
        forms = {}
        for lemma, why in [
            ("km", "unknown"),
            ("milano", "name"),
            ("giovanni", "name"),
            ("k", "letter"),
            ("b", "letter"),
            # A letter's name filed as a noun is a letter's, read by its gloss.
            ("u", "letter"),
            # A letter, an abbreviation, another word's spelling: no word's sense.
            ("r", "letter"),
            ("à", "letter"),
            ("the", "spelling"),
            ("perchè", "spelling"),
            # The preposition and the two conjunctions are words; a contraction is no spelling.
            ("a", None),
            ("e", None),
            ("o", None),
            ("del", None),
            ("di", None),
        ]:
            self.assertEqual(red.no_level(lemma, forms, self.senses), why, lemma)

    def test_a_word_of_more_than_one_character_is_never_a_letter(self):
        senses = {"di": [("noun", "The name of the Latin script letter D/d.; dee", frozenset({"feminine"}))]}
        self.assertIsNone(red.no_level("di", {}, senses))

    def test_a_spelling_whose_other_entry_is_a_name_s(self):
        senses = {"zombie": [("noun", "alternative spelling of zombi", frozenset({"alt-of"})), ("name", "Zombie", frozenset())]}
        self.assertEqual(red.no_level("zombie", {}, senses), "spelling")

    def test_spec_scenario_a_lemma_whose_own_form_reads_as_another(self):
        # `porta` ranked while the forms table maps it to *portare*: a guard — the builder files a level
        # by the lemma's id and refuses such a lemma, and change 9 ranks none. It takes no level;
        # portare keeps its own rank's, and the next lemma takes the place.
        senses = {word: [("noun", "a meaning", frozenset())] for word in ("di", "portare", "porta", "casa", "vita")}
        ranks = {"di": 1, "portare": 2, "porta": 3, "casa": 4, "vita": 5}
        forms = {"di": "di", "portare": "portare", "porta": "portare", "porte": "porta", "casa": "casa", "vita": "vita"}
        self.assertEqual(red.no_level("porta", forms, senses), "elsewhere")
        levels, left_out = red.estimated_levels(ranks, forms, senses, bands=(("A1", 2), ("A2", 2)))
        self.assertEqual(levels, {"di": "A1", "portare": "A1", "casa": "A2", "vita": "A2"})
        self.assertEqual(left_out["elsewhere"], ["porta"])

    def test_the_bands_go_in_rank_order_to_the_lemmas_a_cefr_list_would_hold(self):
        ranks = {"di": 1, "e": 2, "km": 3, "the": 4, "milano": 5, "del": 6, "a": 7, "k": 8, "o": 9, "u": 10}
        levels, left_out = red.estimated_levels(ranks, {}, self.senses, bands=(("A1", 2), ("A2", 1), ("B1", 2)))
        self.assertEqual(levels, {"di": "A1", "e": "A1", "del": "A2", "a": "B1", "o": "B1"})
        # Each rule's lemmas, up to the last level given: `u` comes after it.
        self.assertEqual(
            left_out,
            {"unknown": ["km"], "name": ["milano"], "elsewhere": [], "letter": ["k"], "spelling": ["the"], "unlisted": []},
        )
        # Equal ranks go by the lemma, so the order never depends on the source's.
        tied, _ = red.estimated_levels({"o": 1, "e": 1}, {}, self.senses, bands=(("A1", 1), ("A2", 1)))
        self.assertEqual(tied, {"e": "A1", "o": "A2"})
        # Too few lemmas: the bands stop where they run out.
        short, _ = red.estimated_levels({"di": 1}, {}, self.senses)
        self.assertEqual(short, {"di": "A1"})

    def test_a_lemma_left_out_gives_its_slot_to_the_next(self):
        ranks = {"di": 1, "milano": 2, "e": 3, "k": 4, "o": 5}
        levels, _ = red.estimated_levels(ranks, {}, self.senses, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(levels, {"di": "A1", "e": "A1", "o": "A2"})
        # With English's bands, as many lemmas as there are slots take one.
        many = {f"parola{i}": i for i in range(1, 9000)}
        many_senses = {word: [("noun", "a word", frozenset())] for word in many}
        many_senses["parola3"] = [("name", "a name", frozenset())]
        levels, left_out = red.estimated_levels(many, {}, many_senses)
        self.assertEqual(len(levels), 8302)
        self.assertEqual(left_out["name"], ["parola3"])
        self.assertEqual(levels["parola1021"], "A1")
        self.assertEqual(levels["parola1022"], "A2")
        self.assertEqual(levels["parola8303"], "C2")
        self.assertNotIn("parola8304", levels)

    def test_dictionary_words_narrow_the_levels_when_given(self):
        # Change 16 passes Italian's dictionary words (D4): a lemma they do not list takes no level,
        # read last, and gives its slot; without them the rule is not read.
        ranks = {"di": 1, "roma": 2, "e": 3, "milano": 4, "o": 5}
        senses = {**self.senses, "roma": [("name", "Rome", frozenset()), ("noun", "a type of rice", frozenset())]}
        self.assertIsNone(red.no_level("roma", {}, senses))
        words = {"di", "e", "o"}
        self.assertEqual(red.no_level("roma", {}, senses, words), "unlisted")
        # Another rule names a lemma first: `milano` is a name, listed or not.
        self.assertEqual(red.no_level("milano", {}, senses, words), "name")
        levels, left_out = red.estimated_levels(ranks, {}, senses, words, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(levels, {"di": "A1", "e": "A1", "o": "A2"})
        self.assertEqual(left_out["unlisted"], ["roma"])
        self.assertEqual(red.LEVEL_RULES[-1], "unlisted")
        unnarrowed, _ = red.estimated_levels(ranks, {}, senses, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(unnarrowed, {"di": "A1", "roma": "A1", "e": "A2"})

    def test_spec_scenario_the_table_does_not_wait_for_the_glosses(self):
        # The derivation reads the ranks, the forms, the section's senses and, once change 16 passes
        # them, Italian's dictionary words: glosses that add or remove no dictionary word leave it as
        # it is, and another pair's glosses never reach it.
        self.assertEqual(list(inspect.signature(red.estimated_levels).parameters), ["ranks", "forms", "senses", "words", "bands"])
        ranks = {"di": 1, "e": 2, "del": 3}
        bands = (("A1", 2), ("A2", 1))
        self.assertEqual(
            red.estimated_levels(ranks, {}, self.senses, {"di", "e", "del", "NATO"}, bands),
            red.estimated_levels(ranks, {}, self.senses, None, bands),
        )

    def test_english_bands_hold_its_8302_cefr_lemmas_as_fr_en_s_do(self):
        self.assertEqual([level for level, _ in red.ENGLISH_BANDS], ["A1", "A2", "B1", "B2", "C1", "C2"])
        self.assertEqual([size for _, size in red.ENGLISH_BANDS], [1020, 1158, 2015, 2347, 886, 876])
        self.assertEqual(sum(size for _, size in red.ENGLISH_BANDS), 8302)
        spec = importlib.util.spec_from_file_location("reduce_fr_en", os.path.join(_HERE, "reduce-fr-en.py"))
        fr_en = importlib.util.module_from_spec(spec)
        spec.loader.exec_module(fr_en)
        self.assertEqual(red.ENGLISH_BANDS, fr_en.ENGLISH_BANDS)

    def test_the_report_names_each_level_s_span_and_each_rule(self):
        ranks = {"di": 1, "km": 2, "e": 3, "o": 4}
        levels, left_out = red.estimated_levels(ranks, {}, self.senses, bands=(("A1", 2), ("A2", 1)))
        self.assertEqual(
            red.levels_report(levels, left_out, ranks, bands=(("A1", 2), ("A2", 1))),
            "levels=3 estimated: A1 2 (1–3); A2 1 (4–4); left out: unknown 1, name 0, elsewhere 0, letter 0, spelling 0, "
            "unlisted 0",
        )


class EstimatedLevelsReduced(unittest.TestCase):
    """The reduction of the fixture section, the levels' entries added and wordfreq ranking them."""

    LEVEL_FREQUENCY = {
        "di": 6.95, "e": 6.9, "a": 6.85, "o": 6.6, "milano": 5.5, "giovanni": 5.2, "km": 5.1, "the": 5.0,
        "b": 4.9, "r": 4.85, "u": 4.8, "k": 4.6, "à": 4.4, "perchè": 4.2,
    }

    @classmethod
    def setUpClass(cls):
        reduction = Reduction()
        reduction.setUp()
        cls.addClassCleanup(reduction.doCleanups)
        with tempfile.TemporaryDirectory() as work:
            reduction.write_work(work, extra=LEVEL_ENTRIES)
            cls.summary, cls.lines = reduction.run_reduction(work, None, extra=cls.LEVEL_FREQUENCY)
            cls.out = {name: _read(work, name) for name in ("level.tsv", "freq.tsv", "forms.tsv", "manifest.json", "NOTICE")}
        cls.levels = dict(line.split("\t") for line in cls.out["level.tsv"].splitlines())
        cls.ranks = dict(line.split("\t") for line in cls.out["freq.tsv"].splitlines())
        cls.forms = dict(line.split("\t") for line in cls.out["forms.tsv"].splitlines())

    def test_spec_scenario_the_commonest_words_are_a1(self):
        for word in ["di", "e", "il", "essere", "avere", "a", "del", "della", "o"]:
            self.assertEqual(self.levels.get(word), "A1", word)
        # `al` and `nel` are no words of Italian's tables.
        for word in ["al", "nel"]:
            self.assertNotIn(word, self.ranks)
            self.assertNotIn(word, self.levels)

    def test_spec_scenario_what_a_cefr_list_leaves_out(self):
        for word in ["milano", "giovanni", "km", "k", "b", "u", "r", "à", "the", "perchè"]:
            self.assertIn(word, self.ranks, word)
            self.assertNotIn(word, self.levels, word)
        # Besides them, the fixture's `cent` and `nepalese`, which the section does not know, and
        # `nepal`, a name alone.
        left_out = self.summary["levels left out"]
        self.assertEqual((left_out["unknown"], left_out["name"], left_out["letter"], left_out["spelling"]), (3, 3, 5, 2))
        self.assertEqual((left_out["elsewhere"], left_out["unlisted"]), (0, 0))

    def test_every_level_is_a_ranked_lemma_s_and_its_own_form_s(self):
        for lemma in self.levels:
            self.assertIn(lemma, self.ranks)
            self.assertEqual(self.forms[lemma], lemma)
        rows = self.out["level.tsv"].splitlines()
        self.assertEqual(rows, sorted(rows))
        # The fixture ranks fewer lemmas than A1 holds: every lemma a CEFR list would hold is A1.
        self.assertEqual(set(self.levels.values()), {"A1"})
        self.assertEqual(self.summary["levels"], {"A1": len(self.levels)})
        self.assertEqual(len(self.levels) + sum(self.summary["levels left out"].values()), len(self.ranks))
        self.assertTrue(any(line.startswith(f"reduced it-fr: levels={len(self.levels)} estimated: A1 ") for line in self.lines), self.lines)

    def test_spec_scenario_every_italian_pack_says_so(self):
        manifest = json.loads(self.out["manifest.json"])
        self.assertIs(manifest["meta"]["levels_estimated"], True)
        notice = self.out["NOTICE"].replace("\n", " ")
        self.assertIn("The levels are estimated, not taken from a CEFR list", notice)
        self.assertIn("the English Wiktionary's Italian section saying which lemmas take one", notice)
        self.assertIn("every pack studying Italian reads them as committed", notice)
        for list_name in ("Profilo", "CEFR-J", "Octanove", "FLELex"):
            self.assertNotIn(list_name, notice)


class NativeSide(unittest.TestCase):
    """it-fr's native side (add-lingua-pack-it-fr D2, D4–D6), over a small French Wiktionary's Italian
    section shaped as kaikki writes it — the senses quoted are the section's — and the two
    translation tables."""

    SECTION = [
        fr_entry("casa", "noun", "Maison.", "Domicile."),
        # D4: a gloss filed as a link to the French « Du »; the hamlet `Del` kept off it (D5).
        fr_entry("del", "article", senses=[fr_link("del", "article", "Du. Note d’usage : devant une consonne.", "Du")]),
        fr_entry("Del", "name", "Del, hameau de Saint-Denis."),
        # « Pluriel de storia », which kaikki files as a form of « Pluriel »: an inflection by its text.
        fr_entry("storie", "noun", senses=[fr_link("storie", "noun", "Pluriel de storia.", "Pluriel")]),
        fr_entry("storia", "noun", "Histoire."),
        # A lowercase target no Italian entry heads, the word glossed by nothing else: a gloss.
        fr_entry("degli", "article", senses=[fr_link("degli", "article", "Article contracté souvent traduit par des.", "impures")]),
        # … beside the word's own senses: left a form.
        fr_entry("pelle", "noun", "Peau.", "Fourrure, peau, pelage, poil d’animal."),
        fr_entry("pelle", "article", senses=[fr_link("pelle", "article", "Article contracté souvent traduit par pour les.", "les")]),
        # A link to the Italian `la`: a form, which the tables gloss.
        fr_entry("della", "article", senses=[fr_link("della", "article", "Article contracté souvent traduit par De la.", "la")]),
        fr_entry("la", "article", "La."),
        fr_entry(
            "coloro",
            "pron",
            senses=[
                fr_link("coloro", "pron", "Pluriel de colui : Ceux.", "Ceux"),
                fr_link("coloro", "pron", "Celles et ceux.", "Celles et ceux"),
            ],
        ),
        # D5: a proper noun's sense that is no name's note stays beside the common word's.
        fr_entry("luna", "noun", "Lune."),
        fr_entry("Luna", "name", "Lune, satellite de la Terre."),
        # D5: a surname kept off a word of form-of senses only, which borrows its base's gloss.
        fr_entry("soldi", "noun", senses=[fr_link("soldi", "noun", "Pluriel de soldo.", "soldo")]),
        fr_entry("soldo", "noun", "Sou."),
        fr_entry("Soldi", "name", "Nom de famille."),
        # D5: a province's acronym kept off `te`, and the letter left out. `te`'s link names `toi`,
        # whose entry is *togliere*'s form alone, as the section writes it: no Italian word heads it
        # (the review), so the link glosses `te` — not the tables' « Te », the letter's French name.
        fr_entry(
            "te",
            "pron",
            senses=[
                fr_link("te", "pron", "Toi : pronom disjoint de la deuxième personne du singulier.", "toi"),
                {"glosses": ["Variante de ti : forme objet de la deuxième personne du singulier."], "tags": ["alt-of"], "alt_of": [{"word": "ti"}]},
            ],
        ),
        fr_entry("te", "noun", "Te, vingtième lettre de l’alphabet cyrillique."),
        fr_entry("TE", "name", "Province de Térame."),
        fr_entry("toi", "verb", senses=[fr_link("toi", "verb", "Deuxième personne du singulier de l’indicatif présent de togliere.", "togliere")]),
        fr_entry("togliere", "verb", "Ôter."),
        # The review's leaks: examples filed as senses (`stesso`, `età`, `io` with its missing
        # translation and the placeholder link), a colour's codes (`bianco`), a head line merged into
        # a sense (`sedere`'s meaning kept, `signore`'s form dropped).
        fr_entry(
            "stesso",
            "adj",
            "Même.",
            "io sono stanco di dover ricordare le stesse cose.\n#*: moi, je suis fatigué de rappeler les mêmes choses.",
        ),
        fr_entry("età", "noun", "Âge.", "Époque, période.", "età glaciale.\n#*: ère glaciaire."),
        fr_entry(
            "io",
            "pron",
            "Pronom de la première personne du singulier.",
            "Io mi chiamo Fabrizio. — (Il figlio, 2014)\n#*: La traduction en français de l’exemple manque. ([https://server.dummy/x Ajouter])",
        ),
        fr_entry("bianco", "noun", "Blanc, la couleur comme celle des os. #FFFFFF", "Blanc, personne dont la couleur de peau est claire."),
        fr_entry("Bianco", "name", "Commune, commune d’Italie de la ville métropolitaine de Reggio de Calabre."),
        fr_entry("verde", "adj", "Vert, couleur verte. — #C2F732 #00FF00 #10E010"),
        fr_entry("sedere", "noun", "Derrière."),
        fr_entry("sedere", "verb", "\\se.ˈde.re\\intransitif2ᵉ groupe (voir la conjugaison)* Asseoir."),
        fr_entry("signore", "noun", "Monsieur."),
        fr_entry("signore", "noun", "signore\\si.ˈɲɔ.re\\fémininpluriel:Pluriel de signora.", tags=("form-of",)),
        # D5 since the review: a commune's note kept off a common word, a planet or Christ kept.
        fr_entry("ora", "noun", "Heure."),
        fr_entry("ora", "adv", "Maintenant."),
        fr_entry("Ora", "name", "Ora, commune d’Italie de la province de Bolzano."),
        fr_entry("terra", "noun", "Terre, le sol sur lequel nous marchons."),
        fr_entry("Terra", "name", "La Terre, planète tellurique sur laquelle nous vivons."),
        fr_entry("cristo", "noun", "Hère, bougre."),
        fr_entry("Cristo", "name", "Christ."),
        fr_entry("Luna", "name", "Luna, commune d’Espagne, située dans la province de Saragosse."),
        # The letters the section names bare: off a word holding another meaning, kept on a letter.
        fr_entry("ci", "pron", "Nous, pronom clitique objet de la première personne du pluriel."),
        fr_entry("ci", "noun", "Tsy."),
        fr_entry("chi", "pron", "Qui, celui qui."),
        fr_entry("chi", "noun", "Khi."),
        fr_entry("gamma", "noun", "Gamma."),
        fr_entry("nu", "noun", "Nu."),
        fr_entry("NU", "name", "Province de Nuoro."),
        fr_entry("nudo", "adj", "Nu."),
        fr_entry("nudo", "noun", "Nu."),
        # A sense repeating another up to case, across parts of speech.
        fr_entry("italiano", "adj", "Italien."),
        fr_entry("italiano", "noun", "Italien."),
        fr_entry("italiano", "noun", "italien."),
        # D5: an acronym's sense by its text, off a word with a sense of its own.
        fr_entry("a", "prep", "À."),
        fr_entry("A", "name", "Abréviation de Azione, nom d’un parti politique italien."),
        # A first name with no lowercase entry: its note glosses it, and it is no dictionary word.
        fr_entry("Francesco", "name", "Prénom masculin, équivalant à François."),
        # The acronym row's synonyms, proper nouns both (D5).
        fr_entry("America", "name", "Amérique (continent).", "États-Unis d’Amérique."),
        fr_entry("Stati Uniti", "name", "États-Unis."),
        # D6: expressions, the typographic apostrophe read as `'`.
        fr_entry("essere d’accordo", "verb", "Être d’accord."),
        fr_entry("all’improvviso", "adv", "Soudainement."),
        fr_entry("c’è", "verb", senses=[fr_link("c’è", "verb", "Il y a.", "Il y a")]),
        # A proper noun with a space, and one the pre-pass splits: no expression.
        fr_entry("Valle d’Aosta", "name", "Vallée d’Aoste."),
        fr_entry("all’Aia", "name", "À La Haye."),
        {"word": "x", "lang_code": "it", "pos": "character", "senses": [{"glosses": ["Lettre x."]}]},
    ]
    # The Italian Wiktionary's French translations (direct), in the table's order.
    DIRECT = [
        translations("app", "noun", "appli", "application"),
        translations("gratis", "adv", "gratuitement", "gratis", "gratuit", "à l’œil"),
        translations("senz’altro", "adv", "sans faute"),
        translations("dentifricio", "noun", "dentifrice"),
    ]
    # The French Wiktionary's Italian translations (inverted, read backwards).
    INVERTED = [
        translations("application", "noun", "app"),
        translations("dentifrice", "noun", "pasta dentifricia", "dentifricio"),
        translations("téléphone intelligent", "noun", "smartphone"),
        translations("ordiphone", "noun", "smartphone"),
        translations("smartphone", "noun", "smartphone"),
        translations("d'accord", "adv", "d’accordo"),
        translations("mécanique quantique", "noun", "meccanica quantica"),
        translations("te", "noun", "te"),
        translations("beige", "adj", "beige"),
        translations("beige", "noun", "beige"),
        translations("de la", "article", "della"),
        translations("des", "article", "della"),
    ]
    LEMMAS = [
        "a", "del", "della", "degli", "te", "casa", "coloro", "luna", "pelle", "soldi", "storie",
        "francesco", "app", "gratis", "smartphone", "dentifricio", "stesso", "età", "io", "bianco",
        "verde", "sedere", "signore", "ora", "terra", "cristo", "ci", "chi", "gamma", "nu", "nudo", "italiano",
        "beige",
    ]
    RANKS = {lemma: rank for rank, lemma in enumerate(LEMMAS, 1)}
    FORMS = {lemma: lemma for lemma in LEMMAS}
    # The French words' frequencies, doubled.
    FRENCH = {"smartphone": 4.2, "ordiphone": 1.5, "téléphone intelligent": 2.0, "des": 7.0, "de la": 6.0}
    ROWS = {"USA": ((), ("America", "Stati Uniti"), "PROPN", "the country"), "NATO": ((), (), "PROPN", "the alliance")}

    @classmethod
    def setUpClass(cls):
        tmp = tempfile.TemporaryDirectory()
        cls.addClassCleanup(tmp.cleanup)
        work = tmp.name
        write_native(work, cls.SECTION, cls.DIRECT, cls.INVERTED)
        cls.stats = collections.Counter()
        cls.section = red.section_as_read(os.path.join(work, red.SECTION), os.path.join(work, red.SECTION_READ), cls.stats)
        direct = red.common.read_translations(
            red.straight_translations(os.path.join(work, red.DIRECT), os.path.join(work, red.DIRECT_READ), inverted=False),
            inverted=False,
            studied=red.IT,
        )
        inverted = red.common.read_translations(
            red.straight_translations(os.path.join(work, red.INVERTED), os.path.join(work, red.INVERTED_READ), inverted=True),
            inverted=True,
            studied=red.IT,
        )
        cls.sources = [(direct, list), (inverted, red.common.by_native_frequency(lambda w: cls.FRENCH.get(w, 0.0)))]
        cls.glosses, cls.runs, cls.expressions, cls.steps = red.native_side(cls.section, cls.RANKS, cls.sources)
        cls.shared = (dict(cls.glosses), dict(cls.runs))
        cls.repeated = red.without_repeated_senses(cls.glosses, cls.runs)
        with mock.patch.dict(red.acronyms.ACRONYMS, cls.ROWS, clear=True):
            cls.split, cls.entries = red.section_words(cls.section, cls.FORMS, red.row_words())
            cls.split_tables = red.table_split_words(cls.sources, cls.FORMS, cls.split)
            cls.acronym_glosses = red.acronym_glosses(cls.glosses, cls.expressions, cls.entries)
        with open(cls.section, encoding="utf-8") as f:
            cls.read = [json.loads(line) for line in f]

    def test_spec_scenario_a_word_glossed_by_its_definition(self):
        self.assertEqual(self.glosses["casa"], "Maison; Domicile")
        self.assertEqual(self.runs["casa"], [("NOUN", 2)])
        self.assertIn("casa", self.steps["definitions"])

    def test_the_native_side_is_the_shared_one(self):
        # Step by step, the tables `reduce_common.native_tables` gives (D1).
        glosses, runs, expressions, primary = red.common.native_tables(
            self.section, self.RANKS, studied=red.IT, edition=red.EDITION, fallbacks=self.sources
        )
        self.assertEqual((glosses, runs, expressions), (*self.shared, self.expressions))
        self.assertEqual(primary, len(self.steps["definitions"]))

    def test_spec_scenario_the_fallbacks_order(self):
        # `app`: the Italian table's French words first, in its order — not « Application » read
        # backwards from the French entry.
        self.assertEqual(self.glosses["app"], "Appli, application")
        self.assertIn("app", self.steps["direct"])
        # At most three words per part of speech from a table, in the table's order.
        self.assertEqual(self.glosses["gratis"], "Gratuitement, gratis, gratuit")
        # Read backwards only where the Italian table has none, the commonest French word first.
        self.assertEqual(self.glosses["smartphone"], "Smartphone, téléphone intelligent, ordiphone")
        self.assertEqual(self.runs["smartphone"], [("NOUN", 1)])
        self.assertIn("smartphone", self.steps["inverted"])
        # A word both tables list takes the Italian table's.
        self.assertEqual(self.glosses["dentifricio"], "Dentifrice")
        self.assertEqual(red.source_of("dentifricio", self.steps), "direct")
        self.assertIsNone(red.source_of("missing", self.steps))

    def test_spec_scenario_a_gloss_read_as_a_link(self):
        # `del` « Du. », a link to the French « Du », which no Italian entry heads: a gloss — its usage
        # note out since refine-lingua-it-fr-glosses D8; the hamlet `Del` kept off it (D5).
        self.assertEqual(self.glosses["del"], "Du")
        self.assertNotIn("hameau", self.glosses["del"])
        # `storie`: « Pluriel de storia » is an inflection by its text, whatever its target: still a
        # form, its pointer naming « Pluriel », which lends nothing.
        self.assertNotIn("storie", self.glosses)
        # `coloro`: its « Pluriel de… » senses stay forms, « Celles et ceux » glosses it.
        self.assertEqual(self.glosses["coloro"], "Celles et ceux")
        # `c'è`, an expression: « Il y a ».
        self.assertEqual(self.split["c'è"], "Il y a")

    def test_spec_scenario_a_link_to_a_lowercase_word_beside_a_gloss(self):
        # `degli`, glossed by nothing else: its link is its gloss, read through its translation since
        # refine-lingua-it-fr-glosses D8.
        self.assertEqual(self.glosses["degli"], "Des")
        # `pelle`: its own senses alone, the article's link left a form.
        self.assertEqual(self.glosses["pelle"], "Peau; Fourrure, peau, pelage, poil d’animal")
        self.assertEqual(self.runs["pelle"], [("NOUN", 2)])
        # The exact spelling: `della` links the Italian `la`, whose entry holds a meaning: a form for D4 —
        # the `di` family's link read as its translation since refine-lingua-it-fr-glosses D8, not the
        # tables' « Des, de la ».
        self.assertEqual(self.glosses["della"], "De la")
        self.assertIn("della", self.steps["definitions"])
        # `te`'s link names `toi`, whose entry is a form alone: no Italian word heads it, so the link
        # glosses `te` (the review) — not the tables' « Te ».
        self.assertEqual(self.glosses["te"], "Toi : pronom disjoint de la deuxième personne du singulier")
        self.assertIn("te", self.steps["definitions"])
        self.assertEqual(
            {k: n for k, n in self.stats.items() if k.startswith(("links", "lowercase"))},
            {"links read as a gloss": 3, "links read as a gloss, a lowercase target": 2, "lowercase links kept beside another gloss": 1},
        )

    def test_spec_scenario_a_proper_noun_beside_a_common_word(self):
        self.assertEqual(self.glosses["luna"], "Lune; Lune, satellite de la Terre")
        self.assertEqual(self.runs["luna"], [("NOUN", 1), ("PROPN", 1)])

    def test_spec_scenario_a_surname_keeps_off_a_common_word(self):
        # `soldi`, a form: its base's gloss, never the surname's note.
        self.assertEqual(self.glosses["soldi"], "Sou")
        # `te`: neither the province's acronym nor the letter.
        self.assertNotIn("Térame", self.glosses["te"])
        self.assertNotIn("lettre", self.glosses["te"])
        # `a`: its own sense alone, the party's abbreviation kept off it.
        self.assertEqual(self.glosses["a"], "À")
        self.assertEqual((self.stats["name notes"], self.stats["name notes, communes"]), (2, 3))
        self.assertEqual(self.stats["acronym senses"], 3)
        self.assertEqual((self.stats["letter entries"], self.stats["letter senses"]), (1, 1))
        # The entries left with no sense go: `Del`, `Soldi`, `TE`, `A`, `NU`, the letter's `te`, the
        # communes `Bianco`, `Ora` and `Luna`, `ci`'s and `chi`'s letters, and `signore`'s leaked form.
        self.assertEqual(self.stats["entries left with no sense"], 12)
        headwords = [e["word"] for e in self.read]
        for gone in ("Del", "Soldi", "TE", "A", "x"):
            self.assertNotIn(gone, headwords)
        # A word that is only a name keeps its note: `Francesco`, no lowercase entry.
        self.assertEqual(self.glosses["francesco"], "Prénom masculin, équivalant à François")
        self.assertIn("Francesco", headwords)

    def test_what_kaikki_leaks_into_a_sense(self):
        # The review: an example filed as a sense goes, the word's other senses stay.
        self.assertEqual(self.glosses["stesso"], "Même")
        self.assertEqual(self.glosses["età"], "Âge; Époque, période")
        # `io`: the example with its missing translation and kaikki's placeholder link.
        self.assertEqual(self.glosses["io"], "Pronom de la première personne du singulier")
        # A colour's codes go, its meaning stays.
        self.assertEqual(self.glosses["bianco"], "Blanc, la couleur comme celle des os; Blanc, personne dont la couleur de peau est claire")
        self.assertEqual(self.glosses["verde"], "Vert, couleur verte")
        # A head line merged into a sense: the meaning after it stays, a form with no meaning goes.
        self.assertEqual(self.glosses["sedere"], "Derrière; Asseoir")
        self.assertEqual(self.runs["sedere"], [("NOUN", 1), ("VERB", 1)])
        self.assertEqual(self.glosses["signore"], "Monsieur")
        self.assertEqual(
            {k: self.stats[k] for k in ("examples filed as a sense", "colour codes taken out", "head lines read for their meaning", "head lines with no meaning")},
            {"examples filed as a sense": 3, "colour codes taken out": 2, "head lines read for their meaning": 1, "head lines with no meaning": 1},
        )
        for gloss in self.glosses.values():
            for leak in ("#*", "#FFFFFF", "\\", "server.dummy", "exemple manque"):
                self.assertNotIn(leak, gloss)
        # Read alone, sense by sense.
        self.assertEqual(red.read_sense({"glosses": ["Même."]}), ({"glosses": ["Même."]}, None))
        self.assertEqual(red.read_sense({"glosses": ["Voir https://example.org."]}), (None, "senses holding an address"))
        self.assertEqual(red.read_sense({"glosses": ["#FFFFFF"]}), (None, "colour codes taken out"))
        self.assertEqual(red.read_sense({}), ({}, None))

    def test_spec_scenario_a_commune_keeps_off_a_common_word(self):
        # The review: a commune's note is a name's note (D5) — off `ora`, `bianco` and `luna`; the
        # Earth, Christ and the Moon stay beside their words.
        self.assertEqual(self.glosses["ora"], "Heure; Maintenant")
        self.assertNotIn("commune", self.glosses["bianco"])
        self.assertEqual(self.glosses["luna"], "Lune; Lune, satellite de la Terre")
        self.assertEqual(self.glosses["terra"], "Terre, le sol sur lequel nous marchons; La Terre, planète tellurique sur laquelle nous vivons")
        self.assertEqual(self.glosses["cristo"], "Hère, bougre; Christ")
        for note in ("Ora, commune d’Italie de la province…", "Commune, commune d’Italie de…", "commune de Rome"):
            self.assertTrue(red._NAME_NOTE.match(red._without_headword(note, "Ora")), note)
        for name in ("La Terre, planète tellurique", "Christ", "Lune, satellite de la Terre", "Communauté de communes"):
            self.assertFalse(red._NAME_NOTE.match(name), name)

    def test_a_letter_s_name_written_bare(self):
        # The review: `ci` « Tsy », `chi` « Khi » off the pronouns; `gamma`, only a letter, keeps its
        # name; `nudo` « Nu » is French, never a letter here.
        self.assertEqual(self.glosses["ci"], "Nous, pronom clitique objet de la première personne du pluriel")
        self.assertEqual(self.glosses["chi"], "Qui, celui qui")
        self.assertEqual(self.glosses["gamma"], "Gamma")
        # A province's acronym is no common word's meaning: `nu` keeps its letter.
        self.assertEqual(self.glosses["nu"], "Nu")
        self.assertEqual(self.glosses["nudo"], "Nu")
        self.assertEqual(self.stats["letter names written bare"], 2)
        self.assertEqual(sorted(red.LETTER_NAMES), list(red.LETTER_NAMES), "kept sorted")
        self.assertNotIn("nudo", red.LETTER_NAMES)
        self.assertNotIn("guado", red.LETTER_NAMES)

    def test_a_sense_repeating_another(self):
        # The review: `italiano` « Italien; italien » across its parts of speech, `beige` across a
        # table's; a run shortened with its sense.
        self.assertEqual((self.glosses["italiano"], self.runs["italiano"]), ("Italien", [("ADJ", 1)]))
        self.assertEqual((self.glosses["beige"], self.runs["beige"]), ("Beige", [("ADJ", 1)]))
        self.assertEqual(self.shared[0]["italiano"], "Italien; italien")
        self.assertEqual(self.repeated, ["beige", "italiano"])
        glosses, runs = {"x": "A; B; a.; C"}, {"x": [("NOUN", 2), ("VERB", 2)]}
        self.assertEqual(red.without_repeated_senses(glosses, runs), ["x"])
        self.assertEqual((glosses["x"], runs["x"]), ("A; B; C", [("NOUN", 2), ("VERB", 1)]))
        # A gloss whose runs do not count its senses is left as it is.
        glosses, runs = {"y": "A; a"}, {"y": [("NOUN", 1)]}
        self.assertEqual(red.without_repeated_senses(glosses, runs), [])
        self.assertEqual(glosses["y"], "A; a")

    def test_the_section_as_read_keeps_what_it_does_not_change(self):
        # The typographic apostrophe read as `'` in a headword; an entry the pass does not change is
        # written as its line was.
        headwords = [e["word"] for e in self.read]
        self.assertIn("essere d'accordo", headwords)
        self.assertNotIn("essere d’accordo", headwords)
        casa = next(e for e in self.read if e["word"] == "casa")
        self.assertEqual(casa, self.SECTION[0])
        # A line that is no JSON object is left out, and so are the thirteen entries left with no sense
        # or a letter's.
        self.assertEqual(len(self.read), len(self.SECTION) - 13)

    def test_spec_scenario_an_acronym_through_a_proper_noun_synonym(self):
        # `USA`: `America`'s first sense and `Stati Uniti`'s, a proper noun's entry, in the row's
        # order — one run of the row's part of speech; `NATO`, which names neither, none.
        self.assertEqual(self.acronym_glosses, {"USA": ("Amérique (continent); États-Unis", [("PROPN", 2)])})
        self.assertNotIn("stati uniti", self.expressions)
        self.assertEqual(self.entries["stati uniti"], "États-Unis")
        with mock.patch.dict(red.acronyms.ACRONYMS, self.ROWS, clear=True):
            self.assertEqual(red.row_words(), {"america", "stati uniti"})
            # The pair's own gloss of a synonym wins over its entry read whatever its part of speech.
            own = red.acronym_glosses({"america": "Amérique; Amériques"}, {}, self.entries)
        self.assertEqual(own["USA"][0], "Amérique; États-Unis")

    def test_spec_scenario_expressions_from_every_source(self):
        # The section's headword written with ’, a word the pre-pass splits from the section, one
        # with a space and one the pre-pass splits from the tables — keyed at build.
        self.assertEqual(self.expressions["essere d'accordo"], "Être d’accord")
        self.assertEqual(self.split["all'improvviso"], "Soudainement")
        self.assertEqual(self.expressions["meccanica quantica"], "Mécanique quantique")
        self.assertEqual(self.split_tables, {"senz'altro": "Sans faute", "d'accordo": "D'accord"})
        self.assertIn("essere d'accordo", self.steps["section expressions"])
        self.assertNotIn("meccanica quantica", self.steps["section expressions"])
        # No proper noun: neither `Valle d'Aosta` nor `all'Aia`.
        self.assertNotIn("valle d'aosta", self.expressions)
        self.assertNotIn("all'aia", self.split)

    def test_a_compound_s_link_is_no_gloss(self):
        # The review: `gli` heads only form-of entries, so a clitic compound's link « Agglutination du
        # verbe… avec le pronom gli » names no word a meaning heads — still a pointer by its wording,
        # with a pronominal form, an old spelling and a person written « seconde ».
        for text in (
            "Agglutination du verbe abalienare avec le pronom gli (« lui »).",
            "Agglutination de la deuxième personne du singulier de l’impératif présent du verbe dare avec le pronom le.",
            "Forme pronominale de maritare : se marier.",
            "Ancienne orthographe de asti, pluriel de astio.",
            "Seconde personne du pluriel de l'indicatif présent de enumerare.",
        ):
            self.assertIsNone(red._link({"glosses": [text], "tags": ["form-of"], "form_of": [{"word": "gli"}]}), text)
        self.assertIsNotNone(red._link({"glosses": ["Du."], "tags": ["form-of"], "form_of": [{"word": "Du"}]}))

    def test_what_the_pre_pass_splits(self):
        forms = {"po'": "poco", "nell'": "nello"}
        for word in ("d'accordo", "all'improvviso", "c'è", "senz'altro", "mezz'ora", "dov'è", "sott'olio"):
            self.assertTrue(red.split_word(word, forms), word)
        # A space, no elided piece, an articulated preposition written whole, a truncation, a form.
        for word in ("un po'", "perché", "alla", "nel", "po'", "anglo-americano", "nell'"):
            self.assertFalse(red.split_word(word, forms), word)

    def test_measures(self):
        got = red.measures(self.RANKS, self.glosses, self.steps, top=10)
        self.assertEqual(got["top"], 10)
        self.assertEqual(got["by source"], {k: len(self.steps[k]) for k in ("definitions", "direct", "inverted")})
        self.assertEqual(got["glossed"], sum(1 for lemma in self.LEMMAS[:10] if lemma in self.glosses))
        self.assertEqual(len(got["sample"]), len([l for l in self.LEMMAS if l in self.glosses]))
        lemma, rank, source, gloss = got["sample"][0]
        self.assertEqual((rank, source, gloss), (self.RANKS[lemma], red.source_of(lemma, self.steps), self.glosses[lemma]))
        self.assertEqual([s[1] for s in got["sample"]], sorted(s[1] for s in got["sample"]))
        self.assertEqual(red.coverage({"a": 1, "b": 2}, {"a": "A"}, tops=(1, 2)), [100.0, 50.0])

    def test_the_straightened_tables(self):
        # Each Italian word as Italian is written; the French words as they are.
        self.assertIn("senz'altro", self.sources[0][0])
        self.assertIn("d'accordo", self.sources[1][0])
        self.assertEqual(self.sources[0][0]["gratis"]["ADV"][-1], "à l’œil")


class NativeSideReduced(unittest.TestCase):
    """The reduction end to end with a native side (add-lingua-pack-it-fr D5): Italian's dictionary
    words, the levels narrowed to them, the report."""

    def setUp(self):
        self.reduction = Reduction()
        self.reduction.setUp()
        self.addCleanup(self.reduction.doCleanups)

    def test_spec_scenario_the_levels_go_to_dictionary_words(self):
        words = sorted({e["word"].lower() for e in ENTRIES} | set(TOP) | set(red.prepass_words()))
        # `casa` glossed by nothing, `roma` by a name alone: no dictionary word, so no level.
        section = [fr_entry(w, "noun", f"Sens de {w}.") for w in words if w not in ("casa", "roma")]
        section.append(fr_entry("Roma", "name", "Rome."))
        with tempfile.TemporaryDirectory() as work:
            self.reduction.write_work(work, native=(section, [], []))
            summary, lines = self.reduction.run_reduction(work, None)
            levels = dict(line.split("\t") for line in _read(work, "level.tsv").splitlines())
            lexical = _read(work, "lexical.tsv").splitlines()
            glosses = dict(line.split("\t") for line in _read(work, "gloss.tsv").splitlines())
            senses = dict(line.split("\t", 1) for line in _read(work, "senses.tsv").splitlines())
        self.assertTrue(set(levels) <= set(lexical), set(levels) - set(lexical))
        self.assertNotIn("casa", lexical)
        self.assertNotIn("casa", levels)
        self.assertEqual((glosses["roma"], senses["roma"]), ("Rome", "PROPN:1"))
        self.assertNotIn("roma", lexical)
        self.assertGreaterEqual(summary["levels left out"]["unlisted"], 1)
        self.assertEqual(summary["names"], 1)
        self.assertEqual(summary["dictionary words"], len(lexical))
        self.assertEqual(lexical, sorted(lexical))
        self.assertTrue(any(line.startswith("reduced it-fr: glosses=") for line in lines), lines)
        self.assertTrue(any(line.startswith("reduced it-fr: sample: ") for line in lines), lines)

    def test_the_native_sources_are_read(self):
        with tempfile.TemporaryDirectory() as work:
            self.reduction.write_work(work)
            os.remove(os.path.join(work, red.DIRECT))
            with self.assertRaises(SystemExit) as raised:
                self.reduction.run_reduction(work, None)
            self.assertIn(red.DIRECT, str(raised.exception))
            self.assertIn("DUMPS['it-fr']", str(raised.exception))

    def test_the_credits(self):
        with tempfile.TemporaryDirectory() as work:
            self.reduction.write_work(work)
            self.reduction.run_reduction(work, None)
            notice = _read(work, "NOTICE")
            manifest = json.loads(_read(work, "manifest.json"))
            measured = json.loads(_read(work, "measures.json"))
        self.assertIn("French Wiktionary (frwiktionary), Italian section", notice)
        self.assertIn("Italian Wiktionary (itwiktionary)", notice)
        self.assertEqual(manifest["meta"]["licences"][0], "kaikki / enwiktionary, frwiktionary, itwiktionary (CC BY-SA 4.0 + GFDL)")
        self.assertEqual(set(measured), {"top", "glossed", "share", "direct", "inverted", "coverage", "by source", "sample"})


# — refine-lingua-it-fr-glosses (the Italian programme's change 17): Italian's forms and readings (D11, D12) —

FIRST_PLURAL = ["first-person", "indicative", "plural", "present"]
SECOND_PLURAL = ["indicative", "plural", "present", "second-person"]
# The English section's entries as kaikki writes them, for D11's four rules and its reviewed rows.
REFINED_ENTRIES = [
    # Rule 1: a verb's person whose own entry holds no meaning reads as its verb — an ellipsis (`sto`),
    # another part of speech's pointer marked obsolete and poetic (`vado`), the verb's inflection written
    # without its tag (`sapete`, kaikki's raw tag « of sapere »).
    entry("stare", pos="verb", forms=[*TABLE_HEAD, ("stò", PRESENT_1S), ("stà", PRESENT_3S)]),
    form_of("sto", "stare", tags=("first-person", "form-of", "indicative", "present", "singular")),
    entry(
        "sto",
        pos="phrase",
        senses=[
            {
                "glosses": ["ellipsis of sto bene (“I'm fine”)"],
                "tags": ["abbreviation", "alt-of", "colloquial", "ellipsis"],
                "alt_of": [{"word": "sto bene", "extra": "I'm fine"}],
            }
        ],
    ),
    entry("andare", pos="verb", forms=[*TABLE_HEAD, ("vàdo", PRESENT_1S)]),
    form_of("vado", "andare", tags=("first-person", "form-of", "indicative", "present", "singular")),
    entry(
        "vado",
        senses=[
            {
                "glosses": ["alternative form of guado"],
                "tags": ["alt-of", "alternative", "literary", "masculine", "obsolete", "poetic"],
                "alt_of": [{"word": "guado"}],
            }
        ],
    ),
    entry("guado"),
    entry("sapere", pos="verb", forms=[*TABLE_HEAD, ("sapéte", SECOND_PLURAL)]),
    entry(
        "sapete",
        pos="verb",
        senses=[{"glosses": ["(you) know"], "tags": [*SECOND_PLURAL, "singular"], "raw_tags": ["of sapere"]}],
    ),
    # … and what it leaves: another word's inflection (`dee`, dea's plural), a contraction (`cogli`), a
    # function word's homograph (`contra`), a form the section heads as an acronym (`tari`), a participle
    # (`elisa`) and an alternative infinitive (`cagare`), which no table gives as a person.
    entry("dovere", pos="verb", forms=[*TABLE_HEAD, ("dèe", PRESENT_3S)]),
    entry("dea", forms=[("dee", ["plural"])]),
    form_of("dee", "dea", pos="noun", tags=("feminine", "form-of", "plural")),
    entry(
        "dee",
        pos="verb",
        senses=[
            {
                "glosses": ["alternative form of deve, third-person singular present indicative of dovere"],
                "tags": ["alt-of", "alternative", "archaic", "poetic"],
                "alt_of": [{"word": "deve"}],
            }
        ],
    ),
    entry("cogliere", pos="verb", forms=[*TABLE_HEAD, ("còlgo", PRESENT_1S), ("cògli", ["imperative", "second-person", "singular"])]),
    alt_of("cogli", "con gli", pos="contraction", tags=("abbreviation", "alt-of", "contraction", "dated")),
    form_of("cogli", "cogliere", tags=("form-of", "imperative", "second-person", "singular")),
    entry("contrare", pos="verb", forms=[*TABLE_HEAD, ("cóntra", PRESENT_3S)]),
    alt_of("contra", "contro", pos="prep", tags=("alt-of", "alternative", "archaic")),
    form_of("contra", "contrare", tags=("form-of", "indicative", "present", "singular", "third-person")),
    entry("contro", pos="prep"),
    entry("tarare", pos="verb", forms=[*TABLE_HEAD, ("tàri", ["indicative", "present", "second-person", "singular"])]),
    form_of("tari", "tarare", tags=("form-of", "indicative", "present", "second-person", "singular")),
    entry("TARI", senses=[{"glosses": ["abbreviation of tassa sui rifiuti"], "tags": ["abbreviation", "alt-of"], "alt_of": [{"word": "tassa sui rifiuti"}]}]),
    entry("elidere", pos="verb", forms=[*TABLE_HEAD, ("elìso", PARTICIPLE)]),
    entry("eliso", pos="verb", forms=[("elisa", ["feminine"])], senses=[{"tags": ["form-of", "participle", "past"], "form_of": [{"word": "elidere"}]}]),
    entry("elisa", pos="verb", senses=[{"glosses": ["elided"], "tags": ["feminine"]}]),
    # `mei`: meare's person, and meo's plural along an adjective's link — another word's.
    entry("meare", pos="verb", forms=[*TABLE_HEAD, ("mèi", ["indicative", "present", "second-person", "singular"])]),
    form_of("mei", "meare", tags=("form-of", "indicative", "present", "second-person", "singular")),
    form_of("mei", "meo", pos="adj", tags=("form-of", "masculine", "plural")),
    alt_of("mei", "meglio", pos="adv", tags=("alt-of", "alternative", "archaic")),
    entry("meo", pos="adj", senses=[{"glosses": ["Old Italian form of mio"]}]),
    entry("cacare", pos="verb", forms=[*TABLE_HEAD, ("càco", PRESENT_1S)]),
    entry("cagare", pos="verb", forms=[*TABLE_HEAD], senses=[{"glosses": ["alternative form of cacare"], "tags": ["alt-of", "alternative"], "alt_of": [{"word": "cacare"}]}]),
    # Rule 2: no form of a form through a noun's plural (`viti` → `vite` → *vita*), but through its own
    # table's row (`crochi` → *croco*) or a feminine (`maestre` → `maestra` → *maestro*).
    entry("vita", forms=[("vite", ["plural"])], senses=[{"glosses": ["life"], "tags": ["feminine"]}]),
    entry("vite", forms=[("viti", ["plural"])], senses=[{"glosses": ["vine"], "tags": ["feminine"]}]),
    form_of("vite", "vita", pos="noun", tags=("feminine", "form-of", "plural")),
    form_of("viti", "vite", pos="noun", tags=("feminine", "form-of", "plural")),
    entry("croco", forms=[("crochi", ["plural"])], senses=[{"glosses": ["crocus"], "tags": ["masculine"]}]),
    entry("croce", forms=[("croci", ["plural"])], senses=[{"glosses": ["cross"], "tags": ["feminine"]}]),
    form_of("croci", "croce", pos="noun", tags=("feminine", "form-of", "plural")),
    entry("crochi", senses=[{"glosses": ["plural of croco crocuses, croci"], "tags": ["form-of", "masculine", "plural"], "form_of": [{"word": "croco crocuses"}, {"word": "croci"}]}]),
    entry("faccia", forms=[("facce", ["plural"])], senses=[{"glosses": ["face"], "tags": ["feminine"]}]),
    form_of("facce", "faccia", pos="noun", tags=("feminine", "form-of", "plural")),
    form_of("faccie", "facce", pos="noun", tags=("form-of", "misspelling")),
    entry("maestro", forms=[("maestra", ["feminine"])], senses=[{"glosses": ["master"], "tags": ["masculine"]}]),
    form_of("maestra", "maestro", pos="noun", tags=("feminine", "form-of")),
    form_of("maestre", "maestra", pos="noun", tags=("feminine", "form-of", "plural")),
    # Rule 3: a feminine noun's masculine row (`signorina`'s `signorini`), unless the form's own entry
    # names the noun (`echi`, *eco*'s).
    headed(
        entry("signorina", forms=[("signorine", ["plural"]), ("signorino", ["masculine"]), ("signorini", ["masculine", "plural"])], senses=[{"glosses": ["miss"], "tags": ["feminine"]}]),
        **{"1": "f"},
    ),
    headed(entry("signorino", forms=[("signorini", ["plural"])], senses=[{"glosses": ["young master"], "tags": ["masculine"]}]), **{"1": "m"}),
    form_of("signorini", "signorino", pos="noun", tags=("form-of", "masculine", "plural")),
    headed(entry("eco", forms=[("echi", ["masculine", "plural"])], senses=[{"glosses": ["echo"]}]), **{"1": "f,m<l:colloquial>"}),
    form_of("echi", "eco", pos="noun", tags=("form-of", "masculine", "plural")),
    # Rule 4: *avere*'s Classical Roman Dialect table (no first person plural in -iamo) gives `avi` and
    # `ava`, which their own entries name *avo*'s; *essere*'s gives `son`, whose own entry names `sono`, no
    # ranked word; `ate`'s names *avere* itself. *essere*'s usage notes write headings as cells.
    entry(
        "avere",
        pos="verb",
        forms=[
            *TABLE_HEAD,
            ("abbiàmo", FIRST_PLURAL),
            ("àte", SECOND_PLURAL),
            ("regular", ["table-tags"]),
            ("avémo", FIRST_PLURAL),
            ("àvi", ["indicative", "present", "second-person", "singular"]),
            ("àva", PRESENT_3S),
            ("àmo", FIRST_PLURAL),
            ("crési", ["first-person", "historic", "past", "singular"]),
        ],
    ),
    # `cresi`: the dialect's row, and its own entries naming *creso* and *credere*'s obsolete past historic.
    entry("creso", forms=[("cresi", ["plural"])], senses=[{"glosses": ["rich man"], "tags": ["masculine"]}]),
    form_of("cresi", "creso", pos="noun", tags=("form-of", "masculine", "plural")),
    form_of("cresi", "avere", tags=("first-person", "form-of", "historic", "obsolete", "past", "singular")),
    # … where the dialect's verb is not the one chosen, nothing moves: `amo`, the hook and amare's person.
    entry("amo", senses=[{"glosses": ["fishhook"], "tags": ["masculine"]}]),
    entry("amare", pos="verb", forms=[*TABLE_HEAD, ("àmo", PRESENT_1S)]),
    form_of("amo", "amare", tags=("first-person", "form-of", "indicative", "present", "singular")),
    entry("avo", forms=[("avi", ["plural"]), ("ava", ["feminine"])], senses=[{"glosses": ["ancestor"], "tags": ["masculine"]}]),
    form_of("avi", "avo", pos="noun", tags=("form-of", "masculine", "plural")),
    form_of("ava", "avo", pos="noun", tags=("feminine", "form-of")),
    form_of("ate", "avere", tags=("form-of", "indicative", "plural", "present", "second-person")),
    entry(
        "essere",
        pos="verb",
        forms=[
            *TABLE_HEAD,
            ("siàmo", FIRST_PLURAL),
            ("sóno", PRESENT_1S),
            ("regular", ["table-tags"]),
            ("semo", FIRST_PLURAL),
            ("són", PRESENT_1S),
            ("no-table-tags", ["table-tags"]),
            ("archaic", []),
            ("subjunctive", []),
            ("fósse", ["imperfect", "singular", "subjunctive", "third-person", "archaic"]),
        ],
    ),
    form_of("sono", "essere", tags=("first-person", "form-of", "indicative", "present", "singular")),
    alt_of("son", "sono", pos="verb", tags=("abbreviation", "alt-of", "apocopic")),
    # The reviewed rows: the apocope `signor` and the euphonic `od`; `usa` → *usare*; `dati` → *dati*,
    # the plural noun of its own, while `dato` stays *dare*'s.
    headed(entry("signore", forms=[("signori", ["plural"])], senses=[{"glosses": ["gentleman"], "tags": ["masculine"]}]), **{"1": "m"}),
    alt_of("signor", "signore", pos="noun", tags=("abbreviation", "alt-of", "apocopic", "masculine")),
    entry("o", pos="conj"),
    entry("od", pos="conj", senses=[{"glosses": ["or (used before a vowel for euphony instead of o)"]}]),
    entry("uso", pos="adj", forms=[("usa", ["feminine"])]),
    entry("usare", pos="verb", forms=[*TABLE_HEAD, ("ùsa", PRESENT_3S)]),
    form_of("usa", "uso", pos="adj", tags=("feminine", "form-of", "singular")),
    form_of("usa", "usare", tags=("form-of", "indicative", "present", "singular", "third-person")),
    entry("dare", pos="verb", forms=[*TABLE_HEAD, ("dàto", PARTICIPLE), ("dàti", ["masculine", "participle", "past", "plural"])]),
    entry("dato", forms=[("dati", ["plural"])], senses=[{"glosses": ["datum"], "tags": ["masculine"]}]),
    form_of("dato", "dare", tags=("form-of", "participle", "past")),
    entry("dati", senses=[{"glosses": ["data (plural of dato)"], "tags": ["masculine"]}]),
    form_of("dati", "dato", tags=("form-of", "masculine", "participle", "plural")),
]
REFINED_TOP = [
    "essere", "avere", "stare", "andare", "sapere", "dovere", "dare", "dato", "dati", "sto", "vado", "sapete", "usa",
    "usare", "uso", "signore", "signor", "o", "od", "vita", "vite", "viti", "croce", "croci", "croco", "crochi", "maestro",
    "maestra", "maestre", "signorina", "signorino", "signorini", "eco", "echi", "avo", "avi", "ava", "ate", "son", "sono",
    "dea", "dee", "cogliere", "cogli", "contrare", "contra", "contro", "tarare", "tari", "elidere", "elisa", "cacare", "cagare",
    "guado", "amo", "amare", "faccia", "facce", "faccie", "mei", "meo", "meare", "creso", "cresi",
]
REFINED_FREQUENCY = {word: 7.0 - i * 0.05 for i, word in enumerate(REFINED_TOP)}
# MarkIT's counts, doubled: `dato` read as dare, as MarkIT reads it 9 times against 3, and `vite` as vita.
REFINED_COUNTS = collections.Counter({("dato", "dare"): 9, ("dato", "dato"): 3, ("vite", "vita"): 2})


def refined_forms(entries=REFINED_ENTRIES, counts=REFINED_COUNTS):
    """The forms and ranks over `entries` as the reduction reads them (rules 1 to 4, the reviewed rows)."""
    lexicon = section(entries)
    red.name_or_word(lexicon.candidates, lexicon.poses, lambda w: REFINED_FREQUENCY.get(w, 0.0))
    red.own_words(lexicon)
    red.read_apocopes(lexicon)
    frequency = lambda w: REFINED_FREQUENCY.get(w, 0.0)  # noqa: E731
    forms, ranks, step = red.reduce_forms(lexicon, counts, {}, frequency, lambda n: REFINED_TOP[:n], 200)
    return lexicon, forms, ranks, step


class RefinedForms(unittest.TestCase):
    """refine-lingua-it-fr-glosses D11: a form's own entry counts only by a meaning, no chain through a plural,
    a feminine noun's masculine row and the tables the readings set aside, the reviewed rows; D12: a reading
    set aside by name."""

    @classmethod
    def setUpClass(cls):
        cls.lexicon, cls.forms, cls.ranks, cls.step = refined_forms()

    def test_spec_scenario_a_verb_s_first_person_read_as_its_verb(self):
        # `vado`: its noun is « alternative form of guado », obsolete and poetic; `sapete` only « (you)
        # know », sapere's inflection written without its tag; `sto` beside an ellipsis.
        self.assertEqual((self.forms["vado"], self.forms["sapete"], self.forms["sto"]), ("andare", "sapere", "stare"))
        for word in ("vado", "sapete", "sto"):
            self.assertNotIn(word, self.ranks, word)
            self.assertNotIn(word, self.lexicon.lemmas, word)
        self.assertEqual(sorted(self.lexicon.inflections_read), ["sapete", "sto", "vado"])
        self.assertEqual(self.lexicon.stats["verb forms whose own entry holds no meaning, read as their verb"], 3)

    def test_spec_scenario_what_the_own_entry_rule_leaves(self):
        # `dee`, also dea's plural; `cogli`, a contraction; `contra`, a preposition's homograph; `tari`,
        # which the section heads as an acronym; `elisa`, a participle's feminine; `cagare`, an
        # alternative infinitive: each keeps the reading the other rules give it.
        self.assertIn("dee", self.lexicon.lemmas)
        self.assertEqual(self.forms["dee"], "dee")
        # `mei`, meare's person, also meo's plural along an adjective's link.
        self.assertIn("mei", self.lexicon.lemmas)
        self.assertEqual(self.forms["mei"], "mei")
        for word in ("cogli", "contra", "tari", "elisa", "cagare"):
            self.assertIn(word, self.lexicon.lemmas, word)
            self.assertIn(word, self.lexicon.candidates[word], word)
        self.assertNotIn("elisa", self.lexicon.finite)
        self.assertEqual(self.forms["elisa"], "elisa")

    def test_no_meaning(self):
        # An ellipsis, another part of speech's pointer marked outdated, a verb's inflection written without
        # its tag (the verbs named); a meaning.
        self.assertEqual(red.no_meaning({"glosses": ["ellipsis of sto bene"], "tags": ["ellipsis"]}, "phrase"), ())
        self.assertEqual(red.no_meaning({"glosses": ["x"], "tags": ["archaic"], "alt_of": [{"word": "ruota"}]}, "noun"), ())
        self.assertIsNone(red.no_meaning({"glosses": ["x"], "tags": ["rare"], "alt_of": [{"word": "sanguigna"}]}, "noun"))
        self.assertEqual(
            red.no_meaning({"glosses": ["first-person plural present present subjunctive/imperative of compiere and compire"]}, "verb"),
            frozenset({"compiere", "compire"}),
        )
        self.assertEqual(red.no_meaning({"glosses": ["(you) know"], "raw_tags": ["of sapere"]}, "verb"), frozenset({"sapere"}))
        self.assertIsNone(red.no_meaning({"glosses": ["to know"]}, "verb"))
        self.assertIsNone(red.no_meaning({"glosses": ["league"]}, "noun"))

    def test_spec_scenario_no_chain_through_a_plural(self):
        # `viti` reaches `vite`, itself a noun's plural, and no *vita*: no ranked lemma, so no form.
        self.assertNotIn("viti", self.forms)
        self.assertTrue(self.lexicon.plural_links("viti", "vite"))
        # `crochi`'s own entry names croco too: a plural of a word that is no plural.
        self.assertFalse(self.lexicon.plural_links("crochi", "croco"))
        # A spelling of a plural is no plural of it: `faccie` reaches *faccia*.
        self.assertEqual(self.forms["faccie"], "faccia")
        # `crochi`: through `croci`, croce's plural, no *croce*; *croco*'s own row reads it.
        self.assertEqual(self.forms["crochi"], "croco")

    def test_spec_scenario_a_chain_through_a_feminine(self):
        self.assertEqual(self.forms["maestre"], "maestro")

    def test_spec_scenario_a_feminine_noun_s_masculine_row(self):
        self.assertEqual(self.forms["signorini"], "signorino")
        self.assertNotIn("signorina", self.lexicon.candidates["signorini"])
        # `echi`'s own entry names *eco*: its row stays.
        self.assertEqual(self.forms["echi"], "eco")
        self.assertIn("eco", self.lexicon.candidates["echi"])
        self.assertTrue(red._masculine_row({"masculine", "plural"}, ("Fem",)))
        self.assertTrue(red._masculine_row({"masculine"}, ("Masc", "Fem")))
        self.assertFalse(red._masculine_row({"masculine", "plural"}, ("Fem", "Masc")))
        self.assertFalse(red._masculine_row({"feminine", "plural"}, ("Fem",)))

    def test_spec_scenario_the_dialect_s_table(self):
        # `avi` and `ava`: the dialect's rows of *avere* do not outrank their own entries naming *avo*.
        self.assertEqual((self.forms["avi"], self.forms["ava"]), ("avo", "avo"))
        self.assertIn(("avi", "avere"), self.lexicon.dialect_only())
        # `son`: its own entry names `sono`, no ranked word: still *essere*'s. `ate` names *avere* itself.
        self.assertEqual((self.forms["son"], self.forms["ate"]), ("essere", "avere"))
        # `amo`: the dialect's *avere* is not the lemma chosen — the hook, its own entry, is.
        self.assertIn(("amo", "avere"), self.lexicon.dialect_only())
        self.assertEqual(self.forms["amo"], "amo")
        # `cresi`: its own entry names the dialect's verb too — no row of the dialect's alone.
        self.assertNotIn(("cresi", "avere"), self.lexicon.dialect_only())
        self.assertEqual(self.forms["cresi"], "avere")

    def test_spec_scenario_a_table_s_tag_is_no_form(self):
        for word in ("archaic", "subjunctive"):
            self.assertNotIn(word, self.forms, word)
            self.assertNotIn(word, self.lexicon.candidates, word)
        self.assertEqual(self.lexicon.stats["a table's headings, no form"], 2)

    def test_spec_scenario_an_override_row(self):
        self.assertEqual(self.forms["usa"], "usare")
        self.assertIn("uso", self.lexicon.candidates["usa"])
        self.assertEqual(red.decided_by("usa", {"uso", "usare"}, REFINED_COUNTS, self.lexicon.lemmas, lambda w: REFINED_FREQUENCY.get(w, 0.0)), "override")

    def test_spec_scenario_a_plural_noun_of_its_own(self):
        # `dati` → *dati*, ranked, its own form read as itself; `dato` still *dare*'s, MarkIT 9 against 3.
        self.assertEqual(self.forms["dati"], "dati")
        self.assertIn("dati", self.ranks)
        self.assertEqual(self.forms["dato"], "dare")
        self.assertNotIn("dato", red.OVERRIDES)
        self.assertNotIn("dato", self.ranks)

    def test_spec_scenario_an_apocope_and_a_euphonic_form(self):
        self.assertEqual((self.forms["signor"], self.forms["od"]), ("signore", "o"))
        for word in ("signor", "od"):
            self.assertNotIn(word, self.ranks, word)
            self.assertNotIn(word, self.lexicon.lemmas, word)
        self.assertEqual(list(red.APOCOPES), sorted(red.APOCOPES), "kept sorted")

    def test_the_reviewed_rows_hold_their_reasons(self):
        for table in (red.OVERRIDES, red.APOCOPES):
            for form, (lemma, reason) in table.items():
                self.assertTrue(lemma and len(reason) > 30, form)
        for (form, lemma, tag), reason in red.READINGS_SET_ASIDE.items():
            self.assertTrue(len(reason) > 30, form)
            self.assertEqual(red.parse_tag(tag)[0], "VERB")


class ReadingsSetAside(unittest.TestCase):
    """refine-lingua-it-fr-glosses D12: a reading the section's tables give a form wrongly, set aside by name."""

    def test_spec_scenario_an_imperative_read_as_a_subjunctive(self):
        sub = "VERB|Mood=Sub|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin"
        ind = "VERB|Mood=Ind|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin"
        imp = "VERB|Mood=Imp|Number=Plur|Person=2|VerbForm=Fin"
        rows = [f"fate\tfare\t{tag}\t-\n" for tag in (imp, ind, sub)] + ["fate\tfata\tNOUN|Gender=Fem|Number=Plur\tother\n"]
        kept, unread = red.without_readings_set_aside(rows)
        self.assertEqual(unread, [])
        self.assertEqual(kept, [rows[0], rows[1], rows[3]])
        self.assertEqual(list(red.READINGS_SET_ASIDE), [("fate", "fare", sub)])

    def test_spec_scenario_a_row_that_finds_nothing(self):
        kept, unread = red.without_readings_set_aside(["fate\tfare\tVERB|Mood=Imp|Number=Plur|Person=2|VerbForm=Fin\t-\n"])
        self.assertEqual(unread, [("fate", "fare", "VERB|Mood=Sub|Number=Plur|Person=2|Tense=Pres|VerbForm=Fin")])
        self.assertEqual(len(kept), 1)


# — refine-lingua-it-fr-glosses (the Italian programme's change 17): it-fr's glosses refined (D2–D10) —


def fr_alt(text, *targets, tags=("alt-of",)):
    """A sense kaikki files as an alternative form of `targets` (`paro` « Variante de paio. »)."""
    return {"glosses": [text], "tags": list(tags), "alt_of": [{"word": t} for t in targets]}


class RefinedSection(unittest.TestCase):
    """refine-lingua-it-fr-glosses D2–D10, over a small French Wiktionary's Italian section shaped as kaikki
    writes it — the senses quoted are the section's —, MarkIT's parts of speech doubled and a small forms
    table."""

    SECTION = [
        # D2: MarkIT's order for a function word; a content word as the section writes it; too little
        # evidence; a row the section opens on a proper noun, whose proper noun never comes first.
        fr_entry("fra", "noun", "Frère."),
        fr_entry("fra", "prep", "Entre, durant."),
        fr_entry("piacere", "verb", "Plaire."),
        fr_entry("piacere", "noun", "Plaisir."),
        fr_entry("subito", "adj", "Immédiat, instantané."),
        fr_entry("subito", "adv", "Aussitôt."),
        fr_entry("Lira", "name", "Lyre (constellation)."),
        fr_entry("lira", "noun", "Lire (monnaie)."),
        # D2: the reviewed table — held against the treebank, a part of speech MarkIT does not read, two
        # entries of one part of speech.
        fr_entry("come", "adv", "Comme.", "Comment forme interrogative."),
        fr_entry("come", "conj", "Dès que.", "Que dans une comparaison"),
        fr_entry("fino", "adj", "Fin."),
        fr_entry("fino", "prep", "Jusque."),
        fr_entry("Loro", "pron", "Pronom de la troisième personne du pluriel, utilisé pour la politesse (équivalent au vouvoiement)."),
        fr_entry("loro", "pron", "Ils, elles, eux."),
        fr_entry("loro", "adj", "Leur."),
        # D3: a name's sense its common entry repeats; D4: a pronoun's and an adverb's homograph names.
        fr_entry("Bibbia", "name", senses=[{"glosses": ["Bible."], "tags": ["Biblical"]}]),
        fr_entry("bibbia", "noun", senses=[{"glosses": ["Bible."], "tags": ["Biblical"]}]),
        fr_entry("luna", "noun", "Lune."),
        fr_entry("Luna", "name", "Lune, satellite de la Terre."),
        fr_entry(
            "io",
            "pron",
            "Pronom de la première personne du singulier.",
            "io sono in Francia. — moi, je suis en France.",
            "Pago io o paghi tu ? — je paie ou tu paies ?",
            "io pago. — c’est moi qui paie.",
        ),
        fr_entry("Io", "name", "Io, fille d’Inachus, maîtresse de Zeus.", "Io, satellite naturel de la planète Jupiter."),
        fr_entry("qui", "adv", "Ici."),
        fr_entry("Qui", "name", "Riri."),
        fr_entry("mai", "adv", "Jamais, en nul temps. — Note d’usage : Se rencontre combiné avec la particule négative non."),
        # D5: an untagged plural, an alternative form, an obsolete one, a superlative's and a plural's
        # quoted meaning, a clitic compound's pronoun.
        fr_entry("lettere", "noun", "Pluriel de lettera.", tags=("feminine", "form-of")),
        fr_entry("lettera", "noun", "Lettre."),
        fr_entry("paro", "noun", senses=[fr_alt("Variante de paio.", "paio")]),
        fr_entry("paio", "noun", "Paire."),
        fr_entry("fora", "adv", senses=[fr_alt("Variante de fuori.", "fuori", tags=("alt-of", "obsolete"))]),
        fr_entry("fuori", "adv", "Dehors."),
        fr_entry("maio", "adj", senses=[fr_alt("Variante de maggio.", "maggio", tags=("alt-of", "obsolete"))]),
        fr_entry("maggio", "noun", "Mai."),
        fr_entry("pochissimo", "adj", senses=[fr_link("pochissimo", "adj", "Superlatif de poco : « très peu ».", "peu")]),
        fr_entry("ceneri", "noun", senses=[fr_link("ceneri", "noun", "Pluriel de cenere (« cendre »).", "cendre")]),
        fr_entry("cenere", "noun", "Cendre, résidu de la combustion."),
        fr_entry(
            "vallo",
            "verb",
            senses=[fr_link("vallo", "verb", "Agglutination de la deuxième personne du singulier de l’impératif présent du verbe andare avec le pronom personnel masculin singulier lo (« le »).", "lo", tags=("enclitic", "form-of"))],
        ),
        fr_entry("lo", "article", "Le."),
        # D6: a plural kept from lending by a constellation, by an archaic sense; a verb form's pointer beside
        # a name.
        fr_entry("gemelli", "adj", senses=[fr_link("gemelli", "adj", "Masculin pluriel de gemello.", "gemello")]),
        fr_entry("gemelli", "noun", senses=[fr_link("gemelli", "noun", "Pluriel de gemello.", "gemello")]),
        fr_entry("Gemelli", "name", "Gémeaux (constellation et signe zodiacal)."),
        fr_entry("gemello", "adj", "Jumeau."),
        fr_entry("gemello", "noun", "Jumeau.", "Bouton de manchette."),
        fr_entry("spezie", "noun", senses=[{"glosses": ["Espèce"], "tags": ["archaic"]}]),
        fr_entry("spezie", "noun", senses=[fr_link("spezie", "noun", "Pluriel de spezia.", "spezia")]),
        fr_entry("spezia", "noun", "Épice, condiment végétal."),
        fr_entry("regolo", "verb", senses=[fr_link("regolo", "verb", "Première personne du singulier de l’indicatif présent de regolare.", "regolare")]),
        fr_entry("Regolo", "name", "Constellation de la Règle."),
        fr_entry("regolare", "verb", "Régler."),
        # D8: the `di` family read through their translations, a word with a gloss of its own, the usage
        # notes.
        fr_entry("della", "article", senses=[fr_link("della", "article", "Article contracté souvent traduit par De la.", "la")]),
        fr_entry("la", "article", "La."),
        fr_entry("degli", "article", senses=[{"glosses": ["Article contracté souvent traduit par des. Note d’usage : il s’emploie alternativement à dei devant les consonnes impures : s, gn, pn, ps, x, z."]}]),
        fr_entry("pelle", "noun", "Peau."),
        fr_entry("pelle", "article", senses=[fr_link("pelle", "article", "Article contracté souvent traduit par pour les.", "les")]),
        fr_entry("del", "article", "Du. Note d’usage : il s’utilise devant un mot commençant par une consonne."),
        fr_entry("sebbene", "conj", "Quoique. Note d’usage : suivi d’un verbe au subjonctif."),
        # D9: communes written otherwise, a provincial capital, cities named more than a commune, a commune
        # the reviewed list keeps, a name alone.
        fr_entry("vittoria", "noun", "Victoire."),
        fr_entry("Vittoria", "name", "Vittoria, commune et ville du libre consortium municipal de Raguse dans la région de Sicile en Italie."),
        fr_entry("ro", "noun", "Rhô."),
        fr_entry("Ro", "name", "Ro (commune italienne de la province de Ferrare, dans la région d’Émilie-Romagne."),
        fr_entry("matera", "noun", senses=[{"glosses": ["Matière."], "tags": ["archaic"]}]),
        fr_entry("Matera", "name", "Matera, commune et ville de la région de la Basilicate en Italie.", "Province autour de cette ville."),
        fr_entry("bordeaux", "noun", "Bordeaux, vin de la région de Bordeaux."),
        fr_entry("Bordeaux", "name", "Bordeaux, commune, ville et chef-lieu de département français, situé dans le département de la Gironde."),
        fr_entry("colonia", "noun", "Colonie."),
        fr_entry("Colonia", "name", "Cologne, ville, commune, arrondissement et district d’Allemagne."),
        fr_entry("Piacenza", "name", "Plaisance, commune italienne."),
        fr_entry("piacenza", "noun", "plaisance, agrément."),
        fr_entry("trani", "verb", senses=[fr_link("trani", "verb", "Deuxième personne du singulier de l’indicatif présent de tranare.", "tranare")]),
        fr_entry("Trani", "name", "Trani, commune et ville de la province de Barletta-Andria-Trani dans la région des Pouilles en Italie."),
        # D10: the corrections — a meaning the section misses, an example filed as senses, a participle's lent
        # meaning, a plural noun of its own.
        fr_entry("che", "pron", senses=[{"glosses": ["Quoi."]}], tags=("interrogative",)),
        fr_entry("che", "adj", senses=[{"glosses": ["Quel."]}], tags=("interrogative",)),
        fr_entry(
            "volere",
            "verb",
            "Vouloir.",
            "Volerci, falloir.",
            "Il faut dix heures pour aller à Brindisi.",
            "Ci vuole un uovo per la torta.",
            "Il faut un œuf pour le gâteau.",
        ),
        fr_entry("dirotto", "verb", senses=[fr_link("dirotto", "verb", "Participe passé de dirompere.", "dirompere")]),
        fr_entry("dirompere", "verb", "Écanguer."),
        fr_entry("dati", "adj", senses=[fr_link("dati", "adj", "Masculin pluriel de dato.", "dato")]),
        fr_entry("dati", "noun", senses=[fr_link("dati", "noun", "Pluriel de dato.", "dato")], tags=("form-of", "masculine")),
        fr_entry("dato", "adj", "Donné."),
        fr_entry("dato", "noun", "Donnée."),
        # D10: words split after their elided piece, one no Italian.
        fr_entry("d’n’b", "noun", "Drum and bass."),
        fr_entry("d’altronde", "adv", "D’ailleurs."),
    ]
    INVERTED = [
        translations("qwerty", "noun", "c’hwerty"),
        translations("d'accord", "adv", "d’accordo"),
    ]
    LEMMAS = [
        "fra", "piacere", "subito", "lira", "come", "fino", "loro", "bibbia", "luna", "io", "qui", "mai", "lettere",
        "paro", "fora", "maio", "pochissimo", "ceneri", "vallo", "gemelli", "spezie", "regolo", "della", "degli",
        "pelle", "del", "sebbene", "vittoria", "ro", "matera", "bordeaux", "colonia", "piacenza", "trani", "che",
        "volere", "dirotto", "dati",
    ]
    RANKS = {lemma: rank for rank, lemma in enumerate(LEMMAS, 1)}
    # The forms table the example rule reads: `io`'s forms and the words of its examples.
    FORMS = {
        **{lemma: lemma for lemma in LEMMAS},
        **{w: w for w in ("sono", "in", "francia", "pago", "o", "paghi", "tu", "essere", "pagare")},
        "sono": "essere", "pago": "pagare", "paghi": "pagare", "me": "io",
    }
    # MarkIT's parts of speech, doubled (`reduce_italian_treebank.markit_pos_counts`): `fra` 13 prepositions,
    # `piacere` more often a noun, `subito` 3 adverbs, `come` 39 conjunctions against 16 adverbs, `fino` 8
    # adverbs, `pure` 3; `lira` read as a proper noun.
    COUNTS = collections.Counter(
        {
            ("fra", "ADP"): 13,
            ("piacere", "NOUN"): 20,
            ("piacere", "VERB"): 5,
            ("subito", "ADV"): 3,
            ("come", "SCONJ"): 39,
            ("come", "ADV"): 16,
            ("fino", "ADV"): 8,
            ("lira", "PROPN"): 30,
            ("che", "PRON"): 700,
            ("che", "SCONJ"): 406,
        }
    )

    @classmethod
    def setUpClass(cls):
        tmp = tempfile.TemporaryDirectory()
        cls.addClassCleanup(tmp.cleanup)
        work = tmp.name
        write_native(work, cls.SECTION, [], cls.INVERTED)
        cls.stats = collections.Counter()
        cls.refined = red.Refinement(cls.COUNTS, cls.FORMS)
        cls.section = red.section_as_read(
            os.path.join(work, red.SECTION), os.path.join(work, red.SECTION_READ), cls.stats, refined=cls.refined
        )
        inverted = red.common.read_translations(
            red.straight_translations(os.path.join(work, red.INVERTED), os.path.join(work, red.INVERTED_READ), inverted=True),
            inverted=True,
            studied=red.IT,
        )
        cls.sources = [({}, list), (inverted, red.common.by_native_frequency(lambda w: 0.0))]
        cls.glosses, cls.runs, cls.expressions, cls.steps = red.native_side(cls.section, cls.RANKS, cls.sources)
        red.without_repeated_senses(cls.glosses, cls.runs)
        cls.split, _ = red.section_words(cls.section, cls.FORMS, set())
        cls.split_tables = red.table_split_words(cls.sources, cls.FORMS, cls.split)
        cls.words = red.french.dictionary_words(cls.glosses, cls.runs)
        with open(cls.section, encoding="utf-8") as f:
            cls.read = [json.loads(line) for line in f]
        cls.left = os.listdir(work)

    def test_spec_scenario_a_function_word_s_commonest_part_of_speech(self):
        self.assertEqual(self.glosses["fra"], "Entre, durant; Frère")
        self.assertEqual(self.runs["fra"], [("ADP", 1), ("NOUN", 1)])

    def test_spec_scenario_a_content_word_keeps_the_section_s_order(self):
        self.assertEqual(self.glosses["piacere"], "Plaire; Plaisir")

    def test_spec_scenario_too_little_evidence(self):
        self.assertEqual(self.glosses["subito"], "Immédiat, instantané; Aussitôt")

    def test_a_proper_noun_never_first(self):
        # `lira` opens on `Lira`; MarkIT reads lira as a proper noun: no other part of speech is promoted.
        self.assertEqual(self.glosses["lira"], "Lyre (constellation); Lire (monnaie)")

    def test_a_fixed_token_counts_for_none(self):
        sample = (
            "1\tfra\tfra\tADP\t_\t_\t2\tcase\t_\t_\n"
            "2\tpoco\tpoco\tADV\t_\t_\t0\troot\t_\t_\n"
            "3\tfra\tfra\tADP\t_\t_\t2\tfixed\t_\t_\n"
        )
        with tempfile.TemporaryDirectory() as folder:
            path = os.path.join(folder, "it_markit-ud-train.conllu")
            with open(path, "w", encoding="utf-8") as f:
                f.write(sample)
            counts = treebank.markit_pos_counts([path])
        self.assertEqual(counts[("fra", "ADP")], 1)

    def test_spec_scenario_a_row_the_table_holds_against_the_treebank(self):
        self.assertTrue(self.glosses["come"].startswith("Comme; "), self.glosses["come"])
        self.assertIsNotNone(red.french_treebank.commonest_first("come", ["ADV", "SCONJ"], "ADV", self.COUNTS))

    def test_spec_scenario_a_row_the_reviewed_table_orders(self):
        self.assertEqual(self.glosses["fino"], "Jusque; Fin")

    def test_spec_scenario_two_entries_of_one_part_of_speech(self):
        self.assertTrue(self.glosses["loro"].startswith("Ils, elles, eux; "), self.glosses["loro"])
        self.assertEqual(self.runs["loro"], [("PRON", 2), ("ADJ", 1)])

    def test_the_reviewed_table(self):
        # Each row a case, a part of speech and a reason; the rows that found their entries recorded.
        self.assertEqual(sorted(red.FIRST_RUN), ["come", "fino", "gente", "loro", "pure"])
        for word, (case, upos, reason) in red.FIRST_RUN.items():
            self.assertIn(case, red._FIRST_RUN_CASES, word)
            self.assertIn(upos, {"ADV", "ADP", "NOUN", "PRON"}, word)
            self.assertGreater(len(reason), 30, word)
        self.assertEqual(self.refined.first_runs, {"come", "fino", "loro"})
        self.assertEqual(self.stats["rows opened by the reviewed table"], 2)
        self.assertEqual(self.stats["rows opened by MarkIT's commonest part of speech"], 1)

    def test_spec_scenario_a_common_word_its_name_repeats(self):
        self.assertEqual((self.glosses["bibbia"], self.runs["bibbia"]), ("Bible", [("NOUN", 1)]))
        self.assertIn("bibbia", self.words)
        # A proper noun's sense the common entry does not repeat stays.
        self.assertEqual(self.glosses["luna"], "Lune; Lune, satellite de la Terre")

    def test_spec_scenario_a_pronoun_s_and_an_adverb_s_homograph_names(self):
        self.assertNotIn("Io,", self.glosses["io"])
        self.assertEqual(self.glosses["qui"], "Ici")
        self.assertEqual(self.stats["names under a function word or an adverb"], 2)

    def test_spec_scenario_an_example_and_its_translation(self):
        self.assertEqual(self.glosses["io"], "Pronom de la première personne du singulier")
        self.assertEqual(self.stats["examples with their translation"], 3)
        refined = red.Refinement(None, self.FORMS)
        self.assertTrue(red.italian_example("io pago. — c’est moi qui paie.", "io", refined))
        # A French sentence before the dash, or no form of the headword in it, is no example.
        self.assertFalse(red.italian_example("Je paie. — je paie.", "io", refined))
        self.assertFalse(red.italian_example("tu paghi. — tu paies.", "io", refined))
        # No forms table: the rule reads nothing.
        self.assertFalse(red.italian_example("io pago. — c’est moi qui paie.", "io", red.Refinement()))

    def test_spec_scenario_a_note_after_a_dash(self):
        self.assertEqual(self.glosses["mai"], "Jamais, en nul temps. — Note d’usage : Se rencontre combiné avec la particule négative non")

    def test_spec_scenario_a_plural_the_source_does_not_tag(self):
        self.assertEqual(self.glosses["lettere"], "Lettre")

    def test_spec_scenario_an_alternative_form(self):
        self.assertEqual(self.glosses["paro"], "Paire")

    def test_spec_scenario_an_obsolete_alternative_form(self):
        self.assertNotIn("fora", self.glosses)
        self.assertNotIn("maio", self.glosses)

    def test_spec_scenario_a_superlative_s_meaning(self):
        self.assertEqual(self.glosses["pochissimo"], "Très peu")
        self.assertEqual(self.glosses["ceneri"], "Cendre")

    def test_spec_scenario_a_clitic_compound_s_pronoun(self):
        self.assertNotEqual(self.glosses.get("vallo"), "Le")

    def test_spec_scenario_a_plural_kept_from_lending_by_a_constellation(self):
        self.assertEqual(self.glosses["gemelli"], "Jumeau; Gémeaux (constellation et signe zodiacal); Bouton de manchette")
        self.assertIn("gemelli", self.words)

    def test_spec_scenario_an_archaic_sense_beside_a_plural(self):
        self.assertEqual(self.glosses["spezie"], "Épice, condiment végétal; Espèce")

    def test_a_verb_form_s_pointer_lends_nothing_beside_a_name(self):
        self.assertEqual(self.glosses["regolo"], "Constellation de la Règle")
        self.assertEqual(self.stats["pointers lending beside a name or an old sense"], 2)

    def test_spec_scenario_an_article_read_through_its_translation(self):
        self.assertEqual(self.glosses["della"], "De la")
        self.assertEqual(self.glosses["degli"], "Des")

    def test_spec_scenario_a_word_with_a_gloss_of_its_own(self):
        self.assertEqual(self.glosses["pelle"], "Peau")

    def test_spec_scenario_an_article_s_usage_note(self):
        self.assertEqual(self.glosses["del"], "Du")

    def test_spec_scenario_a_conjunction_s_usage_note(self):
        self.assertEqual(self.glosses["sebbene"], "Quoique. Note d’usage : suivi d’un verbe au subjonctif")

    def test_spec_scenario_a_commune_written_otherwise(self):
        self.assertEqual((self.glosses["vittoria"], self.glosses["ro"]), ("Victoire", "Rhô"))

    def test_spec_scenario_a_provincial_capital(self):
        self.assertEqual(
            self.glosses["matera"],
            "Matière; Matera, commune et ville de la région de la Basilicate en Italie; Province autour de cette ville",
        )

    def test_spec_scenario_a_city_named_more_than_a_commune(self):
        self.assertIn("chef-lieu", self.glosses["bordeaux"])
        self.assertIn("Cologne", self.glosses["colonia"])

    def test_spec_scenario_a_commune_the_reviewed_list_keeps(self):
        self.assertEqual(self.glosses["piacenza"], "Plaisance, commune italienne; plaisance, agrément")
        self.assertEqual(self.refined.kept, {"piacenza"})
        self.assertEqual(sorted(red.COMMUNES_KEPT), ["cesena", "piacenza"])

    def test_a_name_alone_left_with_no_gloss(self):
        # `trani`: a verb form's pointer and a commune alone — the town set aside, no gloss left.
        self.assertNotIn("trani", self.glosses)
        self.assertEqual(self.stats["communes' notes set aside"], 3)

    def test_spec_scenario_a_meaning_the_section_misses(self):
        self.assertEqual(self.glosses["che"], "Qui, que; Quoi; Que; Quel")
        self.assertEqual(self.runs["che"], [("PRON", 2), ("SCONJ", 1), ("ADJ", 1)])

    def test_spec_scenario_an_example_filed_as_senses(self):
        self.assertEqual(self.glosses["volere"], "Vouloir; Volerci, falloir")

    def test_spec_scenario_a_participle_s_lent_meaning_corrected(self):
        self.assertEqual((self.glosses["dirotto"], self.runs["dirotto"]), ("À verse (dans a dirotto)", [("ADJ", 1)]))

    def test_spec_scenario_a_plural_noun_of_its_own(self):
        self.assertEqual((self.glosses["dati"], self.runs["dati"]), ("Données", [("NOUN", 1)]))

    def test_the_corrections(self):
        # Every one fired; each names its source, its page and its reason.
        self.assertEqual(self.refined.fired, set(red.CORRECTIONS))
        self.assertEqual(red.unfired(self.refined.fired), [])
        for correction in red.CORRECTIONS:
            self.assertEqual(correction.source, "section")
            self.assertTrue(correction.report.startswith("https://fr.wiktionary.org/wiki/"), correction)
            self.assertGreater(len(correction.reason), 30, correction)
            for text in correction.corrected or ():
                self.assertEqual(red.read_sense({"glosses": [text]}), ({"glosses": [text]}, None), text)
        added = [e for e in self.read if e["word"] == "che"]
        self.assertEqual([e["pos"] for e in added], ["pron", "conj", "pron", "adj"])

    def test_spec_scenario_the_page_corrected_upstream(self):
        # `volere`'s examples no longer filed as senses: the correction fires on nothing, and says so.
        section = [fr_entry("volere", "verb", "Vouloir.", "Volerci, falloir.")]
        with tempfile.TemporaryDirectory() as work:
            write_native(work, section)
            refined = red.Refinement()
            red.section_as_read(os.path.join(work, red.SECTION), os.path.join(work, red.SECTION_READ), refined=refined)
        self.assertIn("volere (section, verb)", red.unfired(refined.fired))

    def test_spec_scenario_a_split_word_that_is_no_italian(self):
        self.assertNotIn("d'n'b", self.split)
        self.assertNotIn("c'hwerty", self.split_tables)
        self.assertIn("d'altronde", self.split)
        self.assertIn("d'accordo", self.split_tables)
        for word in ("c'hwerty", "l'wha", "d'n'b", "l'xeno", "d'jazz", "l'kappa"):
            self.assertFalse(red.split_word(word, {}), word)
        for word in ("d'accordo", "d'antan", "all'improvviso"):
            self.assertTrue(red.split_word(word, {}), word)

    def test_the_section_as_read_writes_what_it_does_not_change(self):
        # An entry no rule changes is written as its line was, where it stood; the intermediate file goes.
        casa_like = next(e for e in self.read if e["word"] == "luna")
        self.assertEqual(casa_like, self.SECTION[self.SECTION.index(fr_entry("luna", "noun", "Lune."))])
        self.assertEqual(sorted(self.left), sorted([red.SECTION, red.DIRECT, red.INVERTED, red.SECTION_READ, red.INVERTED_READ]))
        # A headword's entries — every case of it — written together where its first stood.
        words = [e["word"] for e in self.read]
        self.assertEqual(words[words.index("fra") : words.index("fra") + 2], ["fra", "fra"])
        self.assertEqual([e["pos"] for e in self.read if e["word"] == "fra"], ["prep", "noun"])


class RefinedReduction(unittest.TestCase):
    """refine-lingua-it-fr-glosses end to end: the reviewed lists checked, the report."""

    def setUp(self):
        self.reduction = Reduction()
        self.reduction.setUp()
        self.addCleanup(self.reduction.doCleanups)

    def test_a_first_run_row_naming_no_entry_fails_the_reduction(self):
        # D2: the reduction fails on a row of the reviewed table whose word has no such entry.
        with mock.patch.dict(red.FIRST_RUN, {"casa": ("lowercase", "ADV", "a row naming an entry the section does not give")}):
            with tempfile.TemporaryDirectory() as work:
                self.reduction.write_work(work)
                with self.assertRaises(SystemExit) as raised:
                    self.reduction.run_reduction(work, None)
        self.assertIn("FIRST_RUN", str(raised.exception))
        self.assertIn("casa (lowercase ADV)", str(raised.exception))

    def test_a_reading_set_aside_that_finds_nothing_fails_the_reduction(self):
        # D12: a row naming a reading the reduction does not give.
        row = ("casa", "casa", "NOUN|Number=Plur")
        with mock.patch.dict(red.READINGS_SET_ASIDE, {row: "a reading the tables do not give"}):
            with tempfile.TemporaryDirectory() as work:
                self.reduction.write_work(work)
                with self.assertRaises(SystemExit) as raised:
                    self.reduction.run_reduction(work, None)
        self.assertIn("READINGS_SET_ASIDE", str(raised.exception))
        self.assertIn("casa casa NOUN|Number=Plur", str(raised.exception))

    def test_the_report(self):
        # The corrections that fired on nothing named, to remove (D10); the « decided by » counts and M8's
        # cost printed as before (D11); the verb forms rule 1 read as their verb.
        with tempfile.TemporaryDirectory() as work:
            self.reduction.write_work(work)
            summary, lines = self.reduction.run_reduction(work, None)
        report = "\n".join(lines)
        # This section's `che` is a noun alone: the conjunction is added, the relative pronoun finds no
        # interrogative to go before.
        self.assertIn(
            f"reduced it-fr: 1 of {len(red.CORRECTIONS)} corrections fired; fired on nothing, to remove: che (section, pron), "
            "volere (section, verb), dirotto (section, adj), dati (section, noun)",
            report,
        )
        self.assertEqual(summary["corrections that fired on nothing"], [c for c in red.unfired(set()) if c != "che (section, conj)"])
        self.assertIn("reduced it-fr: one lemma per form, the forms reaching several: ", report)
        self.assertIn("by override", report)
        self.assertIn("reduced it-fr: M8's cost, dictionary nouns read as a verb among wordfreq's 1,000 / 2,000 / 5,000: ", report)
        self.assertIn("reduced it-fr: verb forms whose own entry holds no meaning, read as their verb=", report)
        self.assertEqual(summary["communes kept"], [])


if __name__ == "__main__":
    unittest.main()
