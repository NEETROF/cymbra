# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the it-en reducer (add-lingua-pack-it-en) and its pre-pass (`reduce_italian_en.py`).

The senses are recorded from the English Wiktionary's Italian section as `pack_sources.derive` writes it
from the English dump of 2026-10-21 (`kaikki-Italian.jsonl`, sha256 ae7de484…, it-fr's release
`lingua-pack-sources-it-fr-2026.10.23`), and the translations from the Italian Wiktionary's English
translations derived from its dump (`kaikki-it-traductions-en.jsonl`, sha256 e192643a…), cut down to the
fields the rules read. A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import pack_sources as ps  # noqa: E402
import reduce_common as common  # noqa: E402
import reduce_edition_en as english  # noqa: E402
import reduce_italian_acronyms as acronyms  # noqa: E402
import reduce_italian_en as italian  # noqa: E402


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(f"reduce_{pair.replace('-', '_')}_it_en", os.path.join(_HERE, f"reduce-{pair}.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


it_en = _reducer("it-en")
it_fr = _reducer("it-fr")


def sense(gloss, tags=(), form_of=None, alt_of=None):
    out = {"glosses": [gloss]}
    if tags:
        out["tags"] = list(tags)
    if form_of is not None:
        out["form_of"] = [{"word": w, **({"extra": x} if x else {})} for w, x in form_of]
    if alt_of is not None:
        out["alt_of"] = [{"word": w, **({"extra": x} if x else {})} for w, x in alt_of]
    return out


def entry(word, pos, *senses):
    return {"word": word, "pos": pos, "senses": list(senses)}


# — Recorded entries of the section —

DEL = entry(
    "del",
    "contraction",
    sense("contraction of di + il; some, of the, from the", ("abbreviation", "alt-of", "contraction"), alt_of=[("di", "+ il; some, of the, from the")]),
)
DELLA = entry(
    "della",
    "contraction",
    sense("contraction of di la; of the, from the", ("abbreviation", "alt-of", "contraction"), alt_of=[("di la", "of the, from the")]),
)
DEI = entry(
    "dei",
    "contraction",
    sense("contraction of di + i; of the, from the", ("abbreviation", "alt-of", "contraction"), alt_of=[("di", "+ i; of the, from the")]),
    sense("some", ("contraction",)),
)
DEI_NOUN = entry("dei", "noun", sense("plural of dio", ("form-of", "masculine", "plural"), form_of=[("dio", None)]))
DEI_VERB = entry(
    "dei",
    "verb",
    sense(
        "alternative form of devi, second-person singular present indicative of dovere",
        ("Tuscany", "alt-of", "alternative", "archaic", "colloquial", "poetic"),
        alt_of=[("devi", "second-person singular present indicative of dovere")],
    ),
)
ED = entry(
    "ed",
    "conj",
    sense(
        "alternative form of e for euphony, especially before /e/ or /ɛ/; and",
        ("alt-of", "alternative", "before-vowel"),
        alt_of=[("e for euphony", "especially before /e/ or /ɛ/; and")],
    ),
)
AD = entry(
    "ad",
    "prep",
    sense(
        "alternative form of a for euphony, especially before /a/; to, at, in",
        ("alt-of", "alternative", "before-vowel"),
        alt_of=[("a for euphony", "especially before /a/; to, at, in")],
    ),
)
AD_ACRONYM = entry(
    "AD",
    "noun",
    sense(
        "initialism of amministratore delegato (“CEO”)",
        ("abbreviation", "alt-of", "initialism"),
        alt_of=[("amministratore delegato", "CEO")],
    ),
)
AMMINISTRATORE_DELEGATO = entry("amministratore delegato", "noun", sense("chief executive officer, CEO", ("masculine",)))
GLI_ARTICLE = entry(
    "gli",
    "article",
    sense(
        "form of the article i (“the”) used before a vowel, impure s, gn, pn, ps, sc, x, y, and z, and also with the plural noun dei (“gods”); the",
        ("form-of", "masculine", "plural"),
        form_of=[("the article i", "(“the”) used before a vowel…")],
    ),
)
GLI_PRONOUN = entry("gli", "pron", sense("him, to him; it; to it", ("dative", "masculine")))
MI_PRONOUN = entry(
    "mi",
    "pron",
    sense("clitic accusative of io. me", ("accusative", "clitic", "first-person", "form-of", "objective"), form_of=[("io. me", None)]),
    sense("clitic dative of io. (to) me", ("clitic", "dative", "first-person", "form-of", "objective"), form_of=[("io", "(to) me")]),
    sense("Used as ethical dative.", ("colloquial", "first-person", "objective")),
)
MI_NOTE = entry("mi", "noun", sense("the third note, mi"), sense("E (musical note or key)"))
MI_LETTER = entry("mi", "noun", sense("mu (Greek letter)", ("feminine", "invariable", "masculine")))
CHI = entry("chi", "pron", sense("who, whom, whoever"))
CHI_LETTER = entry("chi", "noun", sense("chi (Greek letter)", ("feminine", "invariable", "masculine")))
ETA = entry("eta", "noun", sense("eta (Greek letter)", ("feminine", "invariable", "masculine")))
QUANTI_DET = entry("quanti", "det", sense("masculine plural of quanto", ("form-of", "masculine", "plural"), form_of=[("quanto", None)]))
QUANTI_PRON = entry("quanti", "pron", sense("synonym of chi sg (“who”): those who", ("formal", "masculine", "plural", "uncommon")))
QUANTO = entry("quanto", "det", sense("how much, how many, however much, however many"))
PIL = entry("pil", "noun", sense("alternative form of PIL", ("alt-of", "alternative", "masculine"), alt_of=[("PIL", None)]))
PIL_ACRONYM = entry(
    "PIL",
    "noun",
    sense(
        "initialism of prodotto interno lordo (“GDP”)",
        ("abbreviation", "alt-of", "initialism", "masculine"),
        alt_of=[("prodotto interno lordo", "GDP")],
    ),
)
PRODOTTO_INTERNO_LORDO = entry("prodotto interno lordo", "noun", sense("gross domestic product", ("masculine",)))
NATALE_ADJ = entry("natale", "adj", sense("native (of one's birth)", ("by-personal-gender", "feminine", "masculine")))
NATALE_NOUN = entry("natale", "noun", sense("birthday", ("masculine",)), sense("birth", ("masculine",)))
NATALE_NAME = entry("Natale", "name", sense("Christmas", ("masculine",)), sense("a male given name, feminine equivalent Natalia", ("masculine",)))
IO = entry("io", "pron", sense("I (the first-person singular nominative pronoun)", ("first-person", "personal")))
IO_NOUN = entry("io", "noun", sense("ego, self", ("invariable", "masculine")))
IO_MOON = entry("Io", "name", sense("Io", ("Greek", "feminine")), sense("85 Io", ("feminine",)))
MENO = entry("meno", "adv", sense("less"))
MENO_DET = entry("meno", "det", sense("less, fewer"))
MENO_RIVER = entry("Meno", "name", sense("Main (German river)"))
PERFINO = entry("perfino", "adv", sense("synonym of persino"))
AVERE = entry(
    "avere",
    "verb",
    sense("to have", ("transitive",)),
    sense("to have (done something); See Category:Italian transitive verbs and Category:Italian reflexive verbs.", ("auxiliary",)),
)
LAVORO = entry("lavoro", "noun", sense("work (all senses)", ("masculine",)), sense("job, task", ("masculine",)))
MERCATO = entry("mercato", "noun", sense("market (in all senses)", ("masculine",)))
MIO = entry("mio", "det", sense("my; of mine (used attributively)"))
MIO_HONORIFIC = entry("Mio", "pron", sense("Same as entry above.", ("masculine",)))
COLLA_CONTRACTION = entry(
    "colla",
    "contraction",
    sense("contraction of con + la; with the", ("abbreviation", "alt-of", "contraction", "dated"), alt_of=[("con", "+ la; with the")]),
)
COLLA_NOUN = entry("colla", "noun", sense("glue or similar sticky material", ("feminine",)))
COGLI_CONTRACTION = entry(
    "cogli",
    "contraction",
    sense("contraction of con gli; with the", ("abbreviation", "alt-of", "contraction", "dated"), alt_of=[("con gli", "with the")]),
)
COGLI_VERB = entry(
    "cogli",
    "verb",
    sense("second-person singular present indicative", ("form-of", "indicative", "present", "second-person", "singular"), form_of=[("cogliere", None)]),
)
COGLIERE = entry("cogliere", "verb", sense("to pick, to gather"), sense("to grasp, to seize, to take"))
CUORI = entry(
    "cuori",
    "noun",
    sense("plural of cuore (when talking about the organ)", ("form-of", "masculine", "plural"), form_of=[("cuore", "when talking about the organ")]),
    sense("hearts (suit of playing cards)", ("masculine", "plural")),
)
SOMMI = entry(
    "sommi",
    "contraction",
    sense(
        "contraction of sono + mi: mi sono",
        ("Tuscany", "abbreviation", "alt-of", "contraction", "obsolete"),
        alt_of=[("sono", "+ mi: mi sono")],
    ),
)
ANCH_IO = entry(
    "anch'io",
    "contraction",
    sense("contraction of anche + io; me too (subject)", ("abbreviation", "alt-of", "contraction"), alt_of=[("anche", "+ io; me too (subject)")]),
)
UN_PO = entry("un po'", "adv", sense("a bit, a little"))
D_ACCORDO = entry("d'accordo", "adv", sense("agreed, OK, all right"))
C_E = entry("c'è", "verb", sense("third-person singular present indicative of esserci", ("form-of",), form_of=[("esserci", None)]))
CANCRO_NAME = entry("Cancro", "name", sense("Cancer (constellation)"))
CANCRO = entry("cancro", "noun", sense("cancer (disease)"))
USA = {
    "word": "USA",
    "pos": "name",
    "synonyms": [{"word": "America"}, {"word": "Stati Uniti"}],
    "senses": [sense("the United States of America", ("plural",))],
}
AMERICA = entry("America", "name", sense("America, the Americas"))
STATI_UNITI = entry("Stati Uniti", "name", sense("United States (a country in North America)"))
A_LETTER = entry("a", "character", sense("The first letter of the Italian alphabet."))
SECTION = [
    DEL, DELLA, ED, AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, GLI_ARTICLE, GLI_PRONOUN, MI_PRONOUN, MI_NOTE, MI_LETTER,
    CHI, CHI_LETTER, ETA, QUANTI_DET, QUANTI_PRON, QUANTO, PIL, PIL_ACRONYM, PRODOTTO_INTERNO_LORDO, NATALE_ADJ,
    NATALE_NOUN, NATALE_NAME, IO, IO_NOUN, IO_MOON, MENO, MENO_DET, MENO_RIVER, PERFINO, AVERE, LAVORO, MERCATO, MIO,
    MIO_HONORIFIC, COLLA_CONTRACTION, COLLA_NOUN, COGLI_CONTRACTION, COGLI_VERB, COGLIERE, CUORI, SOMMI, ANCH_IO,
    UN_PO, D_ACCORDO, C_E, CANCRO_NAME, CANCRO, USA, AMERICA, STATI_UNITI, A_LETTER,
]  # fmt: skip

# — Recorded entries of the Italian Wiktionary's English translations —

TRANSLATED = [
    {"word": "perfino", "pos": "adv", "translations": [{"word": "even"}]},
    {"word": "ex", "pos": "adj", "translations": [{"word": "ex"}, {"word": "former"}]},
    {"word": "ex", "pos": "noun", "translations": [{"word": "ex"}, {"word": "former"}]},
    {"word": "meccanica quantica", "pos": "noun", "translations": [{"word": "quantum mechanics"}]},
    {"word": "all’inizio", "pos": "adv", "translations": [{"word": "at the beginning"}]},
    {"word": "b", "pos": "noun", "translations": [{"word": "b"}]},
    {"word": "Roma", "pos": "name", "translations": [{"word": "Rome"}]},
]
# The English Wiktionary's English entries' Italian translations, which it-en never reads backwards (D2):
# made up, an English entry « etc. » listing `etc`.
INVERTED = [{"word": "etc.", "pos": "adv", "translations": [{"word": "etc"}]}]

# The lemmas the fixture ranks: the forms that are their own lemma, `gli` among them (made up: Italian's
# tables read it as a form of `il`), and the acronym rows, which are their own forms with no rank.
RANKED = [
    "del", "della", "ed", "ad", "gli", "mi", "chi", "eta", "quanti", "quanto", "pil", "natale", "io", "meno", "perfino",
    "avere", "lavoro", "mercato", "mio", "colla", "cogli", "cogliere", "cuori", "sommi", "cancro", "ex", "etc",
]  # fmt: skip


# — refine-lingua-it-en-glosses (change 23): recorded entries as the section reaches `read_as_italian`
# (after the English edition's `read_as_meanings`), each cut to the fields the rules read; an entry
# whose senses are cut says so. —


def nested(parent, child, tags=()):
    """A sense the section nests under another: its glosses, the parent's then its own."""
    out = {"glosses": [parent, child]}
    if tags:
        out["tags"] = list(tags)
    return out


SU = [
    entry("su", "adv", sense("up, upstairs")),
    entry("su", "intj", sense("come on!")),
    entry("su", "prep", sense("on, upon, onto, on top"), sense("over"), sense("about, on"), sense("above")),  # cut
]
PIACERE = [
    entry("piacere", "verb", sense("to be pleasing to", ("intransitive",)), sense("to be fond of", ("intransitive",))),  # cut
    entry("piacere", "noun", sense("pleasure, enjoyment", ("masculine",)), sense("delight", ("masculine",))),  # cut
    entry("piacere", "intj", sense("pleased to meet you")),
]
POLIZIA = [entry("polizia", "noun", sense("polity", ("archaic", "feminine")), sense("police", ("feminine",)))]
_RARE = ("by-personal-gender", "feminine", "masculine", "rare")
GRANDE = [
    entry(
        "grande",
        "adj",
        nested("of greater physical dimensions or numerosity", "big, large (in size or quantity)", _RARE),
        nested("of greater physical dimensions or numerosity", "tall", _RARE),
        sense("great (importance)", _RARE),
    ),  # cut
    entry("grande", "adv", sense("really (intensifier)")),
    entry("grande", "intj", sense("great!")),
]
TESTO = [
    entry("testo", "verb", sense("synonym of tessuto, past participle of tessere", ("archaic", "participle"))),
    entry(
        "testo",
        "noun",
        sense("an earthenware lid", ("masculine", "uncommon")),
        sense("an earthenware baking tray", ("masculine",)),
        sense("earthenware", ("archaic", "masculine")),
    ),  # cut
    entry("testo", "noun", sense("text (writing)", ("masculine",)), sense("wording", ("masculine",)), sense("textbook", ("masculine",))),  # cut
    entry("testo", "noun", sense("alternative form of test", ("alt-of", "alternative", "masculine", "rare"), alt_of=[("test", None)])),
]
COME = [
    entry("come", "adv", sense("how"), sense("as, like"), sense("such as")),
    entry("come", "conj", sense("as soon as")),
]
_OLD_POETIC = ("archaic", "feminine", "poetic")
GENTE = [
    entry(
        "gente",
        "adj",
        sense("noble", ("archaic", "by-personal-gender", "feminine", "masculine", "poetic")),
        sense("elegant, refined, gentle", ("archaic", "broadly", "by-personal-gender", "feminine", "masculine", "poetic")),
    ),
    entry(
        "gente",
        "noun",
        sense("gens (legally defined unit of Roman society)", ("Ancient-Rome", "archaic", "feminine", "historical", "poetic")),
        sense("lineage", _OLD_POETIC),
        sense("people, guys, folks (body of human beings)", ("archaic", "collective", "feminine", "poetic")),
    ),  # cut
]
PARLAMENTO = [
    entry(
        "parlamento",
        "noun",
        sense("conversation, talk", ("archaic", "literary", "masculine")),
        sense("public assembly or meeting", ("masculine",)),
        nested("public assembly or meeting", "parliament", ("masculine",)),
        sense("the collective members of a parliament", ("broadly", "masculine")),
    ),  # cut
    entry(
        "parlamento",
        "verb",
        sense("first-person singular present indicative of parlamentare", ("first-person", "form-of"), form_of=[("parlamentare", None)]),
    ),
]
A_VANVERA = entry(
    "a vanvera",
    "adv",
    sense("at random, without any effort to achieve particular outcome", ("dated",)),
    sense("without thinking"),
)
CHE = [
    entry("che", "pron", sense("what; which", ("interrogative",)), sense("who, whom, which, that", ("relative",))),
    entry("che", "conj", sense("that"), sense("than")),  # cut
    entry("che", "conj", sense("proscribed spelling of ché")),
]
DI = entry(
    "di",
    "prep",
    sense("used to indicate possession, after the thing owned and before the owner; of; ’s"),
    sense("from"),
    sense("by, of, ’s"),
    sense("than"),
    sense("used in superlative forms; in, of"),
    sense("about, on, concerning"),
    sense("expresses composition; of, made of, in or more often omitted"),
    sense("to or omitted", ("with-infinitive",)),
    sense("used with the definite article in partitive constructions; some"),
)  # cut
DE = [
    entry("de", "prep", sense("alternative form of di", ("alt-of", "alternative", "error-lua-exec"), alt_of=[("di", None)])),
    entry("de", "prep", sense("Marche, Milan, Rome, and Venice form of di")),
]
IL = [
    entry("il", "article", sense("the", ("masculine", "singular"))),
    entry(
        "il",
        "pron",
        sense("Tuscany form of lo", ("Tuscany", "alt-of", "masculine", "obsolete", "singular"), alt_of=[("lo", None)]),
        sense("Tuscany form of gli", ("Tuscany", "alt-of", "masculine", "obsolete", "singular"), alt_of=[("gli", None)]),
    ),
]
EL = [
    entry("el", "article", sense("archaic, regional, and Rome form of il", ("masculine", "singular"))),
    entry("el", "pron", sense("he", ("abbreviation", "apocopic", "literary", "masculine", "rare"))),
    entry(
        "el",
        "contraction",
        sense("contraction of e + il: and the", ("abbreviation", "alt-of", "contraction", "obsolete"), alt_of=[("e", "+ il: and the")]),
    ),
]
IN = entry("in", "prep", sense("in"), sense("to"), sense("into"), sense("by"), sense("on"), sense("marker of way or manner"))
_NE_UNSAID = ", sometimes not translated in English"
NE = [
    entry(
        "NE",
        "noun",
        sense("abbreviation of nordest; northeast", ("abbreviation", "alt-of", "masculine"), alt_of=[("nordest", "northeast")]),
    ),
    entry("ne", "adv", sense("from there")),
    entry(
        "ne",
        "pron",
        sense("from this; from that; from these; from those" + _NE_UNSAID),
        sense("about this; about that; about these; about those"),
        sense("of this; of that; of these; of those" + _NE_UNSAID),
        sense("of them (sometimes not translated in English)"),
        sense("for this; of that; of these; of those" + _NE_UNSAID),
        sense(
            "intensive particle, used in forms of certain verbs suffixed with -ne, where it indicates a particular way of "
            "carrying out the verb's action"
        ),
        sense("only used in forms of the verb of volerne (“to hold a grudge”)"),
        sense("only used in forms of the verb of andarne (“to be at stake”)"),
    ),
    entry("ne", "prep", sense("alternative form of in", ("alt-of", "alternative"), alt_of=[("in", None)])),
    entry("ne", "pron", sense("synonym of ci: us, to us", ("Tuscany", "obsolete", "poetic"))),
]
GLI_FEMININE = entry("gli", "pron", sense("her, to her", ("dative", "feminine", "nonstandard")))
GLI_PLURAL = entry("gli", "pron", sense("them, to them", ("dative", "feminine", "masculine", "plural")))
LE_PRONOUN = entry("le", "pron", sense("them (third-person plural feminine)", ("accusative", "feminine", "plural")))
JE = entry(
    "je",
    "pron",
    sense(
        "alternative form of gli (3ʳᵈ-person masculine singular dative pronoun; 3ʳᵈ-person masculine/feminine plural dative pronoun)",
        ("alt-of", "alternative"),
        alt_of=[("gli", "3ʳᵈ-person masculine singular dative pronoun; 3ʳᵈ-person masculine/feminine plural dative pronoun")],
    ),
    sense(
        "alternative form of le (3ʳᵈ-person feminine singular dative pronoun)",
        ("alt-of", "alternative"),
        alt_of=[("le", "3ʳᵈ-person feminine singular dative pronoun")],
    ),
)
LO = [
    entry(
        "lo",
        "article",
        sense(
            "the form of il that is used before the so-called impure consonants, that is, s+consonant (impure s), gn, pn, ps, "
            "x, y, or z, and before i+vocal, bd, cn, ct, dm, ft, mn, pt, tm and also ts; before a vowel it becomes l'; the",
            ("masculine", "singular"),
        ),
    ),
    entry(
        "lo",
        "pron",
        sense("him", ("accusative", "masculine", "singular")),
        sense("it, this or that thing", ("accusative", "masculine", "singular")),
    ),
]
APRILE = [
    entry("aprile", "noun", sense("April", ("masculine", "uncountable"))),
    entry("aprile", "verb", sense("compound of apri, the second-person singular imperative form of aprire, with le", ("compound-of",))),
]
VENDESI = [
    entry(
        "vendesi",
        "verb",
        sense(
            "compound of vende, the third-person singular present indicative form of vendere, with si. it gets sold, for sale",
            ("compound-of",),
        ),
    ),
    entry("vendesi", "noun", sense("for-sale sign or announcement", ("informal", "invariable", "masculine"))),
]
THE = entry("the", "noun", sense("misspelling of tè", ("alt-of", "misspelling"), alt_of=[("tè", None)]))
TE_TEA = entry("tè", "noun", sense("tea (Camellia sinensis)", ("invariable", "masculine")))  # cut
SI = entry(
    "si",
    "pron",
    sense("oneself, himself, herself, itself, themselves", ("pronoun", "reflexive")),
    sense("each other, one another", ("pronoun", "reciprocal")),
    sense("one, you, we, they, people", ("indefinite",)),
    sense("used to form the passive voice of a verb; it"),
    sense("reflexive and reciprocal first person pronoun, where Standard Italian uses ci", ("dialectal",)),
)
SE = [
    entry("se", "conj", sense("if", ("error-lua-exec",)), sense("whether", ("error-lua-exec",)), sense("if only", ("error-lua-exec",))),
    entry("se", "pron", sense("alternative form of si", ("alt-of", "alternative"), alt_of=[("si", None)])),
]
TE = [
    entry(
        "te",
        "pron",
        sense("you, thee", ("disjunctive", "emphatic")),
        sense("you, thou", ("colloquial", "error-lua-exec", "regional", "subjective")),
    ),
    entry("te", "pron", sense("alternative form of ti", ("alt-of", "alternative", "clitic"), alt_of=[("ti", None)])),
]
TI = entry("ti", "pron", sense("you", ("accusative", "dative")))  # cut
CI = entry(
    "ci",
    "pron",
    sense("us", ("error-lua-exec",)),
    sense("ourselves; each other", ("error-lua-exec", "pronoun", "reflexive")),
    sense("to us", ("error-lua-exec",)),
    sense("it, to it", ("error-lua-exec",)),
)  # cut
CE = [
    entry("ce", "pron", sense("alternative form of ci (“us”)", ("alt-of", "alternative"), alt_of=[("ci", "us")])),
    entry(
        "ce",
        "adv",
        sense("alternative form of ci", ("alt-of", "alternative"), alt_of=[("ci", None)]),
        sense("reinforcing particle, “do”"),
    ),
]
EO = entry("eo", "pron", sense("Old Italian form of io", ("first-person", "personal")))
ITRIA = entry("itria", "noun", sense("# alternative form of lontra", ("dialectal", "feminine")))
LONTRA = entry("lontra", "noun", sense("otter", ("feminine",)))
MAR = entry("mar", "noun", sense("(“sea”) (used in poetry and in names of some seas)", ("abbreviation", "apocopic", "masculine")))
MINIMO = [
    entry(
        "minimo",
        "adj",
        nested("superlative degree of piccolo", "minimum; (the) least, (the) smallest, (the) slightest, (the) lowest", ("relational",)),
        nested("superlative degree of piccolo", "very small, very little, tiny, very low, minimum"),
    ),
    entry("minimo", "noun", sense("minimum, the least", ("masculine",)), sense("idle speed (of an engine)", ("masculine",))),
]
MINORE = [
    entry(
        "minore",
        "adj",
        nested("comparative degree of piccolo", "less"),
        nested("comparative degree of piccolo", "shorter"),
        nested("superlative degree of piccolo", "least"),
    ),  # cut
    entry("minore", "noun", sense("younger", ("by-personal-gender", "feminine", "masculine"))),  # cut
]
PIU = [
    entry(
        "più",
        "adv",
        sense("comparative degree of molto; more", ("comparative", "form-of"), form_of=[("molto", "more")]),
        sense("anymore, any longer"),
    ),
    entry("più", "adj", sense("more", ("invariable",))),
    entry("più", "prep", sense("plus")),
]
AUTO = [
    entry(
        "auto",
        "noun",
        sense("clipping of automobile: car", ("abbreviation", "alt-of", "clipping", "feminine", "invariable"), alt_of=[("automobile", "car")]),
    ),
    entry(
        "auto",
        "noun",
        sense("clipping of autobus: bus", ("abbreviation", "alt-of", "clipping", "masculine", "regional"), alt_of=[("autobus", "bus")]),
    ),
    entry(
        "auto",
        "noun",
        sense("auto (genre of dramatic literature)", ("historical", "invariable", "masculine", "uncountable")),
        sense("an auto literary work", ("historical", "invariable", "masculine")),
    ),
]
GUAI = [
    entry("guai", "intj", sense("woe!")),
    entry("guai", "noun", sense("plural of guaio (“trouble”)", ("form-of", "masculine", "plural"), form_of=[("guaio", "trouble")])),
]
FUSA = [
    entry(
        "fusa",
        "noun",
        sense("plural of fuso (“spindle”)", ("archaic", "feminine", "form-of", "literary", "plural"), form_of=[("fuso", "spindle")]),
        sense("purr (sound made by a cat)", ("feminine", "plural", "plural-only")),
    ),
]
SIGNORA = [
    entry(
        "signora",
        "noun",
        sense("female equivalent of signore", ("feminine", "form-of"), form_of=[("signore", None)]),
        sense("lady", ("feminine",)),
        sense("a title of address for women; madam (ladies is used for the plural)", ("feminine",)),
    ),  # cut
]
SIGNORE = entry("signore", "noun", sense("lord (ruler)", ("masculine",)), sense("gentleman", ("masculine",)))  # cut
_PERSON = ("by-personal-gender", "feminine", "masculine")
MUSICA = [entry("musica", "noun", sense("music", ("feminine",))), entry("Musica", "name", sense("a surname", _PERSON))]
MATTARELLA = entry("Mattarella", "name", sense("a surname", _PERSON))
GARIBALDI = entry(
    "Garibaldi", "name", sense("a surname", _PERSON), sense("Anita Garibaldi, Italian patriot and wife of Giuseppe Garibaldi", _PERSON)
)
CLINTON = [
    entry("Clinton", "name", sense("a surname in English", _PERSON)),
    entry("Clinton", "name", sense("any of numerous placenames; see Clinton", ("feminine",))),
    entry("clinton", "noun", sense("alternative form of clinto", ("alt-of", "alternative", "invariable", "masculine"), alt_of=[("clinto", None)])),
]
CLINTO = entry("clinto", "noun", sense("a red wine from Veneto", ("capitalized", "masculine", "sometimes")))
MOLINO = [
    entry("molino", "noun", sense("alternative form of mulino", ("alt-of", "alternative", "masculine"), alt_of=[("mulino", None)])),
    entry("Molino", "name", sense("a surname", _PERSON)),
    entry("Molino", "name", sense("a frazione in Messina, Messina, Sicily, Italy", ("feminine",))),
]  # cut
MULINO = entry("mulino", "noun", sense("mill", ("masculine",)))
DELFINO = [
    entry("delfino", "noun", sense("dolphin", ("masculine",))),
    entry("delfino", "adj", sense("hunchbacked")),
    entry("delfino", "noun", sense("dauphin", ("masculine",)), sense("probable successor", ("masculine",))),
    entry("Delfino", "name", sense("a male given name", ("masculine",))),
]
ALICE = [
    entry("Alice", "name", sense("a female given name, equivalent to English Alice", ("feminine",))),
    entry("alice", "noun", sense("anchovy", ("feminine",))),
]
NOTO = [
    entry("noto", "adj", sense("of common knowledge"), sense("well-known, known"), sense("famous, notorious")),
    entry("noto", "noun", sense("the known", ("masculine",))),
    entry("Noto", "name", sense("a town in Syracuse, Sicily, Italy", ("feminine",))),
    entry("Noto", "name", sense("a habitational surname", _PERSON)),
]
CERVIA = [
    entry("Cervia", "name", sense("a town in Ravenna, Emilia-Romagna, Italy", ("feminine",))),
    entry("cervia", "noun", sense("hind (female deer)", ("feminine",))),
]
ANDRIA = [
    entry("Andria", "name", sense("a municipality of Barletta-Andria-Trani province, Apulia, Italy", ("feminine",))),
    entry("andria", "noun", sense("any salamander of the Andrias taxonomic genus", ("invariable", "masculine"))),
]
RUSSO = [
    entry("russo", "adj", sense("Russian")),
    entry("russo", "noun", sense("the Russian language", ("masculine", "uncountable"))),
    entry("Russo", "name", sense("a surname; the second most popular surname in Italy", _PERSON)),
    entry("Russo", "name", sense("a frazione in Rizziconi, Metropolitan City of Reggio Calabria, Calabria, Italy", ("feminine",))),
]  # cut
FERMO = [
    entry("fermo", "adj", sense("still"), sense("firm")),
    entry("Fermo", "name", sense("a town and province of Marche, Italy", ("feminine",)), sense("a male given name", ("feminine",))),
]  # cut
MASSA = [
    entry("massa", "noun", sense("mass (all senses)", ("feminine",)), sense("crowd", ("feminine",))),
    entry("Massa", "name", sense("Massa (a town in Tuscany, Italy)", ("feminine",))),
    entry("Massa", "name", sense("a surname", _PERSON)),
]
PINUCCIO = entry(
    "Pinuccio",
    "name",
    sense(
        "endearing form of Pino: a diminutive of the male given name Giuseppe, feminine equivalent Pinuccia",
        ("endearing", "form-of", "masculine"),
        form_of=[("Pino", "a diminutive of the male given name Giuseppe, feminine equivalent Pinuccia")],
    ),
)
PINO = [
    entry("pino", "noun", sense("pine, pine tree (tree of the genus Pinus)", ("masculine",))),
    entry("Pino", "name", sense("a diminutive of the male given name Giuseppe, feminine equivalent Pina", ("masculine",))),
]  # cut
OBBIETTIVO = [
    entry("obbiettivo", "adj", sense("alternative form of obiettivo", ("alt-of", "alternative"), alt_of=[("obiettivo", None)])),
    entry("Obbiettivo", "name", sense("a surname transferred from the nickname", _PERSON)),
]  # cut
OBIETTIVO = entry("obiettivo", "adj", sense("objective"))
PREGO = [
    entry("prego", "noun", sense("prayer", ("masculine",))),
    entry("prego", "intj", sense("you're welcome, don't mention it etc."), sense("yes, of course")),
]  # cut
AMICA = [
    entry("amica", "noun", sense("female equivalent of amico", ("feminine", "form-of"), form_of=[("amico", None)]), sense("girlfriend", ("feminine",))),
    entry("amica", "adj", sense("feminine singular of amico", ("feminine", "form-of", "singular"), form_of=[("amico", None)])),
]  # cut
PARMIGIANO = [
    entry("parmigiano", "adj", sense("of, from or relating to Parma", ("archaic",))),
    entry("parmigiano", "noun", sense("native or inhabitant of Parma (usually male)", ("archaic", "masculine"))),
    entry("parmigiano", "noun", sense("the dialect of Emilian spoken in Parma", ("masculine", "uncountable"))),
    entry("parmigiano", "noun", sense("parmesan cheese", ("also", "masculine", "uncountable"))),
]
ALL_ANTICA = {"word": "all'antica", "pos": "phrase", "translations": [{"word": "ancient"}, {"word": "anshent"}, {"word": "antique"}]}
# UD Italian-MarkIT's counts of the words above, as `reduce_italian_treebank.markit_pos_counts` reads its
# training and development sections at it-en's pin (e210079d).
MARKIT_COUNTS = collections.Counter(
    {
        ("su", "ADP"): 92, ("su", "ADV"): 1,
        ("piacere", "NOUN"): 19, ("piacere", "VERB"): 6,
        ("come", "ADP"): 97, ("come", "SCONJ"): 39, ("come", "ADV"): 16,
        ("ne", "PRON"): 49, ("ne", "ADP"): 3,
    }
)  # fmt: skip
# Made up: a sentence of MarkIT's shape, what the reducer reads from its work folder.
MARKIT_SECTION = "# sent_id = made-up\n# text = Su.\n1\tSu\tsu\tADP\tE\t_\t0\troot\t_\t_\n\n"


class Fixture(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def jsonl(self, *entries, name="section.jsonl"):
        path = self.dir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        return str(path)

    def studied(self, ranked=RANKED, extra_forms=()):
        studied = self.dir / "it"
        studied.mkdir(exist_ok=True)
        forms = {w: w for w in ranked} | dict(extra_forms) | acronyms.acronym_forms()
        (studied / "forms.tsv").write_text("".join(f"{f}\t{l}\n" for f, l in sorted(forms.items())), encoding="utf-8")
        (studied / "freq.tsv").write_text("".join(f"{w}\t{i}\n" for i, w in enumerate(ranked, start=1)), encoding="utf-8")
        return studied

    def read(self, *entries, rules=italian.RULES, **kwargs):
        src = self.jsonl(*entries)
        dst, stats = italian.read_as_italian(src, str(self.dir / "read.jsonl"), rules=rules, **kwargs)
        return [json.loads(line) for line in Path(dst).read_text(encoding="utf-8").splitlines()], stats

    def glosses(self, *entries, lemmas, rules=italian.RULES, **kwargs):
        src = self.jsonl(*entries)
        dst, _ = italian.read_as_italian(src, str(self.dir / "read.jsonl"), rules=rules, **kwargs)
        return common.reduce_gloss(dst, set(lemmas), **common.WORD_GLOSS, studied=it_en.IT, edition=english.EN)

    def reduce(self, section=SECTION, translated=TRANSLATED, ranked=RANKED, first_run=None):
        work = self.dir / "work"
        work.mkdir(exist_ok=True)
        studied = self.studied(ranked)
        self.jsonl(*section, name="work/kaikki-Italian.jsonl")
        self.jsonl(*translated, name="work/kaikki-it-traductions-en.jsonl")
        self.jsonl(*INVERTED, name="work/kaikki-en-traductions-it.jsonl")
        for name in it_en.MARKIT:
            (work / name).write_text(MARKIT_SECTION, encoding="utf-8")
        argv = [
            "reduce-it-en.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-25", "--pack-version", "test"),
        ]
        # The reviewed table names the section's real words, which this small section has only where a
        # test writes them (refine-lingua-it-en-glosses D2c).
        with (
            mock.patch.dict(italian.FIRST_RUN, first_run or {}, clear=True),
            mock.patch.object(sys, "argv", argv),
            contextlib.redirect_stderr(io.StringIO()) as err,
        ):
            it_en.main()
        self.report = err.getvalue()

        def table(name):
            rows = {}
            for line in (work / name).read_text(encoding="utf-8").splitlines():
                key, _, value = line.partition("\t")
                rows[key] = value
            return rows

        return work, table("gloss.tsv"), table("senses.tsv"), table("mwe.tsv")


class CarriedMeanings(Fixture):
    """D4.1: the meaning a pointer carries, by a reviewed list of wordings."""

    def test_the_di_family_reads_its_contraction(self):
        self.assertEqual(italian.carried_meaning(DEL["senses"][0], "contraction"), "some, of the, from the")
        self.assertEqual(italian.carried_meaning(DELLA["senses"][0], "contraction"), "of the, from the")

    def test_the_di_family_s_contraction_is_read_wherever_it_stands(self):
        # Recorded: `dei`'s other entries are the plural of *dio* and two alternative forms, pointers that
        # hold it out of the narrowing; its contraction is the `di` family's, an Italian shape read beside
        # them, before the row's own « some ».
        self.assertEqual(italian.carried_meaning(DEI["senses"][0], "contraction", alone=False), "of the, from the")
        self.assertEqual(italian.carried_meaning(DELLA["senses"][0], "contraction", alone=False), "of the, from the")
        self.assertIsNone(italian.carried_meaning(COLLA_CONTRACTION["senses"][0], "contraction", alone=False), "a con contraction")
        glosses = self.glosses(DEI, DEI_NOUN, DEI_VERB, lemmas={"dei"})
        self.assertEqual(glosses["dei"], "of the, from the; some")
        without = self.glosses(DEI, DEI_NOUN, DEI_VERB, lemmas={"dei"}, rules=())
        self.assertEqual(without["dei"], "some")

    def test_italian_s_three_shapes(self):
        self.assertEqual(italian.carried_meaning(ED["senses"][0], "conj"), "and")
        self.assertEqual(italian.carried_meaning(AD["senses"][0], "prep"), "to, at, in")
        self.assertEqual(italian.carried_meaning(GLI_ARTICLE["senses"][0], "article"), "the")
        accusative, dative, ethical = MI_PRONOUN["senses"]
        self.assertEqual(italian.carried_meaning(accusative, "pron"), "me")
        self.assertEqual(italian.carried_meaning(dative, "pron"), "(to) me")
        self.assertIsNone(italian.carried_meaning(ethical, "pron"), "no pointer")
        # A case form is a pronoun's alone.
        self.assertIsNone(italian.carried_meaning(accusative, "noun"))
        # The shapes are read beside the word's other senses: they are its function word's own reading.
        self.assertEqual(italian.carried_meaning(ED["senses"][0], "conj", alone=False), "and")

    def test_never_synonym_nor_every_wording(self):
        # `quanti`'s synonym would read « who »; an abbreviation would read `giu` « June ».
        self.assertIsNone(italian.carried_meaning(QUANTI_PRON["senses"][0], "pron"), "untagged and unlisted")
        giu = sense("abbreviation of giugno (“June”)", ("abbreviation", "alt-of"), alt_of=[("giugno", "June")])
        self.assertIsNone(italian.carried_meaning(giu, "noun"))
        self.assertIsNone(italian.carried_meaning(QUANTI_DET["senses"][0], "det"), "« masculine plural of » carries nothing")

    def test_a_carrying_wording_only_where_the_word_holds_nothing_else(self):
        # The implementation's narrowing: a contraction or a plural never opens or joins a row its own
        # senses gloss, nor replaces a verb form's lent senses.
        self.assertEqual(italian.carried_meaning(COLLA_CONTRACTION["senses"][0], "contraction"), "with the")
        self.assertIsNone(italian.carried_meaning(COLLA_CONTRACTION["senses"][0], "contraction", alone=False))
        glosses = self.glosses(COLLA_CONTRACTION, COLLA_NOUN, COGLI_CONTRACTION, COGLI_VERB, COGLIERE, CUORI, lemmas={"colla", "cogli", "cuori"})
        self.assertEqual(glosses["colla"], "glue or similar sticky material")
        self.assertEqual(glosses["cogli"], "to pick, to gather; to grasp, to seize, to take")
        self.assertEqual(glosses["cuori"], "hearts (suit of playing cards)")

    def test_a_meaning_from_the_gloss_never_from_extra_s_notes(self):
        # Recorded: kaikki's `extra` drops the quotation marks and keeps a note as readily as a meaning.
        cuori = CUORI["senses"][0]
        self.assertIsNone(italian.carried_meaning(cuori, "noun"))
        guai = sense("plural of guaio (“trouble”)", ("form-of", "masculine", "plural"), form_of=[("guaio", "trouble")])
        self.assertEqual(italian.carried_meaning(guai, "noun"), "trouble")
        domi = sense("plural of domo (“dome, vault; sky”)", ("form-of", "literary", "plural"), form_of=[("domo", "dome, vault; sky")])
        self.assertEqual(italian.carried_meaning(domi, "noun"), "dome, vault; sky", "a semicolon inside the quotation is no separator")
        stabile = sense("superlative degree of stabile (“stable”): most stable", ("form-of", "superlative"), form_of=[("stabile", "(“stable”): most stable")])
        self.assertEqual(italian.carried_meaning(stabile, "adj"), "most stable")

    def test_a_meaning_made_of_the_pointer_s_own_words_is_none(self):
        self.assertIsNone(italian.carried_meaning(SOMMI["senses"][0], "contraction"), "« mi sono » is Italian")

    def test_spec_scenario_a_meaning_a_pointer_carries(self):
        glosses = self.glosses(DEL, DELLA, ED, AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"del", "della", "ed", "ad"})
        self.assertEqual(glosses["della"], "of the, from the")
        self.assertEqual(glosses["del"], "some, of the, from the")
        self.assertEqual(glosses["ed"], "and")
        self.assertEqual(glosses["ad"], "to, at, in")


class AcronymsAndNames(Fixture):
    """D4.2: acronyms off common words, names off function words."""

    def test_spec_scenario_an_acronym_off_a_common_word(self):
        # Without the rule, `ad` borrows « chief executive officer, CEO » through `AD`'s pointer.
        without = self.glosses(AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"ad"}, rules=())
        self.assertEqual(without["ad"], "chief executive officer, CEO")
        alone = self.glosses(AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"ad"}, rules=("acronyms and names",))
        self.assertNotIn("ad", alone)
        glosses = self.glosses(AD, AD_ACRONYM, AMMINISTRATORE_DELEGATO, lemmas={"ad"})
        self.assertEqual(glosses["ad"], "to, at, in")
        self.assertNotIn("CEO", glosses["ad"])

    def test_an_acronym_its_lowercase_entry_points_at_is_kept(self):
        glosses = self.glosses(PIL, PIL_ACRONYM, PRODOTTO_INTERNO_LORDO, lemmas={"pil"})
        self.assertEqual(glosses["pil"], "gross domestic product")

    def test_spec_scenario_a_name_beside_a_common_word_s_senses(self):
        glosses = self.glosses(NATALE_ADJ, NATALE_NOUN, NATALE_NAME, lemmas={"natale"})
        self.assertIn("Christmas", glosses["natale"].split("; "))

    def test_spec_scenario_a_name_off_a_function_word(self):
        glosses = self.glosses(IO, IO_NOUN, IO_MOON, MENO, MENO_DET, MENO_RIVER, lemmas={"io", "meno"})
        self.assertEqual(glosses["io"], "I (the first-person singular nominative pronoun); ego, self")
        self.assertEqual(glosses["meno"], "less; less, fewer")
        read, stats = self.read(IO, IO_NOUN, IO_MOON, MENO, MENO_DET, MENO_RIVER)
        self.assertEqual([e["word"] for e in read], ["io", "io", "meno", "meno"])
        self.assertEqual(stats["names under a function word"], 2)
        # An adverb's word keeps its name (made up): the rule reads function words alone.
        read, _ = self.read(MENO, MENO_RIVER)
        self.assertEqual([e["word"] for e in read], ["meno", "Meno"])


class LettersAndNotes(Fixture):
    """D4.3 and D4.4: no word glossed with a letter's name, and the page's notes out."""

    def test_spec_scenario_a_greek_letter(self):
        glosses = self.glosses(MI_PRONOUN, MI_NOTE, MI_LETTER, CHI, CHI_LETTER, ETA, lemmas={"mi", "chi", "eta"})
        self.assertEqual(glosses["mi"].split("; ")[0], "me")
        self.assertNotIn("Greek letter", glosses["mi"])
        self.assertNotIn("mu", glosses["mi"].split("; "))
        self.assertEqual(glosses["chi"], "who, whom, whoever")
        self.assertEqual(glosses["eta"], "eta (Greek letter)", "a word that is only the letter keeps it")
        # Recorded: `rho`'s other sense is a proper noun's, the town: no common word's entry holds another
        # meaning, so the letter stays — and the town goes from a word with a common meaning (change 23's D6).
        rho = entry("rho", "noun", sense("rho (Greek letter)", ("feminine", "invariable", "masculine")))
        town = entry("Rho", "name", sense("a town in Milan, Lombardy, Italy"))
        self.assertEqual(self.glosses(rho, town, lemmas={"rho"}, rules=CHANGE_22)["rho"], "rho (Greek letter); a town in Milan, Lombardy, Italy")
        self.assertEqual(self.glosses(rho, town, lemmas={"rho"})["rho"], "rho (Greek letter)")

    def test_a_latin_letter_goes_with_the_english_edition_s_pre_passes(self):
        cut = it_en.without_letters(self.jsonl(A_LETTER, CHI), str(self.dir / "words.jsonl"))
        self.assertEqual([json.loads(line)["word"] for line in Path(cut).read_text().splitlines()], ["chi"])

    def test_the_page_s_notes_are_no_glosses(self):
        glosses = self.glosses(AVERE, LAVORO, MERCATO, MIO, MIO_HONORIFIC, lemmas={"avere", "lavoro", "mercato", "mio"})
        self.assertEqual(glosses["avere"], "to have; to have (done something)")
        self.assertEqual(glosses["lavoro"], "work; job, task")
        self.assertEqual(glosses["mercato"], "market")
        self.assertEqual(glosses["mio"], "my, of mine (used attributively)")
        self.assertIsNone(italian.without_notes("Same as entry above."))
        kept = "table (all senses, except for the furniture element)"
        self.assertEqual(italian.without_notes(kept), kept, "a note that says more stays")

    def test_each_rule_counts_what_it_moved(self):
        _, stats = self.read(*SECTION, rules=CHANGE_22)
        self.assertEqual(stats["names under a function word"], 2)
        self.assertEqual(stats["acronym senses off a common word"], 1, "AD's; PIL's kept")
        self.assertEqual(stats["Greek letters beside a common word"], 2)
        self.assertEqual(stats["senses that are only a note"], 1)
        self.assertEqual(stats["notes taken out"], 3)
        # del, della, ed, ad, gli, mi twice, anch'io.
        self.assertEqual(stats["pointers read as their meaning"], 8)
        self.assertEqual(sum(stats.values()), 2 + 1 + 2 + 1 + 3 + 8 + stats["entries left with no sense"])

    def test_a_line_the_pre_pass_cannot_read_is_written_as_it_is(self):
        src = self.dir / "section.jsonl"
        src.write_text(json.dumps(DEL) + "\nnot json\n[1]\n", encoding="utf-8")
        dst, _ = italian.read_as_italian(str(src), str(self.dir / "read.jsonl"))
        self.assertEqual(Path(dst).read_text(encoding="utf-8").splitlines()[1:], ["not json", "[1]"])


class TheReducer(Fixture):
    """reduce-it-en.py over a work folder holding the two sources."""

    def test_the_native_side_alone_from_the_committed_studied_tables(self):
        work, glosses, senses, mwe = self.reduce()
        self.assertEqual(glosses["della"], "of the, from the")
        self.assertEqual(glosses["del"], "some, of the, from the")
        self.assertEqual(glosses["ed"], "and")
        self.assertEqual(glosses["ad"], "to, at, in")
        self.assertEqual(glosses["gli"], "the; him, to him, it, to it")
        self.assertEqual(senses["gli"], "DET:1\tPRON:1")
        self.assertEqual(glosses["mi"], "me; (to) me; used as ethical dative; the third note, mi; E (musical note or key)")
        self.assertEqual(glosses["quanti"], "how much, how many, however much, however many")
        self.assertEqual(glosses["pil"], "gross domestic product")
        self.assertEqual(glosses["eta"], "eta (Greek letter)")
        self.assertIn("Christmas", glosses["natale"])
        self.assertNotIn("Io", glosses["io"])
        self.assertEqual(glosses["cancro"], "Cancer (constellation); cancer (disease)", "case keeps a name's sense apart")
        # Nothing of the studied side is computed (D1); measures.json stays in the work folder.
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith(("kaikki-", "it_markit-")))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        self.assertNotIn("NATO", glosses, "a row with no expansion or synonym glossed, none")

    def test_spec_scenario_a_gloss_from_the_italian_wiktionary(self):
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["perfino"], "even")
        self.assertEqual(senses["perfino"], "ADV:1")
        measured = json.loads((self.dir / "work" / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(measured["by source"], {"definitions": 23, "direct": 2})
        self.assertEqual(measured["direct"], ["ex", "perfino"])

    def test_a_sense_repeated_word_for_word_is_shown_once(self):
        # Recorded: the Italian Wiktionary lists « ex, former » under the adjective and the noun.
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["ex"], "ex, former")
        self.assertEqual(senses["ex"], "ADJ:1")
        self.assertIn("1 glosses repeating a sense", self.report)

    def test_spec_scenario_no_table_read_backwards(self):
        # `etc` is ranked, the section and the Italian Wiktionary say nothing of it, and the work folder
        # holds the English entries' Italian translations: it-en reads none of them.
        _, glosses, _, _ = self.reduce()
        self.assertNotIn("etc", glosses)
        self.assertEqual(it_en.SECTION, "kaikki-Italian.jsonl")
        self.assertEqual(it_en.DIRECT, "kaikki-it-traductions-en.jsonl")
        self.assertEqual(ps.DUMPS["it-en"], {"en": ("kaikki-Italian.jsonl",), "it": ("kaikki-it-traductions-en.jsonl",)})

    def test_spec_scenario_expressions_from_every_source(self):
        _, _, _, mwe = self.reduce()
        self.assertEqual(mwe["un po'"], "a bit, a little")
        self.assertEqual(mwe["d'accordo"], "agreed, OK, all right")
        self.assertEqual(mwe["meccanica quantica"], "quantum mechanics")
        self.assertEqual(mwe["anch'io"], "me too (subject)", "a split word whose pointer carries its meaning")
        self.assertEqual(mwe["all'inizio"], "at the beginning", "the Italian Wiktionary's split word, its apostrophe straight")
        self.assertEqual(mwe["amministratore delegato"], "chief executive officer, CEO")
        self.assertNotIn("c'è", mwe, "a pointer only: its pieces gloss it")
        self.assertNotIn("stati uniti", mwe, "a proper noun's entry")

    def test_the_acronym_rows_glossed_through_their_synonyms(self):
        _, glosses, senses, _ = self.reduce()
        self.assertEqual(glosses["USA"], "America, the Americas; United States (a country in North America)")
        self.assertEqual(senses["USA"], "PROPN:2")

    def test_the_credits_name_both_sides(self):
        work, *_ = self.reduce()
        notice = (work / "NOTICE").read_text(encoding="utf-8")
        for source in (
            "English Wiktionary (enwiktionary), Italian section",
            "Italian Wiktionary (itwiktionary)",
            "wordfreq",
            "UD Italian-MarkIT",
            "Morph-it!",
            "French Wiktionary (frwiktionary), Italian section",
            "The levels are estimated",
        ):
            self.assertIn(source, notice)
        self.assertNotIn("PUD", notice, "a gate only, never shipped")
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("it", "en"))
        self.assertTrue(manifest["meta"]["levels_estimated"])
        self.assertEqual(manifest["meta"]["pack_version"], "test")
        self.assertEqual(manifest["meta"]["analyzer_version"], it_fr.analyser_version())
        self.assertEqual(manifest["sources"], it_fr.manifest("x")["sources"])

    def test_a_reduction_without_its_sources_stops(self):
        with self.assertRaisesRegex(SystemExit, r"kaikki-it-traductions-en\.jsonl, .*it_markit-ud-dev\.conllu missing"):
            it_en.reduce(str(self.dir), str(self.studied()), 60000, "test", report=lambda line: None)

    def test_the_native_side_step_by_step_is_native_tables(self):
        section = self.jsonl(*SECTION)
        direct = it_en.read_translated(self.jsonl(*TRANSLATED, name="direct.jsonl"), str(self.dir / "direct-read.jsonl"))
        ranks = {w: i for i, w in enumerate(RANKED, start=1)}
        glosses, runs, expressions, steps = it_en.native_side(section, ranks, direct)
        want = common.native_tables(section, ranks, studied=it_en.IT, edition=english.EN, fallbacks=[(direct, list)])
        self.assertEqual((glosses, runs, expressions), want[:3])
        self.assertEqual(len(steps["definitions"]), want[3])
        self.assertEqual(steps["direct"], frozenset({"perfino", "ex"}))


class TheStudiedSide(Fixture):
    """D1: Italian's studied tables as committed, it-fr's."""

    def test_the_acronym_rows_are_no_ranked_lemmas(self):
        studied = self.studied(["casa", "nato"], extra_forms=[("case", "casa")])
        self.assertEqual(it_en.read_studied(str(studied), 60000), {"casa": 1, "nato": 2})
        self.assertEqual(it_en.read_studied(str(studied), 1), {"casa": 1}, "capped by rank")

    def test_tables_that_disagree_are_not_it_fr_s(self):
        studied = self.dir / "it"
        studied.mkdir()
        (studied / "forms.tsv").write_text("casa\tcasa\nUSA\tUSA\n", encoding="utf-8")
        (studied / "freq.tsv").write_text("casa\t1\ncane\t2\n", encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "disagree on the lemmas"):
            it_en.read_studied(str(studied), 60000)
        with self.assertRaisesRegex(SystemExit, "it-en reads Italian's committed tables"):
            it_en.read_studied(str(self.dir / "missing"), 60000)

    def test_the_committed_studied_tables(self):
        committed = it_en.read_studied(os.path.join(_HERE, "tables", "it"), 60000)
        self.assertEqual(len(committed), 60000)
        self.assertNotIn("USA", committed)

    def test_it_en_s_copy_of_italian_is_it_fr_s(self):
        # reduce-it-en.py and reduce_italian_en.py repeat it-fr's description of Italian because a reducer
        # loads no other pair's (D1): the two must not drift.
        self.assertEqual(it_en.IT, it_fr.IT)
        self.assertEqual(italian.TOKEN.pattern, it_fr._TOKEN.pattern)
        self.assertEqual(italian.ELIDED_PIECES, frozenset(it_fr.ELISIONS) | frozenset(it_fr.ELIDED_PREPOSITIONS))
        forms = {"aujourd'hui": "x", "nell'": "x"}
        for word in ("d'accordo", "all'improvviso", "c'è", "senz'altro", "alla", "un po'", "po'", "nell'", "trent'anni"):
            self.assertEqual(italian.split_word(word, forms), it_fr.split_word(word, forms), word)

    def test_a_translation_s_italian_word_is_read_as_italian_is_written(self):
        src = self.jsonl(*TRANSLATED, name="direct.jsonl")
        direct = it_en.read_translated(src, str(self.dir / "direct-read.jsonl"))
        self.assertIn("all'inizio", direct)
        self.assertNotIn("all’inizio", direct)
        self.assertNotIn("b", direct, "a letter's own translation says nothing")
        self.assertNotIn("roma", direct, "a proper noun's entry glosses nothing")

    def test_native_fields_cut_changes_no_table(self):
        full = [dict(e, forms=[{"form": "x", "tags": ["plural"]}], head_templates=[{"name": "it-noun"}]) for e in SECTION]
        cut, dropped = it_en.native_fields(self.jsonl(*full, name="full.jsonl"), str(self.dir / "cut.jsonl"))
        self.assertEqual(dropped, 0)
        ranks = {w: i for i, w in enumerate(RANKED, start=1)}

        def tables(path):
            return common.native_tables(path, ranks, studied=it_en.IT, edition=english.EN, fallbacks=[({}, list)])

        self.assertEqual(tables(cut), tables(self.jsonl(*SECTION)))


# — refine-lingua-it-en-glosses (change 23) —

CHANGE_22 = italian.RULES[: italian.RULES.index("corrections")]


class HeadwordsTogether(Fixture):
    """Task 1.1: a pass of `read_as_italian` reading each headword's entries together."""

    def test_a_headword_s_entries_kept_in_place_when_no_rule_moves_them(self):
        # `Alice` and `alice` are one headword, read together; no rule moves its row (it opens on the name,
        # which keeps its given name), so each entry is written where it stood, `musica`'s between them.
        section = [ALICE[0], MUSICA[0], ALICE[1]]
        src = self.jsonl(*section)
        dst, _ = italian.read_as_italian(src, str(self.dir / "read.jsonl"))
        self.assertEqual(Path(dst).read_bytes(), Path(src).read_bytes())

    def test_the_section_as_read_written_byte_for_byte_when_no_rule_fires(self):
        section = [MUSICA[0], CHI, QUANTO, ALICE[0], ALICE[1], COLLA_NOUN]
        src = self.jsonl(*section)
        alone, _ = italian.read_as_italian(src, str(self.dir / "alone.jsonl"), rules=CHANGE_22)
        together, stats = italian.read_as_italian(src, str(self.dir / "together.jsonl"), counts=MARKIT_COUNTS)
        self.assertEqual(Path(together).read_bytes(), Path(alone).read_bytes())
        self.assertEqual({k: n for k, n in stats.items() if n}, {})

    def test_a_row_the_rules_order_is_written_where_its_first_entry_stood(self):
        src = self.jsonl(SU[0], MUSICA[0], SU[1], SU[2])
        dst, stats = italian.read_as_italian(src, str(self.dir / "read.jsonl"), counts=MARKIT_COUNTS)
        lines = [json.loads(line) for line in Path(dst).read_text(encoding="utf-8").splitlines()]
        self.assertEqual([(e["word"], e["pos"]) for e in lines], [("su", "prep"), ("su", "adv"), ("su", "intj"), ("musica", "noun")])
        self.assertEqual(stats["rows opened by MarkIT's commonest part of speech"], 1)

    def test_each_rule_can_be_read_alone(self):
        # Every rule of change 23 names itself in RULES, after change 22's, and runs alone.
        self.assertEqual(italian.HEADWORD_RULES, italian.RULES[len(CHANGE_22) :])
        self.assertEqual(len(set(italian.RULES)), len(italian.RULES))
        for rule in italian.HEADWORD_RULES:
            with self.subTest(rule=rule):
                glosses = self.glosses(*POLIZIA, *TESTO, *AUTO, lemmas={"polizia", "testo", "auto"}, rules=(*CHANGE_22, rule))
                self.assertEqual(glosses["polizia"].startswith("police"), rule == "outdated first", rule)


class RowOrder(Fixture):
    """Tasks 1.2–1.4 (design D2): MarkIT for function words, then the page's marks, then the reviewed table."""

    def test_spec_scenario_a_function_word_s_commonest_part_of_speech(self):
        glosses = self.glosses(*SU, lemmas={"su"}, counts=MARKIT_COUNTS)
        self.assertTrue(glosses["su"].startswith("on, upon, onto, on top"), glosses["su"])
        self.assertTrue(self.glosses(*SU, lemmas={"su"})["su"].startswith("up, upstairs"), "no counts: the section's order")

    def test_spec_scenario_a_content_word_keeps_the_section_s_order(self):
        glosses = self.glosses(*PIACERE, lemmas={"piacere"}, counts=MARKIT_COUNTS)
        self.assertTrue(glosses["piacere"].startswith("to be pleasing to"), glosses["piacere"])

    def test_a_proper_noun_is_never_moved_first(self):
        # Made up: a treebank reading the word as a proper noun far more often.
        section = [entry("verso", "noun", sense("verse")), entry("Verso", "name", sense("Verso (a made-up place)"))]
        counts = collections.Counter({("verso", "PROPN"): 400, ("verso", "NOUN"): 1})
        self.assertTrue(self.glosses(*section, lemmas={"verso"}, counts=counts)["verso"].startswith("verse"))

    def test_spec_scenario_an_outdated_first_sense(self):
        self.assertEqual(self.glosses(*POLIZIA, lemmas={"polizia"})["polizia"], "police; polity")

    def test_spec_scenario_a_mark_the_rule_does_not_read(self):
        glosses = self.glosses(*GRANDE, lemmas={"grande"})
        self.assertTrue(glosses["grande"].startswith("of greater physical dimensions or numerosity"), glosses["grande"])

    def test_spec_scenario_a_row_whose_every_sense_is_marked(self):
        glosses = self.glosses(*GENTE, lemmas={"gente"}, first_run={})
        self.assertTrue(glosses["gente"].startswith("noble; elegant, refined, gentle"), glosses["gente"])

    def test_spec_scenario_an_expression_s_outdated_sense(self):
        src = self.jsonl(A_VANVERA)
        dst, _ = italian.read_as_italian(src, str(self.dir / "read.jsonl"))
        expressions = common.reduce_expressions(dst, studied=it_en.IT, edition=english.EN)
        self.assertEqual(expressions["a vanvera"], "without thinking; at random, without any effort to achieve particular outcome")

    def test_spec_scenario_a_row_the_reviewed_table_orders(self):
        self.assertTrue(self.glosses(*TESTO, lemmas={"testo"})["testo"].startswith("text (writing); an earthenware lid"))
        self.assertTrue(self.glosses(*TESTO, lemmas={"testo"}, first_run={})["testo"].startswith("an earthenware lid"))

    def test_spec_scenario_a_row_the_table_holds_against_the_treebank(self):
        glosses = self.glosses(*COME, lemmas={"come"}, counts=MARKIT_COUNTS)
        self.assertEqual(glosses["come"], "how; as, like; such as; as soon as")
        moved = self.glosses(*COME, lemmas={"come"}, counts=MARKIT_COUNTS, first_run={})
        self.assertTrue(moved["come"].startswith("as soon as"), "without the row, MarkIT's rule opens it so")

    def test_spec_scenario_a_row_the_table_opens_where_every_sense_is_marked(self):
        glosses = self.glosses(*GENTE, lemmas={"gente"})
        self.assertTrue(glosses["gente"].startswith("people, guys, folks (body of human beings); gens"), glosses["gente"])

    def test_spec_scenario_a_sense_nested_under_another(self):
        glosses = self.glosses(*PARLAMENTO, lemmas={"parlamento"})
        self.assertTrue(glosses["parlamento"].startswith("parliament; public assembly or meeting; conversation, talk"), glosses["parlamento"])

    def test_spec_scenario_a_sense_the_table_no_longer_finds(self):
        gone = {"testo": ("lowercase", "noun", "a text no longer written", "made up")}
        reviewed = italian.Reviewed()
        self.glosses(*TESTO, lemmas={"testo"}, first_run=gone, reviewed=reviewed)
        self.assertEqual(italian.unfound_first_runs(reviewed.first_runs, gone), ["testo (lowercase noun « a text no longer written »)"])
        with self.assertRaises(SystemExit) as raised:
            self.reduce(first_run=gone)
        self.assertIn("FIRST_RUN", str(raised.exception))
        self.assertIn("testo (lowercase noun « a text no longer written »)", str(raised.exception))

    def test_a_row_the_marks_open_on_a_lesser_sense(self):
        # D2b opens `parmigiano` on the dialect, its adjective being marked archaic; the reviewed table
        # opens it on the cheese, as readers meet it.
        glosses = self.glosses(*PARMIGIANO, lemmas={"parmigiano"})
        self.assertEqual(
            glosses["parmigiano"],
            "parmesan cheese; the dialect of Emilian spoken in Parma; native or inhabitant of Parma (usually male); "
            "of, from or relating to Parma",
        )
        unordered = self.glosses(*PARMIGIANO, lemmas={"parmigiano"}, first_run={})
        self.assertTrue(unordered["parmigiano"].startswith("the dialect of Emilian spoken in Parma"), unordered["parmigiano"])

    def test_ce_opens_on_the_pronoun_its_pointer_lends(self):
        glosses = self.glosses(*CE, CI, lemmas={"ce"})
        self.assertEqual(glosses["ce"], "us; ourselves, each other; to us; it, to it; reinforcing particle, “do”")
        unordered = self.glosses(*CE, CI, lemmas={"ce"}, first_run={})
        self.assertTrue(unordered["ce"].startswith("reinforcing particle, “do”; us"), "the lent senses after the row's own")

    def test_the_reviewed_table(self):
        # The 19 rows the owner settled (task 0.1), the six added on the review of #1065 and `parmigiano`,
        # added on the implementation's gloss diff, each approved by him on 2026-10-25: each a case, a part
        # of speech, a sense and a reason.
        self.assertEqual(len(italian.FIRST_RUN), 26)
        for word, (case, pos, text, reason) in italian.FIRST_RUN.items():
            self.assertIn(case, italian._FIRST_RUN_CASES, word)
            self.assertIn(pos, common._KAIKKI_UPOS, word)
            self.assertTrue(text and reason, word)
        self.assertEqual(italian.unfound_first_runs(set(italian.FIRST_RUN)), [])


class Pointers(Fixture):
    """Task 1.5 (design D3): the pointers the English edition does not know, and a short word's lending."""

    def test_spec_scenario_a_proscribed_spelling(self):
        glosses = self.glosses(*CHE, lemmas={"che"})
        self.assertNotIn("proscribed", glosses["che"])
        self.assertEqual(glosses["che"], "what, which; who, whom, which, that; that; than")

    def test_spec_scenario_a_regional_form_of_a_short_word(self):
        glosses = self.glosses(*DE, DI, lemmas={"de", "di"})
        self.assertEqual(glosses["de"], glosses["di"], "di's first eight senses, the bound")
        self.assertEqual(len(glosses["de"].split("; ")), 8)
        self.assertTrue(glosses["de"].startswith("used to indicate possession"))

    def test_spec_scenario_a_pointer_whose_wording_names_a_label_it_is_not_tagged_with(self):
        self.assertEqual(self.glosses(*EL, IL[0], lemmas={"el"})["el"], "he; the")

    def test_spec_scenario_lent_senses_after_the_row_s_own(self):
        glosses = self.glosses(*NE, IN, lemmas={"ne"}, counts=MARKIT_COUNTS)
        senses = glosses["ne"].split("; ")
        self.assertEqual(len(senses), 8)
        self.assertEqual(senses[-2:], ["from there", "in"], "in's « in » alone, the one sense the bound leaves")
        self.assertTrue(senses[0].startswith("from this, from that"), "MarkIT's pronoun first")

    def test_spec_scenario_a_word_the_shared_rules_lend_to(self):
        glosses = self.glosses(JE, GLI_PRONOUN, GLI_FEMININE, GLI_PLURAL, LE_PRONOUN, lemmas={"je"})
        self.assertEqual(glosses["je"], "him, to him, it, to it; her, to her; them, to them")

    def test_spec_scenario_a_short_word_s_obsolete_form(self):
        self.assertEqual(self.glosses(*IL, *LO, lemmas={"il"})["il"], "the")

    def test_spec_scenario_a_clitic_compound(self):
        self.assertEqual(self.glosses(*APRILE, lemmas={"aprile"})["aprile"], "April")
        glosses = self.glosses(*VENDESI, lemmas={"vendesi"})
        self.assertEqual(glosses["vendesi"], "it gets sold, for sale; for-sale sign or announcement", "its meaning after the full stop")

    def test_a_misspelling_lends_nothing(self):
        self.assertNotIn("the", self.glosses(THE, TE_TEA, lemmas={"the"}))

    def test_a_dialectal_sense_is_never_lent(self):
        glosses = self.glosses(*SE, SI, lemmas={"se"})
        self.assertEqual(
            glosses["se"],
            "if; whether; if only; oneself, himself, herself, itself, themselves; each other, one another; "
            "one, you, we, they, people; used to form the passive voice of a verb, it",
        )

    def test_a_sense_the_row_says_already_is_never_lent(self):
        self.assertEqual(self.glosses(*TE, TI, lemmas={"te"})["te"], "you, thee; you, thou")

    def test_an_old_italian_form_lends_its_word(self):
        self.assertEqual(self.glosses(EO, IO, lemmas={"eo"})["eo"], "I (the first-person singular nominative pronoun)")

    def test_an_untagged_alternative_form_names_its_word_in_its_text(self):
        self.assertEqual(self.glosses(ITRIA, LONTRA, lemmas={"itria"})["itria"], "otter")


class CarriedMeanings23(Fixture):
    """Task 1.6 (design D4): the meanings a degree, a clipping and a plural carry."""

    def test_spec_scenario_a_meaning_nested_under_a_degree(self):
        glosses = self.glosses(*MINIMO, *MINORE, lemmas={"minimo", "minore"})
        self.assertTrue(glosses["minimo"].startswith("minimum, (the) least, (the) smallest, (the) slightest, (the) lowest; very small"))
        self.assertNotIn("degree of", glosses["minimo"])
        self.assertTrue(glosses["minore"].startswith("less; shorter"), glosses["minore"])
        self.assertNotIn("degree of", glosses["minore"])

    def test_spec_scenario_a_degree_beside_the_word_s_own_senses(self):
        self.assertTrue(self.glosses(*PIU, lemmas={"più"})["più"].startswith("more; anymore, any longer"))

    def test_spec_scenario_a_clipping(self):
        self.assertEqual(self.glosses(*AUTO, lemmas={"auto"})["auto"], "car; bus; auto (genre of dramatic literature); an auto literary work")

    def test_spec_scenario_a_plural_beside_another_part_of_speech(self):
        self.assertEqual(self.glosses(*GUAI, lemmas={"guai"})["guai"], "woe!; trouble")

    def test_a_plural_beside_its_own_part_of_speech_carries_nothing(self):
        glosses = self.glosses(*FUSA, COLLA_CONTRACTION, COLLA_NOUN, lemmas={"fusa", "colla"})
        self.assertEqual(glosses["fusa"], "purr (sound made by a cat)")
        self.assertEqual(glosses["colla"], "glue or similar sticky material")

    def test_spec_scenario_a_female_equivalent_beside_the_word_s_own_senses(self):
        glosses = self.glosses(*SIGNORA, SIGNORE, lemmas={"signora"})
        self.assertNotIn("lord", glosses["signora"])
        self.assertEqual(glosses["signora"], "lady; a title of address for women, madam (ladies is used for the plural)")


class NamesNotes(Fixture):
    """Task 1.7 (design D6): the names' notes a common word does not need."""

    def gloss(self, *entries, word, **kwargs):
        return self.glosses(*entries, lemmas={word}, **kwargs)[word]

    def test_spec_scenario_a_surname_after_a_common_word(self):
        self.assertEqual(self.gloss(*MUSICA, word="musica"), "music")

    def test_spec_scenario_a_surname_alone(self):
        self.assertEqual(self.gloss(MATTARELLA, word="mattarella"), "a surname")

    def test_spec_scenario_a_surname_beside_another_name(self):
        self.assertEqual(self.gloss(GARIBALDI, word="garibaldi"), "a surname; Anita Garibaldi, Italian patriot and wife of Giuseppe Garibaldi")
        self.assertEqual(self.gloss(*CLINTON, CLINTO, word="clinton"), "a surname in English; any of numerous placenames, see Clinton")
        self.assertEqual(self.gloss(*MOLINO, MULINO, word="molino"), "a surname; a frazione in Messina, Messina, Sicily, Italy")

    def test_spec_scenario_a_given_name_after_a_common_word(self):
        self.assertEqual(self.gloss(*DELFINO, word="delfino"), "dolphin; dauphin; probable successor; hunchbacked")

    def test_spec_scenario_a_given_name_opening_its_row(self):
        self.assertEqual(self.gloss(*ALICE, word="alice"), "a female given name, equivalent to English Alice; anchovy")

    def test_spec_scenario_a_town_beside_a_common_word(self):
        self.assertEqual(self.gloss(*NOTO, word="noto"), "of common knowledge; well-known, known; famous, notorious; the known")

    def test_spec_scenario_a_town_opening_its_row(self):
        self.assertEqual(self.gloss(*CERVIA, word="cervia"), "a town in Ravenna, Emilia-Romagna, Italy; hind (female deer)")
        self.assertTrue(self.gloss(*ANDRIA, word="andria").startswith("a municipality of Barletta-Andria-Trani province"))

    def test_spec_scenario_a_province_s_metropolitan_city(self):
        self.assertEqual(self.gloss(*RUSSO, word="russo"), "Russian; the Russian language")

    def test_spec_scenario_a_name_pointing_at_a_name(self):
        glosses = self.glosses(PINUCCIO, *PINO, lemmas={"pinuccio", "pino"})
        self.assertEqual(glosses["pinuccio"], "a diminutive of the male given name Giuseppe, feminine equivalent Pina")
        self.assertEqual(glosses["pino"], "pine, pine tree (tree of the genus Pinus)")

    def test_spec_scenario_a_provincial_capital(self):
        self.assertEqual(self.gloss(*FERMO, word="fermo"), "still; firm; a town and province of Marche, Italy")

    def test_spec_scenario_a_town_the_reviewed_list_keeps(self):
        reviewed = italian.Reviewed()
        self.assertEqual(self.gloss(*MASSA, word="massa", reviewed=reviewed), "mass; crowd; Massa (a town in Tuscany, Italy)")
        self.assertEqual(reviewed.kept, {"massa"})
        self.assertEqual(self.gloss(*MASSA, word="massa", towns_kept={}), "mass; crowd", "without the row, Massa goes")
        self.assertEqual(sorted(italian.TOWNS_KEPT), ["massa"])

    def test_a_lowercase_pointer_lends_once_the_names_go(self):
        self.assertEqual(self.gloss(*OBBIETTIVO, OBIETTIVO, word="obbiettivo"), "objective")

    def test_a_proper_noun_s_entry_left_with_no_sense_goes(self):
        read, stats = self.read(*MUSICA)
        self.assertEqual([(e["word"], e["pos"]) for e in read], [("musica", "noun")])
        self.assertEqual(stats["surnames' notes out"], 1)
        self.assertEqual(stats["names' entries left with no sense"], 1)


class PageNotes(Fixture):
    """Task 1.8 (design D7): the page's notes and typography."""

    def test_spec_scenario_an_article_s_usage_note(self):
        self.assertEqual(self.glosses(*LO, lemmas={"lo"})["lo"], "the; him; it, this or that thing")

    def test_spec_scenario_the_page_s_typography(self):
        glosses = self.glosses(ITRIA, LONTRA, MAR, lemmas={"itria", "mar"})
        self.assertEqual(glosses["mar"], "sea (used in poetry and in names of some seas)")
        self.assertNotIn("#", glosses["itria"])
        self.assertEqual(it_en.with_etc_period("you're welcome, don't mention it etc"), "you're welcome, don't mention it etc.")
        self.assertEqual(it_en.with_etc_period("stage (of a theatre etc.)"), "stage (of a theatre etc.)")


class Corrections(Fixture):
    """Task 2.1 (design D5, D8): « etc. », and the sources' slips corrected by name."""

    def test_spec_scenario_a_jocular_spelling_among_the_translations(self):
        _, _, _, mwe = self.reduce(translated=[*TRANSLATED, ALL_ANTICA])
        self.assertEqual(mwe["all'antica"], "ancient, antique")
        self.assertIn("1 of 5 corrections fired", self.report)

    def test_spec_scenario_a_meaning_the_section_leaves_out(self):
        section = [c for c in it_en.CORRECTIONS if c.source == "section"]
        reviewed = italian.Reviewed()
        glosses = self.glosses(*AMICA, lemmas={"amica"}, corrections=section, reviewed=reviewed)
        self.assertEqual(glosses["amica"], "friend; girlfriend")
        self.assertEqual([c.headword for c in reviewed.fired], ["amica"])
        self.assertEqual(self.glosses(*AMICA, lemmas={"amica"})["amica"], "girlfriend", "without the list")

    def test_spec_scenario_the_page_corrected_upstream(self):
        upstream = {**ALL_ANTICA, "translations": [{"word": "ancient"}, {"word": "antique"}]}
        _, _, _, mwe = self.reduce(translated=[*TRANSLATED, upstream])
        self.assertEqual(mwe["all'antica"], "ancient, antique")
        self.assertIn("fired on nothing, to remove: all'antica (translations, phrase)", self.report)

    def test_etc_takes_its_period_in_the_reduction(self):
        _, glosses, _, _ = self.reduce(section=[*SECTION, *PREGO], ranked=[*RANKED, "prego"])
        self.assertEqual(glosses["prego"], "prayer; you're welcome, don't mention it etc.; yes, of course")
        self.assertIn("1 glosses whose « etc » takes its period", self.report)

    def test_the_list(self):
        self.assertEqual([c.headword for c in it_en.CORRECTIONS], ["all'antica", "amica", "bambina", "compagna", "spagnola"])
        for correction in it_en.CORRECTIONS:
            self.assertIn(correction.source, ("section", "translations"), correction.headword)
            self.assertTrue(correction.reason and correction.report.startswith("https://"), correction.headword)
            self.assertEqual(correction.added, correction.source == "section", correction.headword)


class TheRules(unittest.TestCase):
    """it-en's rule digest (D1, D4, D11)."""

    def test_it_en_s_rule_files(self):
        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-it-en.py")],
            [
                "reduce-it-en.py",
                "reduce_common.py",
                "reduce_edition_en.py",
                "reduce_french_treebank.py",
                "reduce_italian_acronyms.py",
                "reduce_italian_en.py",
                "reduce_italian_treebank.py",
            ],
        )

    def test_it_en_s_own_module_is_no_other_pair_s(self):
        for reducer in sorted(Path(_HERE).glob("reduce-*.py")):
            if reducer.name != "reduce-it-en.py":
                self.assertNotIn("reduce_italian_en.py", [p.name for p in ps.rule_files(reducer)], reducer.name)

    def test_spec_scenario_nothing_else_moves(self):
        # The English edition's shared rules and the shared module are not edited: es-en's and fr-en's
        # pins still record the rules they were reduced by, and so does every committed pair.
        for pin in sorted((Path(_HERE) / "tables").glob("*/pin.json")):
            reducer = Path(_HERE) / f"reduce-{pin.parent.name}.py"
            self.assertEqual(ps.rules_sha256(reducer), ps.get(ps.load(pin), "reducer.sha256"), pin.parent.name)


if __name__ == "__main__":
    unittest.main()
