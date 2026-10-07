# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the update report (pin-lingua-pack-sources D6, D7) on small fixtures."""

import io
import json
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

    def test_the_grammar_tables_are_reported_reading_by_reading(self):
        tables(
            self.old,
            grammar_tsv="leaves\tleaf\tNOUN|Number=Plur\tother\nleaves\tleave\tVERB|Tense=Pres\t-\n",
            senses_tsv="can\tNOUN:1\tVERB:2\n",
        )
        new = tables(
            self.root / "new",
            **{k: (self.old / k.replace("_", ".", 1)).read_text() for k in ("gloss_tsv", "freq_tsv", "level_tsv")},
            grammar_tsv="leaves\tleaf\tNOUN|Number=Plur\t-\nwent\tgo\tVERB|Tense=Past\tother\n",
            senses_tsv="can\tNOUN:1\tVERB:1\n",
        )
        code, report = self.run_report(new)
        self.assertEqual(code, 0)
        self.assertIn("| `grammar.tsv` (form lemma reading → may be named as another word) | 2 → 2 | 1 | 1 | 1 |", report)
        self.assertIn("Added: `went go VERB|Tense=Past`", report)
        self.assertIn("Removed: `leaves leave VERB|Tense=Pres`", report)
        self.assertIn("- `leaves leaf NOUN|Number=Plur`: other → -", report)
        self.assertIn("- `can`: NOUN:1\tVERB:2 → NOUN:1\tVERB:1", report)

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


class Identical(unittest.TestCase):
    """--identical (generalise-lingua-gloss-reducer D4): a pair reduced again expecting no change."""

    MANIFEST = {"meta": {"studied": "es", "native": "fr", "pack_version": "2026.10.03+be9a267", "analyzer_version": "1.2.0"}}
    PIN = {
        "snapshot": "2026.10.03",
        "pack": {"sha256": "2c23f1f1", "size": 2190188},
        "reducer": {"sha256": "be9a267e", "files": ["reduce-es-fr.py", "reduce_common.py"]},
        "sources": {"kaikki": {"sha256": "a2cc93ae"}, "wordfreq": {"version": "3.1.1"}},
    }

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.old = self.pair(self.root / "committed")
        self.new = self.pair(self.root / "tables" / "es-fr")

    def tearDown(self):
        self._tmp.cleanup()

    def pair(self, root: Path) -> Path:
        tables(
            root,
            gloss_tsv="casa\tMaison\n",
            freq_tsv="casa\t1\n",
            tags_tsv="NOUN\n",
            lexical_tsv="casa\n",
            NOTICE="kaikki.org, CC BY-SA 4.0\n",
        )
        (root / "manifest.json").write_text(json.dumps(self.MANIFEST, indent=2) + "\n", encoding="utf-8")
        (root / "pin.json").write_text(json.dumps(self.PIN, indent=2) + "\n", encoding="utf-8")
        return root

    def edit(self, name: str, change) -> None:
        path = self.new / name
        if name.endswith(".json"):
            record = json.loads(path.read_text(encoding="utf-8"))
            change(record)
            path.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
        else:
            path.write_text(change(path.read_text(encoding="utf-8")), encoding="utf-8")

    def run_report(self, *args: str) -> tuple[int, str]:
        out = io.StringIO()
        with redirect_stdout(out):
            code = pr.main([str(self.old), str(self.new), "--identical", *args])
        return code, out.getvalue()

    def test_the_pack_version_and_the_record_of_the_pack_and_rules_may_move(self):
        self.edit("manifest.json", lambda m: m["meta"].update(pack_version="2026.10.03+0123abc"))
        self.edit("pin.json", lambda r: r["pack"].update(sha256="99999999"))
        self.edit("pin.json", lambda r: r["reducer"].update(sha256="0123abcd", files=[*r["reducer"]["files"], "reduce_edition_fr.py"]))
        code, report = self.run_report()
        self.assertEqual(code, 0, report)
        self.assertIn("Reduced again expecting no change", report)

    def test_one_byte_of_a_table_fails_naming_the_pair_and_the_file(self):
        self.edit("gloss.tsv", lambda t: t.replace("Maison", "Maison."))
        code, report = self.run_report()
        self.assertEqual(code, 1)
        self.assertIn("- es-fr: gloss.tsv differs", report)
        # Without --identical, a changed gloss is news, not a failure.
        out = io.StringIO()
        with redirect_stdout(out):
            self.assertEqual(pr.main([str(self.old), str(self.new)]), 0)

    def test_the_kept_inputs_and_notice_may_not_move(self):
        for name in ("tags.tsv", "lexical.tsv", "NOTICE"):
            with self.subTest(name):
                before = (self.new / name).read_text(encoding="utf-8")
                self.edit(name, lambda t: t + "x\n")
                code, report = self.run_report()
                self.assertEqual(code, 1)
                self.assertIn(f"- es-fr: {name} differs", report)
                (self.new / name).write_text(before, encoding="utf-8")

    def test_a_table_that_appears_or_goes_is_a_difference(self):
        (self.new / "lexical.tsv").unlink()
        (self.new / "mwe.tsv").write_text("hay que\tIl faut\n", encoding="utf-8")
        code, report = self.run_report()
        self.assertEqual(code, 1)
        self.assertIn("- es-fr: lexical.tsv differs", report)
        self.assertIn("- es-fr: mwe.tsv differs", report)

    def test_the_manifest_moves_in_its_pack_version_only(self):
        self.edit("manifest.json", lambda m: m["meta"].update(pack_version="2026.10.03+0123abc", analyzer_version="1.3.0"))
        code, report = self.run_report()
        self.assertEqual(code, 1)
        self.assertIn("- es-fr: manifest.json differs at meta.analyzer_version", report)

    def test_the_pin_moves_in_the_pack_s_sha256_and_the_rules_only(self):
        self.edit("pin.json", lambda r: (r["pack"].update(size=2190189), r["sources"]["kaikki"].update(sha256="ffffffff")))
        code, report = self.run_report()
        self.assertEqual(code, 1)
        self.assertIn("- es-fr: pin.json differs at pack.size, sources.kaikki.sha256", report)
        self.edit("pin.json", lambda r: r.update(pack={"sha256": "2c23f1f1", "size": 2190188}, snapshot="2026.10.07"))
        code, report = self.run_report()
        self.assertIn("- es-fr: pin.json differs at snapshot, sources.kaikki.sha256", report)

    def test_the_pair_is_named_on_demand(self):
        self.edit("gloss.tsv", lambda t: t + "perro\tChien\n")
        code, report = self.run_report("--pair", "en-fr")
        self.assertEqual(code, 1)
        self.assertIn("- en-fr: gloss.tsv differs", report)


class Studied(unittest.TestCase):
    """A studied language's folder (split-lingua-pack-tables-by-language): what its tables change,
    and the pairs reading it whose pack moves."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.committed, self.tables = self.root / "committed", self.root / "tables"
        for root in (self.committed, self.tables):
            tables(
                root / "es",
                forms_tsv="casas\tcasa\n",
                freq_tsv="casa\t1\ndios\t2\n",
                level_tsv="casa\tA1\ndios\tA2\n",
                grammar_tsv="casas\tcasa\tNOUN|Number=Plur\t-\n",
                tags_tsv="NOUN\n",
                lexical_tsv="casa\ndios\n",
                studied_json='{"reference": "es-fr"}\n',
            )
            for pair, sha in (("es-fr", "aaaa"), ("es-en", "bbbb"), ("en-fr", "cccc")):
                (root / pair).mkdir(parents=True)
                (root / pair / "pin.json").write_text(json.dumps({"pack": {"sha256": sha}}), encoding="utf-8")

    def tearDown(self):
        self._tmp.cleanup()

    def run_report(self, *args: str) -> tuple[int, str]:
        out = io.StringIO()
        with redirect_stdout(out):
            code = pr.main([str(self.committed / "es"), str(self.tables / "es"), *args])
        return code, out.getvalue()

    def test_spec_scenario_spanish_s_tables_change_under_a_second_pair(self):
        # es-fr's update changes Spanish's levels and dictionary words; es-fr and es-en record new
        # packs, en-fr does not.
        tables(self.tables / "es", level_tsv="casa\tA1\ndios\tB1\n", lexical_tsv="casa\ndios\nárbol\n")
        for pair in ("es-fr", "es-en"):
            (self.tables / pair / "pin.json").write_text(json.dumps({"pack": {"sha256": "ffff"}}), encoding="utf-8")
        code, report = self.run_report()
        self.assertEqual(code, 0, report)
        self.assertIn("| `level.tsv` (lemma → CEFR level) | 2 → 2 | 0 | 0 | 1 |", report)
        self.assertIn("- `dios`: A2 → B1", report)
        self.assertIn("| `lexical.tsv` (dictionary word) | 2 → 3 | 1 | 0 | 0 |", report)
        self.assertIn("Added: `árbol`", report)
        self.assertIn("Pairs reading these tables whose pack moves: `es-en`, `es-fr`.", report)
        self.assertNotIn("en-fr", report)

    def test_no_pack_moves_with_tables_that_do_not(self):
        code, report = self.run_report()
        self.assertEqual(code, 0, report)
        self.assertIn("No pack of a pair reading these tables moves.", report)
        # A pair the run did not reduce (no committed pin) is not named, nor a malformed record.
        (self.committed / "es-en" / "pin.json").unlink()
        (self.tables / "es-en" / "pin.json").write_text(json.dumps({"pack": {"sha256": "ffff"}}), encoding="utf-8")
        (self.tables / "es-xx").mkdir()
        (self.committed / "es-xx").mkdir()
        for root in (self.committed, self.tables):
            (root / "es-xx" / "pin.json").write_text("{}", encoding="utf-8")
        self.assertIn("No pack of a pair reading these tables moves.", self.run_report()[1])

    def test_spec_scenario_a_studied_table_meant_not_to_move(self):
        # Reduced again expecting no change: one byte of Spanish's level.tsv names es-fr and es/level.tsv.
        tables(self.tables / "es", level_tsv="casa\tA1\ndios\tA2 \n")
        code, report = self.run_report("--identical")
        self.assertEqual(code, 1)
        self.assertIn("- es-fr: es/level.tsv differs", report)
        tables(self.tables / "es", level_tsv="casa\tA1\ndios\tA2\n", studied_json='{"reference": "es-en"}\n')
        code, report = self.run_report("--identical")
        self.assertEqual(code, 1)
        self.assertIn("- es-en: es/studied.json differs", report)


if __name__ == "__main__":
    unittest.main()
