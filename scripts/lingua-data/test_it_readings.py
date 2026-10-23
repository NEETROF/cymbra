# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Unit tests for the Italian readings' measurement on held-out treebanks
(measure/it_readings.py, add-lingua-italian-grammar-tables D9), on a hand-written CoNLL-U fixture.

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
_spec = importlib.util.spec_from_file_location("it_readings", os.path.join(_HERE, "measure", "it_readings.py"))
it = importlib.util.module_from_spec(_spec)
_spec.loader.exec_module(it)

FORMS = {
    "parla": "parlare",
    "parlerebbe": "parlare",
    "parlato": "parlare",
    "parlante": "parlante",
    "parlando": "parlare",
    "è": "essere",
    "un": "uno",
    "il": "il",
    "la": "il",
    "lo": "lo",
    "casa": "casa",
    "gatto": "cane",
}
GRAMMAR = [
    ("parla", "parlare", "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin", "-"),
    ("parla", "parlare", "VERB|Mood=Imp|Number=Sing|Person=2|VerbForm=Fin", "-"),
    ("parlerebbe", "parlare", "VERB|Mood=Cnd|Number=Sing|Person=3|VerbForm=Fin", "-"),
    ("parlato", "parlare", "VERB|Gender=Masc|Number=Sing|Tense=Past|VerbForm=Part", "-"),
    ("parlando", "parlare", "VERB|VerbForm=Ger", "-"),
    ("è", "essere", "VERB|Mood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin", "-"),
    # `parlante` reads as its own noun; parlare's participle is another word's reading.
    ("parlante", "parlante", "NOUN|Gender=Masc|Number=Sing", "-"),
    ("parlante", "parlare", "VERB|Number=Sing|Tense=Pres|VerbForm=Part", "other"),
    ("il", "il", "DET|Gender=Masc|Number=Sing", "-"),
    # `un`, an apocope, reads nothing.
    ("casa", "casa", "NOUN|Gender=Fem|Number=Sing", "-"),
]
# UD's columns: ID, FORM, LEMMA, UPOS, XPOS, FEATS, HEAD, DEPREL, DEPS, MISC.
TREEBANK = [
    "# sent_id = 1",
    "1\tLei\tlei\tPRON\t_\tGender=Fem|Number=Sing|Person=3|PronType=Prs\t2\tnsubj\t_\t_",
    "2\tparla\tparlare\tVERB\t_\tMood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t0\troot\t_\t_",
    # UD Italian writes the conditional and the imperative with a tense; the tables write none.
    "3\tparlerebbe\tparlare\tVERB\t_\tMood=Cnd|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t2\tconj\t_\t_",
    "4\tParla\tparlare\tVERB\t_\tMood=Imp|Number=Sing|Person=2|Tense=Pres|VerbForm=Fin\t2\tconj\t_\t_",
    "5\tparlato\tparlare\tVERB\t_\tGender=Masc|Number=Sing|Tense=Past|VerbForm=Part\t2\tconj\t_\t_",
    "6\tparlando\tparlare\tVERB\t_\tVerbForm=Ger\t2\tadvcl\t_\t_",
    # An auxiliary is a verb.
    "7\tè\tessere\tAUX\t_\tMood=Ind|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t2\taux\t_\t_",
    # The subjunctive the readings do not give.
    "8\tparla\tparlare\tVERB\t_\tMood=Sub|Number=Sing|Person=3|Tense=Pres|VerbForm=Fin\t2\tconj\t_\t_",
    # A present participle the tables read as a noun of its own: read, not agreeing.
    "9\tparlante\tparlante\tVERB\t_\tNumber=Sing|Tense=Pres|VerbForm=Part\t2\tconj\t_\t_",
    # An article read; an apocope with no reading.
    "10\til\til\tDET\t_\tDefinite=Def|Gender=Masc|Number=Sing|PronType=Art\t11\tdet\t_\t_",
    "11\tun\tuno\tDET\t_\tDefinite=Ind|Gender=Masc|Number=Sing|PronType=Art\t12\tdet\t_\t_",
    "12\tcasa\tcasa\tNOUN\t_\tGender=Fem|Number=Sing\t2\tobj\t_\t_",
    # A contraction's pieces: the multi-word token is no word of its own.
    "13-14\tdella\t_\t_\t_\t_\t_\t_\t_\t_",
    "13\tdi\tdi\tADP\t_\t_\t15\tcase\t_\t_",
    "14\tla\til\tDET\t_\tDefinite=Def|Gender=Fem|Number=Sing|PronType=Art\t15\tdet\t_\t_",
    "14.1\tx\tx\tX\t_\t_\t_\t_\t_\t_",
    # A word whose lemma the tables do not give (`gatto` → cane): not counted.
    "15\tgatto\tgatto\tNOUN\t_\tGender=Masc|Number=Sing\t2\tobl\t_\t_",
    # A pronoun the tables read nothing of.
    "16\tlo\tlo\tPRON\t_\tClitic=Yes|Gender=Masc|Number=Sing|Person=3|PronType=Prs\t2\tobj\t_\t_",
    "",
]


def tables(test, grammar=True):
    """The two committed tables the measurement reads, in a folder removed after `test`."""
    d = tempfile.mkdtemp()
    test.addCleanup(shutil.rmtree, d)
    with open(os.path.join(d, "forms.tsv"), "w", encoding="utf-8") as f:
        f.write("".join(f"{form}\t{lemma}\n" for form, lemma in FORMS.items()))
    if grammar:
        with open(os.path.join(d, "grammar.tsv"), "w", encoding="utf-8") as f:
            f.write("".join("\t".join(row) + "\n" for row in GRAMMAR))
    return d


class Measuring(unittest.TestCase):
    def setUp(self):
        self.forms, self.own = it.read_tables(tables(self))
        self.counts = it.measure([line + "\n" for line in TREEBANK], self.forms, self.own)

    def figures(self, group):
        return tuple(self.counts[group, what] for what in ("words", "read", "agree"))

    def test_spec_scenario_the_readings_figures_are_reported(self):
        # Five finite verbs, the auxiliary among them: read; the subjunctive alone disagreeing.
        self.assertEqual(self.figures("VERB fin"), (5, 5, 4))
        # The past participle agrees; `parlante`, read as its own noun, is read and does not agree.
        self.assertEqual(self.figures("VERB part"), (2, 2, 1))
        self.assertEqual(self.figures("VERB ger"), (1, 1, 1))
        # `il` agrees; `un` reads nothing; `la` (il's) has no reading in this fixture.
        self.assertEqual(self.figures("DET"), (3, 1, 1))
        # `casa` agrees; `gatto`, whose lemma the tables do not give, is not counted.
        self.assertEqual(self.figures("NOUN"), (1, 1, 1))
        self.assertEqual(self.figures("PRON"), (1, 0, 0))
        self.assertEqual(self.figures("ADP"), (0, 0, 0))

    def test_the_report_names_each_group_the_treebank_holds(self):
        lines = it.report("fixture.conllu", self.counts)
        self.assertEqual(lines[0], "Italian readings on fixture.conllu (reported, not gated):")
        self.assertEqual(
            lines[1:],
            [
                "  finite verbs      5 words  100.00 % read   80.00 % agree",
                "  participles       2 words  100.00 % read   50.00 % agree",
                "  gerunds           1 words  100.00 % read  100.00 % agree",
                "  nouns             1 words  100.00 % read  100.00 % agree",
                "  determiners       3 words   33.33 % read  100.00 % agree",
                "  pronouns          1 words    0.00 % read    0.00 % agree",
            ],
        )

    def test_features_as_the_tables_write_them(self):
        self.assertEqual(
            it.project("AUX", {"Mood": "Imp", "Number": "Plur", "Person": "2", "Tense": "Pres", "VerbForm": "Fin"}),
            ("VERB", {"Mood": "Imp", "Number": "Plur", "Person": "2", "VerbForm": "Fin"}),
        )
        # A present participle's number is the reading's, not a gender; a past one's agreement is.
        self.assertEqual(
            it.project("VERB", {"Gender": "Fem", "Number": "Plur", "Tense": "Pres", "VerbForm": "Part"}),
            ("VERB", {"VerbForm": "Part", "Tense": "Pres", "Number": "Plur"}),
        )
        self.assertEqual(
            it.project("VERB", {"Gender": "Fem", "Number": "Plur", "Tense": "Past", "VerbForm": "Part"}),
            ("VERB", {"VerbForm": "Part", "Tense": "Past", "Number": "Plur", "Gender": "Fem"}),
        )
        self.assertEqual(it.project("VERB", {"VerbForm": "Inf"}), ("VERB", {"VerbForm": "Inf"}))
        self.assertEqual(it.group_of(*it.project("VERB", {"VerbForm": "Ger"})), "VERB ger")
        self.assertEqual(it.group_of(*it.project("VERB", {})), "VERB other")
        self.assertEqual(it.project("DET", {"Definite": "Def", "Number": "Plur"}), ("DET", {"Number": "Plur"}))
        # UD Italian-PUD's way: no VerbForm, the imperfetto an imperfective past, the form by XPOS.
        self.assertEqual(
            it.project("VERB", {"Aspect": "Imp", "Mood": "Ind", "Number": "Sing", "Person": "3", "Tense": "Past", "Voice": "Act"}, "VBC"),
            ("VERB", {"Mood": "Ind", "Tense": "Imp", "Person": "3", "Number": "Sing", "VerbForm": "Fin"}),
        )
        self.assertEqual(
            it.project("VERB", {"Aspect": "Perf", "Mood": "Ind", "Number": "Sing", "Person": "3", "Tense": "Past"}, "VBC")[1]["Tense"],
            "Past",
        )
        self.assertEqual(
            it.project("VERB", {"Mood": "Sub", "Number": "Sing", "Person": "3", "Tense": "Past"}, "VBC")[1]["Tense"], "Imp"
        )
        self.assertEqual(it.project("VERB", {"Tense": "Past"}, "VBN"), ("VERB", {"VerbForm": "Part", "Tense": "Past"}))
        self.assertEqual(it.project("VERB", {"Voice": "Act"}, "VB"), ("VERB", {"VerbForm": "Inf"}))
        self.assertEqual(it.project("VERB", {}, "VBG"), ("VERB", {"VerbForm": "Ger"}))
        # A participle naming no agreement agrees with a reading on its tense.
        self.assertTrue(
            it.agrees({"VerbForm": "Part", "Tense": "Past"}, [{"VerbForm": "Part", "Tense": "Past", "Gender": "Masc", "Number": "Sing"}])
        )
        # A reading without a gender agrees with a treebank's gendered word; a contradicting verb
        # feature does not.
        self.assertTrue(it.agrees({"Gender": "Masc", "Number": "Plur"}, [{"Number": "Plur"}]))
        self.assertFalse(it.agrees({"VerbForm": "Fin", "Mood": "Ind", "Tense": "Pres"}, [{"VerbForm": "Fin", "Mood": "Ind", "Tense": "Past"}]))

    def test_main_reports_every_treebank_and_decides_nothing(self):
        d = tables(self)
        path = os.path.join(d, "t.conllu")
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(TREEBANK))
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            status = it.main(["--tables", d, path, path])
        self.assertEqual(status, 0)
        self.assertEqual(out.getvalue().count("Italian readings on t.conllu"), 2)

    def test_before_the_readings_are_committed(self):
        d = tables(self, grammar=False)
        out = io.StringIO()
        with contextlib.redirect_stdout(out):
            status = it.main(["--tables", d, os.path.join(d, "absent.conllu")])
        self.assertEqual(status, 0)
        self.assertIn("holds no grammar.tsv yet", out.getvalue())


if __name__ == "__main__":
    unittest.main()
