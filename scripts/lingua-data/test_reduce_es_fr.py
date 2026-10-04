# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the ES->FR reducer's pure rules (no download, no wordfreq).

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
import json
import importlib.util
import os
import tempfile
import unittest

_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("reduce_es_fr", os.path.join(_HERE, "reduce-es-fr.py"))
red = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(red)


def entry(word, pos="noun", forms=(), senses=None):
    """A kaikki entry: a word, its inflections `(form, [tags])`, and senses (a meaning by default)."""
    return {
        "word": word,
        "pos": pos,
        "forms": [{"form": f, "tags": list(t)} for f, t in forms],
        "senses": senses if senses is not None else [{"glosses": ["a meaning"]}],
    }


def form_of(word, *targets, pos="verb", tags=("form-of",)):
    """An entry that is only a form of `targets`."""
    return entry(word, pos=pos, senses=[{"tags": list(tags), "form_of": [{"word": t} for t in targets]}])


# The tags kaikki gives a combined form's own sense: the verb's, and the pronoun's.
CLITIC_SENSE = ("form-of", "imperative", "second-person", "object-third-person", "object-singular")


def read(*entries):
    """The entries through `read_entry`, as `read_kaikki` reads them."""
    candidates, lemmas, combined = collections.defaultdict(set), set(), set()
    for e in entries:
        red.read_entry(e, candidates, lemmas, combined)
    return candidates, lemmas, combined


class ReadingKaikki(unittest.TestCase):
    def test_a_lemma_lists_its_inflections_and_is_its_own_candidate(self):
        dar = entry("dar", pos="verb", forms=[("doy", ["first-person"]), ("dio", ["preterite"])])
        candidates, lemmas, _ = read(dar)
        self.assertEqual(candidates["dar"], {"dar"})
        self.assertEqual(candidates["doy"], {"dar"})
        self.assertIn("dar", lemmas)

    def test_combined_clitic_forms_are_left_to_the_analyser(self):
        dar = entry("dar", pos="verb", forms=[("dámelo", ["combined-form", "imperative"])])
        candidates, _, combined = read(dar, form_of("dámelo", "dar", tags=CLITIC_SENSE))
        self.assertNotIn("dámelo", candidates)
        self.assertIn("dámelo", combined)

    def test_a_sense_naming_the_pronouns_is_a_combined_form_even_unlisted(self):
        # No lemma lists `cenala`; its own sense names the pronoun.
        candidates, _, combined = read(form_of("cenala", "cenar", tags=CLITIC_SENSE))
        self.assertNotIn("cenala", candidates)
        self.assertIn("cenala", combined)

    def test_a_combined_form_that_is_also_a_plain_form_keeps_that_lemma(self):
        # `principales` is principar + les, and the plural of principal.
        principar = entry("principar", pos="verb", forms=[("principales", ["combined-form", "imperative"])])
        principal = entry("principal", pos="adj", forms=[("principales", ["plural"])])
        own = entry(
            "principales",
            pos="adj",
            senses=[
                {"tags": ["form-of", "plural"], "form_of": [{"word": "principal"}]},
                {"tags": list(CLITIC_SENSE), "form_of": [{"word": "principar"}]},
            ],
        )
        candidates, lemmas, combined = read(principar, principal, own)
        self.assertEqual(candidates["principales"], {"principal"})
        self.assertIn("principales", combined)
        self.assertNotIn("principales", lemmas)

    def test_a_combined_form_that_is_a_word_of_its_own_stays(self):
        # `vete` combines ve + te, but an entry giving it a meaning keeps it.
        ir = entry("ir", pos="verb", forms=[("vete", ["combined-form"])])
        candidates, lemmas, _ = read(ir, entry("vete", pos="intj"))
        self.assertIn("vete", lemmas)
        self.assertEqual(candidates["vete"], {"vete"})

    def test_the_tables_bookkeeping_is_no_form(self):
        dar = entry("dar", pos="verb", forms=[("irregular", ["table-tags"]), ("es-conj", ["inflection-template"])])
        candidates, _, _ = read(dar)
        self.assertNotIn("irregular", candidates)
        self.assertNotIn("es-conj", candidates)

    def test_a_form_of_points_at_its_lemmas_and_a_multi_word_target_is_refused(self):
        candidates, lemmas, _ = read(
            form_of("luces", "luz", "lucir"),
            form_of("vino", "venir", "llamar al pan, pan, y al vino, vino"),
        )
        self.assertEqual(candidates["luces"], {"luz", "lucir"})
        self.assertEqual(candidates["vino"], {"venir"})
        self.assertNotIn("luces", lemmas)

    def test_words_are_lowercased_composed_and_spanish(self):
        decomposed = "está"  # `está`, decomposed
        candidates, _, _ = read(entry("Casa"), form_of(decomposed, "estar"), entry("New York"), entry("e-mail"))
        self.assertIn("casa", candidates)
        self.assertIn("está", candidates)
        self.assertNotIn("new york", candidates)
        self.assertIn("e-mail", candidates)


class ChoosingOneLemma(unittest.TestCase):
    freq = staticmethod(lambda w: {"ser": 7.0, "ir": 6.9, "casa": 5.8, "casar": 4.0, "luz": 5.0, "lucir": 3.5}.get(w, 0))

    def choose(self, form, options, counts=None, lemmas=frozenset(), overrides=None):
        return red.choose_lemma(form, set(options), counts or {}, set(lemmas), self.freq, overrides or {})

    def test_the_treebank_decides_first(self):
        self.assertEqual(self.choose("fue", ["ser", "ir"], {("fue", "ser"): 1458}), "ser")
        self.assertEqual(self.choose("fue", ["ser", "ir"], {("fue", "ir"): 3}), "ir")

    def test_an_override_comes_before_the_counts(self):
        overrides = {"vino": ("venir", "a reason")}
        got = self.choose("vino", ["vino", "venir"], {("vino", "vino"): 20}, {"vino"}, overrides)
        self.assertEqual(got, "venir")
        # An override naming a lemma the form cannot have is no decision.
        self.assertEqual(self.choose("casa", ["casa"], overrides={"casa": ("x", "?")}), "casa")

    def test_without_counts_the_forms_own_entry_wins(self):
        self.assertEqual(self.choose("casa", ["casa", "casar"], lemmas={"casa"}), "casa")

    def test_then_the_commoner_lemma_then_the_alphabet(self):
        self.assertEqual(self.choose("luces", ["luz", "lucir"]), "luz")
        self.assertEqual(self.choose("xx", ["b", "a"]), "a")

    def test_every_override_has_its_reason(self):
        for form, (lemma, reason) in red.OVERRIDES.items():
            self.assertTrue(reason and len(reason) > 20, form)


class ReducingForms(unittest.TestCase):
    def test_forms_of_kept_lemmas_attested_and_ranks(self):
        candidates = collections.defaultdict(set)
        candidates.update(
            {
                "ser": {"ser"},
                "fue": {"ser", "ir"},
                "era": {"era", "ser"},
                "ir": {"ir"},
                "casa": {"casa", "casar"},
                "casar": {"casar"},
                "fuéramos": {"ser", "ir"},
                "rarísimo": {"raro"},
            }
        )
        lemmas = {"ser", "era", "ir", "casa", "casar", "vete"}
        combined = {"hacerlo", "vete"}
        counts = {("fue", "ser"): 1458, ("era", "ser"): 498, ("fuéramos", "ser"): 2}

        def ranks_for(inflected, want):
            # `era` reads as ser by the counts: only inflected, never a lemma.
            self.assertIn("era", inflected)
            self.assertNotIn("casa", inflected)
            # A combined form is the enclitic rule's, unless it is a word of its own.
            self.assertIn("hacerlo", inflected)
            self.assertNotIn("vete", inflected)
            kept = [w for w in ["ser", "hacerlo", "ir", "casa", "casar", "era"] if w not in inflected]
            return {w: i + 1 for i, w in enumerate(kept[:want])}

        attested = lambda f: f != "fuéramos"  # noqa: E731 — no one writes it, in this test
        forms, ranks = red.reduce_forms(candidates, lemmas, combined, counts, lambda w: 0, ranks_for, 10, attested)
        self.assertEqual(ranks, {"ser": 1, "ir": 2, "casa": 3, "casar": 4})
        self.assertNotIn("hacerlo", forms)
        self.assertEqual(forms["fue"], "ser")
        self.assertEqual(forms["era"], "ser")
        self.assertEqual(forms["casa"], "casa")
        self.assertNotIn("fuéramos", forms)  # unattested
        self.assertNotIn("rarísimo", forms)  # its lemma is not kept
        for lemma in ranks:
            self.assertEqual(forms[lemma], lemma)


class Treebank(unittest.TestCase):
    def test_counts_skip_multi_word_tokens_and_empty_nodes(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "t.conllu")
            with open(path, "w", encoding="utf-8") as f:
                f.write(
                    "# sent_id = 1\n"
                    "1\tFue\tser\tAUX\t_\t_\t0\troot\t_\t_\n"
                    "2-3\tdel\t_\t_\t_\t_\t_\t_\t_\t_\n"
                    "2\tde\tde\tADP\t_\t_\t1\tcase\t_\t_\n"
                    "3\tel\tel\tDET\t_\t_\t1\tdet\t_\t_\n"
                    "3.1\tx\tx\tX\t_\t_\t_\t_\t_\t_\n"
                )
            counts = red.read_gsd_counts([path])
        self.assertEqual(counts[("fue", "ser")], 1)
        self.assertEqual(counts[("de", "de")], 1)
        self.assertNotIn(("del", "_"), counts)
        self.assertNotIn(("x", "x"), counts)


PAST = "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Past|VerbForm=Fin"


def readings(*entries):
    """The entries' readings, by (form, lemma), as the reducer collects them."""
    collected = red.Readings()
    for e in entries:
        red.read_readings(e, collected)
    return collected


class VerbReadings(unittest.TestCase):
    def test_verb_forms_read_as_ud_spanish_writes_them(self):
        cases = {
            ("first-person", "indicative", "present", "singular"): "VERB|Mood=Ind|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin",
            ("first-person", "imperfect", "indicative", "plural"): "VERB|Mood=Ind|Number=Plur|Person=1|Tense=Imp|VerbForm=Fin",
            ("indicative", "preterite", "singular", "third-person"): PAST,
            ("imperfect", "imperfect-se", "singular", "subjunctive", "third-person"): "VERB|Mood=Sub|Number=Sing|Person=3|Tense=Imp|VerbForm=Fin",
            ("future", "plural", "subjunctive", "third-person"): "VERB|Mood=Sub|Number=Plur|Person=3|Tense=Fut|VerbForm=Fin",
            # kaikki tags the conditional indicative too; UD Spanish makes it a mood.
            ("conditional", "first-person", "indicative", "singular"): "VERB|Mood=Cnd|Number=Sing|Person=1|VerbForm=Fin",
            ("imperative", "plural", "second-person"): "VERB|Mood=Imp|Number=Plur|Person=2|VerbForm=Fin",
            # usted: a third person, whatever it means.
            ("formal", "imperative", "second-person-semantically", "singular", "third-person"): "VERB|Mood=Imp|Number=Sing|Person=3|VerbForm=Fin",
            ("indicative", "informal", "present", "second-person", "singular", "vos-form"): "VERB|Mood=Ind|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin",
            ("infinitive",): "VERB|VerbForm=Inf",
            ("gerund",): "VERB|VerbForm=Ger",
            ("feminine", "participle", "past", "singular"): "VERB|Gender=Fem|Number=Sing|Tense=Past|VerbForm=Part",
        }
        for tags, want in cases.items():
            self.assertEqual(red.ud_tag("VERB", red.verb_features(set(tags))), want, tags)

    def test_rows_that_are_no_reading_of_their_own(self):
        # The negative imperative is the present subjunctive, listed as such.
        self.assertIsNone(red.verb_features({"imperative", "negative", "second-person", "singular"}))
        # The bare participle repeats the masculine singular.
        self.assertIsNone(red.verb_features({"participle", "past"}))
        # No tense, or no person: nothing to say.
        self.assertIsNone(red.verb_features({"subjunctive", "singular", "first-person"}))
        self.assertIsNone(red.verb_features({"indicative", "present", "singular"}))


class NominalReadings(unittest.TestCase):
    def test_a_form_takes_its_own_gender_or_the_lemmas(self):
        self.assertEqual(red.nominal_features({"feminine"}), [{"Number": "Sing", "Gender": "Fem"}])
        self.assertEqual(red.nominal_features({"feminine", "masculine", "plural"}), [{"Number": "Plur"}])
        self.assertEqual(red.nominal_features({"plural"}, ("Fem",)), [{"Number": "Plur", "Gender": "Fem"}])
        self.assertEqual(len(red.nominal_features({"plural"}, ("Masc", "Fem"))), 2)
        self.assertEqual(red.nominal_features({"superlative"}), [{"Number": "Sing", "Degree": "Sup"}])

    def test_noun_genders_from_the_head_template_else_the_senses(self):
        def head(arg):
            return {"head_templates": [{"name": "es-noun", "args": {"1": arg}}]}

        self.assertEqual(red.noun_genders(head("f")), (("Fem",), False))
        self.assertEqual(red.noun_genders(head("f-p")), (("Fem",), True))
        self.assertEqual(red.noun_genders(head("mfbysense")), (("Masc", "Fem"), False))
        self.assertEqual(red.noun_genders({"senses": [{"tags": ["masculine"]}]}), (("Masc",), False))
        self.assertEqual(red.noun_genders({"senses": [{"glosses": ["x"]}]}), ((), False))

    def test_a_noun_reads_its_gender_on_its_own_form_and_on_its_plural(self):
        casa = entry("casa", forms=[("casas", ["plural"])])
        casa["head_templates"] = [{"name": "es-noun", "args": {"1": "f"}}]
        gafas = entry("gafas")
        gafas["head_templates"] = [{"name": "es-noun", "args": {"1": "f-p"}}]
        got = readings(casa, gafas, form_of("casas", "casa", pos="noun", tags=("form-of", "plural"))).pairs()
        self.assertEqual(got[("casa", "casa")], {"NOUN|Gender=Fem|Number=Sing"})
        # The table's reading wins over the form entry's, which knows no gender.
        self.assertEqual(got[("casas", "casa")], {"NOUN|Gender=Fem|Number=Plur"})
        self.assertEqual(got[("gafas", "gafas")], {"NOUN|Gender=Fem|Number=Plur"})

    def test_an_adjective_agrees_or_takes_one_form_for_both_genders(self):
        rapido = entry("rápido", pos="adj", forms=[("rápida", ["feminine"]), ("rápidos", ["masculine", "plural"])])
        grande = entry("grande", pos="adj", forms=[("grandes", ["feminine", "masculine", "plural"])])
        got = readings(rapido, grande).pairs()
        self.assertEqual(got[("rápido", "rápido")], {"ADJ|Gender=Masc|Number=Sing"})
        self.assertEqual(got[("rápida", "rápido")], {"ADJ|Gender=Fem|Number=Sing"})
        self.assertEqual(got[("rápidos", "rápido")], {"ADJ|Gender=Masc|Number=Plur"})
        self.assertEqual(got[("grande", "grande")], {"ADJ|Number=Sing"})
        self.assertEqual(got[("grandes", "grande")], {"ADJ|Number=Plur"})


class LetterNames(unittest.TestCase):
    def test_a_letter_s_name_gives_no_reading_of_its_inflections(self):
        def letter(word, plural, sense):
            noun = entry(word, forms=[(plural, ["plural"])], senses=[sense])
            noun["head_templates"] = [{"name": "es-noun", "args": {"1": "f"}}]
            return noun

        named = {"tags": ["feminine"], "glosses": ["The name of the Latin script letter E/e."]}
        categorised = {"glosses": ["letter D"], "categories": [{"name": "Greek letter names", "kind": "other"}]}
        tagged = {"tags": ["alt-of", "letter", "name"], "glosses": ["Name of the letter A."]}
        got = readings(
            letter("e", "es", named),
            letter("de", "des", categorised),
            letter("a", "aes", tagged),
            entry("ser", pos="verb", forms=[("es", ["indicative", "present", "singular", "third-person"])]),
            # The plural's own entry, which says nothing of a letter.
            form_of("es", "e", pos="noun", tags=("feminine", "form-of", "plural")),
            # An abbreviation, `E` for east, is no noun `e` of its own.
            entry("E", senses=[{"tags": ["abbreviation", "alt-of", "masculine"], "glosses": ["abbreviation of este; east"]}]),
        ).pairs()
        self.assertEqual(got[("es", "ser")], {"VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin"})
        self.assertEqual({form for form, _ in got}, {"es", "e", "de", "a"})
        # The letter's own form stays, for its gender: the card names no dictionary form (D4).
        self.assertEqual(got[("de", "de")], {"NOUN|Gender=Fem|Number=Sing"})

    def test_a_noun_that_is_also_something_else_keeps_its_readings(self):
        jota = entry(
            "jota",
            forms=[("jotas", ["plural"])],
            senses=[
                {"glosses": ["The name of the Latin script letter J/j."]},
                {"glosses": ["jota (Iberian folk dance)"]},
            ],
        )
        jota["head_templates"] = [{"name": "es-noun", "args": {"1": "f"}}]
        # `be`, the letter B, and another entry, a sheep's bleat: the bleat keeps its plural.
        be_letter = entry("be", senses=[{"tags": ["feminine"], "glosses": ["The name of the Latin script letter B/b."]}])
        be_bleat = entry("be", forms=[("bes", ["plural"])], senses=[{"tags": ["masculine"], "glosses": ["baa"]}])
        got = readings(
            jota, be_letter, be_bleat, form_of("bes", "be", pos="noun", tags=("form-of", "plural"))
        ).pairs()
        self.assertEqual(got[("jotas", "jota")], {"NOUN|Gender=Fem|Number=Plur"})
        self.assertEqual(got[("be", "be")], {"NOUN|Gender=Fem|Number=Sing", "NOUN|Gender=Masc|Number=Sing"})
        self.assertEqual(got[("bes", "be")], {"NOUN|Gender=Masc|Number=Plur"})


class GrammarRows(unittest.TestCase):
    def test_a_pronominal_form_reads_from_its_own_entry_and_a_combined_form_not_at_all(self):
        dar = entry("dar", pos="verb", forms=[("dámelo", ["combined-form", "imperative"]), ("es-conj", ["inflection-template"])])
        got = readings(
            form_of("azotarse", "azotar", tags=("form-of", "infinitive", "reflexive")),
            form_of("dámelo", "dar", tags=CLITIC_SENSE),
            dar,
        ).pairs()
        self.assertEqual(got[("azotarse", "azotar")], {"VERB|VerbForm=Inf"})
        self.assertNotIn(("dámelo", "dar"), got)
        self.assertNotIn(("es-conj", "dar"), got)

    def test_rows_hold_the_tables_forms_and_mark_another_lemma(self):
        third = ["indicative", "preterite", "singular", "third-person"]
        ser = entry("ser", pos="verb", forms=[("fue", third)])
        ir = entry("ir", pos="verb", forms=[("fue", third), ("fuéramos", ["first-person", "imperfect", "plural", "subjunctive"])])
        bajar = entry("bajar", pos="verb", forms=[("bajo", ["first-person", "indicative", "present", "singular"])])
        forms = {"fue": "ser", "ser": "ser", "ir": "ir", "bajo": "bajo"}
        rows = red.grammar_rows(readings(ser, ir, bajar), forms, {"ser": 1, "ir": 2, "bajo": 3})
        # `fuéramos` is not in the table; `bajar` is not kept.
        self.assertEqual(rows, [f"fue\tir\t{PAST}\tother\n", f"fue\tser\t{PAST}\t-\n"])


def translation_file(*entries):
    """A translation file as `pack_sources.derive` writes one, in a temporary folder."""
    d = tempfile.mkdtemp()
    path = os.path.join(d, "t.jsonl")
    with open(path, "w", encoding="utf-8") as f:
        for word, pos, words in entries:
            f.write(json.dumps({"word": word, "pos": pos, "translations": [{"word": w} for w in words]}) + "\n")
    return path


class GlossFallbacks(unittest.TestCase):
    def test_the_spanish_wiktionary_lists_french_words_for_a_spanish_entry(self):
        direct = red.read_translated(
            translation_file(
                ("Sector", "noun", ["secteur"]),
                ("material", "noun", ["matériau", "matériel", "matériau"]),
                ("material", "adj", ["matériel"]),
                ("Alcalá de la Vega", "name", ["Alcalá de la Vega"]),
                ("hello", "intj", []),
            ),
            inverted=False,
        )
        self.assertEqual(direct["sector"], {"NOUN": ["secteur"]})
        self.assertEqual(direct["material"], {"NOUN": ["matériau", "matériel"], "ADJ": ["matériel"]})
        self.assertNotIn("alcalá de la vega", direct)  # a proper noun glosses nothing

    def test_the_french_wiktionary_s_tables_read_backwards(self):
        inverted = red.read_translated(
            translation_file(
                ("arrêté", "noun", ["decreto"]),
                ("décret", "noun", ["decreto"]),
                ("prendre en compte", "verb", ["tener en cuenta", "ça compte"]),
                ("Céline", "name", ["Celina"]),
            ),
            inverted=True,
        )
        self.assertEqual(inverted["decreto"], {"NOUN": ["arrêté", "décret"]})
        self.assertEqual(inverted["tener en cuenta"], {"VERB": ["prendre en compte"]})
        self.assertNotIn("ça compte", inverted)  # not Spanish words
        self.assertNotIn("celina", inverted)

    def test_a_translation_gloss_is_one_sense_per_part_of_speech(self):
        gloss, runs = red.translation_gloss({"NOUN": ["matériau", "matériel"], "ADJ": ["matériel"]}, list)
        self.assertEqual(gloss, "Matériau, matériel; Matériel")
        self.assertEqual(runs, [("NOUN", 1), ("ADJ", 1)])
        many, _ = red.translation_gloss({"ADV": ["a", "b", "c", "d"]}, list)
        self.assertEqual(many, "A, b, c")  # FALLBACK_WORDS

    def test_the_commonest_french_word_comes_first_from_a_table_read_backwards(self):
        order = red.by_french_frequency(lambda w: {"juste": 5.5, "précisément": 4.4}.get(w, 0))
        self.assertEqual(order(["précisément", "juste", "zzz"])[:2], ["juste", "précisément"])
        order = red.by_french_frequency(lambda w: 0)
        self.assertEqual(order(["b", "a"]), ["a", "b"])

    def test_fallbacks_gloss_what_the_french_wiktionary_leaves_out_source_by_source(self):
        direct = {"sector": {"NOUN": ["secteur"]}}
        inverted = {"sector": {"NOUN": ["domaine"]}, "decreto": {"NOUN": ["arrêté"]}, "casa": {"NOUN": ["foyer"]}}
        got = red.fallback_glosses(["sector", "decreto", "casa", "zzz"], {"casa"}, [(direct, list), (inverted, list)])
        self.assertEqual(got, {"sector": ("Secteur", [("NOUN", 1)]), "decreto": ("Arrêté", [("NOUN", 1)])})

    def test_fallback_expressions_add_only_the_multi_word_headwords_none_glosses(self):
        direct = {"tener en cuenta": {"VERB": ["prendre en compte", "tenir compte de"]}, "sector": {"NOUN": ["secteur"]}}
        inverted = {"dar cuenta": {"VERB": ["rendre compte"]}, "sin embargo": {"ADV": ["cependant"]}}
        got = red.fallback_expressions({"sin embargo": "Cependant"}, [(direct, list), (inverted, list)])
        self.assertEqual(got, {"tener en cuenta": "Prendre en compte, tenir compte de", "dar cuenta": "Rendre compte"})

    def test_the_curated_locutions_are_written_by_a_person_each_with_its_gloss(self):
        for expression, gloss in red.LOCUTIONS.items():
            self.assertIn(" ", expression)
            self.assertTrue(gloss and gloss[0].isupper(), expression)


class NounClassRuns(unittest.TestCase):
    def test_a_noun_run_takes_the_gender_of_the_noun_s_own_readings(self):
        casa = entry("casa", forms=[("casas", ["plural"])])
        casa["head_templates"] = [{"name": "es-noun", "args": {"1": "f"}}]
        estudiante = entry("estudiante")
        estudiante["head_templates"] = [{"name": "es-noun", "args": {"1": "mfbysense"}}]
        runs = {
            "casa": [("NOUN", 1), ("VERB", 2)],
            "estudiante": [("NOUN", 1)],
            "hablar": [("VERB", 1)],
        }
        got = red.noun_class_runs(runs, readings(casa, estudiante))
        self.assertEqual(got["casa"], [("NOUN|Gender=Fem", 1), ("VERB", 2)])
        # Of both genders: no gender in the run.
        self.assertEqual(got["estudiante"], [("NOUN", 1)])
        self.assertEqual(got["hablar"], [("VERB", 1)])


class EstimatedLevels(unittest.TestCase):
    def test_bands_go_in_rank_order_to_the_lemmas_a_cefr_list_would_hold(self):
        ranks = {"de": 1, "the": 2, "madrid": 3, "casa": 4, "rosa": 5, "perro": 6, "gato": 7}
        glosses = {"de", "madrid", "casa", "rosa", "perro", "gato"}
        runs = {
            "de": [("ADP", 1)],
            "madrid": [("PROPN", 1)],  # only a place
            "casa": [("NOUN", 1)],
            "rosa": [("NOUN", 1), ("PROPN", 1)],  # a flower too
            "perro": [("NOUN", 1)],
            "gato": [("NOUN", 1)],
        }
        got = red.estimated_levels(ranks, glosses, runs, bands=(("A1", 2), ("A2", 2), ("B1", 5)))
        # `the` has no gloss and `madrid` only a proper noun's: neither takes a level.
        self.assertEqual(got, {"de": "A1", "casa": "A1", "rosa": "A2", "perro": "A2", "gato": "B1"})

    def test_english_bands_hold_its_8302_cefr_lemmas(self):
        self.assertEqual([level for level, _ in red.ENGLISH_BANDS], ["A1", "A2", "B1", "B2", "C1", "C2"])
        self.assertEqual(sum(size for _, size in red.ENGLISH_BANDS), 8302)


class Manifest(unittest.TestCase):
    def test_the_spanish_analyser_version_is_read_from_the_core(self):
        with tempfile.TemporaryDirectory() as d:
            path = os.path.join(d, "mod.rs")
            with open(path, "w", encoding="utf-8") as f:
                f.write('pub const ANALYZER_VERSION: &str = "1.1.0";\npub const SPANISH_ANALYZER_VERSION: &str = "9.9.9";\n')
            self.assertEqual(red.analyser_version(path), "9.9.9")
        # And the real core names one.
        self.assertRegex(red.analyser_version(), r"^\d+\.\d+\.\d+$")

    def test_the_notice_names_every_source_the_manifest_declares(self):
        for name in ["kaikki", "wordfreq", "UD Spanish-GSD"]:
            self.assertIn(name, red.NOTICE)


if __name__ == "__main__":
    unittest.main()
