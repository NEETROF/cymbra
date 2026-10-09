# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the French readings' measurement on held-out treebanks
(measure/fr_readings.py, add-lingua-french-grammar-tables D10), on a hand-written CoNLL-U fixture.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import contextlib
import importlib.util
import io
import os
import shutil
import tempfile
import unittest

_HERE = os.path.dirname(os.path.abspath(__file__))
_spec = importlib.util.spec_from_file_location("fr_readings", os.path.join(_HERE, "measure", "fr_readings.py"))
fr = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(fr)

FORMS = {
    "parle": "parler",
    "parlerait": "parler",
    "parlé": "parler",
    "est": "être",
    "un": "un",
    "le": "le",
    "maison": "maison",
    "chat": "chien",
}
GRAMMAR = [
    ("parle", "parler", "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin", "-"),
    ("parlerait", "parler", "VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin", "-"),
    ("parlé", "parler", "VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part", "-"),
    ("est", "être", "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin", "-"),
    # `un`'s readings are its dictionary noun's (« un »), in both numbers.
    ("un", "un", "NOUN|Gender=Masc|Number=Plur", "-"),
    ("un", "un", "NOUN|Gender=Masc|Number=Sing", "-"),
    ("maison", "maison", "NOUN|Gender=Fem|Number=Sing", "-"),
    # A reading of another word is no reading of the form's own: the card names it.
    ("le", "la", "DET|Gender=Fem|Number=Sing", "other"),
]
# UD's columns: ID, FORM, LEMMA, UPOS, XPOS, FEATS, HEAD, DEPREL, DEPS, MISC.
TREEBANK = [
    "# sent_id = 1",
    "1\tIl\til\tPRON\t_\tGender=Masc|Number=Sing|Person=3\t2\tnsubj\t_\t_",
    "2\tparle\tparler\tVERB\t_\tMood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t0\troot\t_\t_",
    # UD French writes the conditional with a tense; the tables write none.
    "3\tparlerait\tparler\tVERB\t_\tMood=Cnd|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t2\tconj\t_\t_",
    # A participle without a tense, as GSD's recent releases write it.
    "4\tparlé\tparler\tVERB\t_\tGender=Masc|Number=Sing|VerbForm=Part\t2\tconj\t_\t_",
    # An auxiliary is a verb.
    "5\tEst\têtre\tAUX\t_\tMood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t2\taux\t_\t_",
    # The subjunctive the readings do not give.
    "6\tparle\tparler\tVERB\t_\tMood=Sub|Number=Sing|Person=1|Tense=Pres|VerbForm=Fin\t2\tconj\t_\t_",
    # A determiner whose form reads only as a noun: read, not agreeing.
    "7\tun\tun\tDET\t_\tDefinite=Ind|Gender=Masc|Number=Sing|PronType=Art\t8\tdet\t_\t_",
    "8\tmaison\tmaison\tNOUN\t_\tGender=Fem|Number=Sing\t2\tobj\t_\t_",
    # A determiner with no reading of its own.
    "9-10\tdu\t_\t_\t_\t_\t_\t_\t_\t_",
    "9\tde\tde\tADP\t_\t_\t11\tcase\t_\t_",
    "10\tle\tle\tDET\t_\tDefinite=Def|Gender=Masc|Number=Sing|PronType=Art\t11\tdet\t_\t_",
    "10.1\tx\tx\tX\t_\t_\t_\t_\t_\t_",
    # A word whose lemma the tables do not give (`chat` → chien): not counted.
    "11\tchat\tchat\tNOUN\t_\tGender=Masc|Number=Sing\t2\tobl\t_\t_",
    "",
]


def tables(test):
    """The two committed tables the measurement reads, in a folder removed after `test`."""
    d = tempfile.mkdtemp()
    test.addCleanup(shutil.rmtree, d)
    with open(os.path.join(d, "forms.tsv"), "w", encoding="utf-8") as f:
        f.write("".join(f"{form}\t{lemma}\n" for form, lemma in FORMS.items()))
    with open(os.path.join(d, "grammar.tsv"), "w", encoding="utf-8") as f:
        f.write("".join("\t".join(row) + "\n" for row in GRAMMAR))
    return d


class Measuring(unittest.TestCase):
    def setUp(self):
        self.forms, self.own = fr.read_tables(tables(self))
        self.counts = fr.measure([line + "\n" for line in TREEBANK], self.forms, self.own)

    def figures(self, group):
        return tuple(self.counts[group, what] for what in ("words", "read", "agree"))

    def test_spec_scenario_the_readings_figures_are_reported(self):
        # Four finite verbs, the auxiliary among them: read, and the subjunctive alone disagreeing.
        self.assertEqual(self.figures("VERB fin"), (4, 4, 3))
        # The participle without a tense is read either way.
        self.assertEqual(self.figures("VERB part"), (1, 1, 1))
        # `un` is read — its noun's readings — and does not agree as a determiner; `le` reads none
        # of its own (its reading is another word's).
        self.assertEqual(self.figures("DET"), (2, 1, 0))
        # `maison` agrees; `chat`, whose lemma the tables do not give, is not counted.
        self.assertEqual(self.figures("NOUN"), (1, 1, 1))
        # `il` is no form of the tables, and the preposition is not measured.
        self.assertEqual(self.figures("PRON"), (0, 0, 0))
        self.assertEqual(self.figures("ADP"), (0, 0, 0))

    def test_the_report_names_each_group_the_treebank_holds(self):
        lines = fr.report("fixture.conllu", self.counts)
        self.assertEqual(lines[0], "French readings on fixture.conllu (reported, not gated):")
        self.assertEqual(
            lines[1:],
            [
                "  finite verbs      4 words  100.00 % read   75.00 % agree",
                "  participles       1 words  100.00 % read  100.00 % agree",
                "  nouns             1 words  100.00 % read  100.00 % agree",
                "  determiners       2 words   50.00 % read    0.00 % agree",
            ],
        )

    def test_features_as_the_tables_write_them(self):
        self.assertEqual(
            fr.project("AUX", {"Mood": "Imp", "Number": "Plur", "Person": "2", "Tense": "Pres", "VerbForm": "Fin"}),
            ("VERB", {"Mood": "Imp", "Number": "Plur", "Person": "2", "VerbForm": "Fin"}),
        )
        # A present participle's agreement is not the reading's; a past one's is.
        self.assertEqual(
            fr.project("VERB", {"Gender": "Fem", "Tense": "Pres", "VerbForm": "Part"}),
            ("VERB", {"VerbForm": "Part", "Tense": "Pres"}),
        )
        self.assertEqual(
            fr.project("VERB", {"Gender": "Fem", "Number": "Plur", "Tense": "Past", "VerbForm": "Part"}),
            ("VERB", {"VerbForm": "Part", "Tense": "Past", "Gender": "Fem", "Number": "Plur"}),
        )
        self.assertEqual(fr.project("VERB", {"VerbForm": "Inf"}), ("VERB", {"VerbForm": "Inf"}))
        self.assertEqual(fr.group_of(*fr.project("VERB", {"VerbForm": "Ger"})), "VERB other")
        self.assertEqual(fr.project("DET", {"Definite": "Def", "Number": "Plur"}), ("DET", {"Number": "Plur"}))
        # A reading without a gender agrees with a treebank's gendered word; a contradicting verb
        # feature does not.
        self.assertTrue(fr.agrees({"Gender": "Masc", "Number": "Plur"}, [{"Number": "Plur"}]))
        self.assertFalse(fr.agrees({"VerbForm": "Fin", "Mood": "Ind", "Tense": "Pres"}, [{"VerbForm": "Fin", "Mood": "Ind", "Tense": "Past"}]))

    def test_main_reports_every_treebank_and_decides_nothing(self):
        d = tables(self)
        path = os.path.join(d, "t.conllu")
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(TREEBANK))
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            status = fr.main(["--tables", d, path, path])
        self.assertEqual(status, 0)
        self.assertEqual(out.getvalue().count("French readings on t.conllu"), 2)


if __name__ == "__main__":
    unittest.main()
