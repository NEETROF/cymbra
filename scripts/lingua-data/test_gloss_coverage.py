#!/usr/bin/env python3
# Copyright 2026 NEETROF
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""The coverage figures the site publishes (add-site-lingua-spanish-pages D3)."""

from __future__ import annotations

import contextlib
import io
import json
import sys
import tempfile
import unittest
from pathlib import Path
from unittest import mock

sys.path.insert(0, str(Path(__file__).resolve().parent))

import gloss_coverage as coverage  # noqa: E402


def write_table(path: Path, rows: list[tuple[str, str]]) -> None:
    path.write_text("".join(f"{key}\t{value}\n" for key, value in rows), encoding="utf-8")


class MeasureTest(unittest.TestCase):
    def test_the_share_of_the_commonest_lemmas_with_a_gloss(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tables = Path(tmp)
            # Ranked out of file order: the measure sorts by rank, not by line.
            write_table(tables / "freq.tsv", [("c", "3"), ("a", "1"), ("b", "2"), ("d", "4")])
            write_table(tables / "gloss.tsv", [("a", "A"), ("c", "C"), ("d", "D")])
            (tables / "gloss.tsv").write_text("# comment\n\n" + (tables / "gloss.tsv").read_text(encoding="utf-8"))
            self.assertEqual(coverage.measure(tables, (2, 3, 4)), [50.0, 66.7, 75.0])

    def test_a_pair_s_ranks_come_from_its_studied_language_s_folder(self) -> None:
        # split-lingua-pack-tables-by-language: freq.tsv in tables/<studied>/, gloss.tsv in
        # tables/<pair>/ (spec: *The published coverage follows the studied tables*).
        with tempfile.TemporaryDirectory() as tmp:
            root = Path(tmp)
            (root / "tables" / "es").mkdir(parents=True)
            (root / "tables" / "es-fr").mkdir()
            write_table(root / "tables" / "es" / "freq.tsv", [("a", "1"), ("b", "2"), ("c", "3")])
            write_table(root / "tables" / "es-fr" / "gloss.tsv", [("a", "A"), ("c", "C")])
            packs = root / "packs.json"
            packs.write_text(json.dumps({"pairs": ["es-fr"]}), encoding="utf-8")
            before = coverage.figures(root / "tables", packs)
            self.assertEqual(before["glossed"]["es-fr"], [66.7, 66.7, 66.7])
            # Spanish's ranks change: the figures move, so the published ones are stale.
            write_table(root / "tables" / "es" / "freq.tsv", [("a", "1"), ("c", "2"), ("b", "3"), ("d", "4")])
            self.assertNotEqual(coverage.figures(root / "tables", packs), before)
            self.assertEqual(coverage.figures(root / "tables", packs)["glossed"]["es-fr"], [50.0, 50.0, 50.0])


class OnePairTest(unittest.TestCase):
    """`--pair` and `--floor` (add-lingua-pack-es-en D6): a pair measured whether or not it ships,
    held to a floor, and published nowhere."""

    def run_main(self, tables: Path, *argv: str) -> tuple[int, str, str]:
        out, err = io.StringIO(), io.StringIO()
        with mock.patch.object(coverage, "TABLES", tables), contextlib.redirect_stdout(out), contextlib.redirect_stderr(err):
            code = coverage.main(list(argv))
        return code, out.getvalue(), err.getvalue()

    def test_a_pair_not_shipped_is_measured_and_held_to_its_floor(self) -> None:
        # A pair with no floor of its own (made up): held to the one given alone.
        with tempfile.TemporaryDirectory() as tmp:
            tables = Path(tmp)
            (tables / "it").mkdir()
            (tables / "it-en").mkdir()
            write_table(tables / "it" / "freq.tsv", [("a", "1"), ("b", "2"), ("c", "3")])
            write_table(tables / "it-en" / "gloss.tsv", [("a", "A"), ("c", "C")])
            code, out, err = self.run_main(tables, "--pair", "it-en")
            self.assertEqual((code, err), (0, ""))
            self.assertEqual(json.loads(out), {"tops": [5000, 10000, 20000], "glossed": {"it-en": [66.7, 66.7, 66.7]}})
            self.assertEqual(self.run_main(tables, "--pair", "it-en", "--floor", "66.7", "60", "50")[0], 0)
            code, _, err = self.run_main(tables, "--pair", "it-en", "--floor", "66.7", "70", "50")
            self.assertEqual(code, 1)
            self.assertEqual(
                err, "it-en: 66.7 % of the 10,000 commonest lemmas are glossed, under the floor of 70.0 %\n"
            )
            with self.assertRaises(SystemExit):
                self.run_main(tables, "--pair", "de-en")
        with self.assertRaises(SystemExit), contextlib.redirect_stderr(io.StringIO()):
            coverage.main(["--floor", "87.6", "77.2", "63.7"])

    def test_a_pair_s_floor_is_its_own_unless_one_is_given(self) -> None:
        with tempfile.TemporaryDirectory() as tmp:
            tables = Path(tmp)
            (tables / "es").mkdir()
            (tables / "es-en").mkdir()
            write_table(tables / "es" / "freq.tsv", [("a", "1"), ("b", "2"), ("c", "3")])
            write_table(tables / "es-en" / "gloss.tsv", [("a", "A"), ("c", "C")])
            # 66.7 % everywhere: under es-en's floor, which applies with no --floor.
            code, _, err = self.run_main(tables, "--pair", "es-en")
            self.assertEqual(code, 1)
            self.assertEqual(
                err.splitlines()[0], "es-en: 66.7 % of the 5,000 commonest lemmas are glossed, under the floor of 87.6 %"
            )
            # A floor given replaces it; a pair with no floor is measured and held to none.
            self.assertEqual(self.run_main(tables, "--pair", "es-en", "--floor", "60", "60", "60")[0], 0)
            with mock.patch.dict(coverage.FLOORS, {}, clear=True):
                code, _, err = self.run_main(tables, "--pair", "es-en")
            self.assertEqual((code, err), (0, ""))

    def test_spec_scenario_coverage(self) -> None:
        # es-en, on the committed tables, against its floor — es-fr's figures when es-en was
        # proposed — and published nowhere until it ships.
        self.assertEqual(coverage.FLOORS["es-en"], (87.6, 77.2, 63.7))
        code, out, err = self.run_main(coverage.TABLES, "--pair", "es-en")
        self.assertEqual((code, err), (0, ""))
        measured = json.loads(out)["glossed"]["es-en"]
        for top, share, floor in zip(coverage.TOPS, measured, coverage.FLOORS["es-en"]):
            self.assertGreaterEqual(share, floor, f"the {top:,} commonest lemmas")
        published = json.loads(coverage.SITE_DATA.read_text(encoding="utf-8"))
        self.assertNotIn("es-en", published["glossed"])

    def test_spec_scenario_the_floor_of_en_es(self) -> None:
        # en-es (add-lingua-pack-en-es D3): held to a floor the owner sets on the pull request —
        # the study's figures less two points proposed — in FLOORS alone: the reduce job passes no
        # `--floor` (as es-en's line passes none), so the job and this test cannot drift apart; a
        # committed measurement under it fails, naming the figure.
        proposed = (91.4, 83.2, 69.9)
        self.assertEqual(coverage.FLOORS["en-es"], proposed)
        job = (coverage.ROOT / ".github/workflows/lingua-extension-check.yml").read_text(encoding="utf-8")
        self.assertIn("python scripts/lingua-data/gloss_coverage.py --pair en-es\n", job)
        self.assertNotIn("--pair en-es --floor", job)
        self.assertNotIn("--pair es-en --floor", job)
        with tempfile.TemporaryDirectory() as tmp:
            tables = Path(tmp)
            (tables / "en").mkdir()
            (tables / "en-es").mkdir()
            write_table(tables / "en" / "freq.tsv", [("a", "1"), ("b", "2"), ("c", "3"), ("d", "4")])
            write_table(tables / "en-es" / "gloss.tsv", [("a", "Uno"), ("c", "Tres"), ("d", "Cuatro")])
            code, _, err = self.run_main(tables, "--pair", "en-es")
            self.assertEqual(code, 1)
            self.assertEqual(
                err.splitlines(),
                [
                    "en-es: 75.0 % of the 5,000 commonest lemmas are glossed, under the floor of 91.4 %",
                    "en-es: 75.0 % of the 10,000 commonest lemmas are glossed, under the floor of 83.2 %",
                ],
            )
        # On the committed tables, when they are there: at least FLOORS' entry, and published nowhere.
        if (coverage.TABLES / "en-es" / "gloss.tsv").is_file():
            code, out, err = self.run_main(coverage.TABLES, "--pair", "en-es")
            self.assertEqual((code, err), (0, ""))
            measured = json.loads(out)["glossed"]["en-es"]
            for top, share, floor in zip(coverage.TOPS, measured, coverage.FLOORS["en-es"]):
                self.assertGreaterEqual(share, floor, f"the {top:,} commonest lemmas")
            published = json.loads(coverage.SITE_DATA.read_text(encoding="utf-8"))
            self.assertNotIn("en-es", published["glossed"])


class PublishedFiguresTest(unittest.TestCase):
    def test_the_site_reads_what_the_committed_tables_give(self) -> None:
        # The check the site depends on: a table update that moves a figure fails here until
        # `gloss_coverage.py --write` runs (lingua-pack-update runs it).
        published = json.loads(coverage.SITE_DATA.read_text(encoding="utf-8"))
        self.assertEqual(published, coverage.figures())

    def test_every_shipped_pair_is_published(self) -> None:
        published = json.loads(coverage.SITE_DATA.read_text(encoding="utf-8"))
        self.assertEqual(sorted(published["glossed"]), sorted(coverage.shipped_pairs()))
        self.assertEqual(published["tops"], [5_000, 10_000, 20_000])

    def test_check_refuses_stale_figures(self) -> None:
        original = coverage.SITE_DATA
        with tempfile.TemporaryDirectory() as tmp:
            stale = Path(tmp) / "lingua-coverage.json"
            stale.write_text(json.dumps({"tops": [5000], "glossed": {}}), encoding="utf-8")
            try:
                coverage.SITE_DATA = stale
                self.assertEqual(coverage.main(["--check"]), 1)
                coverage.SITE_DATA = original
                self.assertEqual(coverage.main(["--check"]), 0)
            finally:
                coverage.SITE_DATA = original


if __name__ == "__main__":
    unittest.main()
