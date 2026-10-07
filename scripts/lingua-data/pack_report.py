#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""What new tables change, against the committed ones (pin-lingua-pack-sources D6, D7).

    pack_report.py <committed tables> <new tables> [--pack <new pack>] [--max-loss 0.2]
                   [--identical [--pair <pair>]]

Prints a Markdown report — per table, the keys added, removed and whose value changed, with
samples — and the new pack's size against its budget. Exits 1 when a table the committed set has
is missing from the new one, or loses more than `--max-loss` of its rows: an upstream format change
shows as a collapse, not as an error, so the monthly check has to look for one.

With `--identical` (generalise-lingua-gloss-reducer D4), the new tables were reduced again expecting
no change — the rules moved, not what they make — and it also exits 1, naming the pair and the
file, when a table, a kept input (`tags.tsv`, `lexical.tsv`) or NOTICE differs by one byte, when
`manifest.json` differs anywhere but `meta.pack_version`, or when `pin.json` differs anywhere but
the record of the pack's sha256 and of the rules (`reducer`). The pair is the new tables' folder.
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

from pack_sources import KEPT_INPUTS

# What each table maps, as the reader of the report thinks of it.
TABLES = {
    "forms.tsv": "form → lemma",
    "freq.tsv": "lemma → frequency rank",
    "gloss.tsv": "lemma → gloss",
    "level.tsv": "lemma → CEFR level",
    "mwe.tsv": "expression → gloss",
    "grammar.tsv": "form lemma reading → may be named as another word",
    "senses.tsv": "lemma → parts of speech of its senses",
}
# Tables with several lines per first field: the key is every field but the last.
MULTI_KEYED = {"grammar.tsv"}
BUDGET = 5 * 1024 * 1024  # the pack's size budget (lingua-data-packs)
SAMPLES = 8
# Ranks move by one for every lemma above an insertion: a rank change is news past this.
RANK_TOLERANCE = 500

# Reduced again expecting no change (--identical): the files that may not move by one byte, and
# what may move in the two records — the pack's version, which names the rules' digest, and the
# record of the pack those rules build and of the rules themselves.
IDENTICAL = (*TABLES, *KEPT_INPUTS, "NOTICE")
MAY_MOVE = {
    "manifest.json": {("meta", "pack_version")},
    "pin.json": {("pack", "sha256"), ("reducer", "sha256"), ("reducer", "files")},
}


def read_table(path: Path) -> dict[str, str]:
    rows: dict[str, str] = {}
    if not path.is_file():
        return rows
    for line in path.read_text(encoding="utf-8").splitlines():
        if path.name in MULTI_KEYED:
            key, sep, value = line.rpartition("\t")
            key = key.replace("\t", " ")
        else:
            key, sep, value = line.partition("\t")
        if sep:
            rows[key] = value
    return rows


@dataclass
class TableDiff:
    name: str
    before: int
    after: int
    added: list[str] = field(default_factory=list)
    removed: list[str] = field(default_factory=list)
    changed: list[tuple[str, str, str]] = field(default_factory=list)

    @property
    def loss(self) -> float:
        return 0.0 if self.before == 0 else max(0.0, (self.before - self.after) / self.before)


def compare(name: str, old: dict[str, str], new: dict[str, str]) -> TableDiff:
    diff = TableDiff(name, len(old), len(new))
    diff.added = sorted(new.keys() - old.keys())
    diff.removed = sorted(old.keys() - new.keys())
    for key in sorted(old.keys() & new.keys()):
        a, b = old[key], new[key]
        if a == b:
            continue
        if name == "freq.tsv" and a.isdigit() and b.isdigit() and abs(int(a) - int(b)) <= RANK_TOLERANCE:
            continue
        diff.changed.append((key, a, b))
    return diff


def compare_dirs(old: Path, new: Path) -> list[TableDiff]:
    return [
        compare(name, read_table(old / name), read_table(new / name))
        for name in TABLES
        if (old / name).is_file() or (new / name).is_file()
    ]


def problems(diffs: list[TableDiff], old: Path, new: Path, max_loss: float) -> list[str]:
    out = []
    for d in diffs:
        if (old / d.name).is_file() and not (new / d.name).is_file():
            out.append(f"{d.name} is missing from the new tables")
        elif d.loss > max_loss:
            out.append(f"{d.name} lost {d.loss:.0%} of its rows ({d.before} → {d.after})")
    return out


def moved(old, new, at: tuple = ()) -> set[tuple]:
    """The paths of the values that differ between two JSON documents: a key one side lacks, or a
    value — a list is one value — that is not the same."""
    if isinstance(old, dict) and isinstance(new, dict):
        out = set()
        for key in old.keys() | new.keys():
            if key not in old or key not in new:
                out.add((*at, key))
            else:
                out |= moved(old[key], new[key], (*at, key))
        return out
    return set() if old == new else {at}


def not_identical(pair: str, old: Path, new: Path) -> list[str]:
    """What moved, in tables reduced again expecting no change: one line per file, naming the pair."""
    out = []
    for name in IDENTICAL:
        before, after = old / name, new / name
        if not before.is_file() and not after.is_file():
            continue
        if not (before.is_file() and after.is_file()) or before.read_bytes() != after.read_bytes():
            out.append(f"{pair}: {name} differs")
    for name, may in MAY_MOVE.items():
        before, after = old / name, new / name
        if not before.is_file() and not after.is_file():
            continue
        if not (before.is_file() and after.is_file()):
            out.append(f"{pair}: {name} differs")
            continue
        paths = moved(*(json.loads(path.read_text(encoding="utf-8")) for path in (before, after))) - may
        if paths:
            out.append(f"{pair}: {name} differs at {', '.join(sorted('.'.join(map(str, p)) for p in paths))}")
    return out


def render(diffs: list[TableDiff], pack: Path | None, issues: list[str]) -> str:
    lines = ["## Dictionary update — what the new tables change", ""]
    lines.append("| Table | Rows before → after | Added | Removed | Changed |")
    lines.append("|---|---|---|---|---|")
    for d in diffs:
        lines.append(
            f"| `{d.name}` ({TABLES[d.name]}) | {d.before} → {d.after} | {len(d.added)} | {len(d.removed)} | {len(d.changed)} |"
        )
    if pack is not None and pack.is_file():
        size = pack.stat().st_size
        lines += ["", f"Pack: **{size:,} B**, {size / BUDGET:.1%} of the {BUDGET // (1024 * 1024)} MiB budget."]
    if issues:
        lines += ["", "**Failed:**", *[f"- {i}" for i in issues]]
    for d in diffs:
        if not (d.added or d.removed or d.changed):
            continue
        lines += ["", f"### `{d.name}`", ""]
        if d.added:
            lines.append("Added: " + ", ".join(f"`{k}`" for k in d.added[:SAMPLES]) + (" …" if len(d.added) > SAMPLES else ""))
        if d.removed:
            lines.append(
                "Removed: " + ", ".join(f"`{k}`" for k in d.removed[:SAMPLES]) + (" …" if len(d.removed) > SAMPLES else "")
            )
        for key, a, b in d.changed[:SAMPLES]:
            lines.append(f"- `{key}`: {a} → {b}")
        if len(d.changed) > SAMPLES:
            lines.append(f"- … and {len(d.changed) - SAMPLES} more")
    return "\n".join(lines) + "\n"


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    ap.add_argument("old", type=Path)
    ap.add_argument("new", type=Path)
    ap.add_argument("--pack", type=Path)
    ap.add_argument("--max-loss", type=float, default=0.2)
    ap.add_argument("--identical", action="store_true", help="fail on any byte the rules were not meant to move")
    ap.add_argument("--pair", help="the pair the tables are (default: the new tables' folder)")
    a = ap.parse_args(argv)
    diffs = compare_dirs(a.old, a.new)
    issues = problems(diffs, a.old, a.new, a.max_loss)
    moved_files = not_identical(a.pair or a.new.resolve().name, a.old, a.new) if a.identical else []
    issues += moved_files
    if a.pack is not None and a.pack.is_file() and a.pack.stat().st_size > BUDGET:
        issues.append(f"the pack is {a.pack.stat().st_size:,} B, over its budget")
    sys.stdout.write(render(diffs, a.pack, issues))
    if a.identical and not moved_files:
        sys.stdout.write(
            "\nReduced again expecting no change: every table, kept input and NOTICE is byte for byte as "
            "committed; `manifest.json` moves only in `pack_version`, `pin.json` only in the pack's sha256 "
            "and the rules' record.\n"
        )
    return 1 if issues else 0


if __name__ == "__main__":
    sys.exit(main())
