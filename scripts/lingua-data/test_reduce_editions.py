# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the Wiktionary editions' rules (generalise-lingua-gloss-reducer).

The senses are recorded from real kaikki data, cut down to the fields the rules read: the English
Wiktionary's Spanish section (the 2026-09-28 extract es-fr pins) and the Spanish Wiktionary's English
entries (its 2026-10-02 dump, which es-fr pins). A case marked « made up » is not from the data.

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

    def test_nothing_written_no_gloss(self):
        glosses, runs, expressions, primary = common.native_tables(
            self.jsonl(CONGERIES), {"congeries"}, studied=SPANISH, edition=EN, fallbacks=[({}, list)]
        )
        self.assertEqual((glosses, runs, expressions, primary), ({}, {}, {}, 0))


if __name__ == "__main__":
    unittest.main()
