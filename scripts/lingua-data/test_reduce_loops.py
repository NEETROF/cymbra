# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The workflows' loops over the pairs (split-lingua-pack-tables-by-language D1, D5), run as the
workflows run them: the `reduce` job of lingua-extension-check and the Reduce step of
lingua-pack-update, each step's own script, over scratch tables, with `build.sh` doubled — and the
steps around them that read `pack_sources.py`: the update's release step (`gh` doubled), and the
`check` job's build of every pair (add-lingua-pack-es-en).

A loop fed by `pack_sources.py pairs` must stop when the list cannot be made — a folder that is
neither a pair nor a studied language, or no pair at all. Under `set -e`, a failing `$(…)` in a
`for` list, or a failing process substitution feeding an array, runs the loop zero times and
leaves the step green, having reduced nothing.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import json
import os
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
WORKFLOWS = HERE.parent.parent / ".github" / "workflows"


def step_script(workflow: Path, step: str) -> str:
    """The `run: |` script of the step named `step` in `workflow`, dedented."""
    lines = workflow.read_text(encoding="utf-8").splitlines()
    start = next(i for i, line in enumerate(lines) if line.strip() == f"- name: {step}")
    # Its `run: |`, before the next step's name.
    run = next(i for i in range(start + 1, len(lines)) if lines[i].strip() == "run: |" or lines[i].strip().startswith("- name: "))
    if lines[run].strip() != "run: |":
        raise AssertionError(f"{workflow.name}: the step {step!r} runs no script")
    body, indent = [], None
    for line in lines[run + 1 :]:
        if not line.strip():
            body.append("")
            continue
        width = len(line) - len(line.lstrip())
        indent = width if indent is None else indent
        if width < indent:
            break
        body.append(line[indent:])
    return "\n".join(body).rstrip("\n") + "\n"


# build.sh, doubled: it logs its arguments and leaves the raw sources a reduction would — and, for
# en-es, what its reducer measures of its tables (measures.json, add-lingua-pack-en-es D4).
BUILD_SH = """#!/usr/bin/env bash
set -euo pipefail
echo "${1#--} $2" >> "$BUILD_LOG"
mkdir -p "$(dirname "$0")/work/$2"
echo raw > "$(dirname "$0")/work/$2/raw"
if [ "$2" = en-es ]; then echo '{"share": 1.0}' > "$(dirname "$0")/work/$2/measures.json"; fi
"""


class Loops(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.repo = Path(self._tmp.name) / "repo"
        self.data = self.repo / "scripts" / "lingua-data"
        self.tables = self.data / "tables"
        self.data.mkdir(parents=True)
        shutil.copy(HERE / "pack_sources.py", self.data / "pack_sources.py")
        (self.data / "build.sh").write_text(BUILD_SH)
        (self.data / "build.sh").chmod(0o755)
        # The workflows call `python`, as setup-python provides it.
        self.bin = Path(self._tmp.name) / "bin"
        self.bin.mkdir()
        (self.bin / "python").write_text(f'#!/bin/sh\nexec "{sys.executable}" "$@"\n')
        (self.bin / "python").chmod(0o755)
        self.temp = Path(self._tmp.name) / "runner"
        self.temp.mkdir()
        self.log = Path(self._tmp.name) / "build.log"
        # English and Spanish, each with its reference and a second pair reading it.
        for lang, reference in (("en", "en-fr"), ("es", "es-fr")):
            (self.tables / lang).mkdir(parents=True)
            (self.tables / lang / "studied.json").write_text(f'{{"reference": "{reference}"}}\n')
        for pair in ("en-es", "en-fr", "es-en", "es-fr"):
            (self.tables / pair).mkdir()
            (self.tables / pair / "pin.json").write_text("{}\n")

    def tearDown(self):
        self._tmp.cleanup()

    def run_step(self, workflow: str, step: str, cwd=None, **env) -> subprocess.CompletedProcess:
        script = Path(self._tmp.name) / "step.sh"
        script.write_text(step_script(WORKFLOWS / workflow, step))
        return subprocess.run(
            # GitHub's `shell: bash`.
            ["bash", "--noprofile", "--norc", "-eo", "pipefail", str(script)],
            cwd=cwd or self.repo,
            env={
                "PATH": f"{self.bin}{os.pathsep}{os.environ['PATH']}",
                "RUNNER_TEMP": str(self.temp),
                "GITHUB_ENV": str(self.temp / "github_env"),
                "BUILD_LOG": str(self.log),
                "LINGUA_PYTHON": "python",
                **env,
            },
            capture_output=True,
            text=True,
            check=False,
        )

    def built(self) -> list[str]:
        return self.log.read_text().splitlines() if self.log.is_file() else []

    def kept_work(self) -> list[str]:
        work = self.data / "work"
        return sorted(p.name for p in work.iterdir()) if work.is_dir() else []

    def a_folder_that_is_no_pair(self) -> None:
        # The hand-committed tag pool of a language whose first reduction has not run: no
        # studied.json yet, so `pairs` refuses the folder.
        (self.tables / "de").mkdir()
        (self.tables / "de" / "tags.tsv").write_text("NOUN\n")

    # — lingua-extension-check, the `reduce` job —

    def reduce_job(self) -> subprocess.CompletedProcess:
        return self.run_step("lingua-extension-check.yml", "Reduce every pair again from its pinned sources")

    def test_the_reduce_job_reduces_every_pair_each_reference_first(self):
        # en-es after en-fr, es-en after es-fr (add-lingua-pack-en-es, *The reduce job*).
        done = self.reduce_job()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["reduce en-fr", "reduce es-fr", "reduce en-es", "reduce es-en"])
        self.assertEqual(self.kept_work(), [], "the raw sources are dropped after each pair")

    def test_the_reduce_job_fails_when_the_pairs_cannot_be_listed(self):
        self.a_folder_that_is_no_pair()
        done = self.reduce_job()
        self.assertNotEqual(done.returncode, 0, "a step that reduced nothing passed")
        self.assertIn("de is neither a pair", done.stderr)
        self.assertEqual(self.built(), [])

    def test_the_reduce_job_fails_when_there_is_no_pair(self):
        for pair in ("en-es", "en-fr", "es-en", "es-fr"):
            shutil.rmtree(self.tables / pair)
        done = self.reduce_job()
        self.assertNotEqual(done.returncode, 0, "a step that reduced nothing passed")
        self.assertIn("holds no pair", done.stderr)
        self.assertEqual(self.built(), [])

    # — lingua-pack-update, the Reduce step —

    def update(self, mode: str, pair: str) -> subprocess.CompletedProcess:
        return self.run_step("lingua-pack-update.yml", "Reduce", MODE=mode, PAIR=pair)

    def test_every_pair_is_reduced_again_each_reference_first(self):
        done = self.update("reduce", "all")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["reduce en-fr", "reduce es-fr", "reduce en-es", "reduce es-en"])
        self.assertEqual(self.kept_work(), [])
        self.assertIn("PAIRS=en-fr es-fr en-es es-en\n", (self.temp / "github_env").read_text())
        # What a reducer measured of its tables is kept for the report, though its work is dropped.
        self.assertEqual((self.temp / "measures-en-es.json").read_text(), '{"share": 1.0}\n')
        self.assertFalse((self.temp / "measures-en-fr.json").exists(), "en-fr's reducer measures nothing")

    def test_a_reference_s_update_brings_its_readers_and_keeps_its_own_sources_alone(self):
        # The named pair's raw sources are published by a later step; the pairs it brings along
        # are re-reductions, whose raw sources are dropped like any other.
        done = self.update("update", "es-fr")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["update es-fr", "reduce es-en"])
        self.assertEqual(self.kept_work(), ["es-fr"])
        self.assertTrue((self.temp / "committed" / "es").is_dir(), "the studied folder is reported on")

    def test_a_dry_run_checks_its_pair_alone(self):
        done = self.update("dry", "es-fr")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["dry es-fr"])

    def test_an_update_of_a_pair_that_is_no_reference_reduces_it_alone(self):
        # en-es reads tables/en and writes nothing of it: its update brings no pair along, and its
        # raw sources — the files it derives from the dumps — are kept for the release step.
        done = self.update("update", "en-es")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["update en-es"])
        self.assertEqual(self.kept_work(), ["en-es"])
        self.assertTrue((self.temp / "committed" / "en").is_dir())

    def test_an_update_fails_when_the_pairs_cannot_be_listed(self):
        self.a_folder_that_is_no_pair()
        for mode, pair in (("reduce", "all"), ("update", "es-fr")):
            done = self.update(mode, pair)
            self.assertNotEqual(done.returncode, 0, f"{mode} {pair}: a step that reduced nothing passed")
            self.assertIn("de is neither a pair", done.stderr)
        self.assertEqual(self.built(), [])

    # — lingua-pack-update, the release step —

    def gh(self) -> Path:
        """`gh`, doubled: it logs its arguments; no release exists yet."""
        log = Path(self._tmp.name) / "gh.log"
        (self.bin / "gh").write_text(
            f'#!/bin/sh\necho "$*" >> "{log}"\n[ "$1 $2" = "release view" ] && exit 1\nexit 0\n'
        )
        (self.bin / "gh").chmod(0o755)
        return log

    def pin(self, pair: str, record: dict) -> None:
        (self.tables / pair / "pin.json").write_text(json.dumps(record, indent=2) + "\n")

    def own_record(self, pair: str, snapshot: str, derived: str) -> dict:
        release = f"lingua-pack-sources-{pair}-{snapshot}"
        return {
            "snapshot": snapshot,
            "sources": {
                "kaikki": {"release": release, "asset": "kaikki-Spanish.jsonl.zst", "url": "https://kaikki.org/x.jsonl"},
                "kaikki-es": {"release": release, "files": {derived: {"asset": f"{derived}.zst"}}},
                "wordfreq": {"version": "3.1.1"},
            },
        }

    def release(self, pair: str) -> subprocess.CompletedProcess:
        return self.run_step(
            "lingua-pack-update.yml",
            "Keep kaikki's bytes as the snapshot's release",
            PAIR=pair,
            SNAPSHOT="2026.10.08",
            GITHUB_SHA="0" * 40,
            GITHUB_STEP_SUMMARY=str(self.temp / "summary.md"),
        )

    def test_the_release_holds_the_named_pair_s_own_assets(self):
        # es-fr's update brought es-en along: es-fr's release holds es-fr's assets, and nothing of
        # es-en, whose own pin names its own release.
        log = self.gh()
        self.pin("es-fr", self.own_record("es-fr", "2026.10.08", "kaikki-es-traductions.jsonl"))
        self.pin("es-en", self.own_record("es-en", "2026.10.07", "kaikki-es-traductions-en.jsonl"))
        done = self.release("es-fr")
        self.assertEqual(done.returncode, 0, done.stderr)
        created = [line for line in log.read_text().splitlines() if line.startswith("release create")]
        self.assertEqual(len(created), 1, created)
        work = "scripts/lingua-data/work/es-fr"
        self.assertTrue(
            created[0].startswith(
                f"release create lingua-pack-sources-es-fr-2026.10.08 {work}/kaikki-Spanish.jsonl.zst "
                f"{work}/kaikki-es-traductions.jsonl.zst --target "
            ),
            created[0],
        )
        self.assertNotIn("es-en", created[0])

    def test_the_release_of_a_pair_whose_sources_are_dumps_alone_names_no_extract(self):
        # en-es (add-lingua-pack-en-es D2): no `sources.kaikki`; the release holds the files
        # derived from the Spanish Wiktionary's dump and the English Wiktionary's extract, and its
        # notes name no extract.
        log = self.gh()
        release = "lingua-pack-sources-en-es-2026.10.08"
        self.pin(
            "en-es",
            {
                "snapshot": "2026.10.08",
                "sources": {
                    "kaikki-es": {
                        "release": release,
                        "files": {
                            "kaikki-es-English.jsonl": {"asset": "kaikki-es-English.jsonl.zst"},
                            "kaikki-es-traductions-en.jsonl": {"asset": "kaikki-es-traductions-en.jsonl.zst"},
                        },
                    },
                    "kaikki-en": {"release": release, "files": {"kaikki-en-traductions-es.jsonl": {"asset": "kaikki-en-traductions-es.jsonl.zst"}}},
                    "wordfreq": {"version": "3.1.1"},
                },
            },
        )
        done = self.release("en-es")
        self.assertEqual(done.returncode, 0, done.stderr)
        created = [line for line in log.read_text().splitlines() if line.startswith("release create")]
        work = "scripts/lingua-data/work/en-es"
        self.assertEqual(len(created), 1, created)
        self.assertTrue(
            created[0].startswith(
                f"release create {release} {work}/kaikki-es-English.jsonl.zst {work}/kaikki-es-traductions-en.jsonl.zst "
                f"{work}/kaikki-en-traductions-es.jsonl.zst --target "
            ),
            created[0],
        )
        notes = (self.temp / "notes.md").read_text()
        self.assertIn("pins no extract of its own", notes)
        self.assertNotIn("the extract (", notes)
        self.assertIn("tables/en-es/pin.json", notes)

    def test_the_release_step_fails_when_its_assets_cannot_be_listed(self):
        # A record `assets` cannot read (the extract's asset missing): the step stops, and nothing
        # is published — it neither passes having published nothing nor publishes half the list.
        log = self.gh()
        record = self.own_record("es-en", "2026.10.08", "kaikki-es-traductions-en.jsonl")
        del record["sources"]["kaikki"]["asset"]
        self.pin("es-en", record)
        done = self.release("es-en")
        self.assertNotEqual(done.returncode, 0, "a step whose asset list failed passed")
        self.assertIn("KeyError", done.stderr)
        self.assertFalse(any(line.startswith("release create") for line in log.read_text().splitlines()))

    def test_the_release_step_fails_when_no_asset_is_the_pair_s_own(self):
        log = self.gh()
        record = self.own_record("es-en", "2026.10.08", "kaikki-es-traductions-en.jsonl")
        for source in ("kaikki", "kaikki-es"):
            record["sources"][source]["release"] = "lingua-pack-sources-es-fr-2026.10.08"
        self.pin("es-en", record)
        done = self.release("es-en")
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("names no asset under lingua-pack-sources-es-en-2026.10.08", done.stdout)
        self.assertFalse(any(line.startswith("release create") for line in log.read_text().splitlines()))

    # — lingua-extension-check, the `check` job's build of every pair —

    def test_the_check_job_names_a_moved_studied_table_before_building(self):
        # check-reducer runs before build.sh: a reader pair whose studied table moved fails on the
        # check, which names the table, not on the build, which would only say the sha256 moved.
        (self.bin / "python3").write_text(
            "#!/bin/sh\n"
            'pair="$(basename "$(dirname "$4")")"\n'
            'echo "check $pair" >> "$BUILD_LOG"\n'
            'if [ "$pair" = es-en ]; then echo "error: es-en: es/level.tsv is not what es-en\'s tables were built on" >&2; exit 1; fi\n'
        )
        (self.bin / "python3").chmod(0o755)
        (self.data / "build.sh").write_text('#!/usr/bin/env bash\necho "build $1" >> "$BUILD_LOG"\n')
        extension = self.repo / "apps" / "lingua-extension"
        extension.mkdir(parents=True)
        done = self.run_step("lingua-extension-check.yml", "Build the real pack from the committed tables", cwd=extension)
        self.assertNotEqual(done.returncode, 0)
        self.assertIn("es-en: es/level.tsv", done.stderr)
        self.assertEqual(self.built(), ["check en-es", "build en-es", "check en-fr", "build en-fr", "check es-en"])

    def test_the_step_scripts_are_read_whole(self):
        script = step_script(WORKFLOWS / "lingua-pack-update.yml", "Reduce")
        self.assertTrue(script.startswith("set -euo pipefail\n"), script[:80])
        self.assertIn("build.sh", script)
        with self.assertRaises(StopIteration):
            step_script(WORKFLOWS / "lingua-pack-update.yml", "No such step")


if __name__ == "__main__":
    unittest.main()
