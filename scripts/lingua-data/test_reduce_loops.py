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
# en-es, what its reducer measures of its tables (measures.json, add-lingua-pack-en-es D4). An
# update or a dry run reads the run's editions folder (LINGUA_EDITIONS, migrate-lingua-pack-sources-
# to-raw-dumps D4): the double logs what earlier pairs of the run left there, and leaves its own. A
# dry run writes its root as the real one does: the pair's folder, and its studied language's,
# removed and laid down again from the committed one — the reference then writing its drift
# (`level.tsv`). `FAIL_PAIR` names a pair whose dry run fails, half way.
BUILD_SH = r"""#!/usr/bin/env bash
set -euo pipefail
here="$(dirname "$0")"
pair="$2"
studied="${pair%%-*}"
echo "${1#--} $pair" >> "$BUILD_LOG"
mkdir -p "$here/work/$pair"
echo raw > "$here/work/$pair/raw"
if [ "$pair" = en-es ]; then echo '{"share": 1.0}' > "$here/work/$pair/measures.json"; fi
case "$1" in
  --update | --dry)
    editions="${LINGUA_EDITIONS:-$here/work/editions}"
    mkdir -p "$editions"
    echo "$pair: $(ls "$editions" | tr '\n' ' ')" >> "$BUILD_LOG.editions"
    touch "$editions/$pair"
    ;;
esac
if [ "$1" = --dry ]; then
  root="${LINGUA_DRY_ROOT:-$here/work/dry}"
  rm -rf "${root:?}/${pair:?}" "${root:?}/${studied:?}"
  mkdir -p "$root/$pair"
  cp -r "$here/tables/$studied" "$root/$studied"
  echo "$pair" > "$root/$pair/gloss.tsv"
  if [ "${FAIL_PAIR:-}" = "$pair" ]; then exit 1; fi
  if grep -q "\"$pair\"" "$here/tables/$studied/studied.json"; then
    echo "drift by $pair" > "$root/$studied/level.tsv"
  fi
fi
"""

# pack_report.py, doubled: it names the folders it compares and the tables the new one holds.
PACK_REPORT = """import sys
from pathlib import Path
new = Path(sys.argv[2])
held = {f.name: f.read_text().strip() for f in sorted(new.iterdir()) if f.suffix == ".tsv"}
print(f"report {sys.argv[1]} {sys.argv[2]} {held}")
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
        (self.data / "pack_report.py").write_text(PACK_REPORT)
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
            (self.tables / lang / "level.tsv").write_text("committed\n")
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

    def editions_seen(self) -> list[str]:
        seen = Path(f"{self.log}.editions")
        return [line.rstrip() for line in seen.read_text().splitlines()] if seen.is_file() else []

    def github_env(self) -> dict:
        """What a step wrote to GITHUB_ENV, as the next steps read it."""
        env = {}
        for line in (self.temp / "github_env").read_text().splitlines():
            name, _, value = line.partition("=")
            env[name] = value
        return env

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

    def update(self, mode: str, pair: str, **env) -> subprocess.CompletedProcess:
        return self.run_step("lingua-pack-update.yml", "Reduce", MODE=mode, PAIR=pair, **env)

    def report(self, mode: str) -> subprocess.CompletedProcess:
        """The Report step, after the Reduce step: what it wrote to GITHUB_ENV, as GitHub passes it."""
        return self.run_step(
            "lingua-pack-update.yml",
            "Report against the committed tables",
            MODE=mode,
            EXPECT="any",
            MAX_LOSS="0.2",
            GITHUB_STEP_SUMMARY=str(self.temp / "summary.md"),
            **self.github_env(),
        )

    def reported(self) -> dict:
        """The Report step's report, by the folder each line compares."""
        lines = (self.temp / "report.md").read_text().splitlines()
        return {line.split()[2].rsplit("/", 1)[1]: line for line in lines if line.startswith("report ")}

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
        self.assertEqual(self.kept_work(), ["dry"], "its raw sources and the run's editions are dropped")
        done = self.report("dry")
        self.assertEqual(done.returncode, 0, done.stderr)
        reports = self.reported()
        self.assertEqual(sorted(reports), ["es", "es-fr"])
        self.assertIn(f"report {self.temp}/committed/es-fr scripts/lingua-data/work/dry/es-fr/es-fr ", reports["es-fr"])

    def test_spec_scenario_the_dry_run_checks_every_pair_in_one_job(self):
        # The monthly run (D4): every pair in `pairs` order, one job; each pair reduces into a dry
        # root of its own, so the studied folder a reader pair lays down again from the committed
        # copy never hides its reference's drift; the run's editions folder is kept across the
        # pairs — each dump read once — and removed at the end, as is each pair's work folder.
        done = self.update("dry", "all")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual(self.built(), ["dry en-fr", "dry es-fr", "dry en-es", "dry es-en"])
        self.assertEqual(
            self.editions_seen(),
            ["en-fr:", "es-fr: en-fr", "en-es: en-fr es-fr", "es-en: en-es en-fr es-fr"],
            "the later pairs of the run find what the earlier ones derived",
        )
        self.assertEqual(self.kept_work(), ["dry"], "no pair's raw sources, no editions, no dumps")
        dry = self.data / "work" / "dry"
        self.assertEqual(sorted(p.name for p in dry.iterdir()), ["en-es", "en-fr", "es-en", "es-fr"], "a root per pair")
        self.assertEqual((dry / "en-es" / "en" / "level.tsv").read_text(), "committed\n", "en-es laid the committed copy down")
        self.assertEqual(self.github_env()["FAILED"], "")
        done = self.report("dry")
        self.assertEqual(done.returncode, 0, done.stderr)
        reports = self.reported()
        self.assertEqual(sorted(reports), ["en", "en-es", "en-fr", "es", "es-en", "es-fr"])
        # The `en` report is the one en-fr wrote, under en-fr's root: its drift, not en-es's copy.
        self.assertIn("work/dry/en-fr/en {'level.tsv': 'drift by en-fr'}", reports["en"])
        self.assertIn("work/dry/es-fr/es {'level.tsv': 'drift by es-fr'}", reports["es"])
        self.assertIn("work/dry/es-en/es-en {'gloss.tsv': 'es-en'}", reports["es-en"])

    def test_a_failing_pair_does_not_stop_the_loop(self):
        # es-fr fails half way: the loop goes on to en-es and es-en, the Report step skips what
        # es-fr would have written — its folder and Spanish's, which lies under its root — and the
        # job fails at the end, naming it.
        done = self.update("dry", "all", FAIL_PAIR="es-fr")
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertIn("::error::es-fr: the dry run failed", done.stdout)
        self.assertEqual(self.built(), ["dry en-fr", "dry es-fr", "dry en-es", "dry es-en"])
        self.assertEqual(self.kept_work(), ["dry"], "the failed pair's raw sources dropped too")
        self.assertEqual(self.github_env()["FAILED"], "es-fr")
        done = self.report("dry")
        self.assertNotEqual(done.returncode, 0, "a failed pair fails the job")
        self.assertIn("::error::These pairs failed: es-fr.", done.stdout)
        self.assertEqual(sorted(self.reported()), ["en", "en-es", "en-fr", "es-en"])
        self.assertIn("**Failed**: es-fr", (self.temp / "report.md").read_text())

    def test_a_reduction_still_stops_at_its_first_failure(self):
        # Outside a dry run a failure stops the step: nothing is proposed from half a run.
        (self.data / "build.sh").write_text('#!/usr/bin/env bash\necho "${1#--} $2" >> "$BUILD_LOG"\n[ "$2" != es-fr ]\n')
        done = self.update("reduce", "all")
        self.assertNotEqual(done.returncode, 0)
        self.assertEqual(self.built(), ["reduce en-fr", "reduce es-fr"])

    def test_pair_all_is_refused_for_an_update(self):
        for mode, refused in (("update", True), ("reduce", False), ("dry", False)):
            done = self.run_step("lingua-pack-update.yml", "Check the inputs", MODE=mode, PAIR="all", EXPECT="any")
            self.assertEqual(done.returncode != 0, refused, f"{mode}: {done.stdout}")

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
        # en-es's pin as change 22 wrote it (add-lingua-pack-en-es D2): no `sources.kaikki`, derived
        # files alone; the release holds them, and its notes name each record they come from and no
        # extract.
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
        self.assertIn("derived: `kaikki-es-English.jsonl`, `kaikki-es-traductions-en.jsonl`\n", notes)
        self.assertIn("derived: `kaikki-en-traductions-es.jsonl`\n", notes)
        self.assertNotIn("per-language extract", notes)
        self.assertIn("tables/en-es/pin.json", notes)

    def dumped(self, release: str, files: list[str], identity: bool = True) -> dict:
        record = {
            "release": release,
            "url": "https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz",
            "fetched": "2026-10-08",
            "last_modified": "Sat, 03 Oct 2026 08:24:38 GMT",
            "files": {file: {"asset": f"{file}.zst", "sha256": "1" * 64, "size": 1} for file in files},
        }
        if identity:
            record["dump"] = {"sha256": "d" * 64, "size": 25_614_284_530, "compressed_size": 2_981_058_381}
        return record

    def test_spec_scenario_the_release_notes_name_the_dumps(self):
        # A pin written by an update under the dumps: each edition's dump by its address,
        # regeneration date and decompressed sha256, and the files derived from it; no extract.
        log = self.gh()
        release = "lingua-pack-sources-es-en-2026.10.08"
        spanish = self.dumped(release, ["kaikki-es-traductions-en.jsonl"])
        spanish.update(url="https://kaikki.org/eswiktionary/raw-wiktextract-data.jsonl.gz", last_modified="Fri, 02 Oct 2026 12:12:06 GMT")
        self.pin(
            "es-en",
            {
                "snapshot": "2026.10.08",
                "sources": {
                    "kaikki-en": self.dumped(release, ["kaikki-Spanish.jsonl"]),
                    "kaikki-es": spanish,
                    "wordfreq": {"version": "3.1.1"},
                },
            },
        )
        done = self.release("es-en")
        self.assertEqual(done.returncode, 0, done.stderr)
        created = [line for line in log.read_text().splitlines() if line.startswith("release create")]
        work = "scripts/lingua-data/work/es-en"
        self.assertTrue(
            created[0].startswith(
                f"release create {release} {work}/kaikki-Spanish.jsonl.zst {work}/kaikki-es-traductions-en.jsonl.zst --target "
            ),
            created[0],
        )
        notes = (self.temp / "notes.md").read_text()
        self.assertIn("pair derives from kaikki's dumps of whole Wiktionary editions", notes)
        self.assertIn(
            "- `kaikki-en`: the dump https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz, regenerated Sat, 03 Oct "
            f"2026 08:24:38 GMT; decompressed sha256 {'d' * 64}, 25,614,284,530 B (2,981,058,381 B as served); derived: "
            "`kaikki-Spanish.jsonl`\n",
            notes,
        )
        self.assertIn("- `kaikki-es`: the dump https://kaikki.org/eswiktionary/raw-wiktextract-data.jsonl.gz, regenerated Fri", notes)
        self.assertNotIn("per-language extract", notes)
        self.assertIn("tables/es-en/pin.json", notes)

    def test_the_release_notes_of_a_legacy_or_a_mixed_pin_name_what_it_records(self):
        # A pin written before the dumps (es-fr's: an extract, and derived files whose dump was not
        # recorded), or one of each: the notes name each record as it is, the extract as one.
        release = "lingua-pack-sources-es-fr-2026.10.08"
        cases = {
            "legacy": self.own_record("es-fr", "2026.10.08", "kaikki-es-traductions.jsonl")["sources"],
            "mixed": {
                "kaikki-en": self.dumped(release, ["kaikki-Spanish.jsonl"]),
                "kaikki-es": self.dumped(release, ["kaikki-es-traductions.jsonl"], identity=False),
                "wordfreq": {"version": "3.1.1"},
            },
        }
        for case, sources in cases.items():
            with self.subTest(case):
                log = self.gh()
                log.unlink(missing_ok=True)
                self.pin("es-fr", {"snapshot": "2026.10.08", "sources": sources})
                done = self.release("es-fr")
                self.assertEqual(done.returncode, 0, done.stderr)
                created = [line for line in log.read_text().splitlines() if line.startswith("release create")]
                self.assertEqual(len(created), 1, created)
                notes = (self.temp / "notes.md").read_text()
                if case == "legacy":
                    self.assertIn("- `kaikki`: kaikki's per-language extract https://kaikki.org/x.jsonl", notes)
                    self.assertIn("- `kaikki-es`: the dump ?, regenerated on a date not recorded", notes)
                    self.assertIn("the dump's sha256 was not recorded", notes)
                else:
                    self.assertIn("- `kaikki-en`: the dump https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz", notes)
                    self.assertIn(f"decompressed sha256 {'d' * 64}", notes)
                    self.assertIn(
                        "the dump's sha256 was not recorded (a pin written before the dumps were); derived: "
                        "`kaikki-es-traductions.jsonl`",
                        notes,
                    )
                    self.assertNotIn("per-language extract", notes)

    def test_the_release_step_fails_when_its_assets_cannot_be_listed(self):
        # A record `assets` cannot read (a release named, but neither an extract's asset nor derived
        # files): the step stops, and nothing is published — it neither passes having published
        # nothing nor publishes half the list.
        log = self.gh()
        record = self.own_record("es-en", "2026.10.08", "kaikki-es-traductions-en.jsonl")
        del record["sources"]["kaikki"]["asset"]
        self.pin("es-en", record)
        done = self.release("es-en")
        self.assertNotEqual(done.returncode, 0, "a step whose asset list failed passed")
        self.assertIn("kaikki: pin.json names release lingua-pack-sources-es-en-2026.10.08 but neither an extract", done.stderr)
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
