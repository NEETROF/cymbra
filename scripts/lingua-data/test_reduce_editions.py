# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the Wiktionary editions' rules (generalise-lingua-gloss-reducer).

The senses are recorded from real kaikki data, cut down to the fields the rules read: the English
Wiktionary's Spanish section (the 2026-09-28 extract es-fr pins, and for `policía`, the letters and
the full entries its 2026-10-03 one, which es-en pins) and the Spanish Wiktionary's English entries
(its 2026-10-02 dump, which es-fr pins). A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import contextlib
import dataclasses
import gzip
import importlib.util
import io
import json
import os
import subprocess
import sys
import tempfile
import types
import unittest
from pathlib import Path
from unittest import mock

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)

import pack_sources as ps  # noqa: E402
import reduce_common as common  # noqa: E402
import reduce_edition_en as english  # noqa: E402
from reduce_edition_en import EN  # noqa: E402
from reduce_edition_es import ES  # noqa: E402
from reduce_edition_fr import FR  # noqa: E402


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(
        f"reduce_{pair.replace('-', '_')}_editions", os.path.join(_HERE, f"reduce-{pair}.py")
    )
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


en_fr = _reducer("en-fr")
es_fr = _reducer("es-fr")
es_en = _reducer("es-en")
# The studied languages, as the pairs that study them describe them.
ENGLISH = en_fr.EN
SPANISH = es_fr.ES


class Entries(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def jsonl(self, *entries, name="entries.jsonl"):
        path = self.dir / name
        path.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        return str(path)

    def gloss(self, edition, studied, lemmas, *entries):
        runs = {}
        glosses = common.reduce_gloss(
            self.jsonl(*entries), set(lemmas), **common.WORD_GLOSS, runs=runs, studied=studied, edition=edition
        )
        return glosses, runs


# — The English Wiktionary's Spanish section —

CASA = {"word": "casa", "pos": "noun", "senses": [{"glosses": ["house"], "tags": ["feminine"]}]}
CASAS = {
    "word": "casas",
    "pos": "noun",
    "senses": [
        {"glosses": ["plural of casa"], "tags": ["feminine", "form-of", "plural"], "form_of": [{"word": "casa"}]}
    ],
}
POS_CONJ = {"word": "pos", "pos": "conj", "senses": [{"glosses": ["synonym of pues"], "tags": ["colloquial"]}]}
POS_PREP = {"word": "pos", "pos": "prep", "senses": [{"glosses": ["after, behind"], "tags": ["colloquial"]}]}
TESAURO = {
    "word": "tesauro",
    "pos": "noun",
    "senses": [
        {
            "glosses": ["thesaurus (hierarchical index of metadata pointing to information bearing entities)"],
            "tags": ["masculine"],
        },
        {"glosses": ["synonym of diccionario de sinónimos"], "tags": ["masculine", "rare"]},
    ],
}
QUORUM = {
    "word": "quórum",
    "pos": "noun",
    "senses": [
        {
            "glosses": ["superseded spelling of cuórum"],
            "tags": ["alt-of", "archaic", "masculine"],
            "alt_of": [{"word": "cuórum"}],
        }
    ],
}
CUORUM = {
    "word": "cuórum",
    "pos": "noun",
    "senses": [{"glosses": ["quorum (minimum number of votes)"], "tags": ["masculine"]}],
}
ANGULAR_ADJ = {"word": "angular", "pos": "adj", "senses": [{"glosses": ["angular"], "tags": ["feminine", "masculine"]}]}
ANGULAR_VERB = {
    "word": "angular",
    "pos": "verb",
    "senses": [{"raw_glosses": ["(transitive)"], "tags": ["empty-gloss", "no-gloss", "transitive"]}],
}
CONGERIES = {"word": "congeries", "pos": "noun", "senses": [{"tags": ["feminine", "no-gloss"]}]}
DE_LETTER = {
    "word": "de",
    "pos": "noun",
    "senses": [{"glosses": ["The name of the Latin script letter D/d."], "tags": ["feminine"]}],
}
# A word of two noun etymologies, as kaikki writes it: one entry each (2026-10-03 extract).
POLICIA_1 = {
    "word": "policía",
    "pos": "noun",
    "senses": [
        {"glosses": ["Civility, polity, public order, police, fineness, neatness, urbanity"], "tags": ["feminine"]},
        {"glosses": ["police, police department, police force, police service"], "tags": ["feminine"]},
    ],
}
POLICIA_2 = {
    "word": "policía",
    "pos": "noun",
    "senses": [
        {
            "glosses": ["police officer (a member of a police force)"],
            "tags": ["by-personal-gender", "feminine", "masculine"],
        }
    ],
}
# Spanish letters, as the English Wiktionary's Spanish section and the Spanish Wiktionary's English
# translations write them (add-lingua-pack-es-en).
A_NOUN = {"word": "A", "pos": "noun", "senses": [{"glosses": ["bishop"], "tags": ["masculine", "uncountable"]}]}
A_PREP = {
    "word": "a",
    "pos": "prep",
    "senses": [
        {"glosses": ["to"]},
        {"glosses": ["by"]},
        {"glosses": ["at"]},
        {
            "glosses": [
                "Used before words referring to people, pets, or personified objects or places that function as "
                "direct objects: personal a."
            ]
        },
    ],
}
C_NOUN = {
    "word": "C",
    "pos": "noun",
    "senses": [
        {
            "glosses": ["abbreviation of caballo (“knight”): K"],
            "tags": ["abbreviation", "alt-of", "masculine"],
            "alt_of": [{"word": "caballo", "extra": "(“knight”): K"}],
        }
    ],
}
CABALLO = {
    "word": "caballo",
    "pos": "noun",
    "senses": [
        {"glosses": ["horse"], "tags": ["masculine"]},
        {"glosses": ["knight"], "tags": ["masculine"]},
        {"glosses": ["heroin"], "tags": ["masculine", "slang"]},
    ],
}
R_NOUN = {"word": "r", "pos": "noun", "senses": [{"glosses": ["the letter r"], "tags": ["feminine"]}]}
JUEVES = {
    "word": "jueves",
    "pos": "noun",
    "senses": [
        {"glosses": ["Thursday"], "tags": ["masculine"]},
        {"glosses": ["the letter J in the Spanish spelling alphabet"], "tags": ["masculine"]},
    ],
}
ESPANA = {
    "word": "España",
    "pos": "name",
    "senses": [
        {"glosses": ["Spain (a country in Southern Europe, including most of the Iberian peninsula)"], "tags": ["feminine"]},
        {"glosses": ["Peninsular Spain"], "tags": ["Canary-Islands", "colloquial", "feminine"]},
        {"glosses": ["the letter E in the Spanish spelling alphabet"], "tags": ["feminine"]},
    ],
}
ZETA = {
    "word": "zeta",
    "pos": "noun",
    "senses": [
        {"glosses": ["the letter Z"], "tags": ["feminine"]},
        {"glosses": ["zeta; the Greek letter Ζ, ζ"], "tags": ["feminine"]},
    ],
}
B_TRANSLATED = {"pos": "character", "translations": [{"word": "b"}], "word": "b"}
G_TRANSLATED = {"pos": "character", "translations": [{"word": "g"}], "word": "g"}
O_TRANSLATED = {"pos": "conj", "translations": [{"word": "or"}], "word": "o"}
I_TRANSLATED = {"pos": "noun", "translations": [{"sense": "letra", "word": "i"}], "word": "i"}
# Full entries, every field kaikki writes (the 2026-10-03 extract es-en pins): what `native_fields`
# cuts down.
FULL_QUORUM = json.loads(
    r'''{"pos": "noun", "head_templates": [{"name": "es-noun", "args": {"1": "m", "2": "+,#,#es<l:proscribed>"}, "expansion": "quórum m (plural quórums or quórum or (proscribed) quórumes)"}], "forms": [{"form": "quórums", "tags": ["plural"]}, {"form": "quórum", "tags": ["plural"]}, {"form": "quórumes", "tags": ["plural", "proscribed"]}], "etymology_text": "Borrowed from Latin quōrum, genitive plural form of quī (“who, which”).", "etymology_links": [["quōrum", "quorum#Latin"], ["quī", "qui#Latin"]], "etymology_templates": [{"name": "bor+", "args": {"1": "es", "2": "la", "3": "quōrum"}, "expansion": "Borrowed from Latin quōrum"}], "sounds": [{"ipa": "/ˈkwoɾum/"}, {"ipa": "[ˈkwo.ɾũm]"}, {"rhymes": "-oɾum"}], "hyphenation": ["quó‧rum"], "hyphenations": [{"parts": ["quó‧rum"]}], "word": "quórum", "lang": "Spanish", "lang_code": "es", "senses": [{"links": [["cuórum", "cuórum#Spanish"]], "glosses": ["superseded spelling of cuórum"], "tags": ["alt-of", "archaic", "masculine"], "alt_of": [{"word": "cuórum"}], "id": "en-quórum-es-noun-CxQDwg2j", "categories": [{"name": "Pages with 2 entries", "kind": "other", "parents": [], "source": "w"}, {"name": "Pages with entries", "kind": "other", "parents": [], "source": "w"}, {"name": "Spanish entries with incorrect language header", "kind": "other", "parents": [], "source": "w"}, {"name": "Spanish nouns with multiple plurals", "kind": "other", "parents": [], "source": "w"}]}]}'''
)
FULL_CUORUM = json.loads(
    r'''{"pos": "noun", "head_templates": [{"name": "es-noun", "args": {"1": "m", "2": "+,#"}, "expansion": "cuórum m (plural cuórums or cuórum)"}], "forms": [{"form": "cuórums", "tags": ["plural"]}, {"form": "cuórum", "tags": ["plural"]}, {"form": "quorum", "tags": ["alternative"]}, {"form": "quórum", "tags": ["alternative"]}], "sounds": [{"ipa": "/ˈkwoɾum/"}, {"ipa": "[ˈkwo.ɾũm]"}, {"rhymes": "-oɾum"}], "hyphenation": ["cuó‧rum"], "hyphenations": [{"parts": ["cuó‧rum"]}], "word": "cuórum", "lang": "Spanish", "lang_code": "es", "senses": [{"links": [["quorum", "quorum"]], "glosses": ["quorum (minimum number of votes)"], "tags": ["masculine"], "id": "en-cuórum-es-noun-kCTyEEVn", "categories": [{"name": "Pages with 1 entry", "kind": "other", "parents": [], "source": "w"}, {"name": "Pages with entries", "kind": "other", "parents": [], "source": "w"}, {"name": "Spanish entries with incorrect language header", "kind": "other", "parents": [], "source": "w"}, {"name": "Spanish nouns with multiple plurals", "kind": "other", "parents": [], "source": "w"}]}]}'''
)
FULL_DAR_DE_ALTA = json.loads(
    r'''{"pos": "verb", "head_templates": [{"name": "es-verb", "args": {}, "expansion": "dar de alta (first-person singular present doy de alta, first-person singular preterite di de alta, past participle dado de alta)"}], "forms": [{"form": "doy de alta", "tags": ["first-person", "present", "singular"]}, {"form": "di de alta", "tags": ["first-person", "preterite", "singular"]}, {"form": "dado de alta", "tags": ["participle", "past"]}, {"form": "dar el alta", "tags": ["alternative"]}, {"form": "darse de alta", "tags": ["alternative", "reflexive"]}], "word": "dar de alta", "lang": "Spanish", "lang_code": "es", "sounds": [{"ipa": "/ˌdaɾ de ˈalta/"}, {"ipa": "[ˌd̪aɾ ð̞e ˈal̪.t̪a]"}], "hyphenation": ["dar de al‧ta"], "hyphenations": [{"parts": ["dar de al‧ta"]}], "senses": [{"links": [["discharge", "discharge"]], "synonyms": [{"word": "dar de baja"}], "raw_glosses": ["(transitive, idiomatic) to discharge (to release a patient from the hospital)"], "glosses": ["to discharge (to release a patient from the hospital)"], "tags": ["idiomatic", "transitive"], "id": "en-dar_de_alta-es-verb-rcPzMjq1", "categories": [{"name": "Pages with 1 entry", "kind": "other", "parents": [], "source": "w+disamb", "_dis": "51 49"}, {"name": "Pages with entries", "kind": "other", "parents": [], "source": "w+disamb", "_dis": "57 43"}, {"name": "Spanish entries with incorrect language header", "kind": "other", "parents": [], "source": "w+disamb", "_dis": "67 33"}]}, {"links": [["register", "register"], ["sign up", "sign up"]], "antonyms": [{"word": "dar de baja"}], "raw_glosses": ["(transitive, reflexive, idiomatic) to register as, to sign up for (to join a service or an organization)"], "glosses": ["to register as, to sign up for (to join a service or an organization)"], "tags": ["idiomatic", "reflexive", "transitive"], "id": "en-dar_de_alta-es-verb-FJ2AIxqK", "categories": [{"name": "Pages with 1 entry", "kind": "other", "parents": [], "source": "w+disamb", "_dis": "51 49"}]}]}'''
)
SEPULTURA = {
    "word": "sepultura",
    "pos": "noun",
    "senses": [
        {"glosses": ["the act or state of burial"], "tags": ["feminine"]},
        {"glosses": ["grave (a hole made in the Earth to bury a corpse)"], "tags": ["feminine"]},
    ],
}

# — The Spanish Wiktionary's English entries —

CHIPS = {
    "word": "chips",
    "pos": "noun",
    "senses": [{"glosses": ["Forma del plural de chip."], "tags": ["form-of"], "form_of": [{"word": "chip"}]}],
}
READ_VERB = {"word": "read", "pos": "verb", "senses": [{"glosses": ["Leer."]}, {"glosses": ["Ver."]}]}
READ_FORM = {
    "word": "read",
    "pos": "verb",
    "senses": [
        {"glosses": ["Pasado simple del verbo (to) read."]},
        {"glosses": ["Participio pasado del verbo (to) read."]},
    ],
}
READ_NOUN = {"word": "read", "pos": "noun", "senses": [{"glosses": ["Lectura."]}]}
ALLOW = {
    "word": "allow",
    "pos": "verb",
    "senses": [{"glosses": ["Permitir, dejar₉."]}, {"glosses": ["Tener en cuenta al hacer planes."]}],
}
CAPITAL = {
    "word": "capital",
    "pos": "adj",
    "senses": [
        {"glosses": ["Relacionado con la cabeza."]},
        {"glosses": ["Propio o relacionado con el capital₆."]},
        {"glosses": ["Principal o mayor."]},
    ],
}
WOMAN = {"word": "woman", "pos": "adj", "senses": [{"glosses": ["Femenino."]}]}
OUT = {
    "word": "out",
    "pos": "adj",
    "senses": [
        {"glosses": ["Equivocado."]},
        {"glosses": ["En huelga."]},
        {"glosses": ["Pasado de moda."]},
        {"glosses": ["Inaceptable."]},
    ],
}
BLOW = {"word": "blow", "pos": "noun", "senses": [{"glosses": ["Golpe. Véase también el tesauro."]}]}
CUE = {
    "word": "cue",
    "pos": "noun",
    "senses": [{"glosses": ["Indicación."]}, {"glosses": ["Estímulo."]}, {"glosses": ["Nombre de la letra Q."]}],
}


class TheFrenchEdition(unittest.TestCase):
    """Today's rules, unchanged: en-fr and es-fr reduce byte for byte with them."""

    def test_it_holds_today_s_rules_by_identity(self):
        self.assertIs(en_fr._FORM_OF, FR.form_of)
        self.assertIs(en_fr._MWE_FORM_OF, FR.mwe_form_of)
        self.assertIs(en_fr.EDITION, FR)
        self.assertIs(es_fr.EDITION, FR)
        self.assertEqual(FR.code, "fr")
        self.assertEqual((FR.pointer_tags, FR.pointer_fields), (frozenset({"form-of"}), ("form_of",)))
        self.assertTrue(FR.capitalised)
        self.assertEqual(FR.long_parenthesis, 0)

    def test_the_pairs_glossed_in_french_read_it_by_default(self):
        # A pointer, then a coordinator left hanging once it went.
        self.assertEqual(en_fr.clean_gloss("(→ voir bone marrow) ou Moelle.", 80), "Moelle")
        self.assertEqual(en_fr.strip_wiki_notes("Définition manquante ou à compléter. (Ajouter)"), "")
        self.assertTrue(en_fr._is_form_of({}, "Pluriel de datum."))
        self.assertEqual(es_fr.translation_gloss({"NOUN": ["maison"]}, list), ("Maison", [("NOUN", 1)]))

    def test_a_shared_rule_is_told_its_edition(self):
        # The shared module loads no edition, so it has none to fall back on: a pair names its own.
        with self.assertRaises(TypeError):
            common.clean_gloss("Maison.", 80)
        with self.assertRaises(TypeError):
            common._is_form_of({}, "Pluriel de datum.")

    def test_the_shared_rules_load_no_edition(self):
        loaded = subprocess.run(
            [
                sys.executable,
                "-B",
                "-c",
                "import reduce_common, sys; print(sorted(m for m in sys.modules if m.startswith('reduce_')))",
            ],
            cwd=_HERE,
            capture_output=True,
            text=True,
            check=True,
        ).stdout
        self.assertEqual(loaded.strip(), "['reduce_common']")

    def test_it_reads_another_edition_s_senses_as_it_always_did(self):
        # The English edition's pointers are not the French one's: an `alt-of` sense and an untagged
        # « synonym of » are meanings to the French rules, which moved by no byte.
        self.assertFalse(common._is_form_of(QUORUM["senses"][0], "superseded spelling of cuórum", edition=FR))
        self.assertFalse(common._is_form_of(POS_CONJ["senses"][0], "synonym of pues", edition=FR))
        self.assertTrue(common._is_form_of(QUORUM["senses"][0], "superseded spelling of cuórum", edition=EN))


class TheEnglishEdition(Entries):
    def test_a_plural_of_is_a_form_of_its_lemma(self):
        sense = CASAS["senses"][0]
        self.assertTrue(common._is_form_of(sense, "plural of casa", edition=EN))
        self.assertTrue(EN.form_of.match("plural of casa"), "its wording alone says so")
        pointers = {}
        entries = common._read_entries(self.jsonl(CASAS), {"casas"}, 300, pointers, studied=SPANISH, edition=EN)
        self.assertEqual(entries, {}, "no meaning")
        self.assertEqual(pointers, {"casas": [("casa", "NOUN")]})
        # A lemma kept with nothing but a pointer borrows its base's senses, as in French.
        glosses, _ = self.gloss(EN, SPANISH, {"casas"}, CASA, CASAS)
        self.assertEqual(glosses, {"casas": "house"})

    def test_an_untagged_synonym_of_names_a_word_of_the_studied_language(self):
        glosses, runs = self.gloss(EN, SPANISH, {"pos", "tesauro"}, POS_CONJ, POS_PREP, TESAURO)
        self.assertEqual(glosses["pos"], "after, behind")
        self.assertEqual(runs["pos"], [("ADP", 1)])
        self.assertNotIn("synonym", glosses["tesauro"])

    def test_an_alt_of_sense_points_at_the_word_it_names(self):
        sense = QUORUM["senses"][0]
        self.assertTrue(common._is_form_of(sense, sense["glosses"][0], edition=EN))
        glosses, _ = self.gloss(EN, SPANISH, {"quórum"}, QUORUM, CUORUM)
        self.assertEqual(glosses, {"quórum": "quorum (minimum number of votes)"})

    def test_a_sense_with_no_gloss_is_left_out(self):
        glosses, runs = self.gloss(EN, SPANISH, {"angular", "congeries"}, ANGULAR_ADJ, ANGULAR_VERB, CONGERIES)
        self.assertEqual(glosses, {"angular": "angular"}, "a word left with no sense has no gloss")
        self.assertEqual(runs["angular"], [("ADJ", 1)])

    def test_english_glosses_keep_their_lower_case(self):
        glosses, _ = self.gloss(EN, SPANISH, {"tesauro", "casa"}, TESAURO, CASA)
        self.assertEqual(glosses["casa"], "house")
        self.assertTrue(glosses["tesauro"].startswith("thesaurus ("))
        self.assertEqual(
            common.translation_gloss({"NOUN": ["house", "home"]}, list, edition=EN), ("house, home", [("NOUN", 1)])
        )

    def test_a_letter_s_name_is_no_gloss(self):
        out = common.without_letter_senses(self.jsonl(DE_LETTER, CASA), str(self.dir / "out.jsonl"), edition=EN)
        self.assertEqual([json.loads(line)["word"] for line in Path(out).read_text().splitlines()], ["casa"])

    def test_a_sense_that_only_names_a_letter_is_no_gloss(self):
        # Recorded: `r` glossed « the letter r », and a word's place in the Spanish spelling
        # alphabet (36 of es-en's glossed lemmas ended on one). Another sense naming a letter
        # inside it stays a meaning, and so does « the letter » followed by a word.
        out = common.without_letter_senses(
            self.jsonl(R_NOUN, JUEVES, ESPANA, ZETA), str(self.dir / "out.jsonl"), edition=EN
        )
        glosses, _ = self.gloss(EN, SPANISH, {"r", "jueves", "españa", "zeta"}, *map(json.loads, Path(out).read_text().splitlines()))
        self.assertEqual(
            glosses,
            {
                "jueves": "Thursday",
                "españa": "Spain (a country in Southern Europe, including most of the Iberian peninsula); Peninsular Spain",
                "zeta": "zeta, the Greek letter Ζ, ζ",
            },
        )
        for meaning in ("to roll the letter R", "the letter of the law", "the letter-writer"):
            self.assertIsNone(EN.letter.search(meaning), meaning)
        # Read as written, without the rule, each would gloss its word (the committed tables did).
        written, _ = self.gloss(EN, SPANISH, {"r", "jueves"}, R_NOUN, JUEVES)
        self.assertEqual(written, {"r": "the letter r", "jueves": "Thursday; the letter J in the Spanish spelling alphabet"})

    def test_a_single_capital_letter_glosses_no_word(self):
        # Recorded: `A` (the chess bishop) and `C` (an abbreviation of `caballo`, alt-of). Read as
        # written, `a` opens on « bishop » and `c` borrows the gloss of `caballo`.
        written, _ = self.gloss(EN, SPANISH, {"a", "c", "caballo"}, A_NOUN, A_PREP, C_NOUN, CABALLO)
        self.assertTrue(written["a"].startswith("bishop; to; by; at"), written["a"])
        self.assertEqual(written["c"], "horse; knight; heroin")
        kept = english.without_letter_headwords(
            self.jsonl(A_NOUN, A_PREP, C_NOUN, CABALLO), str(self.dir / "headwords.jsonl")
        )
        self.assertEqual([json.loads(line)["word"] for line in Path(kept).read_text().splitlines()], ["a", "caballo"])
        glosses = common.reduce_gloss(kept, {"a", "c", "caballo"}, **common.WORD_GLOSS, studied=SPANISH, edition=EN)
        self.assertTrue(glosses["a"].startswith("to; by; at; Used before words"), glosses["a"])
        self.assertNotIn("c", glosses)
        self.assertEqual(glosses["caballo"], "horse; knight; heroin")
        # An acronym of two capitals or more is the shared rule's; a line it cannot read is passed on.
        made_up = self.dir / "made-up.jsonl"
        made_up.write_text('{"word": "UE", "pos": "name", "senses": []}\nnot json\n[1]\n', encoding="utf-8")
        kept = english.without_letter_headwords(str(made_up), str(self.dir / "headwords.jsonl"))
        self.assertEqual(Path(kept).read_text(encoding="utf-8"), made_up.read_text(encoding="utf-8"))

    def test_no_placeholder_and_no_dangling_coordinator(self):
        # The heraldic « or » is a meaning (made up from the census's 13 such senses).
        self.assertEqual(common.clean_gloss("or (the tincture)", 80, edition=EN), "or (the tincture)")

    def test_long_parentheses_stay_until_m20_is_settled(self):
        grave = SEPULTURA["senses"][1]["glosses"][0]
        self.assertEqual(common.clean_gloss(grave, 300, whole_words=True, edition=EN), grave)
        m20 = dataclasses.replace(EN, long_parenthesis=40)
        self.assertEqual(common.clean_gloss(grave, 300, whole_words=True, edition=m20), "grave")
        self.assertEqual(
            common.clean_gloss("quorum (minimum number of votes)", 300, edition=m20), "quorum (minimum number of votes)"
        )


class TheEnglishEditionSettings(Entries):
    """The two settings es-en's review decides (add-lingua-pack-es-en D5, M20): settings of the
    English edition, which no pair glossed in French loads."""

    def test_both_are_committed_at_their_defaults(self):
        self.assertEqual((english.LONG_PARENTHESIS, EN.long_parenthesis), (0, 0))
        self.assertFalse(english.MERGE_SAME_POS_ETYMOLOGIES)
        # Off, the pre-pass reads nothing and writes nothing: the reducer reads the file as written.
        src = self.jsonl(POLICIA_1, POLICIA_2)
        dst = self.dir / "merged.jsonl"
        self.assertEqual(english.merge_same_pos_etymologies(src, str(dst)), src)
        self.assertFalse(dst.exists())

    def test_the_pre_pass_merges_a_word_s_etymologies_of_one_part_of_speech(self):
        # Recorded: `policía`'s two noun entries. Read as written, the round-robin takes the first
        # sense of each before the second of the first; merged, the first etymology's come first.
        written, _ = self.gloss(EN, SPANISH, {"policía"}, POLICIA_1, POLICIA_2)
        self.assertEqual(
            written["policía"],
            "Civility, polity, public order, police, fineness, neatness, urbanity; "
            "police officer (a member of a police force); police, police department, police force, police service",
        )
        merged = english.merge_same_pos_etymologies(
            self.jsonl(POLICIA_1, POLICIA_2), str(self.dir / "merged.jsonl"), merged=True
        )
        lines = [json.loads(line) for line in Path(merged).read_text(encoding="utf-8").splitlines()]
        self.assertEqual(len(lines), 1)
        self.assertEqual(lines[0]["senses"], POLICIA_1["senses"] + POLICIA_2["senses"])
        runs = {}
        glosses = common.reduce_gloss(merged, {"policía"}, **common.WORD_GLOSS, runs=runs, studied=SPANISH, edition=EN)
        self.assertEqual(
            glosses["policía"],
            "Civility, polity, public order, police, fineness, neatness, urbanity; "
            "police, police department, police force, police service; police officer (a member of a police force)",
        )
        self.assertEqual(runs["policía"], [("NOUN", 3)])

    def test_the_pre_pass_keeps_a_sense_less_entry_in_its_place_and_leaves_out_non_objects(self):
        # Made up: a noun entry with no senses before two with senses, and lines that are no JSON
        # object. The sense-less entry merges into none; the later nouns merge into it, the first
        # noun of the word.
        empty = {"word": "policía", "pos": "noun", "head_templates": [{"name": "es-noun"}]}
        verb = {"word": "policía", "pos": "verb", "senses": []}
        src = self.dir / "entries.jsonl"
        src.write_text(
            "".join(json.dumps(e, ensure_ascii=False) + "\n" for e in (empty, verb, POLICIA_1))
            + "not json\n[1, 2]\n\"a string\"\n"
            + json.dumps(POLICIA_2, ensure_ascii=False)
            + "\n",
            encoding="utf-8",
        )
        merged = english.merge_same_pos_etymologies(str(src), str(self.dir / "merged.jsonl"), merged=True)
        lines = [json.loads(line) for line in Path(merged).read_text(encoding="utf-8").splitlines()]
        self.assertEqual(
            lines,
            [
                {**empty, "senses": POLICIA_1["senses"] + POLICIA_2["senses"]},
                verb,
            ],
        )

    def test_the_pre_pass_keeps_other_parts_of_speech_and_other_spellings_apart(self):
        # Made up: a verb entry between the nouns, and an acronym spelled like the word.
        verb = {"word": "policía", "pos": "verb", "senses": [{"glosses": ["inflection of policiar:"]}]}
        acronym = {"word": "POLICÍA", "pos": "noun", "senses": [{"glosses": ["an acronym"]}]}
        merged = english.merge_same_pos_etymologies(
            self.jsonl(POLICIA_1, verb, POLICIA_2, acronym, CASA), str(self.dir / "merged.jsonl"), merged=True
        )
        lines = [json.loads(line) for line in Path(merged).read_text(encoding="utf-8").splitlines()]
        self.assertEqual([(e["word"], e["pos"], len(e["senses"])) for e in lines], [
            ("policía", "noun", 3),
            ("policía", "verb", 1),
            ("POLICÍA", "noun", 1),
            ("casa", "noun", 1),
        ])

    def test_spec_scenario_a_setting_of_the_english_edition(self):
        # Either setting changed re-pins es-en alone: it is in es-en's rules and in no French-native
        # pair's, and reduce_common.py, which every pair loads, is not edited for it.
        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-es-en.py")],
            ["reduce-es-en.py", "reduce_common.py", "reduce_edition_en.py"],
        )
        copy = self.dir / "rules"
        copy.mkdir()
        for path in Path(_HERE).glob("reduce[-_]*.py"):
            (copy / path.name).write_bytes(path.read_bytes())
        pairs = ("en-fr", "es-fr", "es-en")
        before = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        self.assertEqual(before, {pair: ps.rules_sha256(Path(_HERE) / f"reduce-{pair}.py") for pair in pairs})
        for name, old, new in (
            ("reduce_edition_en.py", "LONG_PARENTHESIS = 0\n", "LONG_PARENTHESIS = 40\n"),
            ("reduce_edition_en.py", "MERGE_SAME_POS_ETYMOLOGIES = False\n", "MERGE_SAME_POS_ETYMOLOGIES = True\n"),
            # The letters' rules of add-lingua-pack-es-en, the English edition's and es-en's own.
            ("reduce_edition_en.py", r"|the letter \w\b)", ")"),
            ("reduce_edition_en.py", "len(headword) == 1 and headword.isupper()", "False"),
            ("reduce-es-en.py", 'if entry.get("pos") == "character":', "if False:"),
        ):
            edition = copy / name
            text = edition.read_text(encoding="utf-8")
            self.assertIn(old, text)
            edition.write_text(text.replace(old, new), encoding="utf-8")
            after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
            self.assertNotEqual(after["es-en"], before["es-en"], new)
            self.assertEqual((after["en-fr"], after["es-fr"]), (before["en-fr"], before["es-fr"]), new)
            before = after


class TheSpanishEdition(Entries):
    def test_forma_del_plural_de_is_a_form_of(self):
        sense = CHIPS["senses"][0]
        self.assertTrue(common._is_form_of(sense, "Forma del plural de chip.", edition=ES))
        self.assertTrue(ES.form_of.match("Forma del plural de chip."), "its wording alone says so")
        pointers = {}
        common._read_entries(self.jsonl(CHIPS), {"chips"}, 300, pointers, studied=ENGLISH, edition=ES)
        self.assertEqual(pointers, {"chips": [("chip", "NOUN")]})

    def test_an_untagged_tense_of_a_verb_is_a_form_of_it(self):
        for gloss in ("Participio pasado del verbo (to) read.", "Pasado simple del verbo (to) read."):
            self.assertTrue(common._is_form_of({"glosses": [gloss]}, gloss, edition=ES), gloss)
        glosses, runs = self.gloss(ES, ENGLISH, {"read"}, READ_VERB, READ_NOUN, READ_FORM)
        self.assertEqual(glosses["read"], "Leer; Ver; Lectura")
        self.assertEqual(runs["read"], [("VERB", 2), ("NOUN", 1)])

    def test_a_wording_without_its_de_stays_a_meaning(self):
        glosses, _ = self.gloss(ES, ENGLISH, {"woman", "out"}, WOMAN, OUT)
        self.assertEqual(glosses["woman"], "Femenino")
        self.assertEqual(glosses["out"], "Equivocado; En huelga; Pasado de moda; Inaceptable")

    def test_a_sense_link_subscript_goes(self):
        glosses, _ = self.gloss(ES, ENGLISH, {"allow", "capital"}, ALLOW, CAPITAL)
        self.assertEqual(glosses["allow"], "Permitir, dejar; Tener en cuenta al hacer planes")
        self.assertIn("Propio o relacionado con el capital;", glosses["capital"])
        # Made up: after a capital letter, a subscript is a formula's, and stays.
        self.assertEqual(common.clean_gloss("Butano (C₄H₁₀).", 80, edition=ES), "Butano (C₄H₁₀)")

    def test_a_link_goes_whole_wherever_it_sits(self):
        # Recorded: a range, two senses, after the word's own period, after a stray space.
        for gloss, cleaned in (
            ("Madrid₁₋₂.", "Madrid"),
            ("Rango₁₋₂, grado, nivel, posición.", "Rango, grado, nivel, posición"),
            ("Entusiasta₁₋₃, entusiástico₁.", "Entusiasta, entusiástico"),
            ("Ejercer el rol de bottom₉ o ₁₀.", "Ejercer el rol de bottom"),
            ("Rey.₃.", "Rey"),
            (
                "Multitud de objetos similares presentes en un espacio limitado; constelación.₅",
                "Multitud de objetos similares presentes en un espacio limitado; constelación",
            ),
            (
                "Persona o cosa de máxima calidad o importancia en algún aspecto.₄; Rey.",
                "Persona o cosa de máxima calidad o importancia en algún aspecto; Rey",
            ),
            ("Cafetería, café ₄.", "Cafetería, café"),
            # Made up: in a sense that goes on, the period closing the sentence stays.
            ("Golpe.₃ Choque.", "Golpe. Choque"),
        ):
            self.assertEqual(common.clean_gloss(gloss, 300, whole_words=True, edition=ES), cleaned, gloss)

    def test_a_subscript_after_a_preposition_is_the_sentence_s_word(self):
        # Recorded: it names one of the entry's own senses; without it, the sense reads « similar a en ».
        for gloss in ("Cualquier parte similar a ₁ en un animal.", "Disparar como en ₁."):
            self.assertEqual(common.clean_gloss(gloss, 300, whole_words=True, edition=ES), gloss.rstrip("."))

    def test_a_see_also_note_goes(self):
        self.assertEqual(common.clean_gloss(BLOW["senses"][0]["glosses"][0], 80, edition=ES), "Golpe")

    def test_a_letter_s_name_is_no_gloss(self):
        out = common.without_letter_senses(self.jsonl(CUE), str(self.dir / "out.jsonl"), edition=ES)
        senses = json.loads(Path(out).read_text())["senses"]
        self.assertEqual([s["glosses"][0] for s in senses], ["Indicación.", "Estímulo."])

    def test_a_translation_gloss_opens_on_a_capital(self):
        self.assertEqual(
            common.translation_gloss({"NOUN": ["casa", "vivienda"]}, list, edition=ES)[0], "Casa, vivienda"
        )


class AGlossIsWrittenInTheReadersLanguage(Entries):
    """lingua-data-packs: a gloss is in the reader's language, written by a person — never the
    studied language, a third language, a machine translation, or a pivot."""

    HOUSE = {
        "word": "house",
        "lang_code": "en",
        "pos": "noun",
        "senses": [
            {
                "glosses": ["A structure serving as an abode of human beings."],
                "translations": [
                    {"lang_code": "fr", "word": "maison", "sense": "abode of a human being"},
                    {"lang_code": "es", "word": "casa", "sense": "abode of a human being"},
                    {"lang_code": "es", "word": "vivienda", "sense": "abode of a human being"},
                ],
            }
        ],
    }

    def derive(self, *files):
        dump = self.dir / "enwiktionary.jsonl.gz"
        dump.write_bytes(gzip.compress((json.dumps(self.HOUSE, ensure_ascii=False) + "\n").encode()))
        ps.derive(dump, {name: spec for name, spec in files}, self.dir)

    def test_a_table_pairing_the_two_languages_directly_qualifies(self):
        self.derive(("en-es.jsonl", ("translations", "en", "es")))
        direct = common.read_translations(str(self.dir / "en-es.jsonl"), inverted=False, studied=ENGLISH)
        self.assertEqual(direct, {"house": {"NOUN": ["casa", "vivienda"]}})
        got = common.fallback_glosses({"house"}, {}, [(direct, list)], edition=ES)
        self.assertEqual(got, {"house": ("Casa, vivienda", [("NOUN", 1)])})

    def test_no_pivot_through_a_third_language(self):
        # The English entry lists both: each file pairs English with one language, and nothing joins
        # two files on the English word.
        self.derive(("en-es.jsonl", ("translations", "en", "es")), ("en-fr.jsonl", ("translations", "en", "fr")))
        spanish = common.read_translations(str(self.dir / "en-es.jsonl"), inverted=True, studied=SPANISH)
        self.assertEqual(spanish, {"casa": {"NOUN": ["house"]}, "vivienda": {"NOUN": ["house"]}})
        self.assertNotIn("maison", (self.dir / "en-es.jsonl").read_text(encoding="utf-8"))
        self.assertNotIn("casa", (self.dir / "en-fr.jsonl").read_text(encoding="utf-8"))

    def test_every_translation_file_a_pair_reads_pairs_its_studied_and_native_languages(self):
        for pair, dumps in ps.DUMPS.items():
            studied, native = pair.split("-")
            for dump in dumps.values():
                for name, (kind, lang, *into) in dump["files"].items():
                    if kind == "translations":
                        self.assertEqual({lang, *into}, {studied, native}, f"{pair}: {name}")
                    else:
                        self.assertEqual(lang, studied, f"{pair}: {name} holds entries of the studied language")

    def test_an_english_gloss_is_not_a_french_one(self):
        # es-fr reduced as its build runs it (`main()`), wordfreq aside. The English Wiktionary's
        # Spanish section — es-fr's forms — glosses all three words in English; the French Wiktionary
        # glosses `perro`, the Spanish one's table lists French `chat` for `gato`, and nothing written
        # in French glosses `casa`. `casa` has no gloss.
        work = self.dir / "work"
        work.mkdir()
        english = {"casa": "house", "perro": "dog", "gato": "cat"}
        sources = {
            "kaikki-Spanish.jsonl": [
                {"word": w, "lang_code": "es", "pos": "noun", "senses": [{"glosses": [g]}]} for w, g in english.items()
            ],
            "kaikki-fr-Espagnol.jsonl": [
                {"word": "perro", "lang_code": "es", "pos": "noun", "senses": [{"glosses": ["Chien."]}]}
            ],
            "kaikki-es-traductions.jsonl": [{"word": "gato", "pos": "noun", "translations": [{"word": "chat"}]}],
            "kaikki-fr-traductions.jsonl": [],
            "es_gsd-ud-train.conllu": [],
            "es_gsd-ud-dev.conllu": [],
        }
        for name, entries in sources.items():
            self.jsonl(*entries, name=f"work/{name}")
        wordfreq = types.ModuleType("wordfreq")
        wordfreq.top_n_list = lambda lang, n: list(english)[:n] if lang == "es" else []
        wordfreq.zipf_frequency = lambda word, lang: 5.0 if lang == "es" and word in english else 0.0
        argv = ["reduce-es-fr.py", "--work", str(work), "--built-at", "2026-10-03", "--pack-version", "test"]
        with mock.patch.dict(sys.modules, {"wordfreq": wordfreq}), mock.patch.object(sys, "argv", argv):
            with contextlib.redirect_stderr(io.StringIO()):
                es_fr.main()
        self.assertEqual((work / "freq.tsv").read_text(encoding="utf-8"), "casa\t1\nperro\t2\ngato\t3\n")
        self.assertEqual((work / "gloss.tsv").read_text(encoding="utf-8"), "gato\tChat\nperro\tChien\n")

    def test_spec_scenario_a_gloss_from_a_translation_table(self):
        # es-en reduced as its build runs it (`main()`), from a studied folder as es-fr writes it.
        # The English Wiktionary glosses `casa`; `sector` and `correr` have no English entry, and
        # the Spanish Wiktionary lists their English translations — at most three per part of
        # speech, in its order, in the case their words have. Nothing glosses `gato` in English: the
        # Spanish Wiktionary's French translations, es-fr's, are not read.
        work, studied = self.dir / "work", self.dir / "es"
        work.mkdir()
        studied.mkdir()
        (studied / "forms.tsv").write_text(
            "casa\tcasa\ncasas\tcasa\ncorrer\tcorrer\ngato\tgato\nsector\tsector\n", encoding="utf-8"
        )
        (studied / "freq.tsv").write_text("casa\t1\nsector\t2\ncorrer\t3\ngato\t4\n", encoding="utf-8")
        self.jsonl(CASA, CASAS, name="work/kaikki-Spanish.jsonl")
        self.jsonl(
            {"word": "casa", "pos": "noun", "translations": [{"word": "home"}]},
            {
                "word": "sector",
                "pos": "noun",
                "translations": [{"word": "sector"}, {"word": "area"}, {"word": "field"}, {"word": "zone"}],
            },
            {"word": "correr", "pos": "verb", "translations": [{"word": "run"}, {"word": "flow", "sense": "of a liquid"}]},
            {"word": "Sevilla", "pos": "name", "translations": [{"word": "Seville"}]},
            name="work/kaikki-es-traductions-en.jsonl",
        )
        self.jsonl({"word": "gato", "pos": "noun", "translations": [{"word": "chat"}]}, name="work/kaikki-es-traductions.jsonl")
        argv = [
            "reduce-es-en.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-08", "--pack-version", "test"),
        ]
        with mock.patch.object(sys, "argv", argv), contextlib.redirect_stderr(io.StringIO()) as err:
            es_en.main()
        self.assertIn("reduced es-en: lemmas=4", err.getvalue())
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "casa\thouse\ncorrer\trun, flow\nsector\tsector, area, field\n",
        )
        self.assertEqual(
            (work / "senses.tsv").read_text(encoding="utf-8"), "casa\tNOUN:1\ncorrer\tVERB:1\nsector\tNOUN:1\n"
        )
        # The native side alone: nothing of the studied side is computed (D1).
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith("kaikki-"))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "mwe.tsv", "senses.tsv"])
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("es", "en"))
        self.assertTrue(manifest["meta"]["levels_estimated"])
        self.assertEqual(manifest["meta"]["pack_version"], "test")

    def test_a_letter_s_translation_is_no_gloss(self):
        # Recorded: the Spanish Wiktionary lists a letter as its own English translation, under
        # its `character` entry or the noun naming it. Read as the direct table, `b`, `g` and `i`
        # would be glossed « b », « g » and « i ». A one-letter word translated is kept (`o` « or »).
        src = self.jsonl(B_TRANSLATED, O_TRANSLATED, G_TRANSLATED, I_TRANSLATED, name="kaikki-es-traductions-en.jsonl")
        self.assertEqual(
            common.read_translations(src, inverted=False, studied=SPANISH),
            {"b": {"SYM": ["b"]}, "o": {"CCONJ": ["or"]}, "g": {"SYM": ["g"]}, "i": {"NOUN": ["i"]}},
        )
        direct = es_en.read_translated(src, str(self.dir / "words.jsonl"))
        self.assertEqual(direct, {"o": {"CCONJ": ["or"]}})
        got = common.fallback_glosses({"b", "g", "i", "o"}, {}, [(direct, list)], edition=EN)
        self.assertEqual(got, {"o": ("or", [("CCONJ", 1)])})

    def test_native_fields_cut_changes_no_table(self):
        # Recorded full entries: the native side is the same from the extract as kaikki writes it
        # and from its cut (`native_fields`), pointers (`alt_of`) and expressions included.
        full = self.jsonl(FULL_QUORUM, FULL_CUORUM, FULL_DAR_DE_ALTA, name="full.jsonl")
        cut, dropped = es_en.native_fields(full, str(self.dir / "cut.jsonl"))
        self.assertEqual(dropped, 0)
        self.assertLess(Path(cut).stat().st_size, Path(full).stat().st_size / 3)
        lemmas = {"quórum": 1, "cuórum": 2}

        def tables(path):
            return common.native_tables(path, lemmas, studied=es_en.ES, edition=es_en.EDITION, fallbacks=[({}, list)])

        self.assertEqual(tables(cut), tables(full))
        glosses, _, expressions, _ = tables(cut)
        self.assertEqual(glosses["quórum"], "quorum (minimum number of votes)", "the pointer kept")
        self.assertIn("dar de alta", expressions)

    def test_native_fields_counts_the_lines_it_leaves_out(self):
        # Made up: an undecodable line and two JSON values that are no object.
        src = self.dir / "extract.jsonl"
        src.write_text(json.dumps(CASA) + "\nnot json\n[1]\n2\n", encoding="utf-8")
        cut, dropped = es_en.native_fields(str(src), str(self.dir / "cut.jsonl"))
        self.assertEqual(dropped, 3)
        self.assertEqual([json.loads(line)["word"] for line in Path(cut).read_text().splitlines()], ["casa"])

    def test_es_en_s_copy_of_spanish_is_es_fr_s(self):
        # reduce-es-en.py repeats es-fr's description of Spanish (`_TOKEN`, the coordinators, the
        # form-of target) because a reducer loads no other pair's: the two must not drift.
        self.assertEqual(es_en._TOKEN, es_fr._TOKEN)
        self.assertEqual(es_en.ES, es_fr.ES)
        self.assertEqual(
            (es_en.ES.token.pattern, es_en.ES.form_of_target.pattern, es_en.ES.coordinators),
            (es_fr.ES.token.pattern, es_fr.ES.form_of_target.pattern, es_fr.ES.coordinators),
        )

    def test_es_en_reads_the_committed_studied_tables_as_es_fr_writes_them(self):
        # A studied folder whose forms and ranks disagree is not what es-fr's reduction writes.
        studied = self.dir / "es"
        studied.mkdir()
        (studied / "forms.tsv").write_text("casa\tcasa\ncasas\tcasa\n", encoding="utf-8")
        (studied / "freq.tsv").write_text("casa\t1\nperro\t2\n", encoding="utf-8")
        with self.assertRaisesRegex(SystemExit, "disagree on the lemmas"):
            es_en.read_studied(str(studied), 60000)
        (studied / "forms.tsv").write_text("casa\tcasa\nperro\tperro\n", encoding="utf-8")
        self.assertEqual(es_en.read_studied(str(studied), 60000), {"casa": 1, "perro": 2})
        self.assertEqual(es_en.read_studied(str(studied), 1), {"casa": 1}, "capped by rank")
        committed = es_en.read_studied(os.path.join(_HERE, "tables", "es"), 60000)
        self.assertEqual(len(committed), 60000)

    def test_nothing_written_no_gloss(self):
        glosses, runs, expressions, primary = common.native_tables(
            self.jsonl(CONGERIES), {"congeries"}, studied=SPANISH, edition=EN, fallbacks=[({}, list)]
        )
        self.assertEqual((glosses, runs, expressions, primary), ({}, {}, {}, 0))


if __name__ == "__main__":
    unittest.main()
