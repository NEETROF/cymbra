# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""The workflows' loops over the pairs (split-lingua-pack-tables-by-language D1, D5), run as the
workflows run them: the `reduce` job of lingua-extension-check and the Reduce step of
lingua-pack-update, each step's own script, over scratch tables, with `build.sh` doubled.

A loop fed by `pack_sources.py pairs` must stop when the list cannot be made — a folder that is
neither a pair nor a studied language, or no pair at all. Under `set -e`, a failing `$(…)` in a
`for` list, or a failing process substitution feeding an array, runs the loop zero times and
leaves the step green, having reduced nothing.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

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


# build.sh, doubled: it logs its arguments and leaves the raw sources a reduction would.
BUILD_SH = """#!/usr/bin/env bash
set -euo pipefail
echo "${1#--} $2" >> "$BUILD_LOG"
mkdir -p "$(dirname "$0")/work/$2"
echo raw > "$(dirname "$0")/work/$2/raw"
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
        # English with its reference, Spanish with its reference and a second pair reading it.
        for lang, reference in (("en", "en-fr"), ("es", "es-fr")):
            (self.tables / lang).mkdir(parents=True)
            (self.tables / lang / "studied.json").write_text(f'{{"reference": "{reference}"}}\n')
        for pair in ("en-fr", "es-en", "es-fr"):
            (self.tables / pair).mkdir()
            (self.tables / pair / "pin.json").write_text("{}\n")

    def tearDown(self):
        self._tmp.cleanup()

    def run_step(self, workflow: str, step: str, **env) -> subprocess.CompletedProcess:
        script = Path(self._tmp.name) / "step.sh"
        script.write_text(step_script(WORKFLOWS / workflow, step))
        return subprocess.run(
            # GitHub's `shell: bash`.
            ["bash", "--noprofile", "--norc", "-eo", "pipefail", str(script)],
            cwd=self.repo,
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
        done = self.reduce_job()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["reduce en-fr", "reduce es-fr", "reduce es-en"])
        self.assertEqual(self.kept_work(), [], "the raw sources are dropped after each pair")

    def test_the_reduce_job_fails_when_the_pairs_cannot_be_listed(self):
        self.a_folder_that_is_no_pair()
        done = self.reduce_job()
        self.assertNotEqual(done.returncode, 0, "a step that reduced nothing passed")
        self.assertIn("de is neither a pair", done.stderr)
        self.assertEqual(self.built(), [])

    def test_the_reduce_job_fails_when_there_is_no_pair(self):
        for pair in ("en-fr", "es-en", "es-fr"):
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
        self.assertEqual(self.built(), ["reduce en-fr", "reduce es-fr", "reduce es-en"])
        self.assertEqual(self.kept_work(), [])
        self.assertIn("PAIRS=en-fr es-fr es-en\n", (self.temp / "github_env").read_text())

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

    def test_an_update_fails_when_the_pairs_cannot_be_listed(self):
        self.a_folder_that_is_no_pair()
        for mode, pair in (("reduce", "all"), ("update", "es-fr")):
            done = self.update(mode, pair)
            self.assertNotEqual(done.returncode, 0, f"{mode} {pair}: a step that reduced nothing passed")
            self.assertIn("de is neither a pair", done.stderr)
        self.assertEqual(self.built(), [])

    def test_the_step_scripts_are_read_whole(self):
        script = step_script(WORKFLOWS / "lingua-pack-update.yml", "Reduce")
        self.assertTrue(script.startswith("set -euo pipefail\n"), script[:80])
        self.assertIn("build.sh", script)
        with self.assertRaises(StopIteration):
            step_script(WORKFLOWS / "lingua-pack-update.yml", "No such step")


if __name__ == "__main__":
    unittest.main()
