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
import collections
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
import reduce_english_it as refine  # noqa: E402
from reduce_edition_it import IT  # noqa: E402
from reduce_edition_it import typography as IT_TYPOGRAPHY  # noqa: E402


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

    def reduce(self, work, studied, rules=None, **zipf):
        argv = [
            "reduce-en-it.py",
            *("--work", str(work), "--studied", str(studied)),
            *("--built-at", "2026-10-26", "--pack-version", "test"),
            *(("--rules", rules) if rules is not None else ()),
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
        self.assertEqual(
            [p.name for p in ps.rule_files(reducer)],
            ["reduce-en-it.py", "reduce_common.py", "reduce_edition_it.py", "reduce_english_it.py"],
        )
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


    def test_no_rule_firing_writes_the_rows_as_before(self):
        # Change 37's rules, every one on, over rows none of them reaches: the tables are written byte
        # for byte as with none (refine-lingua-en-it-glosses task 1.1); an unknown rule is refused.
        written = {}
        for rules in (None, ""):
            work = self.dir / f"work-{rules is None}"
            work.mkdir()
            studied = self.studied(["house", "summer", "interior", "misinformation", "enrichment", "i", "a", "c", "stone"])
            self.jsonl(SUMMER_IT, HOUSE_IT, HOUSE_VERB_IT, GIVE_UP_IT, name=f"{work.name}/kaikki-it-Inglese.jsonl")
            self.jsonl(SUMMER_TRANSLATED, INTERIOR_TRANSLATED, ENRICHMENT_TRANSLATED, LOOK_AFTER_TRANSLATED, name=f"{work.name}/kaikki-en-traductions-it.jsonl")
            self.jsonl(MALINFORMAZIONE, DISINFORMAZIONE, CASA_LISTS_HOUSE, name=f"{work.name}/kaikki-it-traductions-en.jsonl")
            err = self.reduce(work, studied, rules=rules, disinformazione=2.5, arricchimento=3.0)
            written[rules] = {name: (work / name).read_bytes() for name in ("gloss.tsv", "senses.tsv", "mwe.tsv", "NOTICE")}
            self.assertIn("reduced en-it: change 37's rules (" + ("none" if rules == "" else ", ".join(refine.RULES)), err)
        self.assertEqual(written[None], written[""])
        with self.assertRaisesRegex(SystemExit, "no such rule of reduce_english_it.py: grammar"):
            self.reduce(self.dir, self.dir, rules="names,grammar")

    def test_spec_scenario_a_rule_of_en_it_s_own(self):
        # Editing reduce_english_it.py moves en-it's rule digest and no other pair's (D1).
        copy = self.dir / "rules"
        copy.mkdir()
        for path in Path(_HERE).glob("reduce[-_]*.py"):
            shutil.copy(path, copy / path.name)
        pairs = sorted(p.stem.removeprefix("reduce-") for p in copy.glob("reduce-*.py"))
        loading = [pair for pair in pairs if "reduce_english_it.py" in ps.loaded_rules(copy / f"reduce-{pair}.py")]
        self.assertEqual(loading, ["en-it"])
        before = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        module = copy / "reduce_english_it.py"
        module.write_text(module.read_text(encoding="utf-8").replace("MARGIN = 0.5\n", "MARGIN = 0.6\n"), encoding="utf-8")
        after = {pair: ps.rules_sha256(copy / f"reduce-{pair}.py") for pair in pairs}
        self.assertEqual([pair for pair in pairs if before[pair] != after[pair]], ["en-it"])


# — Change 37 (refine-lingua-en-it-glosses): en-it's own rules, reduce_english_it.py —


def zipf_of(italian=None, english=None):
    """wordfreq, made up for the cases: Italian's and English's Zipf frequencies."""
    italian, english = italian or {}, english or {}
    return lambda word, language: (italian if language == "it" else english if language == "en" else {}).get(word, 0.0)


class RefinedRows(unittest.TestCase):
    """The rows (D2, D3, D4, D10a, D11, D12): recorded rows of en-it's committed tables (2026-10-26) and
    the English Wiktionary's first translated senses, cut down; made-up frequencies."""

    def rows(self, rows, *, first=None, direct=None, inverted=None, expressions=None, zipf=None, rules=refine.RULES, entries=None, ranks=None):
        glosses = {lemma: gloss for lemma, (gloss, _) in rows.items()}
        runs = {lemma: list(r) for lemma, (_, r) in rows.items()}
        steps = {"entries": frozenset(rows if entries is None else entries)}
        generated, unfired = refine.refine_rows(
            glosses, runs, steps, ranks=ranks or {lemma: n for n, lemma in enumerate(rows, 1)}, first=first or {},
            direct=direct or {}, inverted=inverted or {}, expressions=expressions or {}, zipf=zipf or zipf_of(), rules=rules,
        )  # fmt: skip
        return glosses, runs, generated, unfired

    def test_each_rule_is_named(self):
        self.assertEqual(len(refine.RULES), len(set(refine.RULES)))
        self.assertEqual(refine.RULES[-1], "repeats", "D10a last, so that a reviewed row's words are not repeated further on")
        self.assertLess(refine.RULES.index("contractions"), refine.RULES.index("part of speech"))
        self.assertLess(refine.RULES.index("described rows"), refine.RULES.index("reviewed rows"))

    def test_spec_scenario_the_part_of_speech_two_signals_agree_on(self):
        # `make`: the English Wiktionary lists its verb first (« fare »), and « fare » rates above « marca ».
        make = ("marca; fare, creare, fabbricare; rendere; raggiungere, arrivare a", [("NOUN", 1), ("VERB", 3)])
        glosses, runs, _, _ = self.rows(
            {"make": make},
            first={"make": ("VERB", ["fare"])},
            direct={"make": {"VERB": ["fare", "creare", "fabbricare"], "NOUN": ["marca"]}},
            zipf=zipf_of({"fare": 6.1, "creare": 5.0, "marca": 4.4}),
        )
        self.assertEqual(glosses["make"], "fare, creare, fabbricare; rendere; raggiungere, arrivare a; marca")
        self.assertEqual(runs["make"], [("VERB", 3), ("NOUN", 1)])
        # Under the margin, the row stays.
        glosses, *_ = self.rows(
            {"make": make}, first={"make": ("VERB", ["fare"])}, direct={"make": {"VERB": ["fare"], "NOUN": ["marca"]}},
            zipf=zipf_of({"fare": 4.8, "marca": 4.4}),
        )  # fmt: skip
        self.assertEqual(glosses["make"], make[0])

    def test_spec_scenario_a_verb_does_not_yield_to_a_noun(self):
        # `check` opens on its verb; the table's noun words rate higher, and the page lists the noun first.
        check = ("esaminare, assaggiare, verificare; assegno", [("VERB", 1), ("NOUN", 1)])
        glosses, *_ = self.rows(
            {"check": check},
            first={"check": ("NOUN", ["controllo", "verifica"])},
            direct={"check": {"NOUN": ["controllo", "verifica", "assegno"], "VERB": ["controllare"]}},
            zipf=zipf_of({"controllo": 5.0, "assegno": 4.0, "controllare": 4.3}),
        )
        self.assertEqual(glosses["check"], check[0])

    def test_a_row_of_one_part_of_speech_and_a_row_the_section_did_not_gloss_stay(self):
        rows = {
            "house": ("casa; abitazione", [("NOUN", 2)]),
            "interior": ("interiore, interno; dentro", [("ADJ", 1), ("ADV", 1)]),
        }
        glosses, *_ = self.rows(
            rows,
            entries={"house"},
            first={"house": ("VERB", ["alloggiare"]), "interior": ("ADV", ["dentro"])},
            direct={"house": {"VERB": ["alloggiare"]}, "interior": {"ADV": ["dentro"]}},
            zipf=zipf_of({"alloggiare": 1.0, "dentro": 6.0}),
            rules=("part of speech",),
        )
        self.assertEqual(glosses, {lemma: gloss for lemma, (gloss, _) in rows.items()})

    def test_that_former_and_none_are_reviewed_rows(self):
        # D2 reads them worse (the design's *Measured*): `FIRST_RUN` names the sense each opens on.
        for lemma in ("that", "former", "none"):
            self.assertIn(lemma, refine.FIRST_RUN)
        that = (
            "così; introduce una proposizione che è il soggetto o l'oggetto di un verbo; quello, quella; "
            "che, il quale, la quale, i quali, le quali; il quale, la quale, i quali, le quali, cui",
            [("ADV", 1), ("SCONJ", 1), ("PRON", 3)],
        )
        glosses, *_ = self.rows({"that": that}, first={"that": ("SCONJ", ["che"])}, direct={"that": {"SCONJ": ["che"]}})
        self.assertTrue(glosses["that"].startswith("quello, quella; che, il quale, la quale, i quali, le quali; "), glosses["that"])

    def test_spec_scenario_the_missing_main_sense(self):
        # `rescue` « salvataggio » alone; the English Wiktionary lists « salvare » under the verb, first.
        glosses, runs, _, _ = self.rows(
            {"rescue": ("salvataggio", [("NOUN", 1)])},
            first={"rescue": ("VERB", ["salvare"])},
            direct={"rescue": {"VERB": ["salvare"], "NOUN": ["salvataggio"]}},
            zipf=zipf_of({"salvare": 4.9, "salvataggio": 3.9}),
        )
        self.assertEqual((glosses["rescue"], runs["rescue"]), ("salvare; salvataggio", [("VERB", 1), ("NOUN", 1)]))

    def test_the_missing_main_sense_is_no_adjective_s_verb_nor_an_interjection_s(self):
        # `smart` « elegante; intelligente, acuto »: the table's verb « fare male » would open an
        # adjective's row; `goddamn` « dannato, maledetto »: its table's first sense is an interjection's.
        rows = {"smart": ("elegante; intelligente, acuto", [("ADJ", 2)]), "goddamn": ("dannato, maledetto", [("ADJ", 1)])}
        glosses, *_ = self.rows(
            rows,
            first={"smart": ("VERB", ["fare male"]), "goddamn": ("INTJ", ["porco Dio!", "porcodio!", "merda!"])},
            direct={"smart": {"VERB": ["fare male"]}, "goddamn": {"INTJ": ["porco Dio!", "porcodio!", "merda!"]}},
            zipf=zipf_of({"fare male": 5.0, "porco Dio!": 4.0, "merda!": 5.0}),
        )
        self.assertEqual(glosses, {lemma: gloss for lemma, (gloss, _) in rows.items()})

    def test_spec_scenario_a_row_of_descriptions(self):
        # `its` « possessivo di it » alone; the table's first sense is « suo ». `should`'s description,
        # « dovere ». `operational`, under the Zipf floor (made up: « operazionale » 1.2), stays.
        rows = {
            "its": ("possessivo di it", [("ADJ", 1)]),
            "should": ("usato per esprimere qualche obbligazione che il soggetto dovrebbe fare", [("VERB", 1)]),
            "operational": ("relativo al funzionamento di un sistema operativo", [("ADJ", 1)]),
        }
        glosses, *_ = self.rows(
            rows,
            first={"its": ("DET", ["suo"]), "should": ("VERB", ["dovere"]), "operational": ("ADJ", ["operazionale"])},
            direct={"its": {"DET": ["suo"]}, "should": {"VERB": ["dovere"]}, "operational": {"ADJ": ["operazionale"]}},
            zipf=zipf_of({"suo": 6.5, "dovere": 5.5, "operazionale": 1.2}),
        )
        self.assertEqual(glosses["its"], "suo; possessivo di it")
        self.assertEqual(glosses["should"], "dovere; usato per esprimere qualche obbligazione che il soggetto dovrebbe fare")
        self.assertEqual(glosses["operational"], rows["operational"][0])

    def test_no_letter_s_row_nor_an_english_first_sense(self):
        # `a`: one letter, the table's letters and codes; `would`: the table's first « sense » is an English
        # example (« We sat on the bench »), not Italian.
        rows = {
            "a": ("un, uno, una, un'", [("DET", 1)]),
            "would": ("usato per formare il modo condizionale", [("VERB", 1)]),
        }
        glosses, *_ = self.rows(
            rows,
            first={"a": ("NOUN", ["la"]), "would": ("VERB", ["We sat on the bench"])},
            direct={"a": {"NOUN": ["la"]}, "would": {"VERB": ["We sat on the bench"]}},
            zipf=zipf_of({"la": 7.0, "we sat on the bench": 3.5}, {"we": 7.0, "sat": 4.5, "on": 7.0, "the": 7.7, "bench": 4.3}),
        )
        self.assertEqual(glosses, {lemma: gloss for lemma, (gloss, _) in rows.items()})

    def test_spec_scenario_a_repeat(self):
        rows = {
            "national": ("nazionale; nazionale", [("ADJ", 1), ("NOUN", 1)]),
            "gradually": ("gradualmente, per gradi, un po' per volta, gradatamente, per gradi", [("ADV", 1)]),
            "one": ("uno / uno; unico", [("NUM", 1), ("ADJ", 1)]),
            "small": ("piccolo, basso; giovane, basso", [("ADJ", 2)]),
        }
        glosses, runs, _, _ = self.rows(rows, rules=("repeats",))
        self.assertEqual(glosses["national"], "nazionale")
        self.assertEqual(runs["national"], [("ADJ", 1)])
        self.assertEqual(glosses["gradually"], "gradualmente, per gradi, un po' per volta, gradatamente")
        self.assertEqual(glosses["one"], "uno; unico")
        self.assertEqual(glosses["small"], rows["small"][0], "an item is not taken across senses")


class Contractions(unittest.TestCase):
    """D11 (task 0.1, answered (a) by the owner, 2026-10-26): contractions read as pointers."""

    def test_spec_scenario_a_contraction_no_source_glosses(self):
        ranks = {w: n for n, w in enumerate(["you're", "he's", "i'd", "how'd", "would've", "let's", "can't", "i'll", "wasn't"], 1)}
        glosses, runs = {}, {}
        generated = refine.contractions(glosses, runs, ranks)
        self.assertEqual(
            glosses,
            {
                "you're": "contrazione di you are",
                "he's": "contrazione di he is; contrazione di he has",
                "i'd": "contrazione di I would; contrazione di I had",
                "how'd": "contrazione di how did; contrazione di how would; contrazione di how had",
                "would've": "contrazione di would have",
                "let's": "contrazione di let us",
                "can't": "contrazione di cannot",
                "i'll": "contrazione di I will",
                "wasn't": "contrazione di was not",
            },
        )
        self.assertEqual(runs["wasn't"], [("VERB", 1)])
        self.assertEqual(runs["he's"], [("X", 2)])
        self.assertEqual(generated, sorted(ranks), "every row listed as generated")

    def test_spec_scenario_not_a_contraction(self):
        for word in ("i's", "you's", "we's", "n't", "someone's", "everyone's", "women's", "world's", "o'clock", "ain't", "x'd"):
            self.assertIsNone(refine.full_forms(word), word)
        glosses = {}
        self.assertEqual(refine.contractions(glosses, {}, {"i's": 1, "women's": 2, "n't": 3}), [])
        self.assertEqual(glosses, {})

    def test_spec_scenario_a_contraction_read_through_another_s_entry(self):
        # `doesn't` reads `don't`'s « contrazione di do not »; a contraction the section glosses with its
        # own full form, or otherwise, stays.
        glosses = {
            "doesn't": "contrazione di do not",
            "isn't": "contrazione di is not",
            "can't": "contrazione di cannot («non riuscire a»)",
            "don't": "contrazione di do not; non farlo!",
        }
        runs = {lemma: [("VERB", 1)] for lemma in glosses}
        generated = refine.contractions(glosses, runs, {lemma: n for n, lemma in enumerate(glosses, 1)})
        self.assertEqual(generated, ["doesn't"])
        self.assertEqual(glosses["doesn't"], "contrazione di does not")
        self.assertEqual(glosses["can't"], "contrazione di cannot («non riuscire a»)")
        self.assertEqual(glosses["don't"], "contrazione di do not; non farlo!")

    def test_the_frame_is_the_section_s_and_no_italian_word_is_written(self):
        # The pointers are composed by rule: « contrazione di » is the section's wording (`isn't`), the rest
        # English; listed as generated.
        self.assertEqual(refine.CONTRACTION, "contrazione di ")
        self.assertEqual(refine.GENERATED, "contractions")
        self.assertEqual(refine.contraction_gloss("they're"), "contrazione di they are")
        self.assertIsNone(refine.contraction_gloss("women's"))


class ReviewedRows(unittest.TestCase):
    """D4's `FIRST_RUN` and D12's `CORRECTIONS`: recorded rows of the committed tables (2026-10-26)."""

    rows = RefinedRows.rows

    def test_every_reviewed_row_names_its_reason_its_sources_and_its_page(self):
        self.assertEqual(len(refine.FIRST_RUN), 34)
        self.assertEqual(len(refine.CORRECTIONS), 23)
        self.assertFalse(set(refine.FIRST_RUN) & set(refine.CORRECTIONS), "one row a word")
        self.assertIn("bye", refine.FIRST_RUN)
        for name in ("trash", "lot", "some", "disposable"):
            self.assertIn(name, refine.CORRECTIONS)
        for lemma, row in {**refine.FIRST_RUN, **refine.CORRECTIONS}.items():
            self.assertTrue(row.reason, lemma)
            self.assertTrue(row.page.startswith("https://") and "wiktionary.org/wiki/" in row.page, lemma)
            for op in row.ops:
                if isinstance(op, refine.Open):
                    names = op.source if isinstance(op.source, tuple) else (op.source,)
                    for name in names:
                        self.assertTrue(name in ("section", "direct", "inverted") or name.startswith("expression:"), lemma)
            if lemma in refine.FIRST_RUN:
                self.assertTrue(all(isinstance(op, refine.Open) for op in row.ops), lemma)
        # `connection`: each word's source recorded, « collegamento » the inverted table's.
        (connection,) = refine.FIRST_RUN["connection"].ops
        self.assertEqual(connection.source, ("direct", "inverted", "direct"))

    def test_every_gloss_written_by_a_person_or_generated(self):
        # Task 0.2 (the owner, 2026-10-26): `lot`'s « (a lot) », the only words a person writes; the
        # contractions' pointers are generated by rule (task 0.1). No FIRST_RUN row is a person's.
        self.assertEqual(refine.BY_A_PERSON, {"lot": "(a lot)"})
        marked = {lemma for lemma, row in refine.CORRECTIONS.items() if "written by a person" in row.reason}
        self.assertEqual(marked, set(refine.BY_A_PERSON))
        self.assertFalse(any("written by a person" in row.reason for row in refine.FIRST_RUN.values()))
        for lemma, words in refine.BY_A_PERSON.items():
            opened = [op.text for op in refine.CORRECTIONS[lemma].ops if isinstance(op, refine.Open)]
            self.assertTrue(any(words in text for text in opened), lemma)
        self.assertEqual(refine.GENERATED, "contractions")

    def test_spec_scenario_a_grammatical_word_reviewed(self):
        that = (
            "così; introduce una proposizione che è il soggetto o l'oggetto di un verbo; quello, quella; "
            "che, il quale, la quale, i quali, le quali; il quale, la quale, i quali, le quali, cui",
            [("ADV", 1), ("SCONJ", 1), ("PRON", 3)],
        )
        glosses, runs, _, unfired = self.rows({"that": that}, rules=("reviewed rows",))
        self.assertEqual(
            glosses["that"],
            "quello, quella; che, il quale, la quale, i quali, le quali; così; "
            "introduce una proposizione che è il soggetto o l'oggetto di un verbo; il quale, la quale, i quali, le quali, cui",
        )
        self.assertEqual(runs["that"], [("PRON", 2), ("ADV", 1), ("SCONJ", 1), ("PRON", 1)])
        self.assertEqual([u for u in unfired if u.startswith("that")], [])

    def test_spec_scenario_a_main_sense_no_rule_reaches(self):
        # `can` « latta, lattina… » alone; `FIRST_RUN` names the English Wiktionary's « potere ».
        can = ("latta, lattina, scatola, barattolo; annaffiatoio; scatoletta; cesso; natica", [("NOUN", 5)])
        glosses, runs, _, unfired = self.rows({"can": can}, direct={"can": {"VERB": ["potere", "sapere"]}}, rules=("reviewed rows",))
        self.assertEqual(glosses["can"], "potere; latta, lattina, scatola, barattolo; annaffiatoio; scatoletta; cesso; natica")
        self.assertEqual(runs["can"], [("VERB", 1), ("NOUN", 5)])
        self.assertNotIn("can", " ".join(unfired))

    def test_spec_scenario_a_reviewed_sense_no_longer_written(self):
        # Made up: the table no longer lists « potere », the section no longer writes `still`'s
        # « ancora, tuttora »: each named, nothing added in its place.
        glosses, _, _, unfired = self.rows(
            {"can": ("latta, lattina", [("NOUN", 1)]), "still": ("calmo, immobile, fermo; ancora, persino, pure (nei comparativi)", [("ADJ", 1), ("ADV", 1)])},
            direct={"can": {"VERB": ["sapere"]}},
            rules=("reviewed rows",),
        )
        self.assertEqual(glosses["can"], "latta, lattina")
        self.assertIn("can: « potere » (direct)", unfired)
        self.assertIn("still: « ancora, tuttora » (section)", unfired)
        self.assertEqual(glosses["still"], "ancora, persino, pure (nei comparativi); calmo, immobile, fermo")

    def test_spec_scenario_a_false_friend(self):
        glosses, *_ = self.rows(
            {"disposable": ("disponibile", [("ADJ", 1)])},
            direct={"disposable": {"ADJ": ["monouso", "usa e getta"]}},
            rules=("reviewed rows",),
        )
        self.assertEqual(glosses["disposable"], "monouso, usa e getta")

    def test_spec_scenario_a_copied_page(self):
        glosses, runs, _, unfired = self.rows(
            {"some": ("molti, tanti; la folla; molti, molte", [("ADJ", 1), ("NOUN", 1), ("PRON", 1)])},
            direct={"some": {"PRON": ["del", "della"]}},
            inverted={"some": {"DET": ["qualche", "alcuni", "alcune"]}},
            rules=("reviewed rows",),
        )
        self.assertEqual(glosses["some"], "qualche, alcuni, alcune; del, della")
        self.assertEqual(runs["some"], [("DET", 1), ("PRON", 1)])
        self.assertEqual([u for u in unfired if u.startswith("some")], [])
        # `trash`: the section's eight senses are *thrash*'s.
        trash = (
            "festa chiassosa o sontuosa; battere, percuotere, picchiare; pestare; frustare, fustigare, sferzare; "
            "battere, sconfiggere; trebbiare, battere; agitare; battere, percuotere, colpire",
            [("NOUN", 1), ("VERB", 7)],
        )
        glosses, *_ = self.rows(
            {"trash": trash},
            direct={"trash": {"NOUN": ["spazzatura", "immondizia", "rifiuti"], "VERB": ["scartare"]}},
            rules=("reviewed rows",),
        )
        self.assertEqual(glosses["trash"], "spazzatura, immondizia, rifiuti; scartare; battere, percuotere, colpire")

    def test_spec_scenario_a_gloss_written_by_a_person(self):
        # `lot` opens on the expression *a lot*'s « molto, parecchio », « (a lot) » a person's (task 0.2).
        lot = ("lotto; destino; set; quota; gruppo di persone; tizio", [("NOUN", 6)])
        glosses, runs, _, unfired = self.rows({"lot": lot}, expressions={"a lot": "molto, parecchio; spesso"}, rules=("reviewed rows",))
        self.assertEqual(glosses["lot"], "molto, parecchio (a lot); lotto; destino; set; quota; gruppo di persone; tizio")
        self.assertEqual(runs["lot"], [("ADV", 1), ("NOUN", 6)])
        # Without the expression's words, nothing is written, and the row is named.
        glosses, _, _, unfired = self.rows({"lot": lot}, rules=("reviewed rows",))
        self.assertEqual(glosses["lot"], lot[0])
        self.assertIn("lot: « molto, parecchio (a lot) » (expression:a lot)", unfired)

    def test_spec_scenario_a_correction_that_finds_nothing(self):
        # Made up: the section no longer writes `scale`'s « squaglia » nor `prove`'s « prove ».
        glosses, _, _, unfired = self.rows(
            {"scale": ("scala, gamma; squama", [("NOUN", 2)]), "prove": ("provare", [("VERB", 1)])},
            direct={"prove": {"VERB": ["dimostrare", "provare"]}},
            rules=("reviewed rows",),
        )
        self.assertIn("scale: « squaglia »", unfired)
        self.assertIn("prove: « prove »", unfired)
        self.assertEqual(glosses["prove"], "dimostrare, provare; provare")
        self.assertIn("media: no row", unfired, "a reviewed row whose lemma no source glosses")


class RefinedSection(unittest.TestCase):
    """The section's entries (D5–D9, D13), recorded from `kaikki-it-Inglese.jsonl` (sha256 79033de9…)."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = Path(self._tmp.name)

    def tearDown(self):
        self._tmp.cleanup()

    def read(self, *entries, rules=refine.RULES, zipf=None):
        src = self.dir / "section.jsonl"
        src.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries), encoding="utf-8")
        out = refine.read_section(str(src), str(self.dir / "refined.jsonl"), zipf=zipf or ITALIAN_AND_ENGLISH, rules=rules)
        senses = collections.defaultdict(list)
        for line in Path(out).read_text(encoding="utf-8").splitlines():
            entry = json.loads(line)
            if not isinstance(entry, dict):
                continue
            senses[(entry["word"], entry["pos"])] += [s["glosses"][0] for s in entry["senses"]]
        return dict(senses)

    def test_spec_scenario_a_frame(self):
        read = self.read(
            {"word": "my", "pos": "adj", "senses": [{"glosses": ["forma possessiva di I usata come aggettivo possessivo, il mio, la mia, i miei, le mie"]}]},
            {"word": "he", "pos": "pron", "senses": [{"glosses": ["in funzione di soggetto: egli, lui, esso"]}]},
            {"word": "first", "pos": "adj", "senses": [{"glosses": ["corrisponde al numero cardinale 1, primo"]}]},
            {"word": "yes", "pos": "noun", "senses": [{"glosses": ["risposta affermativa, sì"]}]},
        )
        self.assertEqual(read[("my", "adj")], ["il mio, la mia, i miei, le mie"])
        self.assertEqual(read[("he", "pron")], ["egli, lui, esso"])
        self.assertEqual(read[("first", "adj")], ["primo"])
        self.assertEqual(read[("yes", "noun")], ["sì"])

    def test_spec_scenario_a_frame_kept(self):
        entries = (
            {"word": "audio", "pos": "adj", "senses": [{"glosses": ["relativo all'ascolto, alla riproduzione sonora"]}]},
            {"word": "leader", "pos": "noun", "senses": [{"glosses": ["capo, guida"]}, {"glosses": ["articolo di fondo, editoriale"]}]},
        )
        read = self.read(*entries)
        self.assertEqual(read[("audio", "adj")], ["relativo all'ascolto, alla riproduzione sonora"])
        self.assertEqual(read[("leader", "noun")], ["capo, guida", "articolo di fondo, editoriale"])

    def test_spec_scenario_a_name_on_a_common_word_and_a_given_name(self):
        read = self.read(
            {"word": "summer", "pos": "noun", "senses": [{"glosses": ["estate"]}]},
            {"word": "Summer", "pos": "name", "senses": [{"glosses": ["nome proprio di persona inglese femminile"]}]},
            {"word": "John", "pos": "name", "senses": [{"glosses": ["nome proprio di persona maschile, corrispondente all'italiano Giovanni"]}]},
            {"word": "Scott", "pos": "name", "senses": [{"glosses": ["nome proprio di persona inglese maschile"]}]},
        )
        self.assertNotIn(("Summer", "name"), read)
        self.assertEqual(read[("John", "name")], ["Giovanni (nome)"])
        self.assertEqual(read[("Scott", "name")], ["nome proprio di persona inglese maschile"], "a name alone keeps its note")

    def test_spec_scenario_heraldry_inside_a_row(self):
        read = self.read(
            {"word": "side", "pos": "noun", "senses": [
                {"glosses": ["lato"]}, {"glosses": ["faccia di una medaglia"]},
                {"glosses": ["fianco: porzione di uno scudo, larga non più di un sesto del totale, delimitata da una linea verticale; può essere destro o sinistro"], "topics": ["heraldry"]},
            ]},
            {"word": "or", "pos": "noun", "senses": [{"glosses": ["oro"], "topics": ["heraldry"]}]},
            {"word": "or", "pos": "conj", "senses": [{"glosses": ["o, ovvero;"]}]},
            {"word": "savage", "pos": "noun", "senses": [{"glosses": ["selvatico; figura utilizzata prevalentemente come tenente dello scudo e consistente in un uomo nudo"], "topics": ["heraldry"]}]},
            {"word": "crown", "pos": "noun", "senses": [{"glosses": ["corona"], "topics": ["heraldry"]}]},
        )  # fmt: skip
        self.assertEqual(read[("side", "noun")], ["lato", "faccia di una medaglia"])
        self.assertNotIn(("or", "noun"), read, "a tincture")
        self.assertEqual(read[("savage", "noun")], ["selvatico"])
        self.assertEqual(read[("crown", "noun")], ["corona"], "a sense tagged heraldic alone, not worded so, stays")

    def test_spec_scenario_an_expansion_and_a_list_of_them(self):
        csi = {"word": "CSI", "pos": "abbrev", "tags": ["abbreviation"], "senses": [
            {"glosses": [text]} for text in ("California Security Institute", "Can't Say It", "Canadian Sugar Institute", "Crime Scene Investigation")
        ]}  # fmt: skip
        read = self.read(
            {"word": "UK", "pos": "abbrev", "tags": ["abbreviation"], "senses": [{"glosses": ["United Kingdom : Regno Unito"]}]},
            {"word": "MA", "pos": "abbrev", "tags": ["abbreviation"], "senses": [{"glosses": ["Moving Average: media mobile"]}, {"glosses": ["Sigla postale per Massachusetts"]}]},
            {"word": "CQC", "pos": "abbrev", "tags": ["abbreviation"], "senses": [{"glosses": ["Close Quarters Combat: vedi CQB"]}]},
            {"word": "DE", "pos": "abbrev", "tags": ["abbreviation"], "senses": [{"glosses": ["Sigla postale per il Delaware"]}]},
            csi,
        )
        self.assertEqual(read[("UK", "abbrev")], ["Regno Unito"])
        self.assertEqual(read[("MA", "abbrev")], ["media mobile"], "a state's code beside another sense")
        self.assertEqual(read[("DE", "abbrev")], ["Sigla postale per il Delaware"], "a code's only sense")
        self.assertEqual(read[("CSI", "abbrev")], [s["glosses"][0] for s in csi["senses"]], "four expansions alone")
        self.assertEqual(read[("CQC", "abbrev")], ["Close Quarters Combat: vedi CQB"], "its only gloss, with D7 and D9 together")

    def test_pointers_and_wiki_text(self):
        read = self.read(
            {"word": "cellphone", "pos": "noun", "senses": [{"glosses": ["variante di cell phone, telefono cellulare"]}]},
            {"word": "ashtray", "pos": "noun", "senses": [{"glosses": ["vedi ash tray"]}]},
            {"word": "sandbox", "pos": "noun", "senses": [{"glosses": ["sabbionaia"]}, {"glosses": ["nei progetti wiki si intende una pagina dove fare delle prove"]}]},
            "not json",
        )
        self.assertEqual(read[("cellphone", "noun")], ["telefono cellulare"])
        self.assertEqual(read[("ashtray", "noun")], ["vedi ash tray"], "the word's only sense")
        self.assertEqual(read[("sandbox", "noun")], ["sabbionaia"])

    def test_no_rule_writes_the_section_as_it_is(self):
        entries = [{"word": "UK", "pos": "abbrev", "tags": ["abbreviation"], "senses": [{"glosses": ["United Kingdom : Regno Unito"]}]}, [1, 2]]
        src = self.dir / "section.jsonl"
        src.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries) + "not json\n", encoding="utf-8")
        out = refine.read_section(str(src), str(self.dir / "refined.jsonl"), zipf=ITALIAN_AND_ENGLISH, rules=())
        self.assertEqual(Path(out).read_bytes(), src.read_bytes())

    def test_spec_scenario_split_forms(self):
        table = {"mx": {"NOUN": ["signorə", "signor", "a"]}, "and": {"CCONJ": ["e", "ed"]}, "primer": {"NOUN": ["primer", "m"]}}
        self.assertEqual(refine.split_forms(table), {"mx": {"NOUN": ["signor/a"]}, "and": {"CCONJ": ["e", "ed"]}, "primer": {"NOUN": ["primer/m"]}})
        self.assertIs(refine.split_forms(table, rules=()), table)
        self.assertEqual(refine.split_forms({"x": {"NOUN": ["xə"]}}), {})

    def test_the_english_wiktionary_s_first_sense(self):
        src = self.dir / "direct.jsonl"
        entries = [
            {"word": "make", "pos": "verb", "translations": [{"word": "fare", "sense": "to construct"}, {"word": "costruire (edifici)", "sense": "to construct"}, {"word": "rendere", "sense": "to cause"}]},
            {"word": "make", "pos": "noun", "translations": [{"word": "marca", "sense": "brand"}]},
            {"word": "Make", "pos": "name", "translations": [{"word": "Make", "sense": "x"}]},
            {"word": "-ness", "pos": "suffix", "translations": [{"word": "-ezza", "sense": "s"}]},
            {"word": "gay", "pos": "adj", "translations": [{"word": "-ə", "sense": "s"}, {"word": "a", "sense": "s"}]},
        ]  # fmt: skip
        src.write_text("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries) + "not json\n", encoding="utf-8")
        self.assertEqual(refine.first_senses(str(src), IT_TYPOGRAPHY), {"make": ("VERB", ["fare", "costruire"])})

    def test_frames_and_expansions_read_alone(self):
        self.assertEqual(refine.without_frame("relativo ai Stati Uniti, americano, statunitense"), "americano, statunitense")
        self.assertEqual(refine.without_frame("relativo all'ascolto, alla riproduzione sonora"), "relativo all'ascolto, alla riproduzione sonora")
        self.assertEqual(refine.without_expansion("abbreviazione di justice of the peace (giudice di pace)", ITALIAN_AND_ENGLISH), "giudice di pace")
        self.assertEqual(refine.without_expansion("British Broadcasting Corporation", ITALIAN_AND_ENGLISH), "")
        self.assertEqual(refine.without_expansion("Federal Bureau, ente federale", ITALIAN_AND_ENGLISH), "ente federale")
        self.assertEqual(refine.without_pointer("troll (vedi in italiano)"), "troll")
        self.assertTrue(refine.describes("possessivo di it"))
        self.assertFalse(refine.describes("suo, sua"))


# Made up for the section's cases: English's words rate in English, Italian's in Italian.
_ENGLISH_WORDS = {
    "united": 5.5, "kingdom": 5.0, "moving": 5.0, "average": 5.0, "close": 5.5, "quarters": 4.5, "combat": 4.5,
    "california": 5.0, "security": 5.0, "institute": 4.8, "can't": 6.0, "say": 6.0, "it": 7.0, "canadian": 4.5,
    "sugar": 4.8, "crime": 5.0, "scene": 5.0, "investigation": 4.8, "justice": 5.0, "of": 7.5, "the": 7.7, "peace": 5.0,
    "british": 5.3, "broadcasting": 4.3, "corporation": 4.8, "federal": 5.0, "bureau": 4.5, "vedi": 0.0, "cqb": 1.0,
}  # fmt: skip
_ITALIAN_WORDS = {"regno": 4.3, "unito": 4.5, "media": 5.0, "mobile": 4.8, "giudice": 4.5, "pace": 5.0, "ente": 4.0, "federale": 4.2, "sigla": 3.5, "postale": 3.6}
ITALIAN_AND_ENGLISH = zipf_of(_ITALIAN_WORDS, _ENGLISH_WORDS)


if __name__ == "__main__":
    unittest.main()
