# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for fr-es's native side (add-lingua-pack-fr-es): French glossed in Spanish.

The entries are recorded from the three files the prototype read — the Spanish Wiktionary's French
section (`kaikki-es-Frances.jsonl`) and the French translations its Spanish entries list
(`kaikki-es-traductions.jsonl`), derived from its dump regenerated 2026-10-02 12:12, and the Spanish
translations the French Wiktionary's French entries list (`kaikki-fr-traductions.jsonl`), from its
dump regenerated 2026-10-02 00:10 — shaped as the dumps write them, cut down to the fields the rules
read. À_SECTION keeps its entry's first senses only. A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import contextlib
import importlib.util
import io
import json
import os
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import reduce_common as common  # noqa: E402
import reduce_edition_es as spanish  # noqa: E402
from reduce_edition_es import ES  # noqa: E402


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(
        f"reduce_{pair.replace('-', '_')}_fr_es_tests", os.path.join(_HERE, f"reduce-{pair}.py")
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


fr_es = _reducer("fr-es")
fr_en = _reducer("fr-en")

# — The Spanish Wiktionary's French section (kaikki-es-Frances.jsonl) —

MAISON = {"word": "maison", "pos": "noun", "tags": ["feminine"], "senses": [{"glosses": ["Casa."]}]}
ET = {"word": "et", "pos": "conj", "senses": [{"glosses": ["Et."]}]}
VENIR = {"word": "venir", "pos": "verb", "tags": ["intransitive"], "senses": [{"glosses": ["Venir."]}]}
TROLL = {"word": "troll", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Troll."]}]}
ELITE = {"word": "élite", "pos": "noun", "tags": ["feminine"], "senses": [{"glosses": ["Élite."]}]}
PIERRE = {"word": "pierre", "pos": "noun", "tags": ["feminine"], "senses": [{"glosses": ["Piedra."]}]}
PIERRE_VERB = {
    "word": "pierre",
    "pos": "verb",
    "tags": ["form-of"],
    "senses": [
        {
            "glosses": ["Primera persona del singular (je) del presente de indicativo de pierrer."],
            "tags": ["form-of"],
            "form_of": [{"word": "pierrer"}],
        },
        {
            "glosses": ["Tercera persona del singular (elle, on, il) del presente de indicativo de pierrer."],
            "tags": ["form-of"],
            "form_of": [{"word": "pierrer"}],
        },
    ],
}
PIERRE_NAME = {
    "word": "Pierre",
    "pos": "name",
    "senses": [{"glosses": ["Nombre de pila de varón, equivalente del español Pedro."]}],
}
JEAN_ADJ = {"word": "jean", "pos": "adj", "senses": [{"glosses": ["Color de mezclilla."]}]}
JEAN_PHRASE = {
    "word": "jean",
    "pos": "phrase",
    "tags": ["substantive"],
    "senses": [{"glosses": ["Mezclilla."]}, {"glosses": ["Tejanos."], "raw_tags": ["Vestimenta"]}],
}
JEAN_NAME = {"word": "Jean", "pos": "name", "senses": [{"glosses": ["Nombre de pila de varón, equivalente del español Juan"]}]}
FRANCOIS_NAME = {
    "word": "François",
    "pos": "name",
    "senses": [{"glosses": ["Nombre de pila de varón, equivalente del español Francisco"]}],
}
MON = {"word": "mon", "pos": "adj", "tags": ["irregular", "possessive"], "senses": [{"glosses": ["Mi."]}]}
MES = {
    "word": "mes",
    "pos": "adj",
    "tags": ["adjectival", "form-of"],
    "senses": [
        {"glosses": ["Forma del masculino plural de mon."], "tags": ["form-of"], "form_of": [{"word": "mon"}]},
        {"glosses": ["Forma del femenino plural de mon."], "tags": ["form-of"], "form_of": [{"word": "mon"}]},
    ],
}
MA = {
    "word": "ma",
    "pos": "adj",
    "tags": ["adjectival", "form-of"],
    "senses": [{"glosses": ["Forma del femenino singular de mon; mi."], "tags": ["form-of"], "form_of": [{"word": "mon"}]}],
}
CE_ADJ = {
    "word": "ce",
    "pos": "adj",
    "tags": ["demonstrative", "irregular"],
    "senses": [
        {
            "glosses": ["Este."],
            "raw_tags": ["toma la forma cet delante de un sustantivo masculino que empieza en una vocal o una h muda"],
        }
    ],
}
CE_PRON = {
    "word": "ce",
    "pos": "pron",
    "tags": ["demonstrative", "neuter"],
    "senses": [
        {
            "glosses": ["Esto, eso, aquello."],
            "raw_tags": ["se usa solamente como sujeto del verbo être o como antecedente de un pronombre relativo"],
        }
    ],
}
SON_PRON = {"word": "son", "pos": "pron", "tags": ["possessive"], "senses": [{"glosses": ["Su."]}]}
SON_NOUN = {"word": "son", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Sonido."]}]}
CHAPELET = {
    "word": "chapelet",
    "pos": "noun",
    "tags": ["masculine"],
    "senses": [{"glosses": ["Guirnalda."], "raw_tags": ["obsoleta"]}, {"glosses": ["Rosario."]}],
}
H_SECTION = {
    "word": "h",
    "pos": "character",
    "senses": [
        {
            "glosses": ["Octava letra y sexta consonante del alfabeto francés. Se llama ache."],
            "tags": ["lower case"],
            "raw_tags": ["puede ser muda, como en español, o sonar como [h], como en inglés"],
        }
    ],
}
X_SECTION = {
    "word": "x",
    "pos": "character",
    "senses": [{"glosses": ["Vigesimocuarta letra y decimonovena consonante del alfabeto francés."], "tags": ["lower case"]}],
}
A_SECTION = {
    "word": "à",
    "pos": "prep",
    "senses": [{"glosses": ["A (un lugar)."]}, {"glosses": ["A, hasta (un tiempo)."]}, {"glosses": ["A (una dirección)."]}],
}
Y_CHARACTER = {
    "word": "y",
    "pos": "character",
    "senses": [{"glosses": ["Vigesimoquinta letra y vigésima consonante del alfabeto francés."], "tags": ["lower case"]}],
}
Y_ADV = {"word": "y", "pos": "adv", "tags": ["place"], "senses": [{"glosses": ["Allí, ahí."]}]}
# Made up: a headword with a typographic apostrophe, as French text and the French Wiktionary write
# it — the section writes 6 (« prud’homme »).
PRUDHOMME = {"word": "prud’homme", "pos": "noun", "tags": ["masculine"], "senses": [{"glosses": ["Juez laboral."]}]}

# — The French Wiktionary's Spanish translations (kaikki-fr-traductions.jsonl), the direct table —

MAISON_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"sense": "Bâtiment servant de logis, d’habitation, de demeure", "word": "casa"},
        {"sense": "Foyer", "word": "casa"},
        {"sense": "Lignée, famille", "word": "linaje"},
        {"sense": "Entreprise, commerce", "word": "casa"},
    ],
    "word": "maison",
}
INTERET_TRANSLATED = {"pos": "noun", "translations": [{"sense": "Ce qui importe", "word": "interés"}], "word": "intérêt"}
ET_TRANSLATED = {
    "pos": "conj",
    "translations": [
        {"sense": "Conjonction", "word": "y"},
        {"sense": "Conjonction", "word": "e"},
        {"sense": "Emphatique en début de phrase", "word": "y"},
    ],
    "word": "et",
}
VENIR_TRANSLATED = {"pos": "verb", "translations": [{"sense": "Survenir, arriver", "word": "venir"}], "word": "venir"}
TROLL_TRANSLATED = {"pos": "noun", "translations": [{"word": "trol"}], "word": "troll"}
ELITE_TRANSLATED = {"pos": "noun", "translations": [{"word": "elite"}], "word": "élite"}
PIERRE_TRANSLATED = {"pos": "noun", "translations": [{"sense": "roche", "word": "piedra"}], "word": "pierre"}
A_TRANSLATED = {
    "pos": "prep",
    "translations": [
        {"sense": "Complément circonstanciel de lieu avec déplacement", "word": "a"},
        {"sense": "Complément circonstanciel de lieu avec déplacement", "word": "en"},
        {"sense": "Complément circonstanciel de lieu sans déplacement", "word": "en"},
        {"sense": "Complément circonstanciel de provenance", "word": "de"},
        {"word": "sobre"},
    ],
    "word": "à",
}
Y_TRANSLATED = {
    "pos": "adv",
    "translations": [{"sense": "À cet endroit", "word": "ahí"}, {"sense": "À cet endroit", "word": "allí"}],
    "word": "y",
}
O_TRANSLATED = {"pos": "particle", "translations": [{"word": "oh"}], "word": "ô"}
I_TRANSLATED = {
    "pos": "character",
    "translations": [{"sense": "Nom de la lettre I, i", "word": "i latina"}, {"sense": "Nom de la lettre I, i", "word": "i"}],
    "word": "i",
}
MAIN_D_OEUVRE_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"sense": "Travail de l’ouvrier considéré surtout par rapport au prix.", "word": "mano de obra"},
        {"sense": "L’ensemble des ouvriers de tel ou tel métier.", "word": "mano de obra"},
    ],
    "word": "main-d’œuvre",
}
ALLER_DE_L_AVANT_TRANSLATED = {"pos": "verb", "translations": [{"word": "echar para adelante"}], "word": "aller de l’avant"}
PARTI_ADJ_TRANSLATED = {"pos": "adj", "translations": [{"sense": "Héraldique", "word": "partido"}], "word": "parti"}
PARTI_NOUN_TRANSLATED = {"pos": "noun", "translations": [{"sense": "parti politique", "word": "partido"}], "word": "parti"}

# — The Spanish Wiktionary's French translations (kaikki-es-traductions.jsonl), read backwards —

TRAVES_LISTS = {"pos": "noun", "translations": [{"word": "travers"}], "word": "través"}
ESTE_ADJ_LISTS = {
    "pos": "adj",
    "translations": [{"word": "ce"}, {"word": "cet"}, {"word": "cette"}, {"word": "celui-ci"}, {"word": "celle-ci"}],
    "word": "este",
}
ESTE_PRON_LISTS = {
    "pos": "pron",
    "translations": [{"word": "ce"}, {"word": "cet"}, {"word": "cette"}, {"word": "celui-ci"}, {"word": "celle-ci"}],
    "word": "este",
}
H_LISTS = {"pos": "character", "translations": [{"word": "h"}], "word": "h"}
X_LISTS = {"pos": "character", "translations": [{"word": "x"}], "word": "x"}
I_LISTS = {"pos": "noun", "translations": [{"sense": "letra", "word": "i"}], "word": "i"}
Y_LISTS_ET = {"pos": "conj", "translations": [{"word": "et"}], "word": "y"}
CASA_LISTS = {"pos": "noun", "translations": [{"word": "maison"}], "word": "casa"}
GOLPE_MILITAR_LISTS = {"pos": "phrase", "translations": [{"word": "coup d’État"}], "word": "golpe militar"}
ADIESTRADOR_ADJ_LISTS = {"pos": "adj", "translations": [{"word": "entraîneur"}, {"word": "dresseur"}], "word": "adiestrador"}
ADIESTRADOR_NOUN_LISTS = {"pos": "noun", "translations": [{"word": "entraîneur"}, {"word": "dresseur"}], "word": "adiestrador"}
CONDOM_LISTS = [
    {"pos": "adj", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "preservativo"},
    {"pos": "noun", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "preservativo"},
    {"pos": "noun", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "condón"},
    {"pos": "adj", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "profiláctico"},
    {"pos": "noun", "translations": [{"word": "préservatif"}, {"word": "condom"}, {"word": "capote"}], "word": "profiláctico"},
]
# French's readings of those words, as tables/fr/grammar.tsv commits them.
CONDOM_ENTRAINEUR_GRAMMAR = (
    "condom\tcondom\tNOUN|Gender=Masc|Number=Sing\t-\n"
    "condoms\tcondom\tNOUN|Gender=Masc|Number=Plur\t-\n"
    "entraîneur\tentraîneur\tNOUN|Gender=Masc|Number=Sing\t-\n"
)

# wordfreq's Spanish Zipf frequency, as the reducer reads it (recorded for the words read here).
SPANISH_ZIPF = {"preservativo": 3.3, "profiláctico": 1.8, "condón": 3.2, "adiestrador": 1.9, "través": 4.9}


def spanish_zipf(word):
    return SPANISH_ZIPF.get(word, 0.0)


class Tables(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def jsonl(self, *entries, name="entries.jsonl"):
        path = self.dir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        return str(path)

    def section(self, *entries):
        """The section through fr-es's passes, in `main`'s order: the file the shared rules read."""
        src = self.jsonl(*entries, name="section.jsonl")
        src = fr_es.straight_apostrophes(src, str(self.dir / "section-apostrophes.jsonl"))
        src = common.without_letter_senses(src, str(self.dir / "section-words.jsonl"), edition=ES)
        src = spanish.read_as_meanings(src, str(self.dir / "section-meanings.jsonl"))
        return fr_es.french_entries(src, str(self.dir / "section-glossing.jsonl"))

    def direct(self, *entries):
        src = self.jsonl(*entries, name="direct.jsonl")
        return fr_es.read_translated(src, str(self.dir / "direct-words.jsonl"), inverted=False, readings={})

    def inverted(self, *entries, readings=None):
        src = self.jsonl(*entries, name="inverted.jsonl")
        return fr_es.read_translated(src, str(self.dir / "inverted-words.jsonl"), inverted=True, readings=readings or {})

    def tables(self, lemmas, *section, direct=(), inverted=()):
        """fr-es's native side over `lemmas`, ranked in their order: `(glosses, runs, expressions,
        steps)`."""
        ranks = {lemma: rank for rank, lemma in enumerate(lemmas, 1)}
        sources = [(self.direct(*direct), list), (self.inverted(*inverted), fr_es.by_spanish_frequency(spanish_zipf))]
        return fr_es.native_side(self.section(*section), ranks, sources)


class FrenchGlossedInSpanish(Tables):
    """*French is glossed in Spanish from the Spanish Wiktionary's French section and the French
    Wiktionary's translation tables*, scenario by scenario."""

    def test_spec_scenario_a_definition_first(self):
        # `maison` has a definition: it wins over the direct table's « casa, linaje » and the inverted
        # table's `casa`.
        glosses, runs, _, steps = self.tables(
            ["maison"], MAISON, direct=[MAISON_TRANSLATED], inverted=[CASA_LISTS]
        )
        self.assertEqual((glosses["maison"], runs["maison"]), ("Casa", [("NOUN", 1)]))
        self.assertEqual(steps["entries"], {"maison"})

    def test_spec_scenario_a_direct_table_gloss(self):
        # No definition of `intérêt`: the French Wiktionary's Spanish word, opening on a capital as
        # the edition's senses do. At most three Spanish words per part of speech, in the table's
        # order, each once: `à` read with no definition (made up — the section defines it).
        glosses, runs, _, steps = self.tables(["intérêt", "à"], direct=[INTERET_TRANSLATED, A_TRANSLATED])
        self.assertEqual((glosses["intérêt"], runs["intérêt"]), ("Interés", [("NOUN", 1)]))
        self.assertEqual((glosses["à"], runs["à"]), ("A, en, de", [("ADP", 1)]))
        self.assertEqual(steps["direct"], {"intérêt", "à"})

    def test_spec_scenario_an_inverted_table_gloss(self):
        glosses, runs, _, steps = self.tables(["travers"], inverted=[TRAVES_LISTS])
        self.assertEqual((glosses["travers"], runs["travers"]), ("Través", [("NOUN", 1)]))
        self.assertEqual(steps["inverted"], {"travers"})

    def test_spec_scenario_the_studied_word_is_no_gloss(self):
        # The section defines the conjunction `et` « Et. »: the French word itself. The French
        # Wiktionary's Spanish words gloss it, and the lemma is the direct table's.
        glosses, runs, _, steps = self.tables(["et"], ET, direct=[ET_TRANSLATED], inverted=[Y_LISTS_ET])
        self.assertEqual((glosses["et"], runs["et"]), ("Y, e", [("CCONJ", 1)]))
        self.assertEqual((steps["entries"], steps["direct"], steps["yielded"]), (set(), {"et"}, {"et"}))
        # Up to case: « Élite. » for `élite` and « Troll. » for `troll`, the table's words being other.
        glosses, _, _, steps = self.tables(
            ["élite", "troll"], ELITE, TROLL, direct=[ELITE_TRANSLATED, TROLL_TRANSLATED]
        )
        self.assertEqual(glosses, {"élite": "Elite", "troll": "Trol"})
        self.assertEqual(steps["yielded"], {"élite", "troll"})

    def test_spec_scenario_a_cognate_keeps_its_definition(self):
        # « Venir. » repeats `venir`, and the table lists « venir » too: a Spanish word spelled alike.
        glosses, _, _, steps = self.tables(["venir"], VENIR, direct=[VENIR_TRANSLATED])
        self.assertEqual(glosses, {"venir": "Venir"})
        self.assertEqual((steps["entries"], steps["yielded"]), ({"venir"}, set()))

    def test_the_studied_word_keeps_its_definition_when_no_table_lists_it(self):
        # Made up: nothing else glosses `et`, or `et` defined beside a sense of its own: kept.
        glosses, _, _, steps = self.tables(["et"], ET)
        self.assertEqual((glosses, steps["yielded"]), ({"et": "Et"}, set()))
        two = {**ET, "senses": [{"glosses": ["Et."]}, {"glosses": ["Y."]}]}
        glosses, _, _, steps = self.tables(["et"], two, direct=[ET_TRANSLATED])
        self.assertEqual((glosses, steps["yielded"]), ({"et": "Et; Y"}, set()))

    def test_no_self_definition_reads_a_gloss_as_written_up_to_case(self):
        glosses = {"et": "Et", "élite": "ÉLITE.", "venir": "Venir", "maison": "Casa", "slip": "Slip; Slip"}
        runs = {lemma: [("X", 1)] for lemma in glosses}
        direct = {"et": {"CCONJ": ["y"]}, "élite": {"NOUN": ["elite"]}, "venir": {"VERB": ["Venir"]}, "slip": {"NOUN": ["bragas"]}}
        self.assertEqual(fr_es.no_self_definition(glosses, runs, direct), ["et", "slip", "élite"])
        self.assertEqual(glosses, {"venir": "Venir", "maison": "Casa"})
        self.assertEqual(set(runs), {"venir", "maison"})

    def test_spec_scenario_a_name_on_a_common_word_s_card(self):
        # `Pierre`'s note is left out of `pierre`'s card, and the entry, left with no sense, goes;
        # `Jean`'s out of `jean`'s.
        glosses, runs, _, _ = self.tables(
            ["pierre", "jean"], PIERRE, PIERRE_VERB, PIERRE_NAME, JEAN_ADJ, JEAN_PHRASE, JEAN_NAME
        )
        self.assertEqual((glosses["pierre"], runs["pierre"]), ("Piedra", [("NOUN", 1)]))
        self.assertEqual(glosses["jean"], "Color de mezclilla; Mezclilla; Tejanos")
        written = [json.loads(line)["word"] for line in Path(self.dir / "section-glossing.jsonl").read_text().splitlines()]
        self.assertEqual(written, ["pierre", "pierre", "jean", "jean"])

    def test_spec_scenario_a_name_s_own_row(self):
        # No `françois` in lower case: the note says what the capitalised token is.
        glosses, runs, _, _ = self.tables(["françois"], FRANCOIS_NAME)
        self.assertEqual(
            (glosses["françois"], runs["françois"]),
            ("Nombre de pila de varón, equivalente del español Francisco", [("PROPN", 1)]),
        )
        # Made up: an entry in lower case that only points at another word holds no meaning.
        pointer = {"word": "françois", "pos": "adj", "senses": [{"glosses": ["Forma del plural de français."]}]}
        glosses, _, _, _ = self.tables(["françois"], FRANCOIS_NAME, pointer)
        self.assertEqual(glosses["françois"], "Nombre de pila de varón, equivalente del español Francisco")

    def test_spec_scenario_possessives_and_their_forms(self):
        # `mon` tagged possessive, `ce` demonstrative: determiners; `mes` and `ma`, every sense a form
        # of `mon`, too — so they borrow `mon`'s « Mi » in its part of speech. Only the runs move.
        glosses, runs, _, _ = self.tables(["mon", "mes", "ma", "ce"], MON, MES, MA, CE_ADJ, CE_PRON)
        self.assertEqual(glosses, {"mon": "Mi", "mes": "Mi", "ma": "Mi", "ce": "Este; Esto, eso, aquello"})
        self.assertEqual(
            runs,
            {"mon": [("DET", 1)], "mes": [("DET", 1)], "ma": [("DET", 1)], "ce": [("DET", 1), ("PRON", 1)]},
        )
        # Without the second clause, `mes` lost its gloss: a form lends in its own part of speech.
        with mock.patch.object(fr_es, "_forms_of", lambda entry, words: False):
            glosses, _, _, _ = self.tables(["mon", "mes"], MON, MES)
        self.assertEqual(glosses, {"mon": "Mi"})
        # The edition heads `son` « pronombre posesivo »: kept as written.
        _, runs, _, _ = self.tables(["son"], SON_PRON, SON_NOUN)
        self.assertEqual(runs, {"son": [("PRON", 1), ("NOUN", 1)]})

    def test_a_form_of_a_word_that_is_no_determiner_stays_an_adjective(self):
        # Made up: an adjective whose senses point at an adjective that is no determiner, or at a
        # determiner and another word.
        grand = {"word": "grande", "pos": "adj", "senses": [{"glosses": ["Forma del femenino de grand."], "form_of": [{"word": "grand"}]}]}
        mixed = {
            "word": "mas",
            "pos": "adj",
            "senses": [
                {"glosses": ["Forma de mon."], "form_of": [{"word": "mon"}]},
                {"glosses": ["Forma de grand."], "form_of": [{"word": "grand"}]},
            ],
        }
        none = {"word": "vide", "pos": "adj", "senses": []}
        written = Path(self.section(MON, grand, mixed, none)).read_text(encoding="utf-8").splitlines()
        self.assertEqual([json.loads(line)["pos"] for line in written], ["det", "adj", "adj", "adj"])

    def test_spec_scenario_a_letter_glosses_no_word(self):
        # `h` and `x`: the section's letters are left out, and the only word a table gives each is
        # the letter itself — the Spanish Wiktionary's `h` and `x` list theirs, its noun `i` the
        # letter, the French Wiktionary's `i` « i latina, i ». `à` and `y`, words of one letter, keep
        # the section's definitions; `ô` the direct table's « Oh ».
        glosses, _, _, _ = self.tables(
            ["h", "x", "i", "à", "y", "ô"],
            H_SECTION,
            X_SECTION,
            A_SECTION,
            Y_CHARACTER,
            Y_ADV,
            direct=[I_TRANSLATED, A_TRANSLATED, Y_TRANSLATED, O_TRANSLATED],
            inverted=[H_LISTS, X_LISTS, I_LISTS, Y_LISTS_ET],
        )
        self.assertEqual(
            glosses,
            {"à": "A (un lugar); A, hasta (un tiempo); A (una dirección)", "y": "Allí, ahí", "ô": "Oh"},
        )
        # Read backwards, a one-letter French word is dropped whatever lists it (made up: a Spanish
        # word, no letter, listing `h`).
        self.assertEqual(self.inverted({"pos": "intj", "translations": [{"word": "h"}], "word": "hache"}), {})

    def test_spec_scenario_a_spanish_word_listed_once(self):
        # `este`, adjective and pronoun, both list `cet`: listed once, under the first listed — no
        # reading of `cet` names either.
        inverted = self.inverted(ESTE_ADJ_LISTS, ESTE_PRON_LISTS)
        self.assertEqual(inverted["cet"], {"ADJ": ["este"]})
        glossed = common.fallback_glosses({"cet"}, {}, [(inverted, list)], edition=ES)
        self.assertEqual(glossed, {"cet": ("Este", [("ADJ", 1)])})
        # French's readings say where a word goes: `entraîneur` and `condom` are nouns.
        studied = self.dir / "fr"
        studied.mkdir()
        (studied / "grammar.tsv").write_text(CONDOM_ENTRAINEUR_GRAMMAR, encoding="utf-8")
        readings = fr_es.read_readings(str(studied))
        self.assertEqual(readings, {"condom": {"NOUN"}, "entraîneur": {"NOUN"}})
        order = fr_es.by_spanish_frequency(spanish_zipf)
        lists = (ADIESTRADOR_ADJ_LISTS, ADIESTRADOR_NOUN_LISTS, *CONDOM_LISTS)
        read = common.fallback_glosses({"condom", "entraîneur"}, {}, [(self.inverted(*lists, readings=readings), order)], edition=ES)
        self.assertEqual(
            read,
            {"condom": ("Preservativo, condón, profiláctico", [("NOUN", 1)]), "entraîneur": ("Adiestrador", [("NOUN", 1)])},
        )
        # Without them, the first listed: an adjective.
        read = common.fallback_glosses({"condom", "entraîneur"}, {}, [(self.inverted(*lists), order)], edition=ES)
        self.assertEqual(
            read,
            {
                "condom": ("Preservativo, profiláctico; Condón", [("ADJ", 1), ("NOUN", 1)]),
                "entraîneur": ("Adiestrador", [("ADJ", 1)]),
            },
        )
        (studied / "grammar.tsv").unlink()
        self.assertEqual(fr_es.read_readings(str(studied)), {}, "no readings: an empty mapping")

    def test_the_direct_table_keeps_the_french_word_s_parts_of_speech(self):
        # `parti`, translated as an adjective and as a noun: « Partido; Partido » (Open Question 3).
        glossed = common.fallback_glosses({"parti"}, {}, [(self.direct(PARTI_ADJ_TRANSLATED, PARTI_NOUN_TRANSLATED), list)], edition=ES)
        self.assertEqual(glossed, {"parti": ("Partido; Partido", [("ADJ", 1), ("NOUN", 1)])})

    def test_spec_scenario_a_typographic_apostrophe(self):
        # The French Wiktionary's `main-d’œuvre` is the committed lemma `main-d'œuvre`; its
        # expression « aller de l’avant », and the Spanish Wiktionary's « coup d’État » read
        # backwards, are keyed as French's tokens read them; a section headword likewise (made up).
        glosses, _, expressions, _ = self.tables(
            ["main-d'œuvre", "prud'homme"],
            PRUDHOMME,
            direct=[MAIN_D_OEUVRE_TRANSLATED, ALLER_DE_L_AVANT_TRANSLATED],
            inverted=[GOLPE_MILITAR_LISTS],
        )
        self.assertEqual(glosses, {"main-d'œuvre": "Mano de obra", "prud'homme": "Juez laboral"})
        self.assertEqual(expressions, {"aller de l'avant": "Echar para adelante", "coup d'état": "Golpe militar"})
        # Without it, neither table glosses the word: `’` is no letter of a French token.
        self.assertEqual(common.read_translations(self.jsonl(MAIN_D_OEUVRE_TRANSLATED), inverted=False, studied=fr_es.FR), {})

    def test_the_outdated_senses_come_after_the_others(self):
        # The Spanish edition's pre-pass, as en-es reads it: `chapelet`'s « Guirnalda », labelled
        # obsolete, after « Rosario ».
        glosses, _, _, _ = self.tables(["chapelet"], CHAPELET)
        self.assertEqual(glosses, {"chapelet": "Rosario; Guirnalda"})

    def test_the_passes_write_what_they_cannot_read_as_they_are(self):
        # Made up: lines that are no JSON object, an entry they do not change, senses and words of no
        # kaikki shape. Each is written as it is, byte for byte, by each pass.
        text = (
            "not json\n[1, 2]\n"
            '{"word":"maison","pos":"noun","senses":[{"glosses":["Casa."]}]}\n'
            '{"word": "raro", "pos": "adj", "tags": "possessive", "senses": ["odd", {"glosses": [1]}, {"form_of": "mon"}]}\n'
            '{"word": 3, "senses": 4, "translations": 5}\n'
            '{"word": "Raro", "pos": "name", "senses": "Apellido."}\n'
            '{"pos": "noun", "translations": [1, {"word": 2}]}\n'
        )
        src = self.dir / "entries.jsonl"
        src.write_text(text, encoding="utf-8")
        for name, run in (
            ("apostrophes", lambda dst: fr_es.straight_apostrophes(str(src), dst)),
            ("glossing", lambda dst: fr_es.french_entries(str(src), dst)),
            ("direct", lambda dst: fr_es.translation_words(str(src), dst, inverted=False)),
            ("inverted", lambda dst: fr_es.translation_words(str(src), dst, inverted=True)),
        ):
            out = run(str(self.dir / f"{name}.jsonl"))
            self.assertEqual(Path(out).read_text(encoding="utf-8"), text, name)


class TheReducer(Tables):
    """fr-es reduced as its build runs it (`main()`), from a studied folder as fr-en writes it (D1),
    and what it measures of its tables (D9)."""

    def studied(self, forms, freq, grammar=None):
        folder = self.dir / "fr"
        folder.mkdir(exist_ok=True)
        (folder / "forms.tsv").write_text(forms, encoding="utf-8")
        (folder / "freq.tsv").write_text(freq, encoding="utf-8")
        if grammar is not None:
            (folder / "grammar.tsv").write_text(grammar, encoding="utf-8")
        return folder

    def reduce(self, work, studied):
        wordfreq = types.ModuleType("wordfreq")
        wordfreq.zipf_frequency = lambda word, lang: spanish_zipf(word) if lang == "es" else 0.0
        argv = [
            "reduce-fr-es.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-10", "--pack-version", "test"),
        ]
        with (
            mock.patch.dict(sys.modules, {"wordfreq": wordfreq}),
            mock.patch.object(sys, "argv", argv),
            contextlib.redirect_stderr(io.StringIO()) as err,
        ):
            fr_es.main()
        return err.getvalue()

    def test_spec_scenarios_through_main(self):
        work = self.dir / "work"
        studied = self.studied(
            "condom\tcondom\ncondoms\tcondom\net\tet\nh\th\nintérêt\tintérêt\nmain-d'œuvre\tmain-d'œuvre\n"
            "maison\tmaison\nmaisons\tmaison\nmes\tmes\nmon\tmon\npierre\tpierre\ntravers\ttravers\n",
            "et\t1\nmon\t2\nmes\t3\nmaison\t4\nh\t5\npierre\t6\nintérêt\t7\ntravers\t8\nmain-d'œuvre\t9\ncondom\t10\n",
            CONDOM_ENTRAINEUR_GRAMMAR,
        )
        self.jsonl(MAISON, ET, MON, MES, PIERRE, PIERRE_NAME, H_SECTION, name="work/kaikki-es-Frances.jsonl")
        self.jsonl(
            ET_TRANSLATED, INTERET_TRANSLATED, MAIN_D_OEUVRE_TRANSLATED, ALLER_DE_L_AVANT_TRANSLATED, I_TRANSLATED,
            name="work/kaikki-fr-traductions.jsonl",
        )
        self.jsonl(TRAVES_LISTS, H_LISTS, *CONDOM_LISTS, name="work/kaikki-es-traductions.jsonl")
        err = self.reduce(work, studied)
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "condom\tPreservativo, condón, profiláctico\n"
            "et\tY, e\n"
            "intérêt\tInterés\n"
            "main-d'œuvre\tMano de obra\n"
            "maison\tCasa\n"
            "mes\tMi\n"
            "mon\tMi\n"
            "pierre\tPiedra\n"
            "travers\tTravés\n",
        )
        self.assertEqual(
            (work / "senses.tsv").read_text(encoding="utf-8"),
            "condom\tNOUN:1\net\tCCONJ:1\nintérêt\tNOUN:1\nmain-d'œuvre\tNOUN:1\nmaison\tNOUN:1\n"
            "mes\tDET:1\nmon\tDET:1\npierre\tNOUN:1\ntravers\tNOUN:1\n",
        )
        self.assertEqual((work / "mwe.tsv").read_text(encoding="utf-8"), "aller de l'avant\tEchar para adelante\n")
        # The native side alone (D1), and what the reducer measured of it (D9).
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith("kaikki-"))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        measures = json.loads((work / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(
            measures,
            {
                "top": 10000,
                "glossed": 9,
                "share": 55.6,
                "direct": ["et", "intérêt", "main-d'œuvre"],
                "inverted": ["condom", "travers"],
            },
        )
        self.assertIn(
            "reduced fr-es: lemmas=10 (French's committed tables) glosses=9 (Spanish Wiktionary 4, French "
            "Wiktionary's Spanish translations 3, Spanish Wiktionary's French translations read backwards 2; "
            "44.4 % from a definition; 1 definitions repeating their headword set aside) expressions=1",
            err,
        )
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("fr", "es"))
        self.assertEqual(manifest["meta"]["pack_version"], "test")
        self.assertEqual(manifest["meta"]["analyzer_version"], fr_en.analyser_version(), "French's analyser version")
        self.assertIs(manifest["meta"]["levels_estimated"], True, "French's levels are fr-en's estimate")
        self.assertEqual([s["name"] for s in manifest["sources"]], ["kaikki", "wordfreq", "UD French-GSD"])
        notice = (work / "NOTICE").read_text(encoding="utf-8")
        for credit in ("enwiktionary", "eswiktionary", "frwiktionary", "wordfreq", "UD French-GSD"):
            self.assertIn(credit, notice)
        self.assertIn("The levels are estimated, not taken from a CEFR list", notice)
        for list_ in ("CEFR-J", "Octanove", "FLELex"):
            self.assertNotIn(list_, notice)

    def test_reduce_fr_es_runs_the_passes_in_order(self):
        # Its apostrophes read as French's, its letters left out, then the edition's pre-pass, then
        # fr-es's, then the shared rules over the last one's file.
        work = self.dir / "work"
        studied = self.studied("maison\tmaison\n", "maison\t1\n")
        self.jsonl(MAISON, name="work/kaikki-es-Frances.jsonl")
        self.jsonl(name="work/kaikki-fr-traductions.jsonl")
        self.jsonl(name="work/kaikki-es-traductions.jsonl")
        calls = []

        def spy(name, real):
            def call(*args, **kwargs):
                calls.append((name, *(Path(arg).name for arg in args if isinstance(arg, str))))
                return real(*args, **kwargs)

            return call

        with (
            mock.patch.object(fr_es, "straight_apostrophes", spy("straight_apostrophes", fr_es.straight_apostrophes)),
            mock.patch.object(common, "without_letter_senses", spy("without_letter_senses", common.without_letter_senses)),
            mock.patch.object(spanish, "read_as_meanings", spy("read_as_meanings", spanish.read_as_meanings)),
            mock.patch.object(fr_es, "french_entries", spy("french_entries", fr_es.french_entries)),
            mock.patch.object(fr_es, "native_side", spy("native_side", fr_es.native_side)),
        ):
            self.reduce(work, studied)
        self.assertEqual(
            calls,
            [
                ("straight_apostrophes", "kaikki-es-Frances.jsonl", "kaikki-es-Frances-apostrophes.jsonl"),
                ("without_letter_senses", "kaikki-es-Frances-apostrophes.jsonl", "kaikki-es-Frances-words.jsonl"),
                ("read_as_meanings", "kaikki-es-Frances-words.jsonl", "kaikki-es-Frances-meanings.jsonl"),
                ("french_entries", "kaikki-es-Frances-meanings.jsonl", "kaikki-es-Frances-glossing.jsonl"),
                ("native_side", "kaikki-es-Frances-glossing.jsonl"),
            ],
        )
        self.assertEqual((work / "gloss.tsv").read_text(encoding="utf-8"), "maison\tCasa\n")

    def test_native_side_gives_the_tables_native_tables_gives(self):
        # The steps written out keep each source's lemmas and change nothing of the tables, when no
        # definition repeats its headword.
        entries = self.section(MAISON, PIERRE, VENIR)
        ranks = {"maison": 1, "pierre": 2, "venir": 3, "intérêt": 4, "travers": 5, "rien": 6}
        direct = self.direct(INTERET_TRANSLATED, VENIR_TRANSLATED, ALLER_DE_L_AVANT_TRANSLATED)
        inverted = self.inverted(TRAVES_LISTS, CASA_LISTS, GOLPE_MILITAR_LISTS)
        sources = [(direct, list), (inverted, fr_es.by_spanish_frequency(spanish_zipf))]
        glosses, runs, expressions, steps = fr_es.native_side(entries, ranks, sources)
        shared = common.native_tables(
            entries, ranks, studied=fr_es.FR, edition=fr_es.EDITION, fallbacks=sources, locutions=fr_es.LOCUTIONS
        )
        self.assertEqual((glosses, runs, expressions), shared[:3])
        self.assertEqual(
            steps,
            {"entries": {"maison", "pierre", "venir"}, "yielded": set(), "direct": {"intérêt"}, "inverted": {"travers"}},
        )
        self.assertEqual(len(steps["entries"]), shared[3], "`primary`")

    def test_the_shares_count_what_they_say(self):
        steps = {"entries": {"a", "c", "z"}, "direct": {"b", "y"}, "inverted": {"d"}, "yielded": set()}
        ranks = {"a": 1, "b": 2, "c": 3, "d": 4, "y": 5, "z": 6}
        self.assertEqual(
            fr_es.translation_share(steps, ranks, top=4),
            {"top": 4, "glossed": 4, "share": 50.0, "direct": ["b"], "inverted": ["d"]},
        )
        self.assertEqual(fr_es.translation_share(steps, ranks)["share"], 50.0)
        self.assertEqual(fr_es.definition_share(steps), 50.0)
        empty = {"entries": set(), "direct": set(), "inverted": set()}
        self.assertEqual((fr_es.translation_share(empty, ranks)["share"], fr_es.definition_share(empty)), (0.0, 0.0))

    def test_fr_es_s_copy_of_french_is_fr_en_s(self):
        # reduce-fr-es.py repeats fr-en's description of French (the token, the coordinators, the
        # form-of target) because a reducer loads no other pair's: the two must not drift.
        self.assertEqual(fr_es.FR, fr_en.FR)
        self.assertEqual(
            (fr_es.FR.token.pattern, fr_es.FR.form_of_target.pattern, fr_es.FR.coordinators),
            (fr_en.FR.token.pattern, fr_en.FR.form_of_target.pattern, fr_en.FR.coordinators),
        )
        self.assertIs(fr_es.EDITION, ES)
        self.assertEqual(fr_es.LOCUTIONS, {})
        self.assertEqual(fr_es.analyser_version(), fr_en.analyser_version())

    def test_fr_es_reads_the_committed_studied_tables_as_fr_en_writes_them(self):
        # A folder whose forms and ranks disagree is refused, naming both; the lemmas are capped by
        # rank; a folder lacking one is named.
        studied = self.studied("maison\tmaison\nmaisons\tmaison\n", "maison\t1\ncasa\t2\n")
        with self.assertRaisesRegex(SystemExit, r"freq\.tsv and .*forms\.tsv disagree on the lemmas"):
            fr_es.read_studied(str(studied), 60000)
        studied = self.studied("et\tet\nmaison\tmaison\nmaisons\tmaison\n", "et\t1\nmaison\t2\n")
        self.assertEqual(fr_es.read_studied(str(studied), 60000), {"et": 1, "maison": 2})
        self.assertEqual(fr_es.read_studied(str(studied), 1), {"et": 1})
        (studied / "freq.tsv").unlink()
        with self.assertRaisesRegex(SystemExit, "freq.tsv is missing"):
            fr_es.read_studied(str(studied), 60000)
        # The committed folder: every lemma French commits within build.sh's cap.
        committed = Path(_HERE, "tables", "fr")
        ranks = fr_es.read_studied(str(committed), 60000)
        ranked = sum(1 for line in (committed / "freq.tsv").read_text(encoding="utf-8").splitlines() if "\t" in line)
        self.assertEqual(len(ranks), ranked)
        self.assertTrue(fr_es.read_readings(str(committed)), "French's committed readings")

    def test_spec_scenario_a_rule_of_the_spanish_edition(self):
        # fr-es's rules are its reducer, the shared rules and the Spanish edition's — no other
        # pair's reducer.
        import pack_sources as ps

        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-fr-es.py")],
            ["reduce-fr-es.py", "reduce_common.py", "reduce_edition_es.py"],
        )


if __name__ == "__main__":
    unittest.main()
