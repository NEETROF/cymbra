# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the en-it reducer (add-lingua-pack-en-it): English glossed in Italian.

The entries are recorded from the files `pack_sources.derive` writes from the dumps served on 2026-10-26
(en-it's local update; design D6): the Italian Wiktionary's English section (`kaikki-it-Inglese.jsonl`,
sha256 79033de9…) and its Italian entries' English translations (`kaikki-it-traductions-en.jsonl`,
e192643a…), the English Wiktionary's Italian translations (`kaikki-en-traductions-it.jsonl`, bb2f8c5a…),
cut down to the fields the rules read. A case marked « made up » is not from the data.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import ast
import contextlib
import importlib.util
import io
import json
import os
import shutil
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
from reduce_edition_it import IT  # noqa: E402


def _reducer(pair):
    spec = importlib.util.spec_from_file_location(f"reduce_{pair.replace('-', '_')}_en_it", os.path.join(_HERE, f"reduce-{pair}.py"))
    module = importlib.util.module_from_spec(spec)
    spec.loader.exec_module(module)
    return module


en_it = _reducer("en-it")
en_es = _reducer("en-es")
en_fr = _reducer("en-fr")

# — The Italian Wiktionary's English section (kaikki-it-Inglese.jsonl), recorded —
SUMMER_IT = {"word": "summer", "pos": "noun", "lang_code": "en", "senses": [{"glosses": ["estate"], "topics": ["astronomy"]}]}
HOUSE_IT = {"word": "house", "pos": "noun", "lang_code": "en", "senses": [{"glosses": ["casa"], "topics": ["architecture"]}]}
HOUSE_VERB_IT = {"word": "house", "pos": "verb", "lang_code": "en", "senses": [{"glosses": ["alloggiare"]}]}
GIVE_UP_IT = {"word": "give up", "pos": "verb", "lang_code": "en", "tags": ["intransitive"], "senses": [{"glosses": ["arrendersi"]}]}

# — The English Wiktionary's Italian translations (kaikki-en-traductions-it.jsonl), recorded —
SUMMER_TRANSLATED = {"word": "summer", "pos": "noun", "translations": [{"word": "estate", "sense": "hottest season"}]}
INTERIOR_TRANSLATED = {
    "word": "interior",
    "pos": "adj",
    "translations": [
        {"word": "interiore", "sense": "having to do with the inner part of something"},
        {"word": "interno", "sense": "having to do with the inner part of something"},
    ],
}
ENRICHMENT_TRANSLATED = {"word": "enrichment", "pos": "noun", "translations": [{"word": "arricchimento (1,2)", "sense": "the act of enriching"}]}
NATIONAL_ADJ_TRANSLATED = {"word": "national", "pos": "adj", "translations": [{"word": "nazionale", "sense": "pertaining to a nation or country; the state"}]}
NATIONAL_NOUN_TRANSLATED = {"word": "national", "pos": "noun", "translations": [{"word": "nazionale", "sense": "born in a given country"}]}
LOOK_AFTER_TRANSLATED = {
    "word": "look after",
    "pos": "verb",
    "translations": [
        {"word": "prendersi cura", "sense": "to care for, keep safe, watch, protect"},
        {"word": "stare dietro", "sense": "to care for, keep safe, watch, protect"},
        {"word": "curare", "sense": "to care for, keep safe, watch, protect"},
    ],
}
# The letters: `A` the letter, `C` the note's name borrowed through it — both nouns, neither glosses —;
# `I` the pronoun « io », kept.
A_NOUN_TRANSLATED = {"word": "A", "pos": "noun", "translations": [{"word": "A"}]}
C_NOUN_TRANSLATED = {"word": "C", "pos": "noun", "translations": [{"word": "do"}]}
I_PRON_TRANSLATED = {"word": "I", "pos": "pron", "translations": [{"word": "io"}]}

# — The Italian Wiktionary's English translations (kaikki-it-traductions-en.jsonl), recorded —
SINISTRO_ADJ = {"word": "sinistro", "pos": "adj", "translations": [{"word": "left-hand"}]}
SINISTRO_NOUN = {"word": "sinistro", "pos": "noun", "translations": [{"word": "left-hand"}]}
SINISTRO_VERB = {"word": "sinistro", "pos": "verb", "translations": [{"word": "left-hand"}]}
DISINFORMAZIONE = {"word": "disinformazione", "pos": "noun", "translations": [{"word": "misinformation"}]}
CASA_LISTS_HOUSE = {"word": "casa", "pos": "noun", "translations": [{"word": "house"}, {"word": "home"}]}
# Made up: a second Italian word listing `misinformation`, rarer than « disinformazione ».
MALINFORMAZIONE = {"word": "malinformazione", "pos": "noun", "translations": [{"word": "misinformation"}]}
# Recorded: the Italian Wiktionary's `e` and `o` list « and » and « or »; a one-letter Italian entry.
E_CHARACTER = {"word": "e", "pos": "character", "translations": [{"word": "and"}]}


class EnglishGlossedInItalian(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def jsonl(self, *entries, name):
        path = self.dir / name
        path.parent.mkdir(parents=True, exist_ok=True)
        path.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        return str(path)

    def studied(self, words, grammar=""):
        folder = self.dir / "en"
        folder.mkdir(exist_ok=True)
        (folder / "forms.tsv").write_text("".join(f"{w}\t{w}\n" for w in words), encoding="utf-8")
        (folder / "freq.tsv").write_text("".join(f"{w}\t{r}\n" for r, w in enumerate(words, 1)), encoding="utf-8")
        (folder / "level.tsv").write_text("", encoding="utf-8")
        (folder / "grammar.tsv").write_text(grammar, encoding="utf-8")
        return folder

    @staticmethod
    def wordfreq(zipf):
        module = types.ModuleType("wordfreq")
        module.zipf_frequency = lambda word, lang: zipf.get(word, 0.0) if lang == "it" else 0.0
        return mock.patch.dict(sys.modules, {"wordfreq": module})

    def reduce(self, work, studied, **zipf):
        argv = [
            "reduce-en-it.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-26", "--pack-version", "test"),
        ]
        with self.wordfreq(zipf), mock.patch.object(sys, "argv", argv), contextlib.redirect_stderr(io.StringIO()) as err:
            en_it.main()
        return err.getvalue()

    def test_spec_scenarios_a_definition_first_then_the_direct_then_the_inverted_table(self):
        # en-it reduced as its build runs it (`main()`), from a studied folder as en-fr writes it.
        # `summer` and `house` have entries in the Italian Wiktionary's English section: its definitions
        # win over the translations the English Wiktionary lists (*A definition first*). `interior` has
        # none, and the English Wiktionary's Italian table translates it (*A word only a table glosses*):
        # « interiore, interno », the words' case kept (`IT.capitalised` is False). `misinformation` has
        # neither: the Italian entries listing it, read backwards, the commonest Italian word first.
        # `enrichment`'s translator's note goes; `a` and `c`, letters, take nothing from either table;
        # `i` takes « io »; nothing glosses `stone`.
        work = self.dir / "work"
        work.mkdir()
        studied = self.studied(["house", "summer", "interior", "misinformation", "enrichment", "i", "a", "c", "stone"])
        self.jsonl(SUMMER_IT, HOUSE_IT, HOUSE_VERB_IT, GIVE_UP_IT, name="work/kaikki-it-Inglese.jsonl")
        self.jsonl(
            SUMMER_TRANSLATED,
            INTERIOR_TRANSLATED,
            ENRICHMENT_TRANSLATED,
            LOOK_AFTER_TRANSLATED,
            A_NOUN_TRANSLATED,
            C_NOUN_TRANSLATED,
            I_PRON_TRANSLATED,
            name="work/kaikki-en-traductions-it.jsonl",
        )
        self.jsonl(MALINFORMAZIONE, DISINFORMAZIONE, CASA_LISTS_HOUSE, E_CHARACTER, name="work/kaikki-it-traductions-en.jsonl")
        err = self.reduce(work, studied, disinformazione=2.5, malinformazione=1.0, arricchimento=3.0)
        self.assertEqual(
            (work / "gloss.tsv").read_text(encoding="utf-8"),
            "enrichment\tarricchimento\n"
            "house\tcasa; alloggiare\n"
            "i\tio\n"
            "interior\tinteriore, interno\n"
            "misinformation\tdisinformazione, malinformazione\n"
            "summer\testate\n",
        )
        self.assertEqual(
            (work / "senses.tsv").read_text(encoding="utf-8"),
            "enrichment\tNOUN:1\nhouse\tNOUN:1\tVERB:1\ni\tPRON:1\ninterior\tADJ:1\nmisinformation\tNOUN:1\nsummer\tNOUN:1\n",
        )
        # The expressions keyed as en-es keys them (D5): the section's, then a table's.
        self.assertEqual((work / "mwe.tsv").read_text(encoding="utf-8"), "give up\tarrendersi\nlook after\tprendersi cura, stare dietro, curare\n")
        written = sorted(p.name for p in work.iterdir() if not p.name.startswith("kaikki-"))
        self.assertEqual(written, ["NOTICE", "gloss.tsv", "manifest.json", "measures.json", "mwe.tsv", "senses.tsv"])
        # The share the tables gave among the glossed lemmas of the commonest (D9): four of six.
        measures = json.loads((work / "measures.json").read_text(encoding="utf-8"))
        self.assertEqual(
            measures,
            {"top": 10000, "glossed": 6, "share": 66.7, "direct": ["enrichment", "i", "interior"], "inverted": ["misinformation"]},
        )
        self.assertIn("reduced en-it: lemmas=9 (English's committed tables) glosses=6 (Italian Wiktionary 2", err)
        manifest = json.loads((work / "manifest.json").read_text(encoding="utf-8"))
        self.assertEqual((manifest["meta"]["studied"], manifest["meta"]["native"]), ("en", "it"))
        self.assertNotIn("levels_estimated", manifest["meta"], "English's levels are CEFR-J's and Octanove's")
        self.assertEqual(manifest["meta"]["analyzer_version"], en_fr.analyser_version())
        self.assertIn("kaikki / itwiktionary, enwiktionary, frwiktionary (CC BY-SA 4.0 + GFDL)", manifest["meta"]["licences"])

    def test_spec_scenario_the_credits(self):
        # Both sides: the Italian and English Wiktionaries, and every source of English's studied tables,
        # as en-es's notice credits them.
        for credit in ("itwiktionary", "enwiktionary", "frwiktionary", "ESDB", "WordNet", "CEFR-J", "Octanove", "wordfreq"):
            self.assertIn(credit, en_it.NOTICE)
        self.assertIn("EN->IT", en_it.NOTICE)
        self.assertNotIn("eswiktionary", en_it.NOTICE)
        studied_side = en_es.NOTICE[en_es.NOTICE.index("ESDB") : en_es.NOTICE.index("wordfreq (")]
        self.assertIn(studied_side, en_it.NOTICE, "the studied side credited as en-es credits it")

    def test_spec_scenario_listed_once(self):
        # The Italian Wiktionary's `sinistro`, read backwards, lists « left-hand » under its adjective,
        # noun and verb: listed once, under the first part of speech English's readings name.
        src = self.jsonl(SINISTRO_ADJ, SINISTRO_NOUN, SINISTRO_VERB, name="it-en.jsonl")
        inverted = en_it.read_translated(src, str(self.dir / "inverted.jsonl"), inverted=True, frequency=lambda w: 0.0, readings={})
        self.assertEqual(inverted, {"left-hand": {"ADJ": ["sinistro"]}})
        inverted = en_it.read_translated(
            src, str(self.dir / "inverted.jsonl"), inverted=True, frequency=lambda w: 0.0, readings={"left-hand": {"NOUN"}}
        )
        self.assertEqual(inverted, {"left-hand": {"NOUN": ["sinistro"]}})
        glossed = common.fallback_glosses({"left-hand"}, {}, [(inverted, list)], edition=IT)
        self.assertEqual(glossed, {"left-hand": ("sinistro", [("NOUN", 1)])})

    def test_spec_scenario_a_translator_s_note(self):
        # A sense number goes (« arricchimento (1,2) »); a note holding an Italian word stays, one holding
        # none (made up: every word under 1.0 Zipf) goes; a word labelled disused is not listed.
        frequency = {"infantile": 3.5}.get
        self.assertEqual(en_it.translators_word("arricchimento (1,2)", "enrichment", lambda w: 0.0), "arricchimento")
        self.assertEqual(en_it.translators_word("asilo (infantile)", "kindergarten", lambda w: frequency(w, 0.0)), "asilo (infantile)")
        self.assertEqual(en_it.translators_word("orchestra (despective)", "orchestra", lambda w: 0.0), "orchestra")
        table = {"enrichment": {"NOUN": ["arricchimento (1,2)"]}, "malign": {"ADJ": ["maligno (disused)"]}}
        self.assertEqual(en_it.translators_words(table, lambda w: 0.0), {"enrichment": {"NOUN": ["arricchimento"]}})
        # The direct table's repeats across an English word's parts of speech are kept (D3).
        src = self.jsonl(NATIONAL_ADJ_TRANSLATED, NATIONAL_NOUN_TRANSLATED, name="en-it.jsonl")
        direct = en_it.read_translated(src, str(self.dir / "direct.jsonl"), inverted=False, frequency=lambda w: 0.0, readings={})
        self.assertEqual(direct, {"national": {"ADJ": ["nazionale"], "NOUN": ["nazionale"]}})

    def test_the_translators_typography_is_the_italian_edition_s(self):
        # Three dots or more written « … », a gender mark after a word taken out (D3, change 35's D9).
        table = {"so": {"ADV": ["così...", "talmente m"]}}
        self.assertEqual(en_it.translators_words(table, lambda w: 0.0), {"so": {"ADV": ["così…", "talmente"]}})

    def test_letters_out_of_both_tables(self):
        src = self.jsonl(A_NOUN_TRANSLATED, C_NOUN_TRANSLATED, I_PRON_TRANSLATED, name="en-it.jsonl")
        direct = en_it.read_translated(src, str(self.dir / "direct.jsonl"), inverted=False, frequency=lambda w: 0.0, readings={})
        self.assertEqual(direct, {"i": {"PRON": ["io"]}})
        # Read backwards, a one-letter English word is left out whatever lists it (made up: « io » « I »).
        io_lists_i = {"word": "io", "pos": "pron", "translations": [{"word": "I"}]}
        src = self.jsonl(io_lists_i, E_CHARACTER, DISINFORMAZIONE, name="it-en.jsonl")
        inverted = en_it.read_translated(src, str(self.dir / "inverted.jsonl"), inverted=True, frequency=lambda w: 0.0, readings={})
        self.assertEqual(inverted, {"misinformation": {"NOUN": ["disinformazione"]}})

    def test_the_readers_are_en_es_s_with_d3_s_two_substitutions(self):
        # en-es's readers, copied (D3): the same code but the Italian edition's typography and Italian's
        # Zipf threshold — a third copy, which must not drift from its source until en-es re-pins.
        def bodies(path):
            out = {}
            for node in ast.parse(Path(path).read_text(encoding="utf-8")).body:
                if isinstance(node, ast.FunctionDef):
                    if node.body and isinstance(node.body[0], ast.Expr) and isinstance(node.body[0].value, ast.Constant):
                        node.body = node.body[1:]
                    out[node.name] = ast.dump(node)
            return out

        es, it = bodies(os.path.join(_HERE, "reduce-en-es.py")), bodies(os.path.join(_HERE, "reduce-en-it.py"))
        copied = (
            "read_table", "read_studied", "without_letters", "without_letter_translations", "_names_a_letter",
            "read_translated", "translators_word", "translators_words", "listed_once", "read_readings",
            "native_side", "translation_share", "analyser_version",
        )  # fmt: skip
        for name in copied:
            expected = (
                es[name].replace("spanish", "italian").replace("SPANISH_WORD_ZIPF", "ITALIAN_WORD_ZIPF").replace("en-es reads", "en-it reads")
            )
            self.assertEqual(it[name], expected, name)
        # en-es's third pre-pass (names, determiners) is change 37's, not copied.
        self.assertNotIn("english_entries", it)
        self.assertEqual(en_it.ITALIAN_WORD_ZIPF, en_es.SPANISH_WORD_ZIPF)
        self.assertEqual(en_it._DISUSED.pattern, en_es._DISUSED.pattern)
        self.assertEqual(en_it._NOTE.pattern, en_es._NOTE.pattern)

    def test_en_it_s_copy_of_english_is_en_fr_s(self):
        self.assertEqual(en_it.EN, en_fr.EN)
        self.assertIs(en_it.EDITION, IT)
        self.assertFalse(en_it.EDITION.capitalised, "a table gloss keeps its words' case (D4)")
        self.assertEqual(en_it.LOCUTIONS, {})

    def test_en_it_reads_the_committed_studied_tables(self):
        committed = en_it.read_studied(os.path.join(_HERE, "tables", "en"), 40000)
        self.assertEqual(committed, en_es.read_studied(os.path.join(_HERE, "tables", "en"), 40000))
        studied = self.studied(["house"])
        (studied / "grammar.tsv").unlink()
        with self.assertRaisesRegex(SystemExit, "en-it reads English's committed tables"):
            en_it.read_readings(str(studied))

    def test_spec_scenario_a_rule_of_the_italian_edition(self):
        # en-it's rule digest is its file, reduce_common.py and reduce_edition_it.py (D1): editing the
        # Italian edition moves it, and no committed pair's.
        reducer = Path(_HERE) / "reduce-en-it.py"
        self.assertEqual([p.name for p in ps.rule_files(reducer)], ["reduce-en-it.py", "reduce_common.py", "reduce_edition_it.py"])
        copy = self.dir / "rules"
        copy.mkdir()
        for path in Path(_HERE).glob("reduce[-_]*.py"):
            shutil.copy(path, copy / path.name)
        committed = sorted(p.parent.name for p in (Path(_HERE) / "tables").glob("*/pin.json"))
        self.assertIn("en-it", committed)
        before = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in committed}
        module = copy / "reduce_edition_it.py"
        module.write_text(module.read_text(encoding="utf-8") + "\n# edited\n", encoding="utf-8")
        after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in committed}
        self.assertEqual([pair for pair in before if before[pair] != after[pair]], ["en-it"])


if __name__ == "__main__":
    unittest.main()
