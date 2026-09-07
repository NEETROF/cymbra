#!/usr/bin/env python3
"""Fail when a buildable unit is watched by no workflow.

The question this answers is narrow and deliberate: **if I change only this unit, does
any workflow do real work?**

The repo answers that in two ways, so this reads both:

* a top-level `on.push.paths` / `on.pull_request.paths` filter, which decides whether the
  workflow starts at all (`backend-image`, `backend-it`, `crawler-image`,
  `openspec-archive`);
* a `dorny/paths-filter` step in a `changes` job, for workflows that always start and then
  gate their real jobs on the outcome (`flutter`, `rust`, `build`, `sonar`, `site`,
  `back-office`, `frb-codegen`). Reading only the top-level filter would report these as
  watching nothing, which is how the first version of this script got it wrong.

A workflow with neither (commitlint, codeql, release-please) runs on everything and is
ignored: counting it would make the check vacuous — it watches every unit and verifies
none.

Coverage is derived from the workflows themselves rather than from a checked-in manifest.
A manifest is a second source of truth that drifts: it keeps claiming a unit is covered
after the workflow that covered it changed. Reading the filters cannot drift, and adding
a workflow that selects a unit makes this check pass with nothing else to update.

Usage:  python3 scripts/check_ci_units.py [--list]
"""

from __future__ import annotations

import fnmatch
import glob
import os
import sys

import yaml

# Directories whose immediate children are independently buildable units.
UNIT_PARENTS = ("apps", "packages", "crates")

# Units that are deliberately not watched, each with the reason. Keep this empty if you
# can: an entry here is a unit whose changes ship without CI.
EXEMPT: dict[str, str] = {}

WORKFLOW_DIR = ".github/workflows"


def units() -> list[str]:
    found = []
    for parent in UNIT_PARENTS:
        if not os.path.isdir(parent):
            continue
        for name in sorted(os.listdir(parent)):
            path = os.path.join(parent, name)
            if os.path.isdir(path) and not name.startswith("."):
                found.append(path)
    return found


def path_filters(workflow: dict) -> set[str]:
    """Every glob that makes this workflow do work. Empty = runs on everything."""
    globs: set[str] = set()

    # PyYAML parses the bare key `on` as the boolean True (YAML 1.1).
    triggers = workflow.get(True, workflow.get("on")) or {}
    if isinstance(triggers, dict):
        for event in ("push", "pull_request"):
            spec = triggers.get(event)
            if isinstance(spec, dict):
                globs.update(spec.get("paths") or [])

    # `dorny/paths-filter` carries its filters as a YAML document in a string.
    for job in (workflow.get("jobs") or {}).values():
        if not isinstance(job, dict):
            continue
        for step in job.get("steps") or []:
            if not isinstance(step, dict) or "paths-filter" not in str(step.get("uses", "")):
                continue
            try:
                declared = yaml.safe_load(step.get("with", {}).get("filters", "")) or {}
            except yaml.YAMLError:
                continue
            values = declared.values() if isinstance(declared, dict) else []
            for value in values:
                globs.update(value if isinstance(value, list) else [value])

    return {g for g in globs if isinstance(g, str)}


# Build output and vendored trees: never the reason a unit is considered watched.
SKIP_DIRS = {
    ".git", ".dart_tool", ".yarn", "node_modules", "target", "build", "dist",
    "__pycache__", ".gradle", "Pods", ".venv",
}


def sample_files(unit: str, limit: int = 400) -> list[str]:
    """Real file paths inside a unit.

    Matching against *actual* files rather than a hypothetical probe matters: an earlier
    version probed `<unit>/rust/src/probe.rs`, which made every app under `apps/` match
    the `apps/*/rust/**` glob of `rust`/`sonar` — so a brand-new app with no Rust in it
    was reported as watched, and the guard passed on exactly the case it exists for.
    """
    found: list[str] = []
    for root, dirs, names in os.walk(unit):
        dirs[:] = [d for d in dirs if d not in SKIP_DIRS and not d.startswith(".")]
        for name in names:
            found.append(os.path.join(root, name))
            if len(found) >= limit:
                return found
    return found


def watchers() -> dict[str, set[str]]:
    """unit -> names of the workflows that do work when it changes."""
    result: dict[str, set[str]] = {unit: set() for unit in units()}
    real_files = {unit: sample_files(unit) for unit in result}
    for path in sorted(glob.glob(f"{WORKFLOW_DIR}/*.yml")):
        with open(path) as handle:
            workflow = yaml.safe_load(handle) or {}
        globs = path_filters(workflow)
        if not globs:
            continue  # runs on everything; verifies nothing in particular
        name = os.path.basename(path)[: -len(".yml")]
        for unit, files in real_files.items():
            if any(fnmatch.fnmatch(f, g) for f in files for g in globs):
                result[unit].add(name)
    return result


def main() -> int:
    found = watchers()
    listing = "--list" in sys.argv

    uncovered = [u for u, w in found.items() if not w and u not in EXEMPT]

    if listing or uncovered:
        for unit, names in found.items():
            if unit in EXEMPT:
                mark, detail = "exempt", EXEMPT[unit]
            elif names:
                mark, detail = "ok", ", ".join(sorted(names))
            else:
                mark, detail = "UNWATCHED", "no workflow starts when this changes"
            print(f"{mark:>9}  {unit:<28} {detail}")

    if not uncovered:
        if listing:
            print(f"\n{len(found)} units, all watched.")
        return 0

    print(
        "\nERROR: the units above start no workflow. A change confined to one of them "
        "would merge with no CI at all.\n"
        "Fix it by adding the unit to the paths filter of the workflow that should check "
        "it — or, if it truly needs none, add it to EXEMPT in this file with the reason.",
        file=sys.stderr,
    )
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
