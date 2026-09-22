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
            {"word": "shrunk", "pos": "verb", "senses": [{"glosses": ["Passé de shrink."]}]},
            # Only kaikki's tag and pointer say this sense is a form of "sweep".
            {
                "word": "swept",
                "pos": "verb",
                "senses": [{"glosses": ["Balayé."], "tags": ["form-of"], "form_of": [{"word": "sweep"}]}],
            },
            {"word": "ignored", "pos": "noun", "senses": [{"glosses": ["Sans intérêt."]}]},
        ]
        path = self.file("kaikki.jsonl", "".join(json.dumps(e) + "\n" for e in entries) + "not json\n")
        words = {"butter", "number", "found", "data", "shrunk", "swept"}
        meanings, targets = red.wiktionary_signals(path, words)
        self.assertEqual(meanings, {"butter": {"noun"}, "number": {"noun"}})
        self.assertIn("numb", targets["number"])
        self.assertIn("find", targets["found"])
        self.assertIn("datum", targets["data"])
        self.assertIn("shrink", targets["shrunk"])
        self.assertIn("sweep", targets["swept"])
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
        "feed": {("fee", "V")},
        "yourselves": {("yourself", "N")},
        "ros": {("ro", "N")},
        "timer": {("time", "A")},
        "could": {("can", "V")},
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
        "feed": {"noun", "verb"},
        "yourselves": {"pron"},
        "ros": {"noun"},
        "timer": {"noun"},
        "could": {"verb"},
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
        "feed": {("noun", "A1"), ("verb", "B1")},
        "fee": {("noun", "B1")},
        "yourself": {("pronoun", "A1")},
        "could": {("modal auxiliary", "A1")},
        "can": {("modal auxiliary", "A1")},
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
        self.assertEqual(self.own(), {"butter", "number", "customer", "owner", "feed", "timer", "could"})

    def test_modals_and_untaught_forms(self):
        # A modal is taught apart from the verb AGID derives it from, even when Wiktionary
        # calls it a form of that verb.
        attested = {**self.targets, "could": {"can"}}
        self.assertIn("could", self.own(targets=attested))
        self.assertNotIn("could", self.own(cefr={}, targets=attested))
        # "timer" is a regular spelling of the taught "time", but "time" is no adjective.
        self.assertNotIn("timer", self.own(meanings={**self.meanings, "time": {"adj"}}))

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
        # The -ed of "feed" is its stem: taught as a noun, "fee" is never a verb.
        self.assertNotIn("feed", self.own(cefr={}))
        # Untaught, next to a taught base or a two-letter one: AGID's base stands.
        self.assertNotIn("yourselves", self.own())
        self.assertNotIn("ros", self.own())
        # -ing forms and the analyser's irregulars never become words of their own.
        self.assertNotIn("running", self.own())
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
    ranks = {"the": 1, "well": 2, "know": 3, "shirt": 4}
    zipf = {"the": 7.0, "well": 6.0, "know": 5.5, "shirt": 4.5, "boring": 5.0, "quokka": 2.0, "zyzzyva": 1.0}
    lemma_of = staticmethod({"well": "well", "known": "know", "shirt": "shirt", "gives": "give"}.get)

    def extras(self, cefr, inflected=()):
        return red.append_level_extras(
            self.ranks, cefr, set(inflected), lambda w: self.zipf.get(w, 0.0), self.lemma_of
        )

    def test_a_compound_ranks_with_its_rarest_part(self):
        out = self.extras({"well-known": set(), "the": set()})
        self.assertEqual(out["well-known"], 3)  # "known" -> "know", the rarer part
        self.assertEqual(out["the"], 1)  # an already-ranked word keeps its rank

    def test_a_word_ranks_with_the_lemmas_of_its_frequency(self):
        # "boring" is an inflection of a dropped base: without a rank it would vanish.
        out = self.extras({"boring": set(), "gives": set()}, inflected={"boring", "gives"})
        self.assertEqual(out["boring"], 3)  # as frequent as "know" or more, less than "well"
        self.assertNotIn("gives", out)  # already reads as the kept "give"

    def test_rarer_words_follow_the_list_commonest_first(self):
        out = self.extras({"t-shirt": set(), "zyzzyva": set(), "quokka": set(), "a.m.": set()})
        # "t" is not a ranked part, so the compound joins the rare tail too.
        self.assertEqual({w: out[w] for w in ("quokka", "zyzzyva", "t-shirt")}, {"quokka": 5, "zyzzyva": 6, "t-shirt": 7})
        self.assertNotIn("a.m.", out)
        self.assertEqual(self.ranks, {"the": 1, "well": 2, "know": 3, "shirt": 4})  # input untouched


class CompoundInflections(unittest.TestCase):
    def test_first_and_last_parts_inflect(self):
        pairs = {("shirts", "shirt"), ("shirt", "shirt"), ("mothers", "mother"), ("cats", "cat")}
        out = red.compound_inflections({"t-shirt", "mother-in-law", "cat"}, pairs)
        self.assertEqual(out, {("t-shirts", "t-shirt"), ("mothers-in-law", "mother-in-law")})


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
        forms = red.resolve_forms(pairs, ranks)
        self.assertEqual(forms["butter"], "butter")  # a kept word of its own maps to itself
        self.assertEqual(forms["leaves"], "leave")  # the commonest base wins
        self.assertEqual(forms["gives"], "give")
        self.assertEqual(forms["leaf"], "leaf")  # every kept lemma resolves as itself
        self.assertEqual(forms["number"], "numb")  # too rare to keep: resolves as before
        self.assertNotIn("rare", forms)  # its lemma is not kept

    def test_a_kept_lemma_is_never_read_as_another(self):
        # "bored" was added for its CEFR level: read as the commoner "bore", it would give
        # "bore" its rank and level in the built pack.
        forms = red.resolve_forms({("bored", "bore"), ("bores", "bore")}, {"bore": 1, "bored": 2})
        self.assertEqual(forms["bored"], "bored")
        self.assertEqual(forms["bores"], "bore")

    def test_the_named_then_the_glossed_base_wins_over_frequency(self):
        pairs = {("uses", "us"), ("uses", "use"), ("fulfilled", "fulfill"), ("fulfilled", "fulfil")}
        ranks = {"us": 1, "use": 2, "fulfill": 3, "fulfil": 4}
        forms = red.resolve_forms(pairs, ranks, targets={"uses": {"use"}}, glossed={"fulfil", "us"})
        self.assertEqual(forms["uses"], "use")  # Wiktionary: "Pluriel de use"
        self.assertEqual(forms["fulfilled"], "fulfil")  # "fulfill" has no gloss


class ReduceExpressions(TempDir):
    def kaikki(self, entries):
        return self.file("kaikki.jsonl", "".join(json.dumps(e) + "\n" for e in entries) + "not json\n")

    def test_only_multi_word_headwords_the_character_set_accepts(self):
        path = self.kaikki(
            [
                {"word": "give up", "pos": "verb", "senses": [{"glosses": ["Abandonner."]}]},
                {"word": "GIVE UP", "pos": "noun", "senses": [{"glosses": ["Abandon."]}]},
                {"word": "abandon", "pos": "verb", "senses": [{"glosses": ["Abandonner."]}]},
                {"word": "café au lait", "pos": "noun", "senses": [{"glosses": ["Café au lait."]}]},
                {"word": "vitamin b12", "pos": "noun", "senses": [{"glosses": ["Vitamine B12."]}]},
            ]
        )
        # One line per headword: the two spellings of "give up" are one entry, the
        # single word is not an expression, and a headword `_TOKEN` rejects is dropped.
        self.assertEqual(red.reduce_expressions(path, 80), {"give up": "Abandonner; Abandon"})

    def test_the_four_wordings_the_shared_filter_lacks_are_dropped(self):
        entries = [
            {"word": "present tense", "pos": "noun", "senses": [{"glosses": ["Présent (temps grammatical)."]}]},
            {"word": "will go", "pos": "verb", "senses": [{"glosses": ["Futur de go."]}]},
            {"word": "to be", "pos": "verb", "senses": [{"glosses": ["Conjugaison du verbe be."]}]},
            {"word": "douche bag", "pos": "noun", "senses": [{"glosses": ["Graphie alternative de douchebag."]}]},
            # What `_FORM_OF` already covered on its own.
            {"word": "gave up", "pos": "verb", "senses": [{"glosses": ["Prétérit de give up."]}]},
            {"word": "given up", "pos": "verb", "senses": [{"glosses": ["Participe passé de give up."]}]},
        ]
        self.assertEqual(red.reduce_expressions(self.kaikki(entries), 80), {})

    def test_the_shared_filter_is_untouched(self):
        # The four extra wordings are dropped in the multi-word path ONLY: the same
        # sense on a single-word entry still feeds gloss.tsv, so forms.tsv, freq.tsv
        # and gloss.tsv come out of a rebuild byte-identical.
        path = self.kaikki([{"word": "present", "pos": "noun", "senses": [{"glosses": ["Présent (temps grammatical)."]}]}])
        self.assertEqual(red.reduce_gloss(path, {"present"}, 80), {"present": "Présent (temps grammatical)"})
        self.assertFalse(red._is_form_of({}, "Présent progressif."))

    def test_a_name_only_entry_is_dropped(self):
        entries = [
            {"word": "new york", "pos": "name", "senses": [{"glosses": ["New York."]}]},
            # A headword that is a proper noun AND a common one keeps the common senses.
            {"word": "big apple", "pos": "name", "senses": [{"glosses": ["New York."]}]},
            {"word": "big apple", "pos": "noun", "senses": [{"glosses": ["Grosse pomme."]}]},
        ]
        self.assertEqual(red.reduce_expressions(self.kaikki(entries), 80), {"big apple": "Grosse pomme"})

    def test_two_spellings_stay_two_lines(self):
        # The builder resolves the collision — it is the one holding the lexicon that
        # says both lemmatise to "break point".
        entries = [
            {"word": "break point", "pos": "noun", "senses": [{"glosses": ["Point d'arrêt."]}]},
            {"word": "breaking point", "pos": "noun", "senses": [{"glosses": ["Point de rupture."]}]},
        ]
        self.assertEqual(
            red.reduce_expressions(self.kaikki(entries), 80),
            {"break point": "Point d'arrêt", "breaking point": "Point de rupture"},
        )

    def test_senses_are_cut_per_sense_then_on_the_joined_string(self):
        entries = [
            {
                "word": "starting point",
                "pos": "noun",
                "senses": [
                    {"glosses": ["Point de départ, origine, commencement, amorce, prémisse."]},
                    {"glosses": ["Base de discussion."]},
                    {"glosses": ["Repère."]},
                    {"glosses": ["Une quatrième acception que personne ne verra."]},
                ],
            },
        ]
        path = self.kaikki(entries)
        # 42 characters per sense, then three senses at most, then the joined cut.
        self.assertEqual(
            red.reduce_expressions(path, 80),
            {"starting point": "Point de départ, origine, commencement, am; Base de discussion; Repère"},
        )
        self.assertEqual(
            red.reduce_expressions(path, 40),
            {"starting point": "Point de départ, origine, commencement,"},
        )


class FormOfGlosses(unittest.TestCase):
    def test_pointers_are_form_of(self):
        for gloss in ("Comparatif de numb.", "Superlatif de fore.", "Pluriel de datum.", "Passé de shrink."):
            self.assertTrue(red._is_form_of({}, gloss), gloss)
        self.assertTrue(red._is_form_of({"tags": ["form-of"]}, "Balayé."))
        self.assertFalse(red._is_form_of({}, "Nombre."))


class DanglingCoordinator(unittest.TestCase):
    """A gloss must not open on a coordinator the extraction left hanging."""

    def test_strips_a_lowercase_leading_coordinator(self):
        # The five shapes the real corpus holds, one per affected entry kind.
        for raw, want in (
            ("ou Pluie", "Pluie"),  # rain
            ("et Biochimie", "Biochimie"),  # biochemistry
            ("ou Céder, abandonner", "Céder, abandonner"),  # cede
            ("ou NEET", "NEET"),  # neet
            ("et Égalité", "Égalité"),  # deuce
        ):
            self.assertEqual(red.clean_gloss(raw, 80), want, raw)

    def test_keeps_a_capitalised_coordinator_that_is_the_translation(self):
        # `etcetera` is glossed "Et cetera": here the coordinator IS the meaning.
        self.assertEqual(red.clean_gloss("Et cetera", 80), "Et cetera")

    def test_leaves_a_coordinator_inside_the_gloss_alone(self):
        self.assertEqual(red.clean_gloss("Nom ou titre", 80), "Nom ou titre")
        self.assertEqual(red.clean_gloss("Étalon et jument", 80), "Étalon et jument")

    def test_leaves_a_word_that_merely_starts_with_those_letters(self):
        for text in ("Outil", "Ouvrir", "Étrange", "Été"):
            self.assertEqual(red.clean_gloss(text, 80), text)

    def test_cuts_to_length_after_stripping(self):
        # The cut counts the text the reader sees, not the coordinator that went.
        self.assertEqual(red.clean_gloss("ou " + "a" * 50, 10), "a" * 10)


if __name__ == "__main__":
    unittest.main()
