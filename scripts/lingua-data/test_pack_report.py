# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the update report (pin-lingua-pack-sources D6, D7) on small fixtures."""

import io
import sys
import tempfile
import unittest
from contextlib import redirect_stdout
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent))

import pack_report as pr  # noqa: E402


def tables(root: Path, **files: str) -> Path:
    root.mkdir(parents=True, exist_ok=True)
    for name, text in files.items():
        (root / name.replace("_", ".", 1)).write_text(text, encoding="utf-8")
    return root


class Report(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.old = tables(
            self.root / "old",
            gloss_tsv="boat\tbateau\nharbour\tport\nship\tnavire\n",
            freq_tsv="the\t1\nboat\t900\nharbour\t3000\n",
            level_tsv="boat\tA1\nharbour\tB1\n",
        )

    def tearDown(self):
        self._tmp.cleanup()

    def run_report(self, new: Path, *args: str) -> tuple[int, str]:
        out = io.StringIO()
        with redirect_stdout(out):
            code = pr.main([str(self.old), str(new), *args])
        return code, out.getvalue()

    def test_counts_and_samples_what_changed(self):
        new = tables(
            self.root / "new",
            gloss_tsv="boat\tbarque\nharbour\tport\nsmartphone\tsmartphone\n",
            freq_tsv="the\t1\nboat\t950\nharbour\t9000\n",
            level_tsv="boat\tA1\nharbour\tB2\n",
        )
        code, report = self.run_report(new)
        self.assertEqual(code, 0)
        self.assertIn("| `gloss.tsv` (lemma → gloss) | 3 → 3 | 1 | 1 | 1 |", report)
        self.assertIn("Added: `smartphone`", report)
        self.assertIn("Removed: `ship`", report)
        self.assertIn("- `boat`: bateau → barque", report)
        self.assertIn("- `harbour`: B1 → B2", report)
        # A rank that moved by a few places is not news; one that moved far is.
        self.assertNotIn("`boat`: 900 → 950", report)
        self.assertIn("- `harbour`: 3000 → 9000", report)

    def test_a_collapsed_table_fails(self):
        new = tables(self.root / "new", gloss_tsv="boat\tbateau\n", freq_tsv="the\t1\nboat\t900\nharbour\t3000\n")
        code, report = self.run_report(new, "--max-loss", "0.2")
        self.assertEqual(code, 1)
        self.assertIn("gloss.tsv lost 67% of its rows (3 → 1)", report)
        self.assertIn("level.tsv is missing from the new tables", report)

    def test_the_pack_against_its_budget(self):
        new = tables(self.root / "new", **{k: (self.old / k.replace("_", ".", 1)).read_text() for k in ("gloss_tsv", "freq_tsv", "level_tsv")})
        pack = self.root / "pack.lingua"
        pack.write_bytes(b"x" * 1_543_744)
        code, report = self.run_report(new, "--pack", str(pack))
        self.assertEqual(code, 0)
        self.assertIn("Pack: **1,543,744 B**, 29.4% of the 5 MiB budget.", report)
        pack.write_bytes(b"x" * (pr.BUDGET + 1))
        code, report = self.run_report(new, "--pack", str(pack))
        self.assertEqual(code, 1)
        self.assertIn("over its budget", report)


if __name__ == "__main__":
    unittest.main()
