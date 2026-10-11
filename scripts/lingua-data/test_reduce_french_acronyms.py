# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for French's acronym table (keep-lingua-french-acronyms-in-capitals D2, D4): the rows,
how the section must back them, and how a pair glosses them — on entries shaped as the English
Wiktionary's dump writes them, and on the committed tables.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import os
import sys
import unittest

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
import reduce_french_acronyms as acronyms  # noqa: E402

_TABLES = os.path.join(_HERE, "tables")

CA_EXPANSIONS = ("conseil d'administration", "chiffre d'affaires", "courant alternatif", "comptable agréé")


def _committed(path, columns=2):
    """A committed table's rows: first column → the rest (or a set of first columns)."""
    with open(os.path.join(_TABLES, *path.split("/")), encoding="utf-8") as f:
        lines = [line.rstrip("\n") for line in f if line.strip()]
    if columns == 1:
        return set(lines)
    return dict(line.split("\t", 1) for line in lines)


def initialism(of, extra=None, typographic=False):
    """A sense of `CA`'s entry as the dump writes it: « initialism of conseil d'administration »."""
    word = of.replace("'", "’") if typographic else of
    ref = {"word": word, **({"extra": extra} if extra else {})}
    return {"glosses": [f"initialism of {of}"], "tags": ["abbreviation", "alt-of", "initialism", "masculine"], "alt_of": [ref]}


CA_ENTRY = {
    "word": "CA",
    "pos": "noun",
    "senses": [
        initialism("conseil d'administration"),
        initialism("chiffre d'affaires"),
        initialism("courant alternatif", extra="AC or alternating current"),
        initialism("comptable agréé", extra="(“chartered accountant”): CA"),
    ],
    "forms": [{"form": "CAs", "tags": ["plural"]}],
}


class TheTable(unittest.TestCase):
    def test_spec_scenario_a_row_is_a_decision(self):
        # The owner's one row (2026-10-10): `CA`, its four expansions in the entry's order, a noun.
        self.assertEqual(list(acronyms.ACRONYMS), ["CA"])
        expansions, pos, reason = acronyms.ACRONYMS["CA"]
        self.assertEqual(expansions, CA_EXPANSIONS)
        self.assertEqual(pos, "NOUN")
        self.assertIn("2026-10-10", reason)
        # The parts of speech the sense runs use: a row's is one of them.
        runs = {run.split(":")[0] for value in _committed("fr-en/senses.tsv").values() for run in value.split("\t")}
        forms = _committed("fr/forms.tsv")
        for acronym, (expansions, pos, reason) in acronyms.ACRONYMS.items():
            # Wholly in capitals, two letters or more, with its reason.
            self.assertTrue(acronyms.is_acronym(acronym), acronym)
            self.assertTrue(reason.strip(), acronym)
            self.assertIn(pos, runs, acronym)
            self.assertTrue(expansions, acronym)
            # A row is only needed where its lower case is a form of the tables, another word's.
            self.assertIn(acronym.lower(), forms, acronym)
            self.assertNotEqual(forms[acronym.lower()], acronym, acronym)
            # No expansion is written twice, and none is in capitals.
            self.assertEqual(len(set(expansions)), len(expansions), acronym)
            self.assertFalse([e for e in expansions if acronyms.is_acronym(e)], acronym)

    def test_what_is_written_as_an_acronym(self):
        for word in ("CA", "ONU", "ÉTÉ", "C.A", "SNCF"):
            self.assertTrue(acronyms.is_acronym(word), word)
        for word in ("Ca", "ca", "A", "L'", "", "12", "Paris"):
            self.assertFalse(acronyms.is_acronym(word), word)

    def test_each_row_is_its_own_lemma_in_the_forms(self):
        self.assertEqual(acronyms.acronym_forms(), {"CA": "CA"})
        self.assertEqual(acronyms.acronym_forms({"ONU": ((), "NOUN", "r"), "CA": ((), "NOUN", "r")}), {"ONU": "ONU", "CA": "CA"})


class TheSectionBacksEachRow(unittest.TestCase):
    """Every expansion a row names is one the section's own entry for it points at (D2)."""

    def pointers(self, *entries, rows=None):
        found = acronyms.Pointers(rows)
        for entry in entries:
            found.read(entry)
        return found

    def test_the_entry_headed_exactly_as_the_row_is_read(self):
        found = self.pointers(
            CA_ENTRY,
            # `ca` « abbreviation of circa » and `Ca` are other headwords: read for nothing.
            {"word": "ca", "pos": "prep", "senses": [{"glosses": ["abbreviation of circa"], "alt_of": [{"word": "circa"}]}]},
            {"word": "Ca", "pos": "name", "senses": [{"glosses": ["calcium"], "form_of": [{"word": "calcium"}]}]},
        )
        self.assertEqual(found.found, {"CA": set(CA_EXPANSIONS)})
        acronyms.check_rows(found)

    def test_a_pointer_is_read_whole_composed_and_straight(self):
        # The typographic apostrophe read as `'`, a decomposed accent composed, `form_of` too.
        entry = {
            "word": "CA",
            "senses": [
                initialism("conseil d'administration", typographic=True),
                {"glosses": ["x"], "form_of": [{"word": "chiffre d'affaires"}]},
                {"glosses": ["y"], "alt_of": [{"word": "courant alternatif"}, {"word": "comptable agréé"}]},
                {"glosses": ["no pointer"]},
                {"glosses": ["a pointer without a word"], "alt_of": [{"extra": "AC"}, "courant"]},
            ],
        }
        self.assertEqual(acronyms.pointed_at(entry), set(CA_EXPANSIONS))
        acronyms.check_rows(self.pointers(entry))

    def test_spec_scenario_every_expansion_is_one_the_entry_points_at(self):
        # An expansion the entry does not point at: the reduction stops, naming the row and it.
        entry = {**CA_ENTRY, "senses": CA_ENTRY["senses"][:3]}
        with self.assertRaisesRegex(SystemExit, "row CA names comptable agréé, which the section's entry CA does not point at"):
            acronyms.check_rows(self.pointers(entry))
        # No entry at all: it stops too.
        with self.assertRaisesRegex(SystemExit, "row CA has no entry headed CA"):
            acronyms.check_rows(self.pointers({"word": "ca", "senses": []}))
        # A row of another table, checked against its own entry.
        rows = {"ONU": (("organisation des Nations unies",), "PROPN", "a reason")}
        onu = {"word": "ONU", "senses": [initialism("organisation des Nations unies")]}
        acronyms.check_rows(self.pointers(onu, rows=rows), rows)
        with self.assertRaisesRegex(SystemExit, "row ONU has no entry"):
            acronyms.check_rows(self.pointers(CA_ENTRY, rows=rows), rows)


class EachPairGlossesTheRow(unittest.TestCase):
    """A pair glosses a row from its own glosses of the expansions (D4)."""

    FR_EN_EXPRESSIONS = {
        "conseil d'administration": "board of directors",
        "chiffre d'affaires": "revenue, turnover, net sales",
        "courant alternatif": "alternating current; domestic electricity supply",
    }

    def test_spec_scenario_each_pair_glosses_the_acronym_in_its_reader_s_language(self):
        # Each expansion's first sense, in the row's order, `comptable agréé` glossed by neither
        # pair and passed over; one noun run of three senses.
        self.assertEqual(
            acronyms.acronym_glosses({}, self.FR_EN_EXPRESSIONS),
            {"CA": ("board of directors; revenue, turnover, net sales; alternating current", [("NOUN", 3)])},
        )
        fr_es = {
            "conseil d'administration": "Administración, consejo de administración",
            "chiffre d'affaires": "Ingreso, cifra de negocios, facturación",
            "courant alternatif": "Corriente alterna",
        }
        self.assertEqual(
            acronyms.acronym_glosses({}, fr_es),
            {
                "CA": (
                    "Administración, consejo de administración; Ingreso, cifra de negocios, facturación; Corriente alterna",
                    [("NOUN", 3)],
                )
            },
        )

    def test_spec_scenario_no_expansion_glossed(self):
        self.assertEqual(acronyms.acronym_glosses({}, {}), {})
        self.assertIsNone(acronyms.gloss_of(CA_EXPANSIONS, {}, {"comptable": "accountant"}))

    def test_an_expansion_without_a_space_is_a_word_s_gloss_and_the_order_is_the_row_s(self):
        rows = {"PC": (("parti communiste", "ordinateur"), "NOUN", "a reason")}
        words = {"ordinateur": "computer; calculator", "parti": "party"}
        expressions = {"parti communiste": "Communist Party", "ordinateur": "not an expression's"}
        self.assertEqual(
            acronyms.acronym_glosses(words, expressions, rows),
            {"PC": ("Communist Party; computer", [("NOUN", 2)])},
        )
        # One expansion glossed: one sense.
        self.assertEqual(acronyms.gloss_of(("ordinateur", "parti communiste"), words, {}), ("computer", 1))
        # The word a headword's first word is never its gloss: an expression is glossed whole.
        self.assertIsNone(acronyms.gloss_of(("parti communiste",), words, {}))


class TheCommittedTables(unittest.TestCase):
    """The tables the re-reduction wrote (D7): each row its own lemma, a dictionary word with no rank,
    level or reading, glossed by fr-en and fr-es; `ca` still `ça`'s."""

    def test_spec_scenario_the_row_in_french_s_tables(self):
        forms = _committed("fr/forms.tsv")
        self.assertEqual(forms["CA"], "CA")
        self.assertEqual(forms["ca"], "ça")
        # The one form holding a capital.
        self.assertEqual([f for f, l in forms.items() if f != f.lower() or l != l.lower()], ["CA"])
        self.assertIn("CA", _committed("fr/lexical.tsv", columns=1))
        for table in ("fr/freq.tsv", "fr/level.tsv"):
            self.assertNotIn("CA", _committed(table), table)
        with open(os.path.join(_TABLES, "fr", "grammar.tsv"), encoding="utf-8") as f:
            self.assertFalse([line for line in f if "CA" in line.split("\t")[:2]])

    def test_spec_scenario_each_pair_s_gloss_as_committed(self):
        self.assertEqual(
            _committed("fr-en/gloss.tsv")["CA"], "board of directors; revenue, turnover, net sales; alternating current"
        )
        self.assertEqual(_committed("fr-en/senses.tsv")["CA"], "NOUN:3")
        self.assertEqual(
            _committed("fr-es/gloss.tsv")["CA"],
            "Administración, consejo de administración; Ingreso, cifra de negocios, facturación; Corriente alterna",
        )
        self.assertEqual(_committed("fr-es/senses.tsv")["CA"], "NOUN:3")
        # Each the pair's own expressions' first senses: the committed expression tables give them.
        for pair in ("fr-en", "fr-es"):
            expressions = _committed(f"{pair}/mwe.tsv")
            expected = acronyms.acronym_glosses(_committed(f"{pair}/gloss.tsv"), expressions)["CA"][0]
            self.assertEqual(_committed(f"{pair}/gloss.tsv")["CA"], expected, pair)
        # The acronym is the one glossed lemma no rank holds.
        ranks = _committed("fr/freq.tsv")
        for pair in ("fr-en", "fr-es"):
            self.assertEqual([lemma for lemma in _committed(f"{pair}/gloss.tsv") if lemma not in ranks], ["CA"], pair)


if __name__ == "__main__":
    unittest.main()
