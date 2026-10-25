# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for Italian's acronym table (keep-lingua-italian-acronyms-and-names D2, D3): the rows,
how the section must back them, and how a pair glosses them — on entries shaped as the English
Wiktionary's dump writes them, and on the committed tables.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import os
import sys
import unittest
from pathlib import Path

_HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, _HERE)
import pack_sources as ps  # noqa: E402
import reduce_italian_acronyms as acronyms  # noqa: E402

_TABLES = os.path.join(_HERE, "tables")

# The section's entries, as the dump writes them (kaikki-Italian.jsonl of it-fr's pin, 2026.10.23).
NATO_ENTRY = {"word": "NATO", "pos": "name", "senses": [{"glosses": ["NATO"], "tags": ["abbreviation", "acronym"]}]}
USA_ENTRY = {
    "word": "USA",
    "pos": "name",
    "senses": [
        {
            "glosses": ["initialism of the United States of America: a country in North America: USA"],
            "tags": ["abbreviation", "alt-of", "initialism", "plural", "plural-only"],
            "alt_of": [{"word": "the United States of America"}],
        }
    ],
    "synonyms": [{"word": "America"}, {"word": "Stati Uniti"}],
}
PIL_ROW = (("prodotto interno lordo",), (), "NOUN", "a made-up row, for the expansions")
PIL_ENTRY = {
    "word": "PIL",
    "pos": "noun",
    "senses": [
        {
            "glosses": ["initialism of prodotto interno lordo (“GDP”)"],
            "tags": ["abbreviation", "alt-of", "initialism", "masculine"],
            "alt_of": [{"word": "prodotto interno lordo", "extra": "(“GDP”)"}],
        }
    ],
}


def _forms():
    with open(os.path.join(_TABLES, "it", "forms.tsv"), encoding="utf-8") as f:
        return dict(line.rstrip("\n").split("\t") for line in f if line.strip())


def _pointers(*entries, rows=None):
    pointers = acronyms.Pointers(rows)
    for entry in entries:
        pointers.read(entry)
    return pointers


class TheTable(unittest.TestCase):
    def test_spec_scenario_a_row_is_a_decision(self):
        # The design's two rows (D2), the measured candidates left to the owner (task 5.1).
        self.assertEqual(sorted(acronyms.ACRONYMS), ["NATO", "USA"])
        self.assertEqual(acronyms.ACRONYMS["NATO"][:3], ((), (), "PROPN"))
        self.assertEqual(acronyms.ACRONYMS["USA"][:3], ((), ("America", "Stati Uniti"), "PROPN"))
        forms = _forms()
        for acronym, (expansions, synonyms, pos, reason) in acronyms.ACRONYMS.items():
            # Wholly in capitals, three letters or more — the two-letter codes stay out —, with its
            # reason and a part of speech a sense run uses.
            self.assertTrue(acronyms.is_acronym(acronym), acronym)
            self.assertGreaterEqual(len(acronym), 3, acronym)
            self.assertIn(acronym, reason)
            self.assertIn("keep-lingua-italian-acronyms-and-names", reason)
            self.assertIn(pos, {"NOUN", "PROPN"}, acronym)
            # No expansion or synonym written twice, none in capitals, none the entry's English.
            named = expansions + synonyms
            self.assertEqual(len(set(named)), len(named), acronym)
            self.assertFalse([w for w in named if acronyms.is_acronym(w)], acronym)
            self.assertNotIn("the United States of America", named)
            # Its lower case is a form of the committed tables read as another word.
            self.assertIn(acronym.lower(), forms, acronym)
            self.assertNotEqual(forms[acronym.lower()], acronym.lower(), acronym)
        self.assertEqual(forms["nato"], "nascere")
        acronyms.check_lowercase(forms)
        for code in ("MI", "SO", "TI", "LE", "NE", "SE", "SU", "AI"):
            self.assertNotIn(code, acronyms.ACRONYMS)

    def test_what_is_written_as_an_acronym(self):
        for word in ("NATO", "USA", "ÉÈ", "PIL"):
            self.assertTrue(acronyms.is_acronym(word), word)
        for word in ("Nato", "nato", "A", "I", "", "U.S"):
            self.assertEqual(acronyms.is_acronym(word), word == "U.S", word)

    def test_the_rows_as_forms(self):
        self.assertEqual(acronyms.acronym_forms(), {"NATO": "NATO", "USA": "USA"})
        self.assertEqual(acronyms.acronym_forms({"PIL": PIL_ROW}), {"PIL": "PIL"})


class TheSection(unittest.TestCase):
    def test_spec_scenario_each_row_is_backed_by_its_entry(self):
        # `NATO` names nothing and its entry gives nothing: accepted. `USA`'s synonyms are the entry's,
        # its English expansion read and never a row's.
        pointers = _pointers(NATO_ENTRY, USA_ENTRY, {"word": "nato", "pos": "verb", "senses": []})
        self.assertEqual(pointers.found["NATO"], (set(), set()))
        self.assertEqual(pointers.found["USA"], ({"the United States of America"}, {"America", "Stati Uniti"}))
        self.assertNotIn("nato", pointers.found)
        acronyms.check_rows(pointers)

    def test_a_synonym_a_sense_names_and_an_entry_read_twice(self):
        sense_synonym = {
            "word": "USA",
            "pos": "name",
            "senses": [{"glosses": ["USA"], "synonyms": [{"word": "Stati Uniti"}, {"no": "word"}, "junk"]}],
        }
        other = {"word": "USA", "pos": "noun", "synonyms": [{"word": "America"}]}
        pointers = _pointers(sense_synonym, other)
        self.assertEqual(pointers.found["USA"], (set(), {"America", "Stati Uniti"}))

    def test_spec_scenario_a_row_with_no_entry_fails_the_reduction(self):
        rows = {"ANSA": ((), (), "PROPN", "the press agency"), **acronyms.ACRONYMS}
        with self.assertRaises(SystemExit) as raised:
            acronyms.check_rows(_pointers(NATO_ENTRY, USA_ENTRY, rows=rows), rows)
        self.assertIn("row ANSA has no entry headed ANSA", str(raised.exception))

    def test_a_row_naming_what_its_entry_does_not_give_fails_the_reduction(self):
        for row in (
            (("prodotto interno lordo", "prodotto nazionale lordo"), (), "NOUN", "x"),
            (("prodotto interno lordo",), ("PNL",), "NOUN", "x"),
        ):
            with self.assertRaises(SystemExit) as raised:
                acronyms.check_rows(_pointers(PIL_ENTRY, rows={"PIL": row}), {"PIL": row})
            self.assertIn("row PIL names", str(raised.exception))
            self.assertIn("does not give", str(raised.exception))
        acronyms.check_rows(_pointers(PIL_ENTRY, rows={"PIL": PIL_ROW}), {"PIL": PIL_ROW})

    def test_a_lower_case_no_other_word_reads_fails_the_reduction(self):
        # `onu` reads as its own entry, and a lower case no form reads needs no row either.
        acronyms.check_lowercase({"nato": "nascere", "usa": "uso"})
        for forms in ({"nato": "nascere", "usa": "usa"}, {"nato": "nascere"}):
            with self.assertRaises(SystemExit) as raised:
                acronyms.check_lowercase(forms)
            self.assertIn("row(s) USA", str(raised.exception))


class TheGloss(unittest.TestCase):
    GLOSSES = {
        "america": "Amérique; Amériques",
        "stati uniti": "États-Unis",
        "prodotto interno lordo": "produit intérieur brut",
        "nascere": "naître",
    }

    def test_spec_scenario_a_pair_glosses_the_synonyms(self):
        # `USA` names no expansion: its synonyms, each cut to its first sense, in the row's order.
        self.assertEqual(
            acronyms.acronym_gloss(acronyms.ACRONYMS["USA"], self.GLOSSES),
            ("Amérique; États-Unis", [("PROPN", 2)]),
        )
        # A synonym the pair does not gloss is passed over.
        self.assertEqual(
            acronyms.acronym_gloss(acronyms.ACRONYMS["USA"], {"america": "Amérique"}),
            ("Amérique", [("PROPN", 1)]),
        )

    def test_spec_scenario_no_expansion_or_synonym_glossed(self):
        self.assertIsNone(acronyms.acronym_gloss(acronyms.ACRONYMS["NATO"], self.GLOSSES))
        self.assertIsNone(acronyms.acronym_gloss(acronyms.ACRONYMS["USA"], {"nascere": "naître"}))

    def test_an_expansion_bearing_row(self):
        # The expansions first; a headword holding a space read in the expression table when the pair
        # has one; the synonyms only when no expansion is glossed.
        self.assertEqual(acronyms.acronym_gloss(PIL_ROW, self.GLOSSES), ("produit intérieur brut", [("NOUN", 1)]))
        self.assertEqual(
            acronyms.acronym_gloss(PIL_ROW, {}, {"prodotto interno lordo": "PIB; produit intérieur brut"}),
            ("PIB", [("NOUN", 1)]),
        )
        both = (("prodotto interno lordo", "reddito nazionale"), ("America",), "NOUN", "x")
        self.assertEqual(acronyms.acronym_gloss(both, self.GLOSSES), ("produit intérieur brut", [("NOUN", 1)]))
        self.assertEqual(acronyms.acronym_gloss(both, {"america": "Amérique"}), ("Amérique", [("NOUN", 1)]))


class TheRuleDigest(unittest.TestCase):
    def test_spec_scenario_the_module_moves_it_fr_s_rules_alone(self):
        # A rule file of the reductions of the pairs glossing Italian — it-fr's, and it-en's since
        # add-lingua-pack-it-en (D1, D6) — and of no other committed pair's: editing it re-pins them.
        pairs = {pin.parent.name for pin in Path(_TABLES).glob("*/pin.json")} | {"it-en"}
        for pair in sorted(pairs):
            loaded = [p.name for p in ps.rule_files(Path(_HERE) / f"reduce-{pair}.py")]
            italian = pair.startswith("it-")
            self.assertEqual("reduce_italian_acronyms.py" in loaded, italian, pair)
            self.assertNotIn("reduce_french_acronyms.py" if italian else "reduce_italian_acronyms.py", loaded, pair)


if __name__ == "__main__":
    unittest.main()
