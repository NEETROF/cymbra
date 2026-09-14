# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the EN->FR reducer's pure rules (no download, no wordfreq).

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import importlib.util
import json
import os
import tempfile
import unittest

_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("reduce_en_fr", os.path.join(_HERE, "reduce-en-fr.py"))
red = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(red)


class TempDir(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.dir = self._tmp.name

    def tearDown(self):
        self._tmp.cleanup()

    def file(self, name, text):
        path = os.path.join(self.dir, name)
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        return path


class ParseAgidRelations(TempDir):
    def test_pairs_and_relation_kind(self):
        path = self.file(
            "infl.txt",
            "but A: butter | buttest?, butest!<? 1\n"
            "gif N?: gives\n"
            "run V: ran | running | runs\n",
        )
        pairs, relations = red.parse_agid_relations(path)
        self.assertIn(("butter", "but"), pairs)
        self.assertIn(("run", "run"), pairs)
        self.assertNotIn(("gives", "gif"), pairs)  # questionable headword dropped
        self.assertEqual(relations["butter"], {("but", "A")})
        self.assertEqual(relations["running"], {("run", "V")})
        self.assertNotIn("run", relations)  # a lemma is not its own inflection


class AnalyserIrregulars(TempDir):
    def test_reads_the_analyser_table(self):
        table = red.analyser_irregulars()
        self.assertEqual(table["found"], "find")
        self.assertEqual(table["left"], "leave")

    def test_fails_loudly_without_a_table(self):
        with self.assertRaises(ValueError):
            red.analyser_irregulars(self.file("lemmatize.rs", "fn lemmatize() {}\n"))


class WiktionarySignals(TempDir):
    def test_meanings_and_form_of_targets(self):
        entries = [
            {"word": "butter", "pos": "noun", "senses": [{"glosses": ["Beurre."]}]},
            {"word": "number", "pos": "adj", "senses": [{"glosses": ["Comparatif de numb."]}]},
            {"word": "number", "pos": "noun", "senses": [{"glosses": ["Nombre."]}]},
            {"word": "found", "pos": "verb", "senses": [{"glosses": ["Prétérit du verbe to find."]}]},
            {"word": "data", "pos": "noun", "senses": [{"glosses": ["Pluriel de datum."]}]},
            {"word": "ignored", "pos": "noun", "senses": [{"glosses": ["Sans intérêt."]}]},
        ]
        path = self.file("kaikki.jsonl", "".join(json.dumps(e) + "\n" for e in entries) + "not json\n")
        meanings, targets = red.wiktionary_signals(path, {"butter", "number", "found", "data"})
        self.assertEqual(meanings, {"butter": {"noun"}, "number": {"noun"}})
        self.assertIn("numb", targets["number"])
        self.assertIn("find", targets["found"])
        self.assertIn("datum", targets["data"])
        self.assertNotIn("butter", targets)


class RegularInflection(unittest.TestCase):
    def test_spelling_rules(self):
        self.assertTrue(red.regular_inflection("cities", "city", "N"))
        self.assertTrue(red.regular_inflection("stopped", "stop", "V"))
        self.assertTrue(red.regular_inflection("tried", "try", "V"))
        self.assertTrue(red.regular_inflection("making", "make", "V"))
        self.assertTrue(red.regular_inflection("bigger", "big", "A"))
        self.assertTrue(red.regular_inflection("butter", "but", "A"))  # spelling alone proves nothing
        self.assertFalse(red.regular_inflection("data", "datum", "N"))
        self.assertFalse(red.regular_inflection("ran", "run", "V"))
        self.assertFalse(red.regular_inflection("runs", "run", "?"))


class OwnWords(unittest.TestCase):
    relations = {
        "butter": {("but", "A")},
        "sales": {("sale", "N")},
        "number": {("numb", "A")},
        "customer": {("custom", "A")},
        "times": {("time", "N"), ("time", "V")},
        "owner": {("own", "A")},
        "running": {("run", "V")},
        "found": {("find", "V")},
    }
    meanings = {
        "butter": {"noun"},
        "but": {"conj"},
        "sales": {"noun"},
        "sale": {"noun"},
        "number": {"noun"},
        "numb": {"adj"},
        "customer": {"noun"},
        "custom": {"noun", "adj"},
        "times": {"noun"},
        "time": {"noun", "verb"},
        "owner": {"noun"},
        "own": {"adj", "verb"},
        "running": {"noun"},
        "found": {"verb"},
    }
    targets = {"number": {"numb"}, "times": {"time"}}
    cefr = {
        "customer": {("noun", "A2")},
        "custom": {("noun", "B1")},
        "times": {("preposition", "B2")},
        "time": {("noun", "A1")},
        "owner": {("noun", "B1")},
        "own": {("adjective", "A1")},
    }
    zipf = {"number": 5.5, "numb": 3.3, "butter": 4.4, "but": 6.6, "sales": 5.0, "sale": 4.9}

    def own(self, **overrides):
        args = dict(
            relations=self.relations,
            meanings=self.meanings,
            targets=self.targets,
            cefr=self.cefr,
            frequency=lambda w: self.zipf.get(w, 4.0),
            never={"found"},
        )
        args.update(overrides)
        return red.own_words(**args)

    def test_words_of_their_own(self):
        self.assertEqual(self.own(), {"butter", "number", "customer", "owner"})

    def test_why_each_form_stays_or_goes(self):
        # "but" is no adjective, so no relation is believable.
        self.assertIn("butter", self.own(relations={"butter": self.relations["butter"]}))
        # Unattested too, but "sale" is a noun that regularly pluralises to "sales".
        self.assertNotIn("sales", self.own())
        # Attested as a comparative, but a hundred times commoner than "numb".
        self.assertNotIn("number", self.own(frequency=lambda w: 4.0))
        # Taught as a noun, and "custom" is only ever taught as a noun.
        self.assertNotIn("customer", self.own(cefr={}))
        # Taught as a preposition, but "time" is taught as the noun it pluralises.
        self.assertNotIn("times", self.own())
        # -ing forms and the analyser's irregulars never become words of their own.
        self.assertNotIn("running", self.own(meanings={**self.meanings, "run": set()}))
        self.assertNotIn("found", self.own())
        self.assertIn("found", self.own(never=set(), cefr={**self.cefr, "found": {("noun", "B2")}}))


class ReadCefr(TempDir):
    def test_variants_levels_and_missing_files(self):
        path = self.file(
            "cefr.csv",
            "headword,pos,CEFR\n"
            "a.m./A.M./am/AM,adverb,A1\n"
            "evening,noun,A1\n"
            "evening,verb,C1\n"
            "odd,adjective,Z9\n",
        )
        cefr = red.read_cefr([path, os.path.join(self.dir, "absent.csv")])
        self.assertEqual(cefr["am"], {("adverb", "A1")})
        self.assertNotIn("a.m.", cefr)  # not a word token
        self.assertEqual(cefr["evening"], {("noun", "A1"), ("verb", "C1")})
        self.assertNotIn("odd", cefr)  # unknown level label

    def test_reduce_levels_keeps_lowest_level_of_kept_lemmas(self):
        cefr = {"evening": {("noun", "B1"), ("verb", "A2")}, "rare": {("noun", "C2")}}
        self.assertEqual(red.reduce_levels(cefr, {"evening"}), {"evening": "A2"})


class AppendLevelExtras(unittest.TestCase):
    ranks = {"the": 1, "well": 7, "know": 12, "shirt": 30}
    lemma_of = staticmethod({"well": "well", "known": "know", "shirt": "shirt"}.get)

    def extras(self, cefr, zipf=None, inflected=()):
        zipf = zipf or {}
        return red.append_level_extras(self.ranks, cefr, set(inflected), lambda w: zipf.get(w, 0.0), self.lemma_of)

    def test_a_compound_ranks_with_its_rarest_part(self):
        out = self.extras({"well-known": set(), "the": set()})
        self.assertEqual(out["well-known"], 12)  # "known" -> "know", the rarer part
        self.assertEqual(out["the"], 1)  # an already-ranked word keeps its rank

    def test_other_words_follow_the_frequency_list_commonest_first(self):
        cefr = {"t-shirt": set(), "zyzzyva": set(), "quokka": set(), "gives": set(), "a.m.": set()}
        out = self.extras(cefr, zipf={"quokka": 2.0, "zyzzyva": 1.0}, inflected={"gives"})
        # "t" is not a ranked part, so the compound joins the rare tail too.
        self.assertEqual({w: out[w] for w in ("quokka", "t-shirt", "zyzzyva")}, {"quokka": 31, "zyzzyva": 32, "t-shirt": 33})
        self.assertNotIn("gives", out)
        self.assertNotIn("a.m.", out)
        self.assertEqual(self.ranks, {"the": 1, "well": 7, "know": 12, "shirt": 30})  # input untouched


class ResolveForms(unittest.TestCase):
    def test_one_lemma_per_form(self):
        pairs = {
            ("butter", "but"),
            ("butter", "butter"),
            ("leaves", "leaf"),
            ("leaves", "leave"),
            ("gives", "give"),
            ("number", "numb"),
            ("rare", "unkept"),
        }
        ranks = {"but": 1, "leave": 2, "give": 3, "numb": 4, "butter": 5, "leaf": 9}
        forms = red.resolve_forms(pairs, ranks, own={"butter", "number"})
        self.assertEqual(forms["butter"], "butter")  # a kept word of its own maps to itself
        self.assertEqual(forms["leaves"], "leave")  # the commonest base wins
        self.assertEqual(forms["gives"], "give")
        self.assertEqual(forms["leaf"], "leaf")  # every kept lemma resolves as itself
        self.assertEqual(forms["number"], "numb")  # too rare to keep: resolves as before
        self.assertNotIn("rare", forms)  # its lemma is not kept


class FormOfGlosses(unittest.TestCase):
    def test_comparisons_are_form_of(self):
        for gloss in ("Comparatif de numb.", "Superlatif de fore.", "Pluriel de datum."):
            self.assertTrue(red._FORM_OF.match(gloss), gloss)
        self.assertFalse(red._FORM_OF.match("Nombre."))


if __name__ == "__main__":
    unittest.main()
