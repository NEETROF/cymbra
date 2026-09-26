#!/usr/bin/env python3
"""Whether a completed OpenSpec change can be archived now, or must wait for another one.

`openspec archive` folds a change's delta into openspec/specs/. A MODIFIED or REMOVED
requirement must already be in the spec, or the archive aborts ("not found / Aborted") and the
change stays in openspec/changes. Two very different things look alike there:

- the requirement was renamed in the spec since the delta was written: the delta is stale and
  must be retargeted — an error;
- the requirement is ADDED by another change that is still open: the delta is right, it just
  archives after that change (add-lingua-expression-table waits for add-lingua-phrase-gloss,
  its design's D7) — nothing to repair.

The openspec-archive workflow reported both as errors, on every push to main. This tells them
apart, by reading the delta files, before any archive runs.

One order cannot be read from the deltas: two open changes that both MODIFY the same requirement.
Whichever archives last wins, since a MODIFIED block replaces the requirement whole — and the
workflow walks changes newest first. A change that must follow another says so in its
`.openspec.yaml`, as a list the `openspec` CLI ignores:

    archiveAfter:
      - add-lingua-translation-android

A listed change still in openspec/changes/ is waited for exactly as an ADDED dependency is
(add-lingua-translation-safari, design D5).

Usage: openspec_archive_order.py <change> [--root DIR]
Exit status: 0 ready, 10 waits for another open change (their names on stdout, one per line),
1 names a requirement no spec and no open change holds (the headers on stdout).
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

READY, WAITS, STALE = 0, 10, 1

SECTION = re.compile(r"^## (ADDED|MODIFIED|REMOVED|RENAMED) Requirements\s*$")
HEADER = re.compile(r"^### Requirement:\s*(.+?)\s*$")
RENAMED_FROM = re.compile(r"^\s*-?\s*FROM:\s*`?###\s*Requirement:\s*(.+?)`?\s*$")
RENAMED_TO = re.compile(r"^\s*-?\s*TO:\s*`?###\s*Requirement:\s*(.+?)`?\s*$")


def delta_headers(spec: Path) -> dict[str, set[str]]:
    """The requirement names a delta spec touches, by operation."""
    ops: dict[str, set[str]] = {"ADDED": set(), "MODIFIED": set(), "REMOVED": set(), "RENAMED_FROM": set()}
    section = None
    for line in spec.read_text(encoding="utf-8").splitlines():
        if m := SECTION.match(line):
            section = m.group(1)
            continue
        if section == "RENAMED":
            if m := RENAMED_FROM.match(line):
                ops["RENAMED_FROM"].add(m.group(1))
            elif m := RENAMED_TO.match(line):
                ops["ADDED"].add(m.group(1))
            continue
        if section and (m := HEADER.match(line)):
            ops[section].add(m.group(1))
    return ops


def archive_after(change_dir: Path) -> list[str]:
    """The changes `.openspec.yaml` says this one archives after (a flat YAML list, read by hand)."""
    meta = change_dir / ".openspec.yaml"
    if not meta.is_file():
        return []
    names: list[str] = []
    listing = False
    for line in meta.read_text(encoding="utf-8").splitlines():
        if re.match(r"^archiveAfter:\s*$", line):
            listing = True
            continue
        if listing:
            if m := re.match(r"^\s+-\s*['\"]?([\w.-]+)['\"]?\s*$", line):
                names.append(m.group(1))
            elif line.strip():
                listing = False
    return names


def spec_headers(spec: Path) -> set[str]:
    if not spec.is_file():
        return set()
    return {m.group(1) for line in spec.read_text(encoding="utf-8").splitlines() if (m := HEADER.match(line))}


def check(root: Path, change: str) -> tuple[int, list[str]]:
    changes = root / "openspec" / "changes"
    specs = root / "openspec" / "specs"
    others = [d for d in changes.iterdir() if d.is_dir() and d.name not in (change, "archive")]
    providers: set[str] = set()
    stale: list[str] = []
    for delta in sorted((changes / change / "specs").glob("*/spec.md")):
        capability = delta.parent.name
        ops = delta_headers(delta)
        # A delta may rename a requirement and modify it under its new name: that name is its own.
        present = spec_headers(specs / capability / "spec.md") | ops["ADDED"]
        for name in sorted(ops["MODIFIED"] | ops["REMOVED"] | ops["RENAMED_FROM"]):
            if name in present:
                continue
            added_by = [
                o.name
                for o in others
                if (o / "specs" / capability / "spec.md").is_file()
                and name in delta_headers(o / "specs" / capability / "spec.md")["ADDED"]
            ]
            if added_by:
                providers.update(added_by)
            else:
                stale.append(f"{capability}: {name}")
    if stale:
        return STALE, stale
    providers.update(name for name in archive_after(changes / change) if (changes / name).is_dir())
    if providers:
        return WAITS, sorted(providers)
    return READY, []


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("change")
    parser.add_argument("--root", type=Path, default=Path("."))
    args = parser.parse_args(argv)
    status, names = check(args.root, args.change)
    for name in names:
        print(name)
    return status


if __name__ == "__main__":
    sys.exit(main())
