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
import reduce_edition_es as spanish  # noqa: E402
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
en_es = _reducer("en-es")
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
        # Either setting changed re-pins es-en and fr-en, the pairs glossed in English
        # (add-lingua-pack-fr-en: *A rule of the English edition*): it is in their rules and in no
        # French-native pair's — nor en-es's, glossed by the Spanish edition — and reduce_common.py,
        # which every pair loads, is not edited for it.
        for pair in ("es-en", "fr-en"):
            self.assertEqual(
                [p.name for p in ps.rule_files(Path(_HERE) / f"reduce-{pair}.py")],
                [f"reduce-{pair}.py", "reduce_common.py", "reduce_edition_en.py"],
            )
        self.assertEqual(
            [p.name for p in ps.rule_files(Path(_HERE) / "reduce-en-es.py")],
            ["reduce-en-es.py", "reduce_common.py", "reduce_edition_es.py"],
        )
        copy = self.dir / "rules"
        copy.mkdir()
        for path in Path(_HERE).glob("reduce[-_]*.py"):
            (copy / path.name).write_bytes(path.read_bytes())
        pairs = ("en-fr", "es-fr", "es-en", "en-es", "fr-en")
        others = ("en-fr", "es-fr", "en-es")
        before = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        self.assertEqual(before, {pair: ps.rules_sha256(Path(_HERE) / f"reduce-{pair}.py") for pair in pairs})
        # The committed pairs' digests are the ones their pins record: fr-en's glosses moved no other.
        for pair in pairs:
            self.assertEqual(before[pair], ps.get(ps.load(Path(_HERE) / "tables" / pair / "pin.json"), "reducer.sha256"), pair)
        for name, old, new in (
            ("reduce_edition_en.py", "LONG_PARENTHESIS = 0\n", "LONG_PARENTHESIS = 40\n"),
            ("reduce_edition_en.py", "MERGE_SAME_POS_ETYMOLOGIES = False\n", "MERGE_SAME_POS_ETYMOLOGIES = True\n"),
            # The letters' rules of add-lingua-pack-es-en, the English edition's.
            ("reduce_edition_en.py", r"|the letter \w\b)", ")"),
            ("reduce_edition_en.py", "len(headword) == 1 and headword.isupper()", "False"),
        ):
            edition = copy / name
            text = edition.read_text(encoding="utf-8")
            self.assertIn(old, text)
            edition.write_text(text.replace(old, new), encoding="utf-8")
            after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
            self.assertNotEqual(after["es-en"], before["es-en"], new)
            self.assertNotEqual(after["fr-en"], before["fr-en"], new)
            self.assertEqual({p: after[p] for p in others}, {p: before[p] for p in others}, new)
            before = after
        # A rule of es-en's own reducer re-pins es-en alone, and one of fr-en's fr-en alone.
        for pair, old, new in (
            ("es-en", 'if entry.get("pos") == "character":', "if False:"),
            ("fr-en", '"à la": (', '"à le": ('),
        ):
            reducer = copy / f"reduce-{pair}.py"
            text = reducer.read_text(encoding="utf-8")
            self.assertIn(old, text)
            reducer.write_text(text.replace(old, new), encoding="utf-8")
            after = {p: ps.rules_sha256(copy / f"reduce-{p}.py") for p in pairs}
            self.assertNotEqual(after[pair], before[pair], new)
            self.assertEqual({p: after[p] for p in pairs if p != pair}, {p: before[p] for p in pairs if p != pair}, new)
            before = after
        # The Spanish edition's rules are en-es's alone (add-lingua-pack-en-es D1).
        edition = copy / "reduce_edition_es.py"
        edition.write_text(edition.read_text(encoding="utf-8") + "\n# edited\n", encoding="utf-8")
        after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        self.assertNotEqual(after["en-es"], before["en-es"])
        self.assertEqual({p: after[p] for p in pairs if p != "en-es"}, {p: before[p] for p in pairs if p != "en-es"})


# — es-en's glosses read as meanings (refine-lingua-es-en-glosses) —
# Recorded from the extract es-en pins (kaikki's of 2026-10-03), as `native_fields` cuts it. HACER,
# ESTAR, SENOR, POR_PREP, SE_PRON and YA_ADV keep their entry's first senses only.
VENIR = {
    "word": "venir",
    "pos": "verb",
    "senses": [
        {"glosses": ["Senses relating to literal movement.", "to come (move closer to some location (most often closer to the speaker), by default the speaker's location)"], "tags": ["reflexive", "sometimes"]},
        {"glosses": ["Senses relating to literal movement.", "to arrive"]},
        {"glosses": ["Figurative senses.", "to come from, originate"]},
        {"glosses": ["Figurative senses.", "to come (happen)"]},
        {"glosses": ["Figurative senses.", "to come (appear)"]},
        {"glosses": ["Figurative senses.", "to come (to have some characteristic or quality) (with an adjective or prepositional phrase)"]},
        {"glosses": ["Figurative senses.", "to receive something, get something"]},
        {"glosses": ["Figurative senses.", "to get (something felt in one's body or perceived in one's mind, a feeling, an illness, pain, an urge, an idea, etc.)"]},
        {"glosses": ["Figurative senses.", "to be coming or coming up (of a date, event, etc.)"]},
        {"glosses": ["Figurative senses.", "translated with next (used with semana, mes, año, days of the week, etc.)"]},
        {"glosses": ["Figurative senses.", "translated with be about, mean"]},
        {"glosses": ["Figurative senses.", "translated with end up"]},
        {"glosses": ["Figurative senses.", "to be (with \"bien\" (convenient, helpful, good), \"mal\" (bad), etc.)"]},
        {"glosses": ["Figurative senses.", "to suit (well, badly)"]},
        {"glosses": ["Figurative senses.", "to fit, to be (of clothing, with adjectives like \"grande\" (be big), etc., or \"bien\" (fit well))"]},
        {"glosses": ["Figurative senses.", "to orgasm, to cum, to come"], "tags": ["colloquial", "reflexive"]},
    ],
}
CUSCO = {
    "word": "Cusco",
    "pos": "name",
    "senses": [
        {"glosses": ["alternative spelling of Cuzco"], "tags": ["alt-of", "alternative", "masculine"], "alt_of": [{"word": "Cuzco"}]},
        {"glosses": ["places in Peru:", "a region of Peru"], "tags": ["masculine"]},
        {"glosses": ["places in Peru:", "a province of Cusco"], "tags": ["masculine"]},
        {"glosses": ["places in Peru:", "a city, the provincial capital of Cusco, Peru"], "tags": ["masculine"]},
    ],
}
HACER = {
    "word": "hacer",
    "pos": "verb",
    "senses": [
        {"glosses": ["to do, perform, execute, carry out"], "tags": ["transitive"]},
        {"glosses": ["to do, perform, execute, carry out", "Forms ad hoc verbs from borrowed nouns."], "tags": ["transitive"]},
        {"glosses": ["to make", "to create, to build, to bring forth"], "tags": ["transitive"]},
        {"glosses": ["to make", "to write, to compose"], "tags": ["transitive"]},
        {"glosses": ["to make", "to prepare (food)"], "tags": ["transitive"]},
    ],
}
CASITA = {
    "word": "casita",
    "pos": "noun",
    "senses": [
        {"glosses": ["diminutive of casa", "small house"], "tags": ["feminine"]},
        {"glosses": ["diminutive of casa", "house"], "tags": ["endearing", "feminine"]},
        {"glosses": ["mother-in-law apartment"], "tags": ["feminine"]},
        {"glosses": ["house (children's activity of pretending to be a family)"], "tags": ["feminine", "in-plural"]},
    ],
}
COSITA = {
    "word": "cosita",
    "pos": "noun",
    "senses": [
        {"glosses": ["diminutive of cosa", "small thing"], "tags": ["feminine"]},
        {"glosses": ["diminutive of cosa", "thingy"], "tags": ["feminine"]},
    ],
}
QUEBEC = {
    "word": "Québec",
    "pos": "name",
    "senses": [
        {"glosses": ["alternative form of Quebec:", "Quebec (a province in eastern Canada)"]},
        {"glosses": ["alternative form of Quebec:", "Quebec, Quebec City (the capital city of the province of Quebec, Canada)"]},
    ],
}
SU = {
    "word": "su",
    "pos": "det",
    "senses": [
        {"glosses": ["apocopic form of suyo"], "tags": ["abbreviation", "alt-of", "apocopic"], "alt_of": [{"word": "suyo"}]},
        {"glosses": ["apocopic form of suyo", "used to express an approximate number: about, approximately"]},
    ],
}
SUYO_PRON = {
    "word": "suyo",
    "pos": "pron",
    "senses": [
        {"glosses": ["his, hers, its"], "tags": ["masculine", "singular"]},
        {"glosses": ["theirs"], "tags": ["masculine", "singular"]},
        {"glosses": ["yours"], "tags": ["formal", "masculine", "singular"]},
    ],
}
SUYO_DET = {
    "word": "suyo",
    "pos": "det",
    "senses": [
        {"glosses": ["his, hers, its, to her"], "tags": ["masculine", "singular"]},
        {"glosses": ["theirs, their, to them"], "tags": ["masculine", "singular"]},
        {"glosses": ["yours, your, to you"], "tags": ["formal", "masculine", "singular"]},
    ],
}
SI_PRON = {
    "word": "sí",
    "pos": "pron",
    "senses": [
        {"glosses": ["prepositional form of se", "himself, herself, itself, themself, themselves"]},
        {"glosses": ["prepositional form of se", "yourself, yourselves"]},
    ],
}
GOMA = {
    "word": "goma",
    "pos": "noun",
    "senses": [
        {"glosses": ["rubber (substance, material)", "ellipsis of cinta de goma or goma elástica (“rubber band”)"], "tags": ["abbreviation", "alt-of", "ellipsis", "feminine"], "alt_of": [{"word": "cinta de goma or goma elástica", "extra": "rubber band"}]},
        {"glosses": ["rubber (substance, material)", "rubber tree"], "tags": ["Bolivia", "feminine"]},
        {"glosses": ["rubber (substance, material)", "teether"], "tags": ["Bolivia", "feminine"]},
        {"glosses": ["rubber (substance, material)", "ellipsis of goma de borrar (“eraser”)"], "tags": ["abbreviation", "alt-of", "ellipsis", "feminine"], "alt_of": [{"word": "goma de borrar", "extra": "eraser"}]},
        {"glosses": ["rubber (substance, material)", "condom"], "tags": ["feminine"]},
        {"glosses": ["rubber (substance, material)", "tire"], "tags": ["feminine"]},
        {"glosses": ["rubber (substance, material)", "boob; tit"], "tags": ["Chile", "Rioplatense", "feminine", "in-plural", "vulgar"]},
        {"glosses": ["gum (substance exuded by certain plants)", "ellipsis of goma de mascar (“chewing gum”)"], "tags": ["Bolivia", "Chile", "Colombia", "Peru", "abbreviation", "alt-of", "ellipsis", "feminine"], "alt_of": [{"word": "goma de mascar", "extra": "chewing gum"}]},
        {"glosses": ["gum (substance exuded by certain plants)", "glue"], "tags": ["Costa-Rica", "Cuba", "Guatemala", "Nicaragua", "Panama", "Uruguay", "feminine"]},
    ],
}
MI_DET = {
    "word": "mi",
    "pos": "det",
    "senses": [
        {"glosses": ["apocopic form of mío, my"], "tags": ["abbreviation", "alt-of", "apocopic", "first-person", "possessive", "singular"], "alt_of": [{"word": "mío"}, {"word": "my"}]},
    ],
}
MI_NOUN_1 = {"word": "mi", "pos": "noun", "senses": [{"glosses": ["mu; the Greek letter Μ, μ"], "tags": ["feminine"]}]}
MI_NOUN_2 = {"word": "mi", "pos": "noun", "senses": [{"glosses": ["mi"], "tags": ["masculine"]}]}
MUY = {
    "word": "muy",
    "pos": "adv",
    "senses": [
        {"glosses": ["apocopic form of mucho; very"], "tags": ["abbreviation", "alt-of", "apocopic"], "alt_of": [{"word": "mucho", "extra": "very"}]},
    ],
}
MUCHO_ADV = {
    "word": "mucho",
    "pos": "adv",
    "senses": [
        {"glosses": ["much, a lot, far, way, many times"]},
        {"glosses": ["very"]},
        {"glosses": ["long (a long time)"]},
    ],
}
UN_ARTICLE = {"word": "un", "pos": "article", "senses": [{"glosses": ["an; a"], "tags": ["indefinite", "masculine"]}]}
UN_NUM = {
    "word": "un",
    "pos": "num",
    "senses": [
        {"glosses": ["apocopic form of uno (“one”)"], "tags": ["abbreviation", "alt-of", "apocopic", "masculine"], "alt_of": [{"word": "uno", "extra": "one"}]},
    ],
}
MAL_ADV = {
    "word": "mal",
    "pos": "adv",
    "senses": [
        {"glosses": ["badly, poorly, ill"]},
        {"glosses": ["awry, amiss, wrong, wrongly"]},
        {"glosses": ["hard (functions as an adverb in Spanish but translates as an adjective in English)"]},
        {"glosses": ["Used as an intensifier, very"], "tags": ["Argentina", "colloquial"]},
        {"glosses": ["yes"], "tags": ["Argentina", "colloquial"]},
    ],
}
MAL_NOUN = {
    "word": "mal",
    "pos": "noun",
    "senses": [
        {"glosses": ["evil, harm; a bad thing or situation"], "tags": ["masculine"]},
        {"glosses": ["disease, illness, ailment"], "tags": ["masculine"]},
        {"glosses": ["worse (substantive)"], "tags": ["masculine"]},
    ],
}
MAL_ADJ = {
    "word": "mal",
    "pos": "adj",
    "senses": [
        {"glosses": ["apocopic form of malo bad; evil"], "tags": ["abbreviation", "alt-of", "apocopic", "masculine"], "alt_of": [{"word": "malo bad", "extra": "evil"}]},
        {"glosses": ["amiss, awry, off, wrong"], "tags": ["abbreviation", "apocopic", "masculine"]},
    ],
}
CINCUENTA_Y_UN = {
    "word": "cincuenta y un",
    "pos": "num",
    "senses": [
        {"glosses": ["apocopic form of cincuenta y uno (“fifty-one”)"], "tags": ["abbreviation", "alt-of", "apocopic", "masculine"], "alt_of": [{"word": "cincuenta y uno", "extra": "fifty-one"}]},
    ],
}
NOS_PRON = {
    "word": "nos",
    "pos": "pron",
    "senses": [
        {"glosses": ["dative of nosotros: to us, for us"], "tags": ["dative", "form-of"], "form_of": [{"word": "nosotros", "extra": "to us, for us"}]},
        {"glosses": ["accusative of nosotros: us"], "tags": ["accusative", "form-of"], "form_of": [{"word": "nosotros", "extra": "us"}]},
        {"glosses": ["reflexive of nosotros: ourselves; each other"], "tags": ["form-of", "pronoun", "reflexive"], "form_of": [{"word": "nosotros", "extra": "ourselves; each other"}]},
        {"glosses": ["first person (except in vocative, and in the oblique it requires a preposition); I (singular; compare vos)"], "tags": ["archaic", "formal"]},
        {"glosses": ["first person nominative, prepositional and vocative plural pronoun"], "tags": ["archaic", "first-person", "formal", "nominative", "plural", "prepositional", "pronoun", "vocative"]},
    ],
}
NOS_NOUN = {
    "word": "nos",
    "pos": "noun",
    "senses": [
        {"glosses": ["plural of no"], "tags": ["form-of", "masculine", "plural"], "form_of": [{"word": "no"}]},
    ],
}
ER = {
    "word": "er",
    "pos": "article",
    "senses": [
        {"glosses": ["eye dialect spelling of el"], "tags": ["Andalusia", "alt-of", "pronunciation-spelling"], "alt_of": [{"word": "el"}]},
        {"glosses": ["pronunciation spelling of el"], "tags": ["Andalusia", "alt-of", "pronunciation-spelling"], "alt_of": [{"word": "el"}]},
    ],
}
EL_1 = {"word": "el", "pos": "article", "senses": [{"glosses": ["masculine singular definite article; the"]}]}
EL_2 = {
    "word": "el",
    "pos": "article",
    "senses": [
        {"glosses": ["feminine singular definite article used before nouns which start with a stressed /a/"]},
    ],
}
LO_PRON = {
    "word": "lo",
    "pos": "pron",
    "senses": [
        {"glosses": ["accusative of él and usted (when referring to a man), and a variant of ello in many constructions; him, you (formal), it, that"], "tags": ["accusative", "form-of"], "form_of": [{"word": "él and usted", "extra": "(when referring to a man), and a variant of ello in many constructions; him, you (formal), it, that"}]},
    ],
}
LO_ARTICLE = {
    "word": "lo",
    "pos": "article",
    "senses": [
        {"glosses": ["neuter definite article used only before nominalized adjectives: the, that which is"]},
    ],
}
LES_PRON = {
    "word": "les",
    "pos": "pron",
    "senses": [
        {"glosses": ["dative of ellos and ellas; to them, for them"], "tags": ["dative", "form-of"], "form_of": [{"word": "ellos and ellas", "extra": "to them, for them"}]},
        {"glosses": ["dative of ustedes; to you all, for you all (formal)"], "tags": ["dative", "form-of"], "form_of": [{"word": "ustedes", "extra": "to you all, for you all (formal)"}]},
        {"glosses": ["accusative of ustedes; you all (formal)"], "tags": ["accusative", "dialectal", "form-of"], "form_of": [{"word": "ustedes", "extra": "you all (formal)"}]},
        {"glosses": ["accusative of ellos and ellas; them"], "tags": ["accusative", "dialectal", "form-of"], "form_of": [{"word": "ellos and ellas", "extra": "them"}]},
        {"glosses": ["dative of elles; to them, for them"], "tags": ["dative", "form-of", "gender-neutral", "neologism"], "form_of": [{"word": "elles", "extra": "to them, for them"}]},
    ],
}
LES_ARTICLE = {
    "word": "les",
    "pos": "article",
    "senses": [
        {"glosses": ["the (plural)"], "tags": ["gender-neutral", "neologism"]},
    ],
}
TOY = {"word": "toy", "pos": "verb", "senses": [{"glosses": ["apheretic form of estoy"], "tags": ["colloquial"]}]}
ESTOY = {
    "word": "estoy",
    "pos": "verb",
    "senses": [
        {"glosses": ["first-person singular present indicative of estar; am"], "tags": ["first-person", "form-of", "indicative", "present", "singular"], "form_of": [{"word": "estar", "extra": "am"}]},
    ],
}
ESTAR = {
    "word": "estar",
    "pos": "verb",
    "senses": [
        {"glosses": ["to be (have a temporary or permanent location in space)"], "tags": ["copulative", "intransitive"]},
        {"glosses": ["to be present"], "tags": ["intransitive"]},
        {"glosses": ["to be (denotes a copula, in a transient fashion)"], "tags": ["copulative", "intransitive"]},
    ],
}
PO_FAVO = {
    "word": "po favó",
    "pos": "intj",
    "senses": [
        {"glosses": ["pronunciation spelling of por favor"], "tags": ["alt-of", "pronunciation-spelling"], "alt_of": [{"word": "por favor"}]},
    ],
}
POR_FAVOR = {
    "word": "por favor",
    "pos": "intj",
    "senses": [
        {"glosses": ["please"]},
        {"glosses": ["you're welcome"], "tags": ["dialectal"]},
    ],
}
SEO = {
    "word": "seó",
    "pos": "noun",
    "senses": [
        {"glosses": ["apocopic form of seor"], "tags": ["abbreviation", "alt-of", "apocopic", "colloquial", "masculine"], "alt_of": [{"word": "seor"}]},
    ],
}
SEOR = {"word": "seor", "pos": "noun", "senses": [{"glosses": ["syncopic form of señor"], "tags": ["masculine"]}]}
SENOR = {
    "word": "señor",
    "pos": "noun",
    "senses": [
        {"glosses": ["mister, sir, lord (title conferred on a married or older male)"], "tags": ["masculine"]},
        {"glosses": ["gentleman"], "tags": ["masculine"]},
    ],
}
COMO_NAME = {
    "word": "Como",
    "pos": "name",
    "senses": [
        {"glosses": ["Como (a city and comune, the capital of the province of Como, Lombardy)"]},
        {"glosses": ["Como (a province of Lombardy, Italy)"]},
        {"glosses": ["a number of places in the United States:", "Como (a town in Panola County, Mississippi)"]},
        {"glosses": ["a number of places in the United States:", "Como (a town in Hertford County, North Carolina)"]},
        {"glosses": ["a number of places in the United States:", "Como (a town in Hopkins County, Texas)"]},
        {"glosses": ["a number of places in the United States:", "Como (a census-designated place in the town of Geneva, Walworth County, Wisconsin)"]},
        {"glosses": ["a number of places in Australia:", "Como (a suburb of Sydney in Sutherland Shire, New South Wales)"]},
        {"glosses": ["a number of places in Australia:", "Como (a suburb of Perth in the City of South Perth, Western Australia)"]},
    ],
}
COMO_ADV = {
    "word": "como",
    "pos": "adv",
    "senses": [
        {"glosses": ["as (to such an extent or degree)"]},
        {"glosses": ["like, about (approximately)"]},
    ],
}
COMO_CONJ = {
    "word": "como",
    "pos": "conj",
    "senses": [
        {"glosses": ["as (introducing a basis of comparison or equality)"]},
        {"glosses": ["as, since (being that)"]},
        {"glosses": ["how (in which way)"]},
        {"glosses": ["if, unless (under the condition that)"]},
    ],
}
COMO_PREP = {
    "word": "como",
    "pos": "prep",
    "senses": [
        {"glosses": ["as (in the manner or role specified)"]},
        {"glosses": ["such as (for example)"]},
        {"glosses": ["like (similar to, reminiscent of)"]},
    ],
}
COMO_VERB = {
    "word": "como",
    "pos": "verb",
    "senses": [
        {"glosses": ["first-person singular present indicative of comer"], "tags": ["first-person", "form-of", "indicative", "present", "singular"], "form_of": [{"word": "comer"}]},
    ],
}
PR_NAME = {
    "word": "PR",
    "pos": "name",
    "senses": [
        {"glosses": ["initialism of Puerto Rico (“Puerto Rico (a commonwealth, island and dependent territory of the United States in the Caribbean)”)"], "tags": ["abbreviation", "alt-of", "initialism"], "alt_of": [{"word": "Puerto Rico", "extra": "(“Puerto Rico (a commonwealth, island and dependent territory of the United States in the Caribbean)”)"}]},
    ],
}
PUERTO_RICO = {
    "word": "Puerto Rico",
    "pos": "name",
    "senses": [
        {"glosses": ["Puerto Rico (a commonwealth, island and dependent territory of the United States in the Caribbean; official name: Estado Libre Asociado de Puerto Rico)"]},
    ],
}
PR_PREP = {
    "word": "pr",
    "pos": "prep",
    "senses": [
        {"glosses": ["abbreviation of por"], "tags": ["Internet", "abbreviation", "alt-of"], "alt_of": [{"word": "por"}]},
        {"glosses": ["abbreviation of para"], "tags": ["Internet", "abbreviation", "alt-of"], "alt_of": [{"word": "para"}]},
    ],
}
PR_CONJ = {
    "word": "pr",
    "pos": "conj",
    "senses": [
        {"glosses": ["abbreviation of pero"], "tags": ["Internet", "abbreviation", "alt-of"], "alt_of": [{"word": "pero"}]},
    ],
}
POR_PREP = {
    "word": "por",
    "pos": "prep",
    "senses": [
        {"glosses": ["by"]},
        {"glosses": ["for (indicates something given in an exchange)"]},
        {"glosses": ["through, out, via (indicating movement)"]},
    ],
}
CHILE_NAME = {
    "word": "Chile",
    "pos": "name",
    "senses": [
        {"glosses": ["Chile (a country in South America; capital: Santiago)"], "tags": ["masculine"]},
    ],
}
CHILE_NOUN = {
    "word": "chile",
    "pos": "noun",
    "senses": [
        {"glosses": ["a chili pepper"], "tags": ["Costa-Rica", "Guatemala", "Honduras", "Mexico", "Nicaragua", "Philippines", "US", "masculine"]},
        {"glosses": ["penis"], "tags": ["El-Salvador", "Guatemala", "Mexico", "masculine", "vulgar"]},
        {"glosses": ["lie"], "tags": ["Guatemala", "colloquial", "masculine"]},
    ],
}
AMOR_NAME = {
    "word": "Amor",
    "pos": "name",
    "senses": [
        {"glosses": ["a surname"], "tags": ["by-personal-gender", "feminine", "masculine"]},
    ],
}
AMOR_NOUN = {
    "word": "amor",
    "pos": "noun",
    "senses": [
        {"glosses": ["love"], "tags": ["masculine"]},
        {"glosses": ["love affair"], "tags": ["masculine"]},
    ],
}
NI_CONJ = {
    "word": "ni",
    "pos": "conj",
    "senses": [
        {"glosses": ["Used when negating two or more elements, and not, not A or B, not A nor B, neither A nor B"]},
    ],
}
NI_ADV = {
    "word": "ni",
    "pos": "adv",
    "senses": [
        {"glosses": ["Used in emphatic negations, not even"]},
        {"glosses": ["Forms an emphatic negative imperative, don't even"]},
    ],
}
SE_PRON = {
    "word": "se",
    "pos": "pron",
    "senses": [
        {"glosses": ["A reflexive or reciprocal pronoun: oneself, himself, herself, itself, yourself; themselves; yourselves; each other; one another"], "tags": ["by-personal-gender", "feminine", "masculine", "plural", "singular", "third-person"]},
    ],
}
YA_ADV = {
    "word": "ya",
    "pos": "adv",
    "senses": [
        {"glosses": ["now, right now, (in the negative) anymore, no longer"]},
        {"glosses": ["now, right now, (in the negative) anymore, no longer", "by now, at this point"]},
        {"glosses": ["indicates completion of an action (difference from sense 4 depends on context)"]},
        {"glosses": ["yet (in questions)"]},
    ],
}
JURADO_NOUN = {
    "word": "jurado",
    "pos": "noun",
    "senses": [
        {"glosses": ["juror, juryman, juryperson (member of a jury [sense 1])"], "tags": ["masculine"]},
        {"glosses": ["judge (member of a jury [sense 2]; officiator of a competitive event)"], "tags": ["masculine"]},
    ],
}
NADA_PRON = {"word": "nada", "pos": "pron", "senses": [{"glosses": ["nothing, zero, zilch, not...anything"]}]}
TANTO_CONJ = {
    "word": "tanto",
    "pos": "conj",
    "senses": [
        {"glosses": ["both ... and (introduces the first of two linked elements)"]},
    ],
}
O_CONJ = {"word": "o", "pos": "conj", "senses": [{"glosses": ["either … or"]}]}
AHORA_CONJ = {
    "word": "ahora",
    "pos": "conj",
    "senses": [
        {"glosses": ["now...now, whether...or..."], "tags": ["literary"]},
    ],
}
OTRO_INTJ = {
    "word": "otro",
    "pos": "intj",
    "senses": [
        {"glosses": ["\"Not again!\" or \"What, again?\" (also Otra vez! or Otra vez?)"], "tags": ["masculine"]},
    ],
}
CANINO_ADJ = {
    "word": "canino",
    "pos": "adj",
    "senses": [
        {"glosses": ["canine"]},
        {"glosses": ["canine"]},
        {"glosses": ["ravenously hungry; hungry as a hog"], "tags": ["idiomatic"]},
        {"glosses": ["ravenously hungry; hungry as a hog\nMarcos siempre estaba canino después de tomar su medicación.\nMarcos was always hungry as a hog after taking his medication.", "Marcos siempre estaba canino después de tomar su medicación."], "tags": ["idiomatic"]},
        {"glosses": ["ravenously hungry; hungry as a hog\nMarcos siempre estaba canino después de tomar su medicación.\nMarcos was always hungry as a hog after taking his medication.", "Marcos was always hungry as a hog after taking his medication."], "tags": ["idiomatic"]},
    ],
}
A_LA_MIERDA = {"word": "a la mierda", "pos": "phrase", "senses": [{"glosses": ["screw this; to hell with..."]}]}
CASCANUECES = {"word": "Cascanueces", "pos": "name", "senses": [{"glosses": ["The Nutcracker (ballet)"]}]}
AGATEADOR_NORTENO = {
    "word": "agateador norteño",
    "pos": "noun",
    "senses": [
        {"glosses": ["The Eurasian treecreeper."], "tags": ["masculine"]},
    ],
}
DIEZ_NOUN = {
    "word": "diez",
    "pos": "noun",
    "senses": [
        {"glosses": ["A (highest grade in testing)"], "tags": ["masculine"]},
    ],
}
JULIO = {"word": "julio", "pos": "noun", "senses": [{"glosses": ["July"], "tags": ["masculine"]}]}
CHAMORRO = {
    "word": "chamorro",
    "pos": "noun",
    "senses": [
        {"glosses": ["shank (of pork, beef, etc.), e.g. 'chamorro de puerco', pork shank"], "tags": ["Mexico", "masculine"]},
    ],
}
EJQUE = {
    "word": "ejque",
    "pos": "phrase",
    "senses": [
        {"glosses": ["pronunciation spelling of es que in the Madrid dialect"], "tags": ["alt-of", "pronunciation-spelling"], "alt_of": [{"word": "es que in the Madrid dialect"}]},
    ],
}
TAS = {"word": "tas", "pos": "verb", "senses": [{"glosses": ["apheretic form of estás"], "tags": ["colloquial"]}]}


class EsEnGlossesReadAsMeanings(Entries):
    """es-en's glosses read a Spanish word's meanings, not its page's layout, in one English typography
    (refine-lingua-es-en-glosses): `reduce_edition_en.read_as_meanings`, a pre-pass of the English
    edition that `reduce-es-en.py` runs before the shared rules read the file."""

    def read(self, *entries):
        """The entries as the pre-pass writes them, in its order."""
        out = english.read_as_meanings(self.jsonl(*entries), str(self.dir / "meanings.jsonl"))
        return [json.loads(line) for line in Path(out).read_text(encoding="utf-8").splitlines()]

    def tables(self, lemmas, *entries):
        """What es-en's shared rules make of the pre-pass's file: `(glosses, runs, expressions)`."""
        out = english.read_as_meanings(self.jsonl(*entries), str(self.dir / "meanings.jsonl"))
        ranks = {lemma: rank for rank, lemma in enumerate(lemmas, start=1)}
        glosses, runs, expressions, _ = common.native_tables(out, ranks, studied=SPANISH, edition=EN)
        return glosses, runs, expressions

    # — D2: a nested sense under a label or a pointer is read by its own gloss —

    def test_spec_scenario_sense_group_labels(self):
        # Recorded: venir's sixteen senses, nested under « Senses relating to literal movement. »
        # and « Figurative senses. ». Read as written, the labels were its gloss.
        written, _ = self.gloss(EN, SPANISH, {"venir"}, VENIR)
        self.assertEqual(written["venir"], "Senses relating to literal movement; Figurative senses")
        glosses, runs, _ = self.tables(["venir"], VENIR)
        self.assertTrue(
            glosses["venir"].startswith(
                "to come (move closer to some location (most often closer to the speaker), by default the speaker's "
                "location); to arrive; to come from, originate; to come (happen); "
            ),
            glosses["venir"],
        )
        self.assertNotIn("senses", glosses["venir"].lower())
        self.assertEqual(runs["venir"], [("VERB", 8)])

    def test_spec_scenario_a_list_s_introduction(self):
        # Recorded: `Cusco`'s places, nested under « places in Peru: »; its pointer stays one.
        written, _ = self.gloss(EN, SPANISH, {"cusco"}, CUSCO)
        self.assertEqual(written["cusco"], "places in Peru")
        glosses, _, _ = self.tables(["cusco"], CUSCO)
        self.assertEqual(
            glosses["cusco"], "a region of Peru; a province of Cusco; a city, the provincial capital of Cusco, Peru"
        )

    def test_spec_scenario_a_parent_that_is_a_meaning(self):
        # Recorded: hacer's first five senses. « to make » is neither a label nor a pointer: it
        # keeps glossing « to create, to build, to bring forth », as before.
        glosses, _, _ = self.tables(["hacer"], HACER)
        self.assertEqual(glosses["hacer"], "to do, perform, execute, carry out; to make")
        written, _ = self.gloss(EN, SPANISH, {"hacer"}, HACER)
        self.assertEqual(glosses, written)
        (entry,) = self.read(HACER)
        self.assertEqual([s["glosses"][0] for s in entry["senses"]], [s["glosses"][0] for s in HACER["senses"]])

    def test_spec_scenario_a_meaning_nested_under_a_diminutive(self):
        # Recorded: `casita`'s « small house » and « house » under « diminutive of casa », and every
        # sense of `cosita` under « diminutive of cosa », which left it unglossed.
        written, _ = self.gloss(EN, SPANISH, {"casita", "cosita"}, CASITA, COSITA)
        self.assertEqual(
            written, {"casita": "mother-in-law apartment; house (children's activity of pretending to be a family)"}
        )
        glosses, _, _ = self.tables(["casita", "cosita"], CASITA, COSITA)
        self.assertEqual(
            glosses,
            {
                "casita": "small house; house; mother-in-law apartment; "
                "house (children's activity of pretending to be a family)",
                "cosita": "small thing; thingy",
            },
        )
        # The new sense keeps its tags: none of them points.
        self.assertEqual(self.read(CASITA)[0]["senses"][1], {"glosses": ["house"], "tags": ["endearing", "feminine"]})

    def test_a_meaning_nested_under_an_alternative_form(self):
        # Recorded: `Québec`'s two places under « alternative form of Quebec: », unglossed before.
        written, _ = self.gloss(EN, SPANISH, {"québec"}, QUEBEC)
        self.assertEqual(written, {})
        glosses, _, _ = self.tables(["québec"], QUEBEC)
        self.assertEqual(
            glosses["québec"],
            "Quebec (a province in eastern Canada); "
            "Quebec, Quebec City (the capital city of the province of Quebec, Canada)",
        )

    def test_a_sense_nested_under_a_pointer_of_its_entry(self):
        # Recorded: su's second sense is nested under its first, a pointer of the same entry, and
        # sí's pronoun senses under « prepositional form of se ». Neither target is in the file, so
        # the pointers themselves stay what they are.
        su, si = self.read(SU, SI_PRON)
        self.assertEqual(su["senses"], [SU["senses"][0], {"glosses": ["used to express an approximate number: about, approximately"]}])
        self.assertEqual(
            si["senses"],
            [{"glosses": ["himself, herself, itself, themself, themselves"]}, {"glosses": ["yourself, yourselves"]}],
        )

    def test_a_nested_pointer_stays_a_pointer(self):
        # Recorded: goma's « ellipsis of goma de mascar (“chewing gum”) », tagged `alt-of`, nested
        # under « gum (substance exuded by certain plants) », a meaning: the sense is untouched and
        # still skipped, and goma's gloss is as before.
        (goma,) = self.read(GOMA)
        self.assertEqual(goma["senses"], GOMA["senses"])
        glosses, _, _ = self.tables(["goma"], GOMA)
        written, _ = self.gloss(EN, SPANISH, {"goma"}, GOMA)
        self.assertEqual(glosses, written)
        self.assertNotIn("chewing gum", glosses["goma"])

    # — D3: a shortened or respelled form, or a pronoun's case form, reads as its meaning —

    def test_spec_scenario_an_apocope_that_carries_its_meaning(self):
        # Recorded: « mi » beside the Greek letter mu and the note mi; « muy » « apocopic form of
        # mucho; very », which borrowed all of « mucho »'s adverb; « un » « (“one”) ».
        entries = (MI_DET, MI_NOUN_1, MI_NOUN_2, MUY, MUCHO_ADV, UN_ARTICLE, UN_NUM)
        written, _ = self.gloss(EN, SPANISH, {"mi", "muy", "un"}, *entries)
        self.assertEqual(
            written,
            {
                "mi": "mu, the Greek letter Μ, μ; mi",
                "muy": "much, a lot, far, way, many times; very; long (a long time)",
                "un": "an, a",
            },
        )
        glosses, runs, _ = self.tables(["mi", "muy", "un"], *entries)
        self.assertEqual(glosses, {"mi": "my; mu, the Greek letter Μ, μ; mi", "muy": "very", "un": "an, a; one"})
        self.assertEqual(runs["mi"], [("DET", 1), ("NOUN", 2)])
        # Its other tags kept, the pointer's tags and fields dropped.
        self.assertEqual(
            self.read(MI_DET)[0]["senses"],
            [{"glosses": ["my"], "tags": ["abbreviation", "apocopic", "first-person", "possessive", "singular"]}],
        )

    def test_spec_scenario_a_form_named_by_its_pointer_s_fields(self):
        # Recorded: mal's adjective « apocopic form of malo bad; evil » (kaikki's target « malo bad »,
        # its `extra` « evil »), and « cincuenta y un » « apocopic form of cincuenta y uno (“fifty-one”) »:
        # its target is the pointer's word, whole, not the gloss's first word (« fifty »).
        entries = (MAL_ADV, MAL_NOUN, MAL_ADJ, CINCUENTA_Y_UN)
        glosses, runs, expressions = self.tables(["mal"], *entries)
        self.assertEqual(
            glosses["mal"],
            "badly, poorly, ill; awry, amiss, wrong, wrongly; hard (functions as an adverb in Spanish but translates "
            "as an adjective in English); evil, harm, a bad thing or situation; disease, illness, ailment; "
            "worse (substantive); evil; amiss, awry, off, wrong",
        )
        self.assertEqual(runs["mal"], [("ADV", 3), ("NOUN", 3), ("ADJ", 2)])
        self.assertNotIn("apocopic form of", glosses["mal"])
        self.assertEqual(expressions["cincuenta y un"], "fifty-one")
        # The carried sense takes a place in the round-robin: « Used as an intensifier, very » is
        # crowded out of the eight (named in the sample for the owner).
        written, _ = self.gloss(EN, SPANISH, {"mal"}, *entries)
        self.assertIn("Used as an intensifier, very", written["mal"])
        self.assertNotIn("intensifier", glosses["mal"])

    def test_spec_scenario_a_tag_is_no_pointer(self):
        # Recorded: mal's « amiss, awry, off, wrong » is tagged `apocopic`, nos's « first person
        # nominative, prepositional and vocative plural pronoun » `prepositional`: neither is worded
        # as a pointer, so both stay meanings.
        mal, nos = self.read(MAL_ADJ, NOS_PRON)
        self.assertEqual(mal["senses"][1], MAL_ADJ["senses"][1])
        self.assertEqual(nos["senses"][4], NOS_PRON["senses"][4])
        glosses, _, _ = self.tables(["mal", "nos"], MAL_ADJ, NOS_PRON)
        self.assertIn("amiss, awry, off, wrong", glosses["mal"].split("; "))
        self.assertIn("first person nominative, prepositional and vocative plural pronoun", glosses["nos"].split("; "))

    def test_spec_scenario_an_apocope_that_carries_none(self):
        # Recorded: « su » « apocopic form of suyo » with a sense nested under it; `suyo` as a
        # pronoun and a determiner. su takes suyo's determiner senses, in the pointer's place, and
        # keeps its nested sense. Without the pre-pass, the edition's wording makes both senses
        # pointers: su borrows suyo's senses whole, and the nested one is lost (the committed tables,
        # before the wording, read « apocopic form of suyo »).
        written, _ = self.gloss(EN, SPANISH, {"su"}, SU, SUYO_PRON, SUYO_DET)
        self.assertEqual(written["su"], "his, hers, its, to her; theirs, their, to them; yours, your, to you")
        glosses, runs, _ = self.tables(["su"], SU, SUYO_PRON, SUYO_DET)
        self.assertEqual(
            glosses["su"],
            "his, hers, its, to her; theirs, their, to them; yours, your, to you; "
            "used to express an approximate number: about, approximately",
        )
        self.assertEqual(runs["su"], [("DET", 4)])
        lent = self.read(SU, SUYO_PRON, SUYO_DET)[0]["senses"][0]
        self.assertEqual(lent, {"glosses": ["his, hers, its, to her"], "tags": ["abbreviation", "apocopic"]})

    def test_spec_scenario_a_target_of_two_letters(self):
        # Recorded: « er », only « pronunciation spelling of el » and « eye dialect spelling of el »:
        # el has two letters, lending nothing (`reduce_common._MIN_BASE`), so they stay pointers.
        er = self.read(ER, EL_1, EL_2)[0]
        self.assertEqual(er["senses"], ER["senses"])
        self.assertTrue(all(EN.form_of.match(s["glosses"][0]) for s in er["senses"]))
        glosses, _, _ = self.tables(["er", "el"], ER, EL_1, EL_2)
        self.assertNotIn("er", glosses)
        self.assertIn("el", glosses)

    def test_spec_scenario_a_pronoun_s_case_form(self):
        # Recorded: lo's, nos's and les's case forms carry their meaning after a colon or a
        # semicolon; read as pointers, lo read only its article and les only « the (plural) ».
        entries = (LO_PRON, LO_ARTICLE, NOS_PRON, NOS_NOUN, LES_PRON, LES_ARTICLE)
        written, _ = self.gloss(EN, SPANISH, {"lo", "nos", "les"}, *entries)
        self.assertEqual(
            written,
            {
                "lo": "neuter definite article used only before nominalized adjectives: the, that which is",
                "nos": "first person (except in vocative, and in the oblique it requires a preposition), "
                "I (singular, compare vos); first person nominative, prepositional and vocative plural pronoun",
                "les": "the (plural)",
            },
        )
        glosses, _, _ = self.tables(["lo", "nos", "les"], *entries)
        self.assertEqual(
            glosses,
            {
                "lo": "him, you (formal), it, that; "
                "neuter definite article used only before nominalized adjectives: the, that which is",
                "nos": "to us, for us; us; ourselves, each other; first person (except in vocative, and in the "
                "oblique it requires a preposition), I (singular, compare vos); "
                "first person nominative, prepositional and vocative plural pronoun",
                "les": "to them, for them; to you all, for you all (formal); you all (formal); them; the (plural)",
            },
        )
        # lo's `extra` opens on the note: a case form's meaning is the text after the first colon or
        # semicolon of its gloss.
        self.assertEqual(self.read(LO_PRON)[0]["senses"], [{"glosses": ["him, you (formal), it, that"], "tags": ["accusative"]}])

    def test_a_form_of_a_form_follows_its_pointer_once(self):
        # Recorded: « toy », untagged « apheretic form of estoy »; estoy only a form of estar (estar's
        # first three senses). Read as a meaning, toy was glossed « apheretic form of estoy ».
        glosses, runs, _ = self.tables(["toy"], TOY, ESTOY, ESTAR)
        self.assertEqual(
            glosses["toy"],
            "to be (have a temporary or permanent location in space); to be present; "
            "to be (denotes a copula, in a transient fashion)",
        )
        self.assertEqual(runs["toy"], [("VERB", 3)])

    def test_a_multi_word_target_is_lent_whole(self):
        # Recorded: « po favó », « pronunciation spelling of por favor », an expression nothing glossed.
        _, _, expressions = self.tables([], PO_FAVO, POR_FAVOR)
        self.assertEqual(expressions["po favó"], "please; you're welcome")

    def test_a_form_with_no_meaning_and_no_lender_stays_a_pointer(self):
        # Recorded: « ejque » names a garbled target, « es que in the Madrid dialect »; « seó » names
        # `seor`, itself only « syncopic form of señor » — whose pointer is in its text, not in a
        # field — so seó stays a pointer, while seor takes señor's senses (its first two). Made up:
        # « tas », « apheretic form of estás », without its target in the file — untagged, it was
        # read as a meaning; the edition's wording makes it a pointer.
        ejque, seo, seor, _, tas = self.read(EJQUE, SEO, SEOR, SENOR, TAS)
        self.assertEqual((ejque, seo, tas), (EJQUE, SEO, TAS))
        self.assertTrue(EN.form_of.match(TAS["senses"][0]["glosses"][0]))
        self.assertEqual(
            seor["senses"],
            [
                {
                    "glosses": ["mister, sir, lord (title conferred on a married or older male)"],
                    "tags": ["masculine"],
                },
                {"glosses": ["gentleman"], "tags": ["masculine"]},
            ],
        )
        written, _ = self.gloss(EN, SPANISH, {"tas"}, TAS)
        self.assertEqual(written, {})
        glosses, _, expressions = self.tables(["ejque", "tas"], EJQUE, TAS)
        self.assertEqual((glosses, expressions), ({}, {}))

    # — D4: a function word does not open on a place's name —

    def test_spec_scenario_a_function_word_spelled_like_a_place(self):
        # Recorded: `Como`, the Italian city, stands before como's adverb, conjunction and
        # preposition in the file, so the round-robin opened como on the city, twice.
        entries = (COMO_NAME, COMO_ADV, COMO_CONJ, COMO_PREP, COMO_VERB)
        written, _ = self.gloss(EN, SPANISH, {"como"}, *entries)
        self.assertTrue(written["como"].startswith("Como (a city and comune"), written["como"])
        self.assertEqual([(e["word"], e["pos"]) for e in self.read(*entries)], [
            ("como", "adv"), ("como", "conj"), ("como", "prep"), ("como", "verb"), ("Como", "name"),
        ])
        glosses, runs, _ = self.tables(["como"], *entries)
        self.assertEqual(
            glosses["como"],
            "as (to such an extent or degree); like, about (approximately); "
            "as (introducing a basis of comparison or equality); as, since (being that); "
            "as (in the manner or role specified); such as (for example); "
            "Como (a city and comune, the capital of the province of Como, Lombardy); "
            "Como (a province of Lombardy, Italy)",
        )
        self.assertEqual(runs["como"], [("ADV", 2), ("SCONJ", 2), ("ADP", 2), ("PROPN", 2)])

    def test_spec_scenario_an_acronym_keeps_its_place(self):
        # Recorded: `pr` has only pointers — « abbreviation of por », « para », « pero » — and `PR`,
        # all in capitals, « initialism of Puerto Rico »: pr borrows from the first in file order,
        # Puerto Rico. Moved last, `PR` would open pr on por's « by » (por's first three senses).
        entries = (PR_NAME, PUERTO_RICO, PR_PREP, PR_CONJ, POR_PREP)
        self.assertEqual(self.read(*entries), [json.loads(json.dumps(e)) for e in entries], "nothing moved")
        glosses, _, _ = self.tables(["pr"], *entries)
        self.assertEqual(
            glosses["pr"],
            "Puerto Rico (a commonwealth, island and dependent territory of the United States in the Caribbean, "
            "official name: Estado Libre Asociado de Puerto Rico)",
        )
        moved, _ = self.gloss(EN, SPANISH, {"pr"}, PUERTO_RICO, PR_PREP, PR_CONJ, POR_PREP, PR_NAME)
        self.assertTrue(moved["pr"].startswith("by; for"), moved["pr"])

    def test_a_proper_noun_before_a_common_word_keeps_its_place(self):
        # Recorded: `Chile` before `chile`, `Amor` (a surname) before `amor`, in file order. Neither
        # common word is a function word: their lines stay where they are (Q3, for the owner).
        entries = (CHILE_NAME, CHILE_NOUN, AMOR_NAME, AMOR_NOUN)
        self.assertEqual(self.read(*entries), [json.loads(json.dumps(e)) for e in entries])
        glosses, _, _ = self.tables(["chile", "amor"], *entries)
        self.assertTrue(glosses["chile"].startswith("Chile (a country in South America"), glosses["chile"])
        self.assertEqual(glosses["amor"], "a surname; love; love affair")

    def test_a_headword_s_moved_lines_stay_one_run(self):
        # Made up from `Como`'s: its senses as two `name` entries, two etymologies, and a made-up
        # place `Ni` written between them. Each headword's moved lines are written together, in
        # their order, after every other line, so the merging still reads them as one run.
        como_a = {**COMO_NAME, "senses": COMO_NAME["senses"][:2]}
        como_b = {**COMO_NAME, "senses": COMO_NAME["senses"][2:]}
        ni = {"word": "Ni", "pos": "name", "senses": [{"glosses": ["a made-up place"]}]}
        lines = self.read(como_a, ni, como_b, COMO_CONJ, NI_CONJ)
        self.assertEqual(
            [(e["word"], e["pos"], len(e["senses"])) for e in lines],
            [("como", "conj", 4), ("ni", "conj", 1), ("Como", "name", 2), ("Como", "name", 6), ("Ni", "name", 1)],
        )
        merged = english.merge_same_pos_etymologies(str(self.dir / "meanings.jsonl"), str(self.dir / "merged.jsonl"), merged=True)
        lines = [json.loads(line) for line in Path(merged).read_text(encoding="utf-8").splitlines()]
        self.assertEqual(
            [(e["word"], e["pos"], len(e["senses"])) for e in lines],
            [("como", "conj", 4), ("ni", "conj", 1), ("Como", "name", 8), ("Ni", "name", 1)],
        )

    # — D5: one English typography —

    def test_spec_scenario_the_edition_s_description_in_lower_case(self):
        # Recorded: ni's conjunction and adverb, se's first sense.
        glosses, _, _ = self.tables(["ni", "se"], NI_CONJ, NI_ADV, SE_PRON)
        self.assertEqual(
            glosses["ni"],
            "used when negating two or more elements, and not, not A or B, not A nor B, neither A nor B; "
            "used in emphatic negations, not even; forms an emphatic negative imperative, don't even",
        )
        self.assertTrue(glosses["se"].startswith("a reflexive or reciprocal pronoun: oneself, "), glosses["se"])

    def test_spec_scenario_a_capital_that_is_no_description(self):
        # Recorded: a ballet's title, a species, a grade and a month keep their capital.
        for gloss in ("The Nutcracker (ballet)", "The Eurasian treecreeper.", "A (highest grade in testing)", "July"):
            self.assertEqual(english.english_typography(gloss), gloss)
        glosses, _, expressions = self.tables(
            ["cascanueces", "diez", "julio"], CASCANUECES, AGATEADOR_NORTENO, DIEZ_NOUN, JULIO
        )
        self.assertEqual(
            glosses, {"cascanueces": "The Nutcracker (ballet)", "diez": "A (highest grade in testing)", "julio": "July"}
        )
        self.assertEqual(expressions["agateador norteño"], "The Eurasian treecreeper")
        # Only the closed list, and only before a letter.
        for gloss in ("Thing (a capital kept)", "Use of", "In 1990", "Interjections"):
            self.assertEqual(english.english_typography(gloss), gloss)

    def test_spec_scenario_one_ellipsis(self):
        # Recorded: nada's pronoun, tanto's and o's conjunctions, ahora's conjunction. The shared
        # cleaning strips a sense's final periods, but « … » is no period: ahora keeps its last one.
        glosses, _, _ = self.tables(
            ["nada", "tanto", "o", "ahora"], NADA_PRON, TANTO_CONJ, O_CONJ, AHORA_CONJ
        )
        self.assertEqual(
            glosses,
            {
                "nada": "nothing, zero, zilch, not … anything",
                "tanto": "both … and (introduces the first of two linked elements)",
                "o": "either … or",
                "ahora": "now … now, whether … or…",
            },
        )
        written, _ = self.gloss(EN, SPANISH, {"ahora"}, AHORA_CONJ)
        self.assertEqual(written["ahora"], "now...now, whether...or")
        self.assertEqual(english.english_typography("let's see..."), "let's see…")
        self.assertEqual(english.english_typography("my name is ..., I am ..."), "my name is …, I am …")

    def test_spec_scenario_paired_quotes(self):
        # Recorded: otro's interjection; chamorro's single quotes stay as written. Made up: an odd
        # number of straight quotes stays.
        glosses, _, _ = self.tables(["otro", "chamorro"], OTRO_INTJ, CHAMORRO)
        self.assertEqual(glosses["otro"], "“Not again!” or “What, again?” (also Otra vez! or Otra vez?)")
        self.assertEqual(glosses["chamorro"], "shank (of pork, beef, etc.), e.g. 'chamorro de puerco', pork shank")
        self.assertEqual(english.english_typography('a 12" record, "vinyl"'), 'a 12" record, "vinyl"')

    def test_spec_scenario_a_numbered_sense_of_the_source(self):
        # Recorded: ya's first four senses, jurado's two. A bracketed sense number goes first, so a
        # parenthesis that says more stays; the sense's own `;` is written `,` by the shared rules.
        glosses, _, _ = self.tables(["ya", "jurado"], YA_ADV, JURADO_NOUN)
        self.assertEqual(
            glosses,
            {
                "ya": "now, right now, (in the negative) anymore, no longer; indicates completion of an action; "
                "yet (in questions)",
                "jurado": "juror, juryman, juryperson (member of a jury); "
                "judge (member of a jury, officiator of a competitive event)",
            },
        )

    def test_spec_scenario_an_example_after_a_line_break(self):
        # Recorded: canino's adjective, an example sentence and its translation after line breaks.
        written, _ = self.gloss(EN, SPANISH, {"canino"}, CANINO_ADJ)
        self.assertIn("Marcos siempre estaba canino", written["canino"])
        glosses, _, _ = self.tables(["canino"], CANINO_ADJ)
        self.assertEqual(glosses["canino"], "canine; ravenously hungry, hungry as a hog")

    def test_an_expression_gets_the_same_rules(self):
        # Recorded: « a la mierda » « screw this; to hell with... », which lost its ellipsis.
        written = common.reduce_expressions(self.jsonl(A_LA_MIERDA), 80, studied=SPANISH, edition=EN)
        self.assertEqual(written["a la mierda"], "screw this; to hell with")
        _, _, expressions = self.tables([], A_LA_MIERDA)
        self.assertEqual(expressions["a la mierda"], "screw this; to hell with…")

    # — D1: where the rules live —

    def test_the_pre_pass_writes_what_it_cannot_read_as_it_is(self):
        # Made up: lines that are no JSON object, an entry it does not change, a sense of no
        # kaikki shape. Each is written as it is, byte for byte.
        src = self.dir / "entries.jsonl"
        text = (
            "not json\n[1, 2]\n"
            '{"word":"casa","pos":"noun","senses":[{"glosses":["house"],"tags":["feminine"]}]}\n'
            '{"word": "raro", "pos": "adj", "senses": ["odd", {"glosses": [1]}, {"tags": ["no-gloss"]}]}\n'
            '{"word": 3, "senses": 4}\n'
        )
        src.write_text(text, encoding="utf-8")
        out = english.read_as_meanings(str(src), str(self.dir / "meanings.jsonl"))
        self.assertEqual(Path(out).read_text(encoding="utf-8"), text)

    def test_reduce_es_en_runs_it_before_the_etymology_merging(self):
        # es-en reduced as its build runs it (`main()`): the pre-pass reads the file the letters'
        # passes wrote, and the merging reads the pre-pass's.
        work, studied = self.dir / "work", self.dir / "es"
        work.mkdir()
        studied.mkdir()
        (studied / "forms.tsv").write_text("venir\tvenir\ncosita\tcosita\n", encoding="utf-8")
        (studied / "freq.tsv").write_text("venir\t1\ncosita\t2\n", encoding="utf-8")
        self.jsonl(VENIR, COSITA, name="work/kaikki-Spanish.jsonl")
        self.jsonl(name="work/kaikki-es-traductions-en.jsonl")
        calls = []

        def spy(name, real):
            def call(src, dst, **kwargs):
                calls.append((name, Path(src).name, Path(dst).name))
                return real(src, dst, **kwargs)

            return call

        argv = [
            "reduce-es-en.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-08", "--pack-version", "test"),
        ]
        with (
            mock.patch.object(sys, "argv", argv),
            mock.patch.object(english, "read_as_meanings", spy("read_as_meanings", english.read_as_meanings)),
            mock.patch.object(
                english, "merge_same_pos_etymologies", spy("merge", english.merge_same_pos_etymologies)
            ),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            es_en.main()
        self.assertEqual(
            calls,
            [
                ("read_as_meanings", "kaikki-Spanish-headwords.jsonl", "kaikki-Spanish-meanings.jsonl"),
                ("merge", "kaikki-Spanish-meanings.jsonl", "kaikki-Spanish-merged.jsonl"),
            ],
        )
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8").splitlines()[0], "cosita\tsmall thing; thingy"
        )

    def test_spec_scenario_nothing_else_moves(self):
        # The rules are the English edition's: es-en's rule digest moves with them — and fr-en's,
        # glossed by the same edition since add-lingua-pack-fr-en — and no French- or
        # Spanish-glossed pair's: en-fr's and es-fr's load the French edition, en-es's the Spanish
        # one — and reduce_common.py, which every pair loads, is not edited. The committed pins
        # record the rules they were reduced with: es-en's, re-pinned with these rules, and the other
        # three, as they were.
        pairs = ("en-fr", "es-fr", "es-en", "en-es")
        for pair in pairs:
            self.assertEqual(
                ps.rules_sha256(Path(_HERE) / f"reduce-{pair}.py"),
                ps.get(ps.load(Path(_HERE) / "tables" / pair / "pin.json"), "reducer.sha256"),
                pair,
            )
        copy = self.dir / "rules"
        copy.mkdir()
        for path in Path(_HERE).glob("reduce[-_]*.py"):
            (copy / path.name).write_bytes(path.read_bytes())
        before = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        for name, old, new in (
            ("reduce_edition_en.py", '_FUNCTION_WORDS = frozenset({"prep", "conj", "pron", "det", "article"})', "_FUNCTION_WORDS = frozenset()"),
            ("reduce_edition_en.py", "(?:pronunciation|eye dialect) spelling", "pronunciation spelling"),
            ("reduce_edition_en.py", 'text.count(\'"\') % 2 == 0', "False"),
            ("reduce-es-en.py", "entries = english.read_as_meanings(", "entries = (lambda src, dst: src)("),
        ):
            edition = copy / name
            text = edition.read_text(encoding="utf-8")
            self.assertIn(old, text)
            edition.write_text(text.replace(old, new), encoding="utf-8")
            after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
            self.assertNotEqual(after["es-en"], before["es-en"], new)
            self.assertEqual({p: after[p] for p in ("en-fr", "es-fr", "en-es")}, {p: before[p] for p in ("en-fr", "es-fr", "en-es")}, new)
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
        for pair, editions in ps.DUMPS.items():
            studied, native = pair.split("-")
            for edition, names in editions.items():
                for name in names:
                    kind, lang, *into = ps.EDITIONS[edition]["files"][name]
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


# — The Spanish Wiktionary's English section, and the two translation tables en-es reads —

HOUSE_ES = {"word": "house", "pos": "noun", "senses": [{"glosses": ["Casa, vivienda."]}, {"glosses": ["Hogar."]}]}
RUN_ES_FORM = {"word": "ran", "pos": "verb", "senses": [{"glosses": ["Pasado simple del verbo (to) run."]}]}
CUE_ES = CUE
B_CHARACTER_ES = {"word": "b", "pos": "character", "senses": [{"glosses": ["Segunda letra del alfabeto."]}]}
GIVE_UP_ES = {"word": "give up", "pos": "verb", "senses": [{"glosses": ["Rendirse."]}]}
# The English Wiktionary's Spanish translations (derived from its English extract).
SECTOR_TRANSLATED = {
    "word": "sector",
    "pos": "noun",
    "translations": [{"word": "sector"}, {"word": "área"}, {"word": "campo"}, {"word": "zona"}],
}
RUN_TRANSLATED = {"word": "run", "pos": "verb", "translations": [{"word": "correr"}, {"word": "fluir", "sense": "of a liquid"}]}
HOUSE_TRANSLATED = {"word": "house", "pos": "noun", "translations": [{"word": "hogar"}]}
B_TRANSLATED_EN = {"pos": "character", "translations": [{"word": "b"}], "word": "b"}
A_TRANSLATED_EN = {"pos": "det", "translations": [{"word": "un"}, {"word": "una"}], "word": "a"}
GIVE_IN_TRANSLATED = {"word": "give in", "pos": "verb", "translations": [{"word": "ceder"}]}
# The Spanish Wiktionary's English translations (es-en's direct table), read backwards.
PERRO_LISTS_DOG = {"word": "perro", "pos": "noun", "translations": [{"word": "dog"}]}
CAN_LISTS_DOG = {"word": "can", "pos": "noun", "translations": [{"word": "dog"}]}
SEVILLA_LISTS = {"word": "Sevilla", "pos": "name", "translations": [{"word": "Seville"}]}
I_LISTS_I = {"word": "i", "pos": "noun", "translations": [{"sense": "letra", "word": "i"}]}
# Recorded: the English Wiktionary's one-letter words — the noun `i` under its Spanish name, the
# pronoun `I`, the vocative `O` — and the Spanish Wiktionary's entries listing one English letter:
# the note `do` (« C »: en-es's `c`, rank 376, was glossed « Do ») and the pronoun `yo`.
I_NOUN_TRANSLATED_EN = {"word": "i", "pos": "noun", "translations": [{"word": "i"}, {"word": "i latina"}]}
I_PRON_TRANSLATED_EN = {"word": "I", "pos": "pron", "translations": [{"word": "yo"}]}
O_TRANSLATED_EN = {"word": "O", "pos": "particle", "translations": [{"word": "oh"}, {"word": "oy"}]}
DO_LISTS_C = {"word": "do", "pos": "noun", "translations": [{"word": "C"}]}
YO_LISTS_I = {"word": "yo", "pos": "pron", "translations": [{"word": "I"}]}


# wordfreq's Spanish Zipf frequency, for a table with no note to judge: no word is frequent.
NO_WORD = lambda word: 0.0  # noqa: E731


class EnglishGlossedInSpanish(Entries):
    """en-es (add-lingua-pack-en-es D1, D4): the native side alone from English's committed tables,
    a definition first, then the English Wiktionary's Spanish translations, then the Spanish
    Wiktionary's English translations read backwards — and the share the tables gave."""

    def studied(self, forms, freq, level="", grammar=""):
        folder = self.dir / "en"
        folder.mkdir(exist_ok=True)
        (folder / "forms.tsv").write_text(forms, encoding="utf-8")
        (folder / "freq.tsv").write_text(freq, encoding="utf-8")
        (folder / "level.tsv").write_text(level, encoding="utf-8")
        (folder / "grammar.tsv").write_text(grammar, encoding="utf-8")
        return folder

    def wordfreq(self, zipf):
        module = types.ModuleType("wordfreq")
        module.zipf_frequency = lambda word, lang: zipf.get(word, 0.0) if lang == "es" else 0.0
        return mock.patch.dict(sys.modules, {"wordfreq": module})

    def reduce(self, work, studied, **zipf):
        argv = [
            "reduce-en-es.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-09", "--pack-version", "test"),
        ]
        with self.wordfreq(zipf), mock.patch.object(sys, "argv", argv), contextlib.redirect_stderr(io.StringIO()) as err:
            en_es.main()
        return err.getvalue()

    def test_spec_scenarios_a_definition_first_then_the_direct_then_the_inverted_table(self):
        # en-es reduced as its build runs it (`main()`), from a studied folder as en-fr writes it.
        # `house` has a Spanish Wiktionary entry: its definitions, by the Spanish edition's rules,
        # win over the translations the English Wiktionary lists. `sector` and `run` have none, and
        # the English Wiktionary lists their Spanish translations: at most three per part of
        # speech, in the table's order, opening on a capital as the edition's own senses do. `dog`
        # has neither, and two Spanish entries list it as their English translation: those lemmas,
        # the commonest Spanish word first. `ran` is a form — « Pasado simple del verbo » — and
        # nothing glosses `stone`.
        work = self.dir / "work"
        work.mkdir()
        studied = self.studied(
            "dog\tdog\nhouse\thouse\nran\tran\nrun\trun\nruns\trun\nsector\tsector\nstone\tstone\n",
            "house\t1\nrun\t2\ndog\t3\nsector\t4\nstone\t5\nran\t6\n",
        )
        self.jsonl(HOUSE_ES, RUN_ES_FORM, CUE_ES, B_CHARACTER_ES, GIVE_UP_ES, name="work/kaikki-es-English.jsonl")
        self.jsonl(
            SECTOR_TRANSLATED,
            RUN_TRANSLATED,
            HOUSE_TRANSLATED,
            B_TRANSLATED_EN,
            A_TRANSLATED_EN,
            GIVE_IN_TRANSLATED,
            name="work/kaikki-en-traductions-es.jsonl",
        )
        self.jsonl(CAN_LISTS_DOG, PERRO_LISTS_DOG, SEVILLA_LISTS, I_LISTS_I, name="work/kaikki-es-traductions-en.jsonl")
        err = self.reduce(work, studied, perro=5.0, can=3.0)
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "dog\tPerro, can\nhouse\tCasa, vivienda; Hogar\nrun\tCorrer, fluir\nsector\tSector, área, campo\n",
        )
        self.assertEqual(
            (work / "senses.tsv").read_text(encoding="utf-8"),
            "dog\tNOUN:1\nhouse\tNOUN:2\nrun\tVERB:1\nsector\tNOUN:1\n",
        )
        self.assertEqual((work / "mwe.tsv").read_text(encoding="utf-8"), "give in\tCeder\ngive up\tRendirse\n")
        # The native side alone (D1), and what the reducer measured of it (D4): among the glossed
        # lemmas of the commonest, the share from a translation table — here three of four.
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith("kaikki-"))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        measures = json.loads((work / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(
            measures, {"top": 10000, "glossed": 4, "share": 75.0, "direct": ["run", "sector"], "inverted": ["dog"]}
        )
        self.assertIn("reduced en-es: lemmas=6 (English's committed tables) glosses=4 (Spanish Wiktionary 1", err)
        self.assertIn("of the 4 glossed lemmas among the 10,000 commonest, 75.0 % come from a translation table", err)
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("en", "es"))
        self.assertNotIn("levels_estimated", manifest["meta"], "English's levels are CEFR-J's and Octanove's")
        self.assertEqual(manifest["meta"]["pack_version"], "test")
        self.assertEqual(manifest["meta"]["analyzer_version"], en_fr.analyser_version())
        notice = (work / "NOTICE").read_text(encoding="utf-8")
        for credit in ("ESDB", "WordNet", "eswiktionary", "enwiktionary", "frwiktionary", "CEFR-J", "Octanove", "wordfreq"):
            self.assertIn(credit, notice)

    def test_native_side_gives_the_tables_native_tables_gives(self):
        # The steps written out keep each source's lemmas, and change nothing of the tables.
        entries = self.jsonl(HOUSE_ES, RUN_ES_FORM, GIVE_UP_ES)
        ranks = {"house": 1, "run": 2, "dog": 3, "sector": 4, "stone": 5}
        direct = {"sector": {"NOUN": ["sector", "área"]}, "run": {"VERB": ["correr"]}, "house": {"NOUN": ["hogar"]}}
        inverted = {"dog": {"NOUN": ["can", "perro"]}, "run": {"VERB": ["andar"]}, "give in": {"VERB": ["ceder"]}}
        sources = [(direct, list), (inverted, en_es.by_spanish_frequency(lambda w: {"perro": 5.0, "can": 3.0}.get(w, 0.0)))]
        glosses, runs, expressions, steps = en_es.native_side(entries, ranks, sources)
        shared = common.native_tables(
            entries, ranks, studied=en_es.EN, edition=en_es.EDITION, fallbacks=sources, locutions=en_es.LOCUTIONS
        )
        self.assertEqual((glosses, runs, expressions), shared[:3])
        self.assertEqual(steps, {"entries": {"house"}, "direct": {"run", "sector"}, "inverted": {"dog"}})
        self.assertEqual(len(steps["entries"]), shared[3], "`primary`")
        self.assertEqual(glosses["dog"], "Perro, can")
        self.assertEqual(expressions, {"give up": "Rendirse", "give in": "Ceder"})

    def test_the_share_counts_the_commonest_alone(self):
        steps = {"entries": {"a", "c", "z"}, "direct": {"b", "y"}, "inverted": {"d"}}
        ranks = {"a": 1, "b": 2, "c": 3, "d": 4, "y": 5, "z": 6}
        self.assertEqual(
            en_es.translation_share(steps, ranks, top=4),
            {"top": 4, "glossed": 4, "share": 50.0, "direct": ["b"], "inverted": ["d"]},
        )
        self.assertEqual(en_es.translation_share(steps, ranks, top=6)["share"], 50.0)
        self.assertEqual(en_es.translation_share({"entries": set(), "direct": set(), "inverted": set()}, ranks)["share"], 0.0)

    def test_a_letter_s_translation_is_no_gloss_in_either_direction(self):
        # A single letter is glossed only by a sense that is neither the letter nor a name borrowed
        # through it. Read forwards: the English Wiktionary's `b` lists « b », and its noun `i`
        # « i, i latina », the letter under its Spanish name — neither glosses; a one-letter word
        # translated is kept: `a` « un, una », `I` « yo », the vocative `O` « oh, oy ».
        src = self.jsonl(
            B_TRANSLATED_EN,
            A_TRANSLATED_EN,
            I_NOUN_TRANSLATED_EN,
            I_PRON_TRANSLATED_EN,
            O_TRANSLATED_EN,
            name="kaikki-en-traductions-es.jsonl",
        )
        direct = en_es.read_translated(src, str(self.dir / "direct.jsonl"), inverted=False, frequency=NO_WORD, readings={})
        self.assertEqual(direct, {"a": {"DET": ["un", "una"]}, "i": {"PRON": ["yo"]}, "o": {"PART": ["oh", "oy"]}})
        # Read backwards, a letter is never glossed: the Spanish Wiktionary's `i` lists « i », and
        # its noun `do` lists « C » — recorded: `c` was glossed « Do », the note's name borrowed
        # through the letter, since the entry is the Spanish word's and the letter test sees
        # nothing of the English side. `yo` « I », the one word of a letter the table reaches, goes
        # with them: the entries and the direct table gloss `I` before it is read.
        src = self.jsonl(I_LISTS_I, PERRO_LISTS_DOG, DO_LISTS_C, YO_LISTS_I, name="kaikki-es-traductions-en.jsonl")
        inverted = en_es.read_translated(src, str(self.dir / "inverted.jsonl"), inverted=True, frequency=NO_WORD, readings={})
        self.assertEqual(inverted, {"dog": {"NOUN": ["perro"]}})
        glossed = common.fallback_glosses({"c", "dog", "i", "o"}, {}, [(direct, list), (inverted, list)], edition=ES)
        self.assertEqual(
            glossed, {"dog": ("Perro", [("NOUN", 1)]), "i": ("Yo", [("PRON", 1)]), "o": ("Oh, oy", [("PART", 1)])}
        )
        # On the committed tables: a one-letter lemma's gloss is neither the letter nor its name.
        committed = Path(_HERE, "tables", "en-es", "gloss.tsv")
        if committed.is_file():
            for line in committed.read_text(encoding="utf-8").splitlines():
                lemma, _, gloss = line.partition("\t")
                if len(lemma) != 1:
                    continue
                for sense in gloss.split("; "):
                    self.assertNotEqual(sense.strip().lower(), lemma, f"{lemma!r} glossed by itself")
                    self.assertNotIn("letra", sense.lower(), f"{lemma!r} glossed as a letter: {gloss!r}")

    def test_en_es_s_copy_of_english_is_en_fr_s(self):
        # reduce-en-es.py repeats en-fr's description of English (`_TOKEN`, the coordinators, the
        # form-of target) because a reducer loads no other pair's: the two must not drift.
        self.assertEqual(en_es.EN, en_fr.EN)
        self.assertEqual(
            (en_es.EN.token.pattern, en_es.EN.form_of_target.pattern, en_es.EN.coordinators),
            (en_fr.EN.token.pattern, en_fr.EN.form_of_target.pattern, en_fr.EN.coordinators),
        )
        self.assertIs(en_es.EDITION, ES)

    def test_en_es_reads_the_committed_studied_tables_as_en_fr_writes_them(self):
        # A studied folder whose forms and ranks disagree is not what en-fr's reduction writes; the
        # level lists' words are kept beyond the cap, as en-fr keeps them.
        studied = self.studied("house\thouse\nhouses\thouse\n", "house\t1\nperro\t2\n")
        with self.assertRaisesRegex(SystemExit, "disagree on the lemmas"):
            en_es.read_studied(str(studied), 40000)
        studied = self.studied("house\thouse\nrun\trun\nseldom\tseldom\n", "house\t1\nrun\t2\nseldom\t3\n", "seldom\tB2\n")
        self.assertEqual(en_es.read_studied(str(studied), 40000), {"house": 1, "run": 2, "seldom": 3})
        self.assertEqual(en_es.read_studied(str(studied), 1), {"house": 1, "seldom": 3}, "capped by rank, the levelled kept")
        (studied / "level.tsv").unlink()
        with self.assertRaisesRegex(SystemExit, "level.tsv is missing"):
            en_es.read_studied(str(studied), 40000)
        committed = en_es.read_studied(os.path.join(_HERE, "tables", "en"), 40000)
        ranked = sum(1 for line in Path(_HERE, "tables", "en", "freq.tsv").read_text(encoding="utf-8").splitlines() if "\t" in line)
        self.assertEqual(len(committed), ranked, "every committed lemma, the level lists' words beyond 40,000 included")
        self.assertGreater(max(committed.values()), 40000)


# — en-es's glosses read as meanings (refine-lingua-en-es-glosses) —
# Recorded from the files en-es pins (lingua-pack-sources-en-es-2026.10.08): the Spanish
# Wiktionary's English entries (`kaikki-es-English.jsonl`, its dump of 2026-10-02), cut down to the
# fields the rules read; GO_VERB, GO_VERB_TR and GO_NOUN keep their entry's first senses only.
STAGE_NOUN = {
    "word": "stage",
    "pos": "noun",
    "senses": [
        {"glosses": ["Fase, etapa."]},
        {"glosses": ["Escenario, escena."]},
        {"glosses": ["Platina de un microscopio."]},
        {"glosses": ["Área de descanso, área de servicio."]},
        {"glosses": ["Piso₃, planta₃ (de un edificio)."]},
    ],
}
STAGE_VERB = {
    "word": "stage",
    "pos": "verb",
    "senses": [
        {"glosses": ["Escenificar, poner en escena, representar."]},
        {"glosses": ["Identificar la fase de un proceso.^([cita requerida])"]},
        {"glosses": ["Producir o dirigir una puesta en escena."]},
        {"glosses": ["Detenerse entre dos etapas (de un viaje o de algún otro proceso)."]},
    ],
}
FAVOR_NOUN = {"word": "favor", "pos": "noun", "tags": ["countable", "uncountable"], "senses": [{"glosses": ["Favor."]}]}
FAVOR_VERB = {
    "word": "favor",
    "pos": "verb",
    "senses": [
        {"glosses": ["Mirar con cariño; preferir."]},
        {"glosses": ["Usar más frecuentemente."]},
        {"glosses": ["Alentar, conducir a."]},
        {"glosses": ["Hacer un favor [sentido del sustantivo] para; mostrar beneficencia hacia."]},
        {"glosses": ["Parecerse; especialmente, parecerse a (otra persona)."]},
    ],
}
HARDCORE_NOUN = {"word": "hardcore", "pos": "noun", "senses": [{"glosses": ["Hardcore (definiciones [1,2])."]}]}
HARDCORE_ADJ = {
    "word": "hardcore",
    "pos": "adj",
    "senses": [{"glosses": ["Hardcore (definiciones [4,5])."]}, {"glosses": ["Incondicional, acérrimo."]}],
}
LEAVEN_NOUN = {
    "word": "leaven",
    "pos": "noun",
    "tags": ["countable", "uncountable"],
    "senses": [
        {"glosses": ["Levadura, fermento."]},
        {"glosses": ["Influencia o causa, generalmente sutil o gradual."], "tags": ["figurative"]},
    ],
}
LEAVEN_VERB = {
    "word": "leaven",
    "pos": "verb",
    "senses": [
        {
            "glosses": [
                "Este lema en este idioma es ampliable. Retira este aviso si la mayor parte de las acepciones ya están "
                "incluidas."
            ]
        }
    ],
}
A_ARTICLE_ES = {
    "word": "a",
    "pos": "article",
    "tags": ["indeterminate"],
    "senses": [
        {"glosses": ["Un, una. A veces se omite en la traducción."]},
        {"glosses": ["Un tal, una tal. Precediendo a nombres personales de gente a la que apenas se conoce."]},
    ],
}
A_PREP_ES = {
    "word": "a",
    "pos": "prep",
    "senses": [
        {"glosses": ["Por, normalmente con sentido proporción."]},
        {
            "glosses": ["Denota la acción de un verbo cuando se halla delante del participio activo."],
            "tags": ["outdated"],
        },
    ],
}
WOW_INTJ = {
    "word": "wow",
    "pos": "intj",
    "senses": [
        {
            "glosses": [
                "Guau. Empleada para expresar sorpresa, perplejidad, entusiasmo, o sarcasticamente desaprobación."
            ]
        }
    ],
}
WOW_VERB = {"word": "wow", "pos": "verb", "senses": [{"glosses": ["Asombrar."], "tags": ["colloquial"]}]}
DEGREE = {
    "word": "degree",
    "pos": "noun",
    "senses": [
        {
            "glosses": [
                "Grado. A no confundir con Licenciatura (en el sistema de educación español) que constan de dos ciclos. "
                "Los actuales grados sería el equivalente de las antiguas Diplomaturas (en el sistema de educación "
                "español)."
            ]
        },
        {"glosses": ["Nivel."]},
        {"glosses": ["Título académico."]},
    ],
}
ISNT = {
    "word": "isn't",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ['Contracción de el verbo is y el adverbio not. Traducida como "no es" o "no está".']}],
}
WERE = {
    "word": "we're",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ["Contracción de we are. Nosotros somos o nosotros estamos."]}],
}
PILOT_NOUN = {
    "word": "pilot",
    "pos": "noun",
    "senses": [
        {"glosses": ["Piloto. Persona que dirige un avión."]},
        {
            "glosses": [
                "Piloto. Persona que conoce bien las profundidades y corrientes de una bahía o zona costera, quien es "
                "contratado para ayudar en la navegación en ese lugar."
            ]
        },
    ],
}
RIVER_PUMPKIN = {
    "word": "river pumpkin",
    "pos": "noun",
    "senses": [
        {"glosses": ["(Gunnera perpensa) Especie de gunera.^([definición imprecisa])"], "raw_tags": ["Plantas"]}
    ],
}
GO_VERB = {
    "word": "go",
    "pos": "verb",
    "tags": ["intransitive"],
    "senses": [
        {"glosses": ["Andar, marchar, caminar."], "tags": ["obsolete", "outdated"]},
        {"glosses": ["Ir."]},
        {"glosses": ["Irse, marcharse, partir."]},
    ],
}
GO_VERB_TR = {
    "word": "go",
    "pos": "verb",
    "tags": ["transitive"],
    "senses": [{"glosses": ["Marchar."]}, {"glosses": ["Repartir."]}],
}
GO_NOUN = {
    "word": "go",
    "pos": "noun",
    "tags": ["irregular"],
    "senses": [{"glosses": ["Turno."]}, {"glosses": ["Intento."]}],
}
GO_INTJ = {
    "word": "go",
    "pos": "intj",
    "senses": [
        {"glosses": ["Se emplea para iniciar un juego o competencia. ¡Ya!, ¡ahora!, ¡fuera!, ¡vamos!."]},
        {"glosses": ["Se emplea para infundir ánimo. ¡Vamos!, ¡adelante!."]},
    ],
}
GO_GAME = {"word": "go", "pos": "noun", "senses": [{"glosses": ["Go (juego)."]}]}
WILL_NOUN = {
    "word": "will",
    "pos": "noun",
    "tags": ["countable", "uncountable"],
    "senses": [
        {"glosses": ["Deseo, inclinación, disposición."], "tags": ["outdated"]},
        {"glosses": ["En particular, deseo sexual."], "tags": ["obsolete"]},
        {"glosses": ["Deleite, placer, goce."], "tags": ["obsolete"]},
        {"glosses": ["Voluntad, albedrío."]},
        {"glosses": ["Decisión, intención."]},
        {"glosses": ["Testamento."]},
    ],
}
WILL_VERB = {
    "word": "will",
    "pos": "verb",
    "tags": ["transitive"],
    "senses": [
        {"glosses": ["Querer, desear."], "tags": ["outdated"]},
        {"glosses": ["Testar."]},
        {"glosses": ["Legar."]},
    ],
}
WILL_MODAL = {
    "word": "will",
    "pos": "verb",
    "tags": ["modal"],
    "senses": [{"glosses": ["Úsase para construir el futuro."]}],
}
WILL_NAME = {
    "word": "Will",
    "pos": "name",
    "senses": [{"glosses": ["Apellido."]}, {"glosses": ["Hipocorístico de William."]}],
}
THOU = {
    "word": "thou",
    "pos": "pron",
    "tags": ["personal"],
    "senses": [
        {
            "glosses": ["Tú, vos (pronombre personal de la segunda persona del singular)."],
            "tags": ["England", "Ireland", "jocular", "literary", "outdated"],
            "raw_tags": ["dialectal", "religión", "requiere la terminación -(e)st en el verbo"],
        }
    ],
}
THY = {
    "word": "thy",
    "pos": "pron",
    "tags": ["possessive"],
    "senses": [{"glosses": ["Vuestro, vuestra, vuestros, vuestras."], "tags": ["obsolete"]}],
}
CLOSE_FIELD = {
    "word": "close",
    "pos": "noun",
    "senses": [
        {"glosses": ["Un campo encerrado."], "tags": ["rare"]},
        {"glosses": ["Una calle sin salida."], "tags": ["UK"]},
    ],
}
ON_PREP = {
    "word": "on",
    "pos": "prep",
    "senses": [
        {"glosses": ["Posicionado sobre la superficie superior de ..."]},
        {"glosses": ["Que ocurre en la fecha especificada."]},
        {"glosses": ["Relacionado con el tema ..., sobre el tema ..."]},
        {"glosses": ["Indica contacto con ..."]},
        {"glosses": ["Utilizado para indicar el estar en el estado o en el proceso que se especifica."]},
    ],
}
NEITHER_CONJ = {"word": "neither", "pos": "conj", "senses": [{"glosses": ["(neither ... nor) Ni."]}]}
NEITHER_ADV = {"word": "neither", "pos": "adv", "tags": ["negative"], "senses": [{"glosses": ["Tampoco."]}]}
WHOD = {
    "word": "who'd",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [
        {"glosses": ["Contracción de el pronombre who y el verbo had; ¿Quién tenía / tuvo / había / hubo....?"]},
        {"glosses": ["Contracción de el pronombre who y el verbo would; ¿Quién (+ condicional)....?"]},
    ],
}
ITS_CONTRACTION = {
    "word": "it's",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ['Contracción de el pronombre it ("ello") y el verbo is ("es").']}],
}
IM = {
    "word": "I'm",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ['Contracción de I y am, "yo soy" o "yo estoy".']}],
}
BYRON = {
    "word": "Byron",
    "pos": "name",
    "senses": [
        {"glosses": ["Nombre de pila de varón."]},
        {"glosses": ["Apellido"]},
        {"glosses": ['Poeta inglés autor de "Don Juan", " Las peregrinaciones de Childe Harold", etc.']},
    ],
}
THATS = {
    "word": "that's",
    "pos": "contraction",
    "tags": ["contraction"],
    "senses": [{"glosses": ["Contracción de 'that is', ese es, esa es, aquel es, aquella es."]}],
}
SMITH = {"word": "smith", "pos": "noun", "senses": [{"glosses": ["Herrero."], "raw_tags": ["Oficios"]}]}
SMITH_NAME = {"word": "Smith", "pos": "name", "senses": [{"glosses": ["Apellido."]}]}
MIKE = {"word": "mike", "pos": "noun", "senses": [{"glosses": ["Micro, micrófono."], "tags": ["colloquial"]}]}
MIKE_NAME = {
    "word": "Mike",
    "pos": "name",
    "senses": [{"glosses": ["Hipocorístico de Michael."]}, {"glosses": ["Hipocorístico de Michaela."]}],
}
WAYNE_NAME = {"word": "Wayne", "pos": "name", "senses": [{"glosses": ["Apellido."]}]}
DONALD_NAME = {
    "word": "Donald",
    "pos": "name",
    "senses": [
        {"glosses": ["Nombre de pila de varón."]},
        {"glosses": ["Apellido."]},
        {"glosses": ["Nombre de varios lugares."]},
    ],
}
SOUTH_NOUN = {
    "word": "south",
    "pos": "noun",
    "tags": ["countable", "uncountable"],
    "senses": [{"glosses": ["Sur."], "raw_tags": ["física"]}],
}
SOUTH_ADJ = {"word": "south", "pos": "adj", "senses": [{"glosses": ["Del sur."]}]}
SOUTH_ADV = {"word": "south", "pos": "adv", "senses": [{"glosses": ["Al sur."]}]}
SOUTH_NAME = {
    "word": "South",
    "pos": "name",
    "senses": [{"glosses": ["Apellido."]}, {"glosses": ["(region) Sur."], "raw_tags": ["Regiones"]}],
}
DON_NOUN = {
    "word": "don",
    "pos": "noun",
    "senses": [{"glosses": ["Profesor de una universidad, en particular Oxford y Cambridge."]}],
}
DON_VERB = {"word": "don", "pos": "verb", "tags": ["transitive"], "senses": [{"glosses": ["Vestirse, ponerse."]}]}
DON_NAMES = {
    "word": "Don",
    "pos": "name",
    "senses": [
        {"glosses": ["Hipocorístico de Donald."]},
        {"glosses": ["Hipocorístico de Donovan."]},
        {"glosses": ["Hipocorístico de Gordon."]},
        {"glosses": ["Apellido."]},
    ],
}
DON_RIVER = {"word": "Don", "pos": "name", "senses": [{"glosses": ["El río Don."], "raw_tags": ["Ríos"]}]}
HER_PRON = {
    "word": "her",
    "pos": "pron",
    "tags": ["personal"],
    "senses": [{"glosses": ["Ella."]}, {"glosses": ["La (pronombre personal de objeto directo)."]}],
}
HER_ADJ = {"word": "her", "pos": "adj", "tags": ["possessive"], "senses": [{"glosses": ["Su (de ella)."]}]}
MY_ADJ = {
    "word": "my",
    "pos": "adj",
    "tags": ["possessive"],
    "senses": [{"glosses": ["Mi, mío, de mí."], "raw_tags": ["se coloca delante del sustantivo"]}],
}
ITS_ADJ = {
    "word": "its",
    "pos": "adj",
    "tags": ["possessive"],
    "senses": [{"glosses": ["Su, sus; que pertenece a un animal o una cosa."]}],
}
ITS_PRON = {
    "word": "its",
    "pos": "pron",
    "tags": ["possessive"],
    "senses": [{"glosses": ["(El) suyo, (la) suya, (los) suyos o (las) suyas; que pertenece a un animal o una cosa."]}],
}
THEIR_ADJ = {
    "word": "their",
    "pos": "adj",
    "tags": ["possessive"],
    "senses": [{"glosses": ["Su, sus (de ellos o de ellas, o de una persona cuando no se conoce su sexo)."]}],
}
THAT_PRON = {
    "word": "that",
    "pos": "pron",
    "tags": ["demonstrative"],
    "senses": [{"glosses": ["Ese, esa, esos, esas."]}, {"glosses": ["Aquel, aquella, aquellos, aquellas."]}],
}
THAT_ADJ = {
    "word": "that",
    "pos": "adj",
    "tags": ["demonstrative"],
    "senses": [{"glosses": ["Ese."]}, {"glosses": ["Aquel."]}],
}
THAT_CONJ = {
    "word": "that",
    "pos": "conj",
    "senses": [{"glosses": ["Que."]}, {"glosses": ["Y pensar que; ojalá."]}, {"glosses": ["Para que."]}],
}
SUCH_ADJ = {
    "word": "such",
    "pos": "adj",
    "tags": ["demonstrative"],
    "senses": [{"glosses": ["Tal, tal es, así, así de, semejante, tan, de este tipo."]}],
}
SUCH_PRON = {"word": "such", "pos": "pron", "tags": ["demonstrative"], "senses": [{"glosses": ["Lo que, como tal."]}]}
ANY_ADJ = {
    "word": "any",
    "pos": "adj",
    "tags": ["indeterminate"],
    "senses": [
        {"glosses": ["Cualquier."]},
        {"glosses": ["Alguno, alguna."], "raw_tags": ["se usa solamente en una oración negativa o interrogativa"]},
    ],
}
MUCH_ADJ = {
    "word": "much",
    "pos": "adj",
    "tags": ["indeterminate"],
    "senses": [{"glosses": ["Mucho."], "raw_tags": ["se usa solamente con un sustantivo no contable"]}],
}
BE_VERB = {
    "word": "be",
    "pos": "verb",
    "tags": ["intransitive"],
    "senses": [{"glosses": ["Ser."]}, {"glosses": ["Estar."]}, {"glosses": ["Encontrarse."]}],
}
BE_AUXILIARY = {
    "word": "be",
    "pos": "verb",
    "tags": ["auxiliary"],
    "senses": [
        {"glosses": ["Ser (be + participio pasado)."], "raw_tags": ["auxiliar de voz pasiva"]},
        {"glosses": ["Estar (be + participio presente)."], "raw_tags": ["auxiliar de progresivo"]},
        {"glosses": ["Se usa en be to (no existe en español)."]},
    ],
}
# The English Wiktionary's Spanish translations (`kaikki-en-traductions-es.jsonl`, derived from its
# English extract of 2026-10-03), in the file's order; HALL keeps its first table's words only.
ORCHESTRA_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"word": "orquesta"},
        {"word": "orquestra (disused)"},
        {"word": "orquestra (disused)"},
        {"word": "orquesta"},
    ],
    "word": "orchestra",
}
BLACKSMITH_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"word": "herrero"},
        {"word": "herrera"},
        {"word": "ferrero (disused)"},
        {"word": "herrador"},
        {"word": "herradora"},
        {"word": "ferrador (disused)"},
    ],
    "word": "blacksmith",
}
SEPTUM_TRANSLATED = {
    "pos": "noun",
    "translations": [{"word": "tabique"}, {"word": "septo"}, {"word": "septum"}, {"word": "septum (séptum)"}],
    "word": "septum",
}
MALIGN_TRANSLATED = {"pos": "verb", "translations": [{"word": "malignar (desus.)"}], "word": "malign"}
HALL_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"word": "pasillo"},
        {"word": "hall (hol)"},
        {"word": "jol"},
        {"word": "antesala"},
        {"word": "zaguán"},
    ],
    "word": "hall",
}
APP_TRANSLATED = {"pos": "noun", "translations": [{"word": "apli"}, {"word": "app (ap)"}], "word": "app"}
INSIDER_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"word": "insider (insáider)"},
        {"word": "adentrino"},
        {"word": "adentreño"},
        {"word": "dentreño"},
    ],
    "word": "insider",
}
BACKWATER_TRANSLATED = {
    "pos": "noun",
    "translations": [{"word": "pueblucho"}, {"word": "villorrio (despective)"}, {"word": "poblacho (despective)"}],
    "word": "backwater",
}
BACKWATER_VERB_TRANSLATED = {"pos": "verb", "translations": [{"word": "ciar"}], "word": "backwater"}
ONSIDE_TRANSLATED = {"pos": "adj", "translations": [{"word": "[4] a favor"}], "word": "onside"}
TO_BE_HONEST_TRANSLATED = {
    "pos": "phrase",
    "translations": [{"word": "para ser honesto [with le and a; or with con]"}, {"word": "si me apuras"}],
    "word": "to be honest",
}
QUIT_TRANSLATED = {
    "pos": "verb",
    "translations": [
        {"word": "dimitir (de)"},
        {"word": "renunciar (a)"},
        {"word": "U.S.: cuitear"},
        {"word": "quitear"},
    ],
    "word": "quit",
}
DAYCARE_TRANSLATED = {
    "pos": "noun",
    "translations": [{"word": "guardería (infantil)"}, {"word": "wawawasi"}, {"word": "nido"}],
    "word": "daycare",
}
FULL_TIME_TRANSLATED = {
    "pos": "adj",
    "translations": [{"word": "[a] tiempo completo"}, {"word": "de jornada completa"}],
    "word": "full-time",
}
PRAM_TRANSLATED = {
    "pos": "noun",
    "translations": [
        {"word": "cochecito [de bebé]"},
        {"word": "carrito [de bebé]"},
        {"word": "carriola"},
        {"word": "coche guagua"},
    ],
    "word": "pram",
}
ISRAELI_NOUN_TRANSLATED = {"pos": "noun", "translations": [{"word": "israelí"}], "word": "Israeli"}
ISRAELI_ADJ_TRANSLATED = {"pos": "adj", "translations": [{"word": "israelí"}], "word": "Israeli"}
MY_NAME_IS_TRANSLATED = {
    "pos": "phrase",
    "translations": [{"word": "me llamo ..."}, {"word": "mi nombre es ..."}],
    "word": "my name is",
}
DEEP_END_TRANSLATED = {"pos": "noun", "translations": [{"word": "fondón (disused)"}], "word": "deep end"}
WITH_BOTH_HANDS_TRANSLATED = {
    "pos": "prep_phrase",
    "translations": [{"word": "a manteniente (disused)"}],
    "word": "with both hands",
}
# The Spanish Wiktionary's English translations (`kaikki-es-traductions-en.jsonl`, derived from its
# dump of 2026-10-02), read backwards, in the file's order.
SPANISH_LISTING_GREY_LENGTHY_SEAMAN = [
    {"pos": "adj", "translations": [{"word": "gray"}, {"word": "grey"}], "word": "gris"},
    {"pos": "noun", "translations": [{"word": "gray"}, {"word": "grey"}], "word": "gris"},
    {"pos": "adj", "translations": [{"word": "long"}, {"word": "lengthy"}], "word": "largo"},
    {"pos": "noun", "translations": [{"word": "long"}, {"word": "lengthy"}], "word": "largo"},
    {"pos": "intj", "translations": [{"word": "long"}, {"word": "lengthy"}], "word": "largo"},
    {"pos": "verb", "translations": [{"word": "long"}, {"word": "lengthy"}], "word": "largo"},
    {"pos": "noun", "translations": [{"word": "lead"}, {"word": "gray"}, {"word": "grey"}], "word": "plomo"},
    {"pos": "adj", "translations": [{"word": "lead"}, {"word": "gray"}, {"word": "grey"}], "word": "plomo"},
    {"pos": "intj", "translations": [{"word": "lead"}, {"word": "gray"}, {"word": "grey"}], "word": "plomo"},
    {"pos": "adj", "translations": [{"word": "sailor"}, {"word": "seaman"}, {"word": "mariner"}], "word": "marinero"},
    {"pos": "noun", "translations": [{"word": "sailor"}, {"word": "seaman"}, {"word": "mariner"}], "word": "marinero"},
    {"pos": "verb", "translations": [{"word": "grey"}, {"word": "gray"}], "word": "agrisar"},
]
# Their readings in English's committed `tables/en/grammar.tsv` (form, lemma, reading, origin).
GREY_LENGTHY_SEAMAN_GRAMMAR = (
    "greyed\tgrey\tVERB|Mood=Ind|Tense=Past|VerbForm=Fin\tother\n"
    "greyer\tgrey\tADJ|Degree=Cmp\tother\n"
    "greys\tgrey\tNOUN|Number=Plur\tother\n"
    "lengthier\tlengthy\tADJ|Degree=Cmp\tother\n"
    "seamen\tseaman\tNOUN|Number=Plur\tother\n"
)
# wordfreq 3.1.1's Spanish Zipf frequency of the notes' words, as `zipf_frequency(word, "es")` gives it.
SPANISH_ZIPF = {
    "a": 7.36,
    "and": 4.95,
    "ap": 3.95,
    "bebé": 4.7,
    "con": 6.97,
    "de": 7.81,
    "despective": 0.0,
    "hol": 2.58,
    "infantil": 4.6,
    "insáider": 0.0,
    "le": 6.4,
    "or": 4.23,
    "with": 4.2,
}


def spanish_zipf(word):
    return SPANISH_ZIPF.get(word, 0.0)


class EnEsGlossesReadAsMeanings(Entries):
    """en-es's glosses read an English word's meanings, not its page's notes, in one Spanish
    typography, and its translation tables the translators' words (refine-lingua-en-es-glosses):
    the Spanish edition's notes and its pre-pass `reduce_edition_es.read_as_meanings`, then en-es's
    `english_entries`, both run by `reduce-en-es.py` before the shared rules read the file, and its
    `read_translated` over the two tables."""

    def read(self, *entries):
        """The entries as the edition's pre-pass writes them, in its order."""
        out = spanish.read_as_meanings(self.jsonl(*entries), str(self.dir / "meanings.jsonl"))
        return [json.loads(line) for line in Path(out).read_text(encoding="utf-8").splitlines()]

    def english(self, *entries):
        """The entries as en-es's pre-pass writes them, in its order."""
        out = en_es.english_entries(self.jsonl(*entries), str(self.dir / "glossing.jsonl"))
        return [json.loads(line) for line in Path(out).read_text(encoding="utf-8").splitlines()]

    def tables(self, lemmas, *entries, sources=()):
        """What en-es's shared rules make of the two pre-passes' file, as `main` runs them, with the
        translation tables `sources` after it: `(glosses, runs, expressions)`."""
        entries = spanish.read_as_meanings(self.jsonl(*entries), str(self.dir / "meanings.jsonl"))
        entries = en_es.english_entries(entries, str(self.dir / "glossing.jsonl"))
        ranks = {lemma: rank for rank, lemma in enumerate(lemmas, start=1)}
        glosses, runs, expressions, _ = en_es.native_side(entries, ranks, list(sources))
        return glosses, runs, expressions

    def direct(self, *entries):
        """The direct table as en-es reads it."""
        src = self.jsonl(*entries, name="kaikki-en-traductions-es.jsonl")
        return en_es.read_translated(
            src, str(self.dir / "direct.jsonl"), inverted=False, frequency=spanish_zipf, readings={}
        )

    # — D2: the edition's notes to its readers, and a usage note after the meaning —

    def test_spec_scenario_the_edition_s_notes_to_its_readers(self):
        # Recorded: stage, favor, hardcore and leaven. hardcore's noun and adjective then read the
        # same « Hardcore », picked once; leaven's verb said nothing but the expansion notice, and goes.
        glosses, runs, _ = self.tables(
            ["stage", "favor", "hardcore", "leaven"],
            STAGE_NOUN,
            STAGE_VERB,
            FAVOR_NOUN,
            FAVOR_VERB,
            HARDCORE_NOUN,
            HARDCORE_ADJ,
            LEAVEN_NOUN,
            LEAVEN_VERB,
        )
        self.assertIn("; Identificar la fase de un proceso; ", glosses["stage"])
        self.assertEqual(
            glosses["favor"],
            "Favor; Mirar con cariño, preferir; Usar más frecuentemente; Alentar, conducir a; "
            "Hacer un favor para, mostrar beneficencia hacia; Parecerse, especialmente, parecerse a (otra persona)",
        )
        self.assertEqual(
            (glosses["hardcore"], runs["hardcore"]), ("Hardcore; Incondicional, acérrimo", [("NOUN", 1), ("ADJ", 1)])
        )
        self.assertEqual(
            (glosses["leaven"], runs["leaven"]),
            ("Levadura, fermento; Influencia o causa, generalmente sutil o gradual", [("NOUN", 2)]),
        )
        # Each note, wherever it sits; recorded: unit's and bittersweet's templates.
        for written, read in (
            ("Un apartamento que alberga una casa.^([cita requerida])", "Un apartamento que alberga una casa"),
            ("Agridulce.^([definición imprecisa])", "Agridulce"),
            ("Hardcore (acepciones [1–3]).", "Hardcore"),
            ("Este lema en este idioma es ampliable. Retira este aviso…", ""),
        ):
            self.assertEqual(common.clean_gloss(written, 300, edition=ES), read)

    def test_spec_scenario_a_usage_note_after_the_meaning(self):
        # Recorded: a's article and preposition, wow, degree. The note goes with the rest of the
        # sense, from the meaning's period on.
        glosses, _, _ = self.tables(["a", "wow", "degree"], A_ARTICLE_ES, A_PREP_ES, WOW_INTJ, WOW_VERB, DEGREE)
        self.assertEqual(
            glosses,
            {
                "a": "Un, una; Un tal, una tal; Por, normalmente con sentido proporción; "
                "Denota la acción de un verbo cuando se halla delante del participio activo",
                "wow": "Guau; Asombrar",
                "degree": "Grado; Nivel; Título académico",
            },
        )
        # The closed list's openers, each after a meaning's period (recorded senses of however,
        # cheese and a); never a period that opens the sense or follows a space.
        for written, read in (
            (
                "En cualquier grado o extensión. Utilizado con un adjetivo o un adverbio.",
                "En cualquier grado o extensión",
            ),
            ("Patata, whisky. Se dice para sonreír cuando tomando un foto.", "Patata, whisky"),
            ("Un tal, una tal. Precediendo a nombres personales.", "Un tal, una tal"),
            ("Contracción de would y have. Usado para «habría sido».", "Contracción de would y have"),
        ):
            self.assertEqual(common.clean_gloss(written, 300, edition=ES), read)

    def test_spec_scenario_a_second_sentence_that_carries_the_meaning(self):
        # Recorded: isn't, we're, pilot — « Traducida », « Nosotros », « Persona » are no openers.
        glosses, _, _ = self.tables(["isn't", "we're", "pilot"], ISNT, WERE, PILOT_NOUN)
        self.assertEqual(
            glosses["isn't"], "Contracción de el verbo is y el adverbio not. Traducida como «no es» o «no está»"
        )
        self.assertEqual(glosses["we're"], "Contracción de we are. Nosotros somos o nosotros estamos")
        self.assertTrue(glosses["pilot"].startswith("Piloto. Persona que dirige un avión; Piloto. Persona que conoce"))

    def test_optional_words_in_brackets_stay(self):
        # Recorded: full-time's and pram's translations — « [a] », « [de bebé] » are Spanish, part
        # of the gloss.
        direct = self.direct(FULL_TIME_TRANSLATED, PRAM_TRANSLATED)
        self.assertEqual(
            direct,
            {
                "full-time": {"ADJ": ["[a] tiempo completo", "de jornada completa"]},
                "pram": {"NOUN": ["cochecito [de bebé]", "carrito [de bebé]", "carriola", "coche guagua"]},
            },
        )

    def test_an_expression_gets_the_same_rules(self):
        # Recorded: « river pumpkin », whose committed gloss was cut inside its note (« Especie de
        # gunera.^([de »), and « my name is » from the direct table, which kept no ellipsis.
        _, _, expressions = self.tables([], RIVER_PUMPKIN, sources=[(self.direct(MY_NAME_IS_TRANSLATED), list)])
        self.assertEqual(
            expressions,
            {"river pumpkin": "(Gunnera perpensa) Especie de gunera", "my name is": "Me llamo …, mi nombre es …"},
        )

    # — D5: current senses first —

    def test_spec_scenario_current_senses_first(self):
        # Recorded: go's verbs, nouns and interjection; will's noun and verbs, and `Will`. The
        # labelled senses go after the others of their entry, in their order: nothing is left out.
        written = self.read(GO_VERB, WILL_NOUN)
        self.assertEqual(
            [[sense["glosses"][0] for sense in entry["senses"]] for entry in written],
            [
                ["Ir.", "Irse, marcharse, partir.", "Andar, marchar, caminar."],
                [
                    "Voluntad, albedrío.",
                    "Decisión, intención.",
                    "Testamento.",
                    "Deseo, inclinación, disposición.",
                    "En particular, deseo sexual.",
                    "Deleite, placer, goce.",
                ],
            ],
        )
        self.assertEqual(written[0]["senses"][2]["tags"], ["obsolete", "outdated"], "its labels kept")
        glosses, _, _ = self.tables(["go"], GO_VERB)
        self.assertEqual(
            glosses["go"], "Ir; Irse, marcharse, partir; Andar, marchar, caminar", "kept when the eight hold it"
        )
        glosses, runs, _ = self.tables(
            ["go", "will"], GO_VERB, GO_VERB_TR, GO_NOUN, GO_INTJ, GO_GAME, WILL_NOUN, WILL_VERB, WILL_MODAL, WILL_NAME
        )
        self.assertEqual(
            glosses["go"],
            "Ir; Marchar; Irse, marcharse, partir; Repartir; Turno; Go (juego); Intento; "
            "Se emplea para iniciar un juego o competencia. ¡Ya!, ¡ahora!, ¡fuera!, ¡vamos!",
        )
        self.assertEqual(
            (glosses["will"], runs["will"]),
            (
                "Voluntad, albedrío; Decisión, intención; Testamento; Deseo, inclinación, disposición; Testar; "
                "Úsase para construir el futuro; Legar; Querer, desear",
                [("NOUN", 4), ("VERB", 4)],
            ),
        )

    def test_an_entry_whose_every_sense_is_labelled_keeps_its_order(self):
        # Recorded: thou and thy, each a single labelled sense; made up: two, both obsolete. Nothing
        # moves, and the lines are written as they were.
        both = {
            "word": "made",
            "pos": "noun",
            "senses": [{"glosses": ["B."], "tags": ["obsolete"]}, {"glosses": ["A."], "raw_tags": ["Arcaico"]}],
        }
        src = self.jsonl(THOU, THY, both)
        out = spanish.read_as_meanings(src, str(self.dir / "meanings.jsonl"))
        self.assertEqual(Path(out).read_text(encoding="utf-8"), Path(src).read_text(encoding="utf-8"))

    def test_a_rare_sense_keeps_its_place(self):
        # Recorded: close's second noun entry opens on a sense tagged rare: not moved.
        self.assertEqual(self.read(CLOSE_FIELD), [CLOSE_FIELD])

    # — D6: one Spanish typography —

    def test_spec_scenario_one_ellipsis(self):
        # Recorded: on's preposition, neither's conjunction and adverb, who'd. The shared cleaning
        # strips a sense's final periods, but « … » is no period: on's first sense keeps it.
        written, _ = self.gloss(ES, ENGLISH, {"on"}, ON_PREP)
        self.assertTrue(written["on"].startswith("Posicionado sobre la superficie superior de; "))
        glosses, _, _ = self.tables(["on", "neither", "who'd"], ON_PREP, NEITHER_CONJ, NEITHER_ADV, WHOD)
        self.assertEqual(
            glosses,
            {
                "on": "Posicionado sobre la superficie superior de …; Que ocurre en la fecha especificada; "
                "Relacionado con el tema …, sobre el tema …; Indica contacto con …; "
                "Utilizado para indicar el estar en el estado o en el proceso que se especifica",
                "neither": "(neither … nor) Ni; Tampoco",
                "who'd": "Contracción de el pronombre who y el verbo had, ¿Quién tenía / tuvo / había / hubo…?; "
                "Contracción de el pronombre who y el verbo would, ¿Quién (+ condicional)…?",
            },
        )
        self.assertEqual(spanish.typography("Invitarlo a salir de cita a ...con"), "Invitarlo a salir de cita a … con")
        self.assertEqual(spanish.typography("¿sabías que...?"), "¿sabías que…?")

    def test_spec_scenario_angular_quotes(self):
        # Recorded: it's, I'm, Byron — its « " Las peregrinaciones… » closed up —, and that's,
        # whose single quotes stay. Made up: an odd number of straight quotes stays as written.
        glosses, _, _ = self.tables(["it's", "i'm", "byron", "that's"], ITS_CONTRACTION, IM, BYRON, THATS)
        self.assertEqual(
            glosses,
            {
                "it's": "Contracción de el pronombre it («ello») y el verbo is («es»)",
                "i'm": "Contracción de I y am, «yo soy» o «yo estoy»",
                "byron": "Nombre de pila de varón; Apellido; "
                "Poeta inglés autor de «Don Juan», «Las peregrinaciones de Childe Harold», etc",
                "that's": "Contracción de 'that is', ese es, esa es, aquel es, aquella es",
            },
        )
        self.assertEqual(spanish.typography('un disco de 12" o "vinilo"'), 'un disco de 12" o "vinilo"')

    def test_spec_scenario_the_ing_form_named_as_the_card_names_it(self):
        # Recorded: be's two verb entries. The card names the form « forma en -ing » (M10).
        glosses, _, _ = self.tables(["be"], BE_VERB, BE_AUXILIARY)
        self.assertEqual(
            glosses["be"],
            "Ser; Ser (be + participio pasado); Estar; Estar (be + forma en -ing); Encontrarse; "
            "Se usa en be to (no existe en español)",
        )

    # — D3: a name does not gloss the common word spelled like it —

    def test_spec_scenario_a_name_on_a_common_word_s_card(self):
        # Recorded: will (above), smith and `Smith`, mike and `Mike`: the name's notes are left out
        # of the common word's card, and an entry left with no sense goes.
        glosses, runs, _ = self.tables(
            ["will", "smith", "mike"], WILL_NOUN, WILL_VERB, WILL_MODAL, WILL_NAME, SMITH, SMITH_NAME, MIKE, MIKE_NAME
        )
        self.assertNotIn("Apellido", glosses["will"])
        self.assertNotIn("Hipocorístico", glosses["will"])
        self.assertEqual((glosses["smith"], runs["smith"]), ("Herrero", [("NOUN", 1)]))
        self.assertEqual((glosses["mike"], runs["mike"]), ("Micro, micrófono", [("NOUN", 1)]))
        # `Will` stays where no `will` is written in lower case.
        self.assertEqual([entry["word"] for entry in self.english(SMITH_NAME, SMITH, WILL_NAME)], ["smith", "Will"])

    def test_spec_scenario_a_name_s_own_row(self):
        # Recorded: `Wayne` and `Donald`, with no entry in lower case: their notes say what they are.
        glosses, _, _ = self.tables(["wayne", "donald"], WAYNE_NAME, DONALD_NAME)
        self.assertEqual(
            glosses, {"wayne": "Apellido", "donald": "Nombre de pila de varón; Apellido; Nombre de varios lugares"}
        )
        # Made up: an entry in lower case that only points at another word holds no meaning.
        pointer = {"word": "wayne", "pos": "noun", "senses": [{"glosses": ["Forma del plural de way."]}]}
        self.assertEqual(self.english(WAYNE_NAME, pointer), [WAYNE_NAME, pointer])

    def test_a_proper_noun_s_other_senses_stay(self):
        # Recorded: south and `South`, don and its two `Don` entries: the region and the river stay,
        # the names' notes go — the first `Don` entry with all of its senses.
        glosses, runs, _ = self.tables(
            ["south", "don"], SOUTH_NOUN, SOUTH_ADJ, SOUTH_ADV, SOUTH_NAME, DON_NOUN, DON_VERB, DON_NAMES, DON_RIVER
        )
        self.assertEqual(
            (glosses["south"], runs["south"]),
            ("Sur; Del sur; Al sur; (region) Sur", [("NOUN", 1), ("ADJ", 1), ("ADV", 1), ("PROPN", 1)]),
        )
        self.assertEqual(
            glosses["don"],
            "Profesor de una universidad, en particular Oxford y Cambridge; Vestirse, ponerse; El río Don",
        )

    # — D4: possessives and demonstratives are determiners —

    def test_spec_scenario_possessives_and_demonstratives(self):
        # Recorded: her, my, its, their, that, such. Only the runs move, no sense.
        glosses, runs, _ = self.tables(
            ["her", "my", "its", "their", "that", "such"],
            HER_PRON,
            HER_ADJ,
            MY_ADJ,
            ITS_ADJ,
            ITS_PRON,
            THEIR_ADJ,
            THAT_PRON,
            THAT_ADJ,
            THAT_CONJ,
            SUCH_ADJ,
            SUCH_PRON,
        )
        self.assertEqual(
            runs,
            {
                "her": [("PRON", 2), ("DET", 1)],
                "my": [("DET", 1)],
                "its": [("DET", 1), ("PRON", 1)],
                "their": [("DET", 1)],
                "that": [("PRON", 2), ("DET", 2), ("SCONJ", 3)],
                "such": [("DET", 1), ("PRON", 1)],
            },
        )
        self.assertEqual(glosses["her"], "Ella; La (pronombre personal de objeto directo); Su (de ella)")
        self.assertEqual(
            glosses["that"],
            "Ese, esa, esos, esas; Aquel, aquella, aquellos, aquellas; Ese; Aquel; Que; Y pensar que, ojalá; Para que",
        )

    def test_the_quantifiers_stay_adjectives(self):
        # Recorded: any's and much's adjectives, which the edition tags indeterminate.
        _, runs, _ = self.tables(["any", "much"], ANY_ADJ, MUCH_ADJ)
        self.assertEqual(runs, {"any": [("ADJ", 2)], "much": [("ADJ", 1)]})

    def test_the_pre_passes_write_what_they_cannot_read_as_they_are(self):
        # Made up: lines that are no JSON object, an entry they do not change, senses of no kaikki
        # shape. Each is written as it is, byte for byte, by both passes.
        src = self.dir / "entries.jsonl"
        text = (
            "not json\n[1, 2]\n"
            '{"word":"house","pos":"noun","senses":[{"glosses":["Casa."]}]}\n'
            '{"word": "raro", "pos": "adj", "tags": "possessive", "senses": ["odd", {"glosses": [1]}, {"tags": ["obsolete"]}]}\n'
            '{"word": 3, "senses": 4}\n'
            '{"word": "Raro", "pos": "name", "senses": "Apellido."}\n'
        )
        src.write_text(text, encoding="utf-8")
        out = spanish.read_as_meanings(str(src), str(self.dir / "meanings.jsonl"))
        self.assertEqual(Path(out).read_text(encoding="utf-8"), text)
        out = en_es.english_entries(str(src), str(self.dir / "glossing.jsonl"))
        self.assertEqual(Path(out).read_text(encoding="utf-8"), text)

    # — D7: the translation tables' words, without their translators' notes —

    def test_spec_scenario_a_disused_word(self):
        # Recorded: orchestra, blacksmith, malign — whose one translation is disused: no gloss from
        # the table, which is better than a word it says is no longer used.
        direct = self.direct(ORCHESTRA_TRANSLATED, BLACKSMITH_TRANSLATED, MALIGN_TRANSLATED)
        self.assertEqual(
            direct,
            {
                "orchestra": {"NOUN": ["orquesta"]},
                "blacksmith": {"NOUN": ["herrero", "herrera", "herrador", "herradora"]},
            },
        )
        glossed = common.fallback_glosses({"orchestra", "blacksmith", "malign"}, {}, [(direct, list)], edition=ES)
        self.assertEqual(
            {lemma: gloss for lemma, (gloss, _) in glossed.items()},
            {"orchestra": "Orquesta", "blacksmith": "Herrero, herrera, herrador"},
        )
        # Recorded: « deep end » and « with both hands », glossed by a disused word alone.
        _, _, expressions = self.tables(
            [], sources=[(self.direct(DEEP_END_TRANSLATED, WITH_BOTH_HANDS_TRANSLATED, MY_NAME_IS_TRANSLATED), list)]
        )
        self.assertEqual(list(expressions), ["my name is"])

    def test_spec_scenario_a_translator_s_note(self):
        # Recorded. A loanword's respelling goes whatever its frequency (« hol » is 2.58 Zipf in
        # Spanish, « ap » 3.95): the text before it is the English headword. A label holding no
        # Spanish word goes (« despective »); so do a sense number and an English usage note.
        direct = self.direct(
            HALL_TRANSLATED,
            APP_TRANSLATED,
            INSIDER_TRANSLATED,
            BACKWATER_TRANSLATED,
            BACKWATER_VERB_TRANSLATED,
            ONSIDE_TRANSLATED,
            TO_BE_HONEST_TRANSLATED,
        )
        self.assertEqual(
            direct,
            {
                "hall": {"NOUN": ["pasillo", "hall", "jol", "antesala", "zaguán"]},
                "app": {"NOUN": ["apli", "app"]},
                "insider": {"NOUN": ["insider", "adentrino", "adentreño", "dentreño"]},
                "backwater": {"NOUN": ["pueblucho", "villorrio", "poblacho"], "VERB": ["ciar"]},
                "onside": {"ADJ": ["a favor"]},
                "to be honest": {"X": ["para ser honesto", "si me apuras"]},
            },
        )
        glossed = common.fallback_glosses({"hall", "backwater", "onside"}, {}, [(direct, list)], edition=ES)
        self.assertEqual(
            {lemma: gloss for lemma, (gloss, _) in glossed.items()},
            {"hall": "Pasillo, hall, jol", "backwater": "Pueblucho, villorrio, poblacho; Ciar", "onside": "A favor"},
        )
        # Recorded: septum's « septum (séptum) », after « septum »: the note's removal makes it a
        # word already listed, listed once.
        self.assertEqual(self.direct(SEPTUM_TRANSLATED), {"septum": {"NOUN": ["tabique", "septo", "septum"]}})

    def test_spec_scenario_a_spanish_note_stays(self):
        # Recorded: quit's « dimitir (de) », « renunciar (a) », daycare's « guardería (infantil) ».
        direct = self.direct(QUIT_TRANSLATED, DAYCARE_TRANSLATED)
        self.assertEqual(
            direct,
            {
                "quit": {"VERB": ["dimitir (de)", "renunciar (a)", "U.S.: cuitear", "quitear"]},
                "daycare": {"NOUN": ["guardería (infantil)", "wawawasi", "nido"]},
            },
        )

    def test_spec_scenario_a_spanish_word_listed_once(self):
        # Recorded: the Spanish entries listing grey, lengthy and seaman, and English's readings of
        # them. Each Spanish word is listed once, under the first of its parts of speech the English
        # word's readings name: lengthy is an adjective, seaman and mariner nouns.
        studied = self.dir / "en"
        studied.mkdir()
        (studied / "grammar.tsv").write_text(GREY_LENGTHY_SEAMAN_GRAMMAR, encoding="utf-8")
        readings = en_es.read_readings(str(studied))
        self.assertEqual(readings, {"grey": {"VERB", "ADJ", "NOUN"}, "lengthy": {"ADJ"}, "seaman": {"NOUN"}})
        src = self.jsonl(*SPANISH_LISTING_GREY_LENGTHY_SEAMAN, name="kaikki-es-traductions-en.jsonl")
        inverted = en_es.read_translated(
            src, str(self.dir / "inverted.jsonl"), inverted=True, frequency=spanish_zipf, readings=readings
        )
        self.assertEqual(inverted["lengthy"], {"ADJ": ["largo"]})
        self.assertEqual(inverted["grey"], {"ADJ": ["gris", "plomo"], "VERB": ["agrisar"]})
        self.assertEqual(inverted["seaman"], {"NOUN": ["marinero"]})
        self.assertEqual(inverted["mariner"], {"ADJ": ["marinero"]}, "no reading of mariner: the first listed")
        glossed = common.fallback_glosses(
            {"lengthy", "grey", "seaman"}, {}, [(inverted, en_es.by_spanish_frequency(lambda w: 0.0))], edition=ES
        )
        self.assertEqual(
            glossed,
            {
                "lengthy": ("Largo", [("ADJ", 1)]),
                "grey": ("Gris, plomo; Agrisar", [("ADJ", 1), ("VERB", 1)]),
                "seaman": ("Marinero", [("NOUN", 1)]),
            },
        )
        (studied / "grammar.tsv").unlink()
        with self.assertRaisesRegex(SystemExit, "grammar.tsv is missing"):
            en_es.read_readings(str(studied))

    def test_spec_scenario_the_english_word_s_own_parts_of_speech(self):
        # Recorded: the English Wiktionary translates Israeli as a noun and as an adjective.
        direct = self.direct(ISRAELI_NOUN_TRANSLATED, ISRAELI_ADJ_TRANSLATED)
        glossed = common.fallback_glosses({"israeli"}, {}, [(direct, list)], edition=ES)
        self.assertEqual(glossed, {"israeli": ("Israelí; Israelí", [("NOUN", 1), ("ADJ", 1)])})

    # — D1: where the rules live —

    def test_reduce_en_es_runs_the_passes_in_order(self):
        # en-es reduced as its build runs it (`main()`): its letters left out, then the edition's
        # pre-pass, then en-es's, then the shared rules over the last one's file.
        work = self.dir / "work"
        work.mkdir()
        studied = self.dir / "en"
        studied.mkdir()
        (studied / "forms.tsv").write_text("go\tgo\nhall\thall\nlengthy\tlengthy\nwill\twill\n", encoding="utf-8")
        (studied / "freq.tsv").write_text("go\t1\nwill\t2\nhall\t3\nlengthy\t4\n", encoding="utf-8")
        (studied / "level.tsv").write_text("", encoding="utf-8")
        (studied / "grammar.tsv").write_text(GREY_LENGTHY_SEAMAN_GRAMMAR, encoding="utf-8")
        self.jsonl(GO_VERB, WILL_NOUN, WILL_NAME, name="work/kaikki-es-English.jsonl")
        self.jsonl(HALL_TRANSLATED, name="work/kaikki-en-traductions-es.jsonl")
        self.jsonl(*SPANISH_LISTING_GREY_LENGTHY_SEAMAN, name="work/kaikki-es-traductions-en.jsonl")
        calls = []

        def spy(name, real):
            def call(*args, **kwargs):
                calls.append((name, *(Path(arg).name for arg in args if isinstance(arg, str))))
                return real(*args, **kwargs)

            return call

        wordfreq = types.ModuleType("wordfreq")
        wordfreq.zipf_frequency = lambda word, lang: spanish_zipf(word) if lang == "es" else 0.0
        argv = [
            "reduce-en-es.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-08", "--pack-version", "test"),
        ]
        with (
            mock.patch.dict(sys.modules, {"wordfreq": wordfreq}),
            mock.patch.object(sys, "argv", argv),
            mock.patch.object(en_es, "without_letters", spy("without_letters", en_es.without_letters)),
            mock.patch.object(spanish, "read_as_meanings", spy("read_as_meanings", spanish.read_as_meanings)),
            mock.patch.object(en_es, "english_entries", spy("english_entries", en_es.english_entries)),
            mock.patch.object(en_es, "native_side", spy("native_side", en_es.native_side)),
            contextlib.redirect_stderr(io.StringIO()),
        ):
            en_es.main()
        self.assertEqual(
            calls,
            [
                ("without_letters", "kaikki-es-English.jsonl", "kaikki-es-English-words.jsonl"),
                ("read_as_meanings", "kaikki-es-English-words.jsonl", "kaikki-es-English-meanings.jsonl"),
                ("english_entries", "kaikki-es-English-meanings.jsonl", "kaikki-es-English-glossing.jsonl"),
                ("native_side", "kaikki-es-English-glossing.jsonl"),
            ],
        )
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "go\tIr; Irse, marcharse, partir; Andar, marchar, caminar\n"
            "hall\tPasillo, hall, jol\n"
            "lengthy\tLargo\n"
            "will\tVoluntad, albedrío; Decisión, intención; Testamento; Deseo, inclinación, disposición; "
            "En particular, deseo sexual; Deleite, placer, goce\n",
        )

    def test_spec_scenario_nothing_else_moves(self):
        # The rules are the Spanish edition's and en-es's reducer's: en-es's rule digest moves with
        # them and no other pair's — en-fr's and es-fr's load the French edition, es-en's the English
        # one — and reduce_common.py, which every pair loads, is not edited. The committed pins
        # record the rules they were reduced with: en-es's, re-pinned with these rules, and the other
        # three, as they were.
        pairs = ("en-fr", "es-fr", "es-en", "en-es")
        for pair in pairs:
            self.assertEqual(
                ps.rules_sha256(Path(_HERE) / f"reduce-{pair}.py"),
                ps.get(ps.load(Path(_HERE) / "tables" / pair / "pin.json"), "reducer.sha256"),
                pair,
            )
        copy = self.dir / "rules"
        copy.mkdir()
        for path in Path(_HERE).glob("reduce[-_]*.py"):
            (copy / path.name).write_bytes(path.read_bytes())
        before = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        others = ("en-fr", "es-fr", "es-en")
        for name, old, new in (
            ("reduce_edition_es.py", '"A no confundir", ', ""),
            ("reduce_edition_es.py", '_NO_LONGER_USED = frozenset({"obsolete", ', "_NO_LONGER_USED = frozenset({"),
            ("reduce_edition_es.py", "if quotes and quotes % 2 == 0:", "if False:"),
            (
                "reduce-en-es.py",
                '_DETERMINER_TAGS = frozenset({"possessive", "demonstrative"})',
                "_DETERMINER_TAGS = frozenset()",
            ),
            ("reduce-en-es.py", "if _DISUSED.search(native):", "if False:"),
            ("reduce-en-es.py", "entries = english_entries(", "entries = (lambda src, dst: src)("),
        ):
            rules = copy / name
            text = rules.read_text(encoding="utf-8")
            self.assertIn(old, text)
            rules.write_text(text.replace(old, new), encoding="utf-8")
            after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
            self.assertNotEqual(after["en-es"], before["en-es"], new)
            self.assertEqual({p: after[p] for p in others}, {p: before[p] for p in others}, new)
            before = after


if __name__ == "__main__":
    unittest.main()
