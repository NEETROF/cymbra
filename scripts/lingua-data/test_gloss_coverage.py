#!/usr/bin/env python3
# Copyright 2026 NEETROF
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""The coverage figures the site publishes (add-site-lingua-spanish-pages D3)."""

from __future__ import annotations

import json
import sys
import tempfile
import unittest
from pathlib import Path

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
