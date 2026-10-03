# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the ES->FR reducer's pure rules (no download, no wordfreq).

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import collections
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
