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


# — ESDB, the inflection source (switch-lingua-inflections-to-esdb) —

# Real lines of ESDB rel-2026.02.25's scowl.txt, groups separated by a blank line as there.
ESDB_LINES = """\
35: run <n_v>: ran, run, running, runs, run's

35: go <v>: went, gone, going, goes
35: go <aj>

35: be <v>: (was | @: wast), were, been, being, am, (are | @: art), is, are

35: bear <n>: (bear | bears*), bear's

35: bear <v>: bore, (borne | ~: born), bearing, bears

35: lie <v> {fib}: lied, lying, lies
35: lie <v> {recline}: lay, lain, lying, lies

35: saw <n_v>: sawed, (sawed | ~: sawn), sawing, saws, saw's

35: focus <n>: (focuses | ~: foci)
35: focus <v>: (A B: focused | AV Bv: focussed), (A B: focusing | AV Bv: focussing), (focuses | ~: focusses)

35: learn <v>: (A B= Z: learned | B Zv: learnt), learning, learns

35: datum <n>: data
35: datum <n> {reference point}

35: leaf <n>: leaves†, leaf's

35: few <d>: fewer, fewest

35: that <d>: those

80: -renown <v>: renowned, renowning, renowns

35: renowned <aj>

35: A B: tire <n_v> {exhausted}: tired, tiring, tires, tire's
35: A B: tired <aj>: tireder, tiredest

60 [brif] 70 [3of6] 80: aah <v>
80: - <v>: aahed-, aahing-, aahs-

85 [ukacd]: abacination <n>: abacinations
"""


class Esdb(TempDir):
    def relations(self, text=ESDB_LINES, **kwargs):
        path = os.path.join(self.dir, "scowl.txt")
        with open(path, "w", encoding="utf-8") as f:
            f.write(text)
        return red.parse_esdb_relations(path, **kwargs)

    def test_same_structure_as_agid(self):
        pairs, relations = self.relations()
        self.assertIn(("ran", "run"), pairs)
        self.assertIn(("run", "run"), pairs)
        self.assertEqual(relations["went"], {("go", "V")})
        self.assertEqual(relations["runs"], {("run", "N"), ("run", "V")})  # a noun-verb's -s form is both
        self.assertEqual(relations["running"], {("run", "V")})

    def test_irregular_verbs(self):
        _, relations = self.relations()
        for form in ("was", "were", "been", "am", "is", "are"):
            self.assertEqual(relations[form], {("be", "V")}, form)
        self.assertEqual(relations["lay"], {("lie", "V")})
        self.assertEqual(relations["lain"], {("lie", "V")})
        self.assertEqual(relations["bore"], {("bear", "V")})
        self.assertEqual(relations["borne"], {("bear", "V")})
        self.assertEqual(relations["data"], {("datum", "N")})
        self.assertEqual(relations["leaves"], {("leaf", "N")})  # the annotation mark is not the word

    def test_lesser_variants_are_no_inflections(self):
        _, relations = self.relations()
        for form in ("born", "art", "wast", "sawn", "focussed", "focussing", "focusses", "foci"):
            self.assertNotIn(form, relations, form)
        # A primary or equal spelling in some region is kept, US and UK alike.
        self.assertEqual(relations["focused"], {("focus", "V")})
        self.assertEqual(relations["learned"], {("learn", "V")})
        self.assertEqual(relations["learnt"], {("learn", "V")})

    def test_possessives_are_never_inflections(self):
        pairs, relations = self.relations()
        self.assertFalse(any(form.endswith("'s") for form, _ in pairs))
        self.assertNotIn("bear's", relations)

    def test_a_determiner_only_for_its_comparisons(self):
        _, relations = self.relations()
        self.assertEqual(relations["fewer"], {("few", "A")})
        self.assertEqual(relations["fewest"], {("few", "A")})
        self.assertNotIn("those", relations)  # a word of its own, not a form of "that"

    def test_an_adjective_of_its_own_commoner_than_its_derivation(self):
        pairs, relations = self.relations()
        self.assertNotIn("renowned", relations)  # an adjective at 35, a verb form only at 80
        self.assertNotIn(("renowned", "renown"), pairs)
        self.assertIn(("renowning", "renown"), pairs)
        # At an equal size the derivation stands, as it always has.
        self.assertEqual(relations["tired"], {("tire", "V")})

    def test_sizes_and_groups(self):
        pairs, relations = self.relations()
        self.assertEqual(relations["aahed"], {("aah", "V")})  # "-" is the group's lemma
        self.assertNotIn("abacinations", relations)  # 85: past "a valid word in current usage"
        _, wider = self.relations(max_size=95)
        self.assertIn("abacinations", wider)

    def test_lesser_variant_levels(self):
        self.assertTrue(red.lesser_variant("~"))
        self.assertTrue(red.lesser_variant("@"))
        self.assertTrue(red.lesser_variant("AV Bv"))
        self.assertFalse(red.lesser_variant("A B"))
        self.assertFalse(red.lesser_variant("B Zv"))
        self.assertFalse(red.lesser_variant("A B= Z"))
        self.assertTrue(red.lesser_variant("D"))  # Australian only: a copyright of its own


class AgidLesserVariants(unittest.TestCase):
    def test_a_numbered_variant_is_no_inflection(self):
        # The same rule as ESDB's, for AGID, explicit rather than a side effect of the token pattern.
        self.assertEqual(red.agid_forms("bore | borne, born 1 | bearing | bears"), ["bore", "borne", "bearing", "bears"])
        self.assertEqual(red.agid_forms("was, wast 2 | were | been"), ["was", "were", "been"])
        self.assertEqual(red.agid_forms("littler, less 1, lesser 1.1 | littlest, least 1"), ["littler", "littlest"])


def kaikki_line(word, gloss, target, pos="noun"):
    return json.dumps(
        {"word": word, "pos": pos, "senses": [{"glosses": [gloss], "form_of": [{"word": target}], "tags": ["form-of"]}]}
    )


class FormOf(TempDir):
    def add(self, *lines, pairs=None, relations=None):
        path = os.path.join(self.dir, "kaikki.jsonl")
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines) + "\n")
        pairs = set() if pairs is None else pairs
        relations = {} if relations is None else relations
        added = red.form_of_relations(path, pairs, relations)
        return added, pairs, relations

    def test_recent_plurals_come_from_wiktionary(self):
        added, pairs, relations = self.add(
            kaikki_line("smartphones", "Pluriel de smartphone.", "smartphone"),
            kaikki_line("influencers", "Pluriel de influencer.", "influencer"),
            kaikki_line("datasets", "Pluriel de dataset.", "dataset"),
        )
        self.assertEqual(added, 3)
        self.assertEqual(relations["smartphones"], {("smartphone", "N")})
        self.assertIn(("dataset", "dataset"), pairs)

    def test_only_regular_inflections(self):
        # Without the condition, kaikki's form links made these (measured, 2026-09-25).
        added, _, relations = self.add(
            kaikki_line("occupied", "Participe passé de nanny.", "nanny", "verb"),
            kaikki_line("stocks", "Pluriel de mot.", "mot"),
            kaikki_line("coats", "Pluriel de coast.", "coast"),
            kaikki_line("born", "Participe passé de bear.", "bear", "verb"),
        )
        self.assertEqual(added, 0)
        self.assertEqual(relations, {})

    def test_never_to_a_one_or_two_letter_word(self):
        added, _, relations = self.add(
            kaikki_line("des", "Pluriel de de.", "de"),
            kaikki_line("dis", "Pluriel de di.", "di"),
        )
        self.assertEqual((added, relations), (0, {}))

    def test_verb_and_comparison_links(self):
        _, _, relations = self.add(
            kaikki_line("texting", "Participe présent de to text.", "to text", "verb"),
            kaikki_line("nicer", "Comparatif de nice.", "nice", "adj"),
        )
        self.assertEqual(relations["texting"], {("text", "V")})
        self.assertEqual(relations["nicer"], {("nice", "A")})

    def test_a_link_already_known_adds_nothing(self):
        added, pairs, _ = self.add(
            kaikki_line("apps", "Pluriel de app.", "app"), pairs={("apps", "app")}, relations={"apps": {("app", "N")}}
        )
        self.assertEqual(added, 0)
        self.assertIn(("apps", "app"), pairs)


class OrphanedForms(unittest.TestCase):
    def test_a_form_with_no_kept_lemma_behind_it_is_a_word_of_its_own(self):
        pairs = {
            ("greed", "gree"),  # ESDB: a rare "gree", never kept
            ("grandkids", "grandkid"),
            ("buildings", "building"), ("building", "build"),  # a chain to a kept lemma
            ("runs", "run"),
        }
        inflected = {"greed", "grandkids", "buildings", "building", "runs"}
        ranks = {"build": 10, "run": 20}
        self.assertEqual(red.orphaned_forms(inflected, pairs, ranks), {"greed", "grandkids"})

    def test_nothing_is_orphaned_when_every_base_is_kept(self):
        self.assertEqual(red.orphaned_forms({"runs"}, {("runs", "run")}, {"run": 1}), set())


# — Word grammar (add-lingua-word-grammar) —

PAST = "VERB|Mood=Ind|Tense=Past|VerbForm=Fin"
PARTICIPLE = "VERB|Tense=Past|VerbForm=Part"
ING = "VERB|VerbForm=Ger"
THIRD = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"
PLURAL = "NOUN|Number=Plur"

MODAL_LINES = """\
35: can <v> {be able}: could, -, (can | @: canst)
"""


class EsdbSlots(TempDir):
    def readings(self, text=ESDB_LINES):
        path = self.file("scowl.txt", text)
        readings = {}
        red.parse_esdb_relations(path, readings=readings)
        return readings

    def test_a_verb_with_four_slots(self):
        r = self.readings()
        self.assertEqual(r[("went", "go")], {PAST})
        self.assertEqual(r[("gone", "go")], {PARTICIPLE})
        self.assertEqual(r[("going", "go")], {ING})
        self.assertEqual(r[("goes", "go")], {THIRD})
        self.assertEqual(r[("lay", "lie")], {PAST})
        self.assertEqual(r[("lain", "lie")], {PARTICIPLE})

    def test_a_verb_whose_participle_is_spelled_like_its_past(self):
        r = self.readings()
        self.assertEqual(r[("lied", "lie")], {PAST, PARTICIPLE})
        self.assertEqual(r[("learned", "learn")], {PAST, PARTICIPLE})
        self.assertEqual(r[("learnt", "learn")], {PAST, PARTICIPLE})
        self.assertEqual(r[("lies", "lie")], {THIRD})

    def test_be_has_eight_slots(self):
        r = self.readings()
        self.assertEqual(r[("was", "be")], {PAST})
        self.assertEqual(r[("were", "be")], {PAST})
        self.assertEqual(r[("been", "be")], {PARTICIPLE})
        self.assertEqual(r[("being", "be")], {ING})
        self.assertEqual(r[("am", "be")], {"VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin"})
        self.assertEqual(r[("are", "be")], {"VERB|Mood=Ind|Tense=Pres|VerbForm=Fin"})
        self.assertEqual(r[("is", "be")], {THIRD})

    def test_a_noun_verb_and_a_form_spelled_like_its_lemma(self):
        r = self.readings()
        self.assertEqual(r[("ran", "run")], {PAST})
        self.assertEqual(r[("run", "run")], {PARTICIPLE})  # "have run"
        self.assertEqual(r[("runs", "run")], {THIRD, PLURAL})
        self.assertEqual(r[("tired", "tire")], {PAST, PARTICIPLE})

    def test_nouns_and_comparisons(self):
        r = self.readings()
        self.assertEqual(r[("data", "datum")], {PLURAL})
        self.assertEqual(r[("leaves", "leaf")], {PLURAL})
        self.assertEqual(r[("fewer", "few")], {"DET|Degree=Cmp"})
        self.assertEqual(r[("fewest", "few")], {"DET|Degree=Sup"})
        self.assertEqual(r[("tireder", "tired")], {"ADJ|Degree=Cmp"})
        self.assertNotIn(("those", "that"), r)

    def test_what_is_no_inflection_carries_no_reading(self):
        r = self.readings()
        for form, lemma in [("born", "bear"), ("art", "be"), ("sawn", "saw"), ("renowned", "renown")]:
            self.assertNotIn((form, lemma), r, form)
        self.assertFalse(any(form.endswith("'s") for form, _ in r))
        self.assertFalse(any(lemma == "abacination" for _, lemma in r))  # beyond the kept sizes

    def test_a_modal_past_is_no_participle_and_its_s_slot_no_reading(self):
        r = self.readings(MODAL_LINES)
        self.assertEqual(r[("could", "can")], {PAST})
        self.assertNotIn(("can", "can"), r)

    def test_every_reading_is_a_pair_the_relations_keep(self):
        path = self.file("scowl.txt", ESDB_LINES)
        readings = {}
        pairs, _ = red.parse_esdb_relations(path, readings=readings)
        self.assertTrue(readings)
        self.assertTrue(set(readings) <= pairs)


class RegularTags(unittest.TestCase):
    def test_by_ending(self):
        self.assertEqual(red._regular_tags("smartphones", "N"), (PLURAL,))
        self.assertEqual(red._regular_tags("texting", "V"), (ING,))
        self.assertEqual(red._regular_tags("walked", "V"), (PAST, PARTICIPLE))
        self.assertEqual(red._regular_tags("tried", "V"), (PAST, PARTICIPLE))
        self.assertEqual(red._regular_tags("walks", "V"), (THIRD,))
        self.assertEqual(red._regular_tags("nicer", "A"), ("ADJ|Degree=Cmp",))
        self.assertEqual(red._regular_tags("nicest", "A"), ("ADJ|Degree=Sup",))

    def test_kind_of_a_tag(self):
        self.assertEqual(red._kind_of(PLURAL), "N")
        self.assertEqual(red._kind_of(PAST), "V")
        self.assertEqual(red._kind_of("ADV|Degree=Sup"), "A")


class FormOfReadings(TempDir):
    def test_a_link_adds_the_readings_of_its_ending(self):
        path = self.file(
            "kaikki.jsonl",
            "\n".join(
                [
                    kaikki_line("smartphones", "Pluriel de smartphone.", "smartphone"),
                    kaikki_line("texting", "Participe présent de to text.", "to text", "verb"),
                    kaikki_line("apps", "Pluriel de app.", "app"),
                ]
            )
            + "\n",
        )
        readings = {("apps", "app"): {"FROM-ESDB"}}
        red.form_of_relations(path, {("apps", "app")}, {}, readings)
        self.assertEqual(readings[("smartphones", "smartphone")], {PLURAL})
        self.assertEqual(readings[("texting", "text")], {ING})
        self.assertEqual(readings[("apps", "app")], {"FROM-ESDB"}, "a pair ESDB gave keeps ESDB's slots")

    def test_one_form_linked_twice_takes_both_readings(self):
        path = self.file(
            "kaikki.jsonl",
            json.dumps(
                {
                    "word": "cooks",
                    "pos": "verb",
                    "senses": [
                        {"glosses": ["Pluriel de cook."], "form_of": [{"word": "cook"}]},
                        {"glosses": ["Présent simple de cook."], "form_of": [{"word": "cook"}]},
                    ],
                }
            )
            + "\n",
        )
        readings = {}
        red.form_of_relations(path, set(), {}, readings)
        self.assertEqual(readings[("cooks", "cook")], {PLURAL, THIRD})


class CompoundReadings(unittest.TestCase):
    def test_a_compound_form_is_what_its_part_is(self):
        pairs = {("shirts", "shirt"), ("shirt", "shirt")}
        readings = {("shirts", "shirt"): {PLURAL}}
        red.compound_inflections({"t-shirt"}, pairs, readings)
        self.assertEqual(readings[("t-shirts", "t-shirt")], {PLURAL})


class SensesByPartOfSpeech(unittest.TestCase):
    def test_senses_of_one_part_of_speech_are_grouped_in_first_appearance_order(self):
        joined, runs = red._join_senses_by_pos([["À"], ["Particule"], ["Vers"]], ["ADP", "PART", "ADP"], 80, 3)
        self.assertEqual(joined, "À; Vers; Particule")
        self.assertEqual(runs, [("ADP", 2), ("PART", 1)])

    def test_the_same_senses_are_picked_as_before(self):
        per_entry = [["Course", "Parcours"], ["Courir"]]
        joined, runs = red._join_senses_by_pos(per_entry, ["NOUN", "VERB"], 80, 3)
        self.assertEqual(sorted(joined.split("; ")), sorted(red._join_senses(per_entry, 80, 3).split("; ")))
        self.assertEqual(runs, [("NOUN", 2), ("VERB", 1)])

    def test_a_separator_inside_a_sense_becomes_a_comma(self):
        joined, runs = red._join_senses_by_pos([["Lettre; caractère"]], ["SYM"], 80, 3)
        self.assertEqual(joined, "Lettre, caractère")
        self.assertEqual(runs, [("SYM", 1)])
        # French typography spaces the semicolon; the comma it becomes is not spaced before.
        joined, _ = red._join_senses_by_pos([["Indigène ; qui est d'ici ;"]], ["ADJ"], 80, 3)
        self.assertEqual(joined, "Indigène, qui est d'ici")

    def test_runs_count_the_senses_that_survive_the_cut(self):
        joined, runs = red._join_senses_by_pos([["a" * 30], ["b" * 30], ["c" * 30]], ["NOUN", "VERB", "VERB"], 62, 3)
        self.assertEqual(joined, "a" * 30 + "; " + "b" * 30)
        self.assertEqual(runs, [("NOUN", 1), ("VERB", 1)])
        joined, runs = red._join_senses_by_pos([["a" * 30], ["b" * 30]], ["NOUN", "VERB"], 32, 3)
        self.assertEqual(joined, "a" * 30)
        self.assertEqual(runs, [("NOUN", 1)])

    def test_a_word_gloss_never_ends_mid_word(self):
        # « Être; Être. Auxiliaire pour former le passif ave » was the card of `is`.
        senses = [["Être", "Être. Auxiliaire pour former le passif avec un participe passé"]]
        joined, runs = red._join_senses_by_pos(senses, ["VERB"], 50, 3)
        self.assertEqual(joined, "Être; Être. Auxiliaire pour former le passif avec…")
        self.assertLessEqual(len(joined), 50)
        self.assertEqual(runs, [("VERB", 2)])

    def test_a_sense_left_too_little_room_is_left_out_rather_than_cut(self):
        joined, runs = red._join_senses_by_pos([["a" * 30], ["Courir vite dans les bois"]], ["NOUN", "VERB"], 45, 3)
        self.assertEqual(joined, "a" * 30)
        self.assertEqual(runs, [("NOUN", 1)])

    def test_kaikki_parts_of_speech(self):
        self.assertEqual(red.kaikki_upos("noun", "cat"), "NOUN")
        self.assertEqual(red.kaikki_upos("conj", "and"), "CCONJ")
        self.assertEqual(red.kaikki_upos("conj", "because"), "SCONJ")
        self.assertEqual(red.kaikki_upos("article", "the"), "DET")
        self.assertEqual(red.kaikki_upos("suffix", "-al"), "X")
        self.assertEqual(red.kaikki_upos("", "odd"), "X")


class ReduceGlossRuns(TempDir):
    def test_a_gloss_and_its_runs(self):
        path = self.file(
            "kaikki.jsonl",
            "\n".join(
                [
                    json.dumps({"word": "can", "pos": "noun", "senses": [{"glosses": ["Boîte de conserve."]}]}),
                    json.dumps({"word": "can", "pos": "verb", "senses": [{"glosses": ["Pouvoir."]}, {"glosses": ["Savoir; connaître."]}]}),
                ]
            )
            + "\n",
        )
        runs = {}
        glosses = red.reduce_gloss(path, {"can"}, 80, runs=runs)
        self.assertEqual(glosses["can"], "Boîte de conserve; Pouvoir; Savoir, connaître")
        self.assertEqual(runs["can"], [("NOUN", 1), ("VERB", 2)])

    def test_an_acronym_does_not_gloss_the_common_word(self):
        # The French Wiktionary's "AND", the logic operator, is a noun and a verb: lowercased
        # into "and", it made the card read « verbe Faire le ET de ».
        path = self.file(
            "kaikki.jsonl",
            "\n".join(
                json.dumps(entry)
                for entry in [
                    {"word": "and", "pos": "conj", "senses": [{"glosses": ["Et."]}]},
                    {"word": "AND", "pos": "noun", "senses": [{"glosses": ["ET."], "topics": ["logic"]}]},
                    {"word": "AND", "pos": "verb", "senses": [{"glosses": ["Faire le ET de."]}]},
                    {"word": "WHO", "pos": "name", "senses": [{"glosses": ["OMS, Organisation mondiale de la santé."]}]},
                    {"word": "who", "pos": "pron", "senses": [{"glosses": ["Qui."]}]},
                    {"word": "He", "pos": "pron", "senses": [{"glosses": ["Il (Dieu)."]}]},
                    {"word": "he", "pos": "pron", "senses": [{"glosses": ["Il."]}]},
                    {"word": "NATO", "pos": "name", "senses": [{"glosses": ["OTAN."]}]},
                ]
            )
            + "\n",
        )
        runs = {}
        glosses = red.reduce_gloss(path, {"and", "who", "he", "nato"}, 80, runs=runs)
        self.assertEqual(glosses["and"], "Et")
        self.assertEqual(runs["and"], [("CCONJ", 1)])
        self.assertEqual(glosses["who"], "Qui")
        # A capitalised word is no acronym: it still glosses its lower-case twin.
        self.assertEqual(glosses["he"], "Il (Dieu); Il")
        # An acronym with no common word of its own keeps its gloss.
        self.assertEqual(glosses["nato"], "OTAN")
        self.assertEqual(runs["nato"], [("PROPN", 1)])

    def test_a_word_keeps_long_senses_whole_up_to_its_own_limits(self):
        long_sense = "Être. Auxiliaire pour former le passif avec un participe passé."
        path = self.file(
            "kaikki.jsonl",
            json.dumps({"word": "be", "pos": "verb", "senses": [{"glosses": ["Être."]}, {"glosses": [long_sense]}]}) + "\n",
        )
        self.assertEqual(
            red.reduce_gloss(path, {"be"}, 160),
            {"be": "Être; Être. Auxiliaire pour former le passif avec un participe passé"},
        )
        # Tighter limits cut at a word boundary, with an ellipsis.
        self.assertEqual(
            red.reduce_gloss(path, {"be"}, 160, per_sense=42),
            {"be": "Être; Être. Auxiliaire pour former le passif…"},
        )

    def test_the_wiktionary_notes_to_its_readers_are_left_out(self):
        # Seen in Safari: `there` read « Y avoir. → voir there be », and 164 glosses held the
        # placeholder of an unfinished page.
        placeholder = "Définition manquante ou à compléter. (Ajouter)"
        path = self.file(
            "kaikki.jsonl",
            "\n".join(
                json.dumps(entry)
                for entry in [
                    {"word": "there", "pos": "adv", "senses": [{"glosses": ["Là, là-bas, y."]}]},
                    {"word": "there", "pos": "verb", "senses": [{"glosses": ["Y avoir. → voir there be"]}]},
                    {"word": "because", "pos": "conj", "senses": [{"glosses": ["Parce que."]}]},
                    {"word": "because", "pos": "adv", "senses": [{"glosses": [placeholder + " → voir because of"]}]},
                    {"word": "pig", "pos": "verb", "senses": [{"glosses": ["Vivre dans la saleté. " + placeholder]}]},
                    {"word": "adviser", "pos": "noun", "senses": [{"glosses": ["→ voir advisor"]}]},
                ]
            )
            + "\n",
        )
        runs = {}
        glosses = red.reduce_gloss(path, {"there", "because", "pig", "adviser"}, 160, runs=runs)
        self.assertEqual(glosses["there"], "Là, là-bas, y; Y avoir")
        self.assertEqual(glosses["because"], "Parce que")
        self.assertEqual(runs["because"], [("SCONJ", 1)])
        self.assertEqual(glosses["pig"], "Vivre dans la saleté")
        # Nothing but a pointer: no gloss at all rather than a dead link.
        self.assertNotIn("adviser", glosses)

    def test_a_word_that_is_only_a_form_borrows_its_base_gloss(self):
        # Seen in Safari: the word list keeps `catacombs` as a word of its own, and the Wiktionary
        # only says it is the plural of `catacomb`, so its card had no translation.
        def form_of(word, pos, base, text):
            return {"word": word, "pos": pos, "senses": [{"glosses": [text], "form_of": [{"word": base}]}]}

        path = self.file(
            "kaikki.jsonl",
            "\n".join(
                json.dumps(entry)
                for entry in [
                    form_of("catacombs", "noun", "catacomb", "Pluriel de catacomb."),
                    {"word": "catacomb", "pos": "noun", "senses": [{"glosses": ["Catacombe."]}]},
                    form_of("holden", "verb", "hold", "Participe passé archaïque de hold."),
                    {"word": "hold", "pos": "noun", "senses": [{"glosses": ["Prise."]}]},
                    {"word": "hold", "pos": "verb", "senses": [{"glosses": ["Tenir."]}]},
                    form_of("hearted", "adj", "heart", "Forme de heart."),
                    {"word": "heart", "pos": "noun", "senses": [{"glosses": ["Cœur."]}]},
                    form_of("fs", "noun", "f", "Pluriel de f."),
                    {"word": "f", "pos": "noun", "senses": [{"glosses": ["Sixième lettre."]}]},
                    form_of("bars", "noun", "bar", "Pluriel de bar."),
                    {"word": "bars", "pos": "noun", "senses": [{"glosses": ["Barres parallèles."]}]},
                    {"word": "bar", "pos": "noun", "senses": [{"glosses": ["Bar."]}]},
                ]
            )
            + "\n",
        )
        runs = {}
        glosses = red.reduce_gloss(path, {"catacombs", "holden", "hearted", "fs", "bars"}, 160, runs=runs)
        self.assertEqual(glosses["catacombs"], "Catacombe")
        self.assertEqual(runs["catacombs"], [("NOUN", 1)])
        # In the part of speech of the form: a verb form borrows the verb's senses.
        self.assertEqual(glosses["holden"], "Tenir")
        # A base only in another part of speech lends nothing, and neither does a letter.
        self.assertNotIn("hearted", glosses)
        self.assertNotIn("fs", glosses)
        # A word with a meaning of its own keeps it.
        self.assertEqual(glosses["bars"], "Barres parallèles")

    def test_a_word_keeps_many_short_senses(self):
        senses = [{"glosses": [f"Sens {i}."]} for i in range(1, 11)]
        path = self.file("kaikki.jsonl", json.dumps({"word": "get", "pos": "verb", "senses": senses}) + "\n")
        runs = {}
        glosses = red.reduce_gloss(path, {"get"}, 800, per_sense=300, max_senses=8, runs=runs)
        self.assertEqual(glosses["get"], "; ".join(f"Sens {i}" for i in range(1, 9)))
        self.assertEqual(runs["get"], [("VERB", 8)])

    def test_what_an_acronym_is(self):
        self.assertTrue(red._acronym("AND"))
        self.assertTrue(red._acronym("B2B"))
        self.assertFalse(red._acronym("I"))  # one letter is a word ("I", "A")
        self.assertFalse(red._acronym("He"))
        self.assertFalse(red._acronym("and"))


class CutAtWord(unittest.TestCase):
    def test_a_text_that_fits_is_untouched(self):
        self.assertEqual(red.cut_at_word("Courir", 10), "Courir")

    def test_the_cut_falls_on_a_space_and_ends_with_an_ellipsis(self):
        cut = red.cut_at_word("Avoir. Auxiliaire utilisé pour former l’aspect accompli", 42)
        self.assertEqual(cut, "Avoir. Auxiliaire utilisé pour former…")
        self.assertLessEqual(len(cut), 42)

    def test_no_separator_or_opening_mark_dangles_before_the_ellipsis(self):
        self.assertEqual(red.cut_at_word("Passif, (avec un participe passé)", 12), "Passif…")
        self.assertEqual(red.cut_at_word("Marcher ; courir", 11), "Marcher…")

    def test_one_long_word_is_cut_where_the_room_ends(self):
        self.assertEqual(red.cut_at_word("a" * 50, 20), "a" * 19 + "…")

    def test_notes_go_wherever_they_sit(self):
        for raw, want in (
            ("Banquette → voir seat of a car et car seat.", "Banquette"),
            ("→ voir hand-off ; Raffut, action de repousser de la main.", "Raffut, action de repousser de la main"),
            ("Moelle (→ voir bone marrow).", "Moelle"),
            ("Suffixe. (→ Comparer avec -ative)", "Suffixe"),
            ("Délictuel. → Voir Responsabilité délictuelle", "Délictuel"),
            ("(Définition manquante ou à compléter. (Ajouter)) Désinvestir.", "Désinvestir"),
            ("Chutes Victoria : Définition manquante ou à compléter. (Ajouter)", "Chutes Victoria"),
            ("Désémantisation. Définition manquante ou à co", "Désémantisation"),
            ("Homme élevant des chiens. Étymologie manquante ou incomplète. Si vous la connaissez, vous pouvez l’ajouter en cliquant ici.", "Homme élevant des chiens"),
            ("Définition manquante ou à compléter. (Ajouter)…", ""),
        ):
            self.assertEqual(red.clean_gloss(raw, 300, whole_words=True), want, raw)

    def test_an_arrow_that_is_no_pointer_stays(self):
        self.assertEqual(red.clean_gloss("Le plus. Ex. hard → hardest.", 80), "Le plus. Ex. hard → hardest")

    def test_an_expression_keeps_the_plain_cut(self):
        self.assertEqual(red.clean_gloss("Initiales de Automobile Association", 15), "Initiales de Au")
        self.assertEqual(red.clean_gloss("Initiales de Automobile Association", 15, whole_words=True), "Initiales de…")


class GrammarRows(unittest.TestCase):
    def test_rows_and_which_may_be_named_as_another_word(self):
        readings = {
            ("leaves", "leaf"): {PLURAL},
            ("leaves", "leave"): {THIRD},
            ("put", "put"): {PAST},
            ("uses", "us"): {PLURAL},
            ("swam", "swim"): {PAST},  # swim is not a kept lemma
        }
        rows = red.grammar_rows(
            readings,
            forms={"leaves": "leave", "uses": "use"},
            lemmas={"leaf", "leave", "put", "us", "use"},
            meanings={"leaf": {"noun"}, "leave": {"verb"}, "us": {"pron"}},
            targets={"leaves": {"leaf"}},
            cefr={},
        )
        self.assertEqual(
            rows,
            [
                ("leaves", "leaf", PLURAL, "other"),
                ("leaves", "leave", THIRD, "other"),
                ("put", "put", PAST, "-"),
                ("uses", "us", PLURAL, "-"),
            ],
        )
