#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""What new tables change, against the committed ones (pin-lingua-pack-sources D6, D7).

    pack_report.py <committed tables> <new tables> [--pack <new pack>] [--max-loss 0.2]

Prints a Markdown report — per table, the keys added, removed and whose value changed, with
samples — and the new pack's size against its budget. Exits 1 when a table the committed set has
is missing from the new one, or loses more than `--max-loss` of its rows: an upstream format change
shows as a collapse, not as an error, so the monthly check has to look for one.
"""

from __future__ import annotations

import argparse
import sys
from dataclasses import dataclass, field
from pathlib import Path

# What each table maps, as the reader of the report thinks of it.
TABLES = {
    "forms.tsv": "form → lemma",
    "freq.tsv": "lemma → frequency rank",
    "gloss.tsv": "lemma → gloss",
    "level.tsv": "lemma → CEFR level",
    "mwe.tsv": "expression → gloss",
}
BUDGET = 5 * 1024 * 1024  # the pack's size budget (lingua-data-packs)
SAMPLES = 8
# Ranks move by one for every lemma above an insertion: a rank change is news past this.
RANK_TOLERANCE = 500


def read_table(path: Path) -> dict[str, str]:
    rows: dict[str, str] = {}
    if not path.is_file():
        return rows
    for line in path.read_text(encoding="utf-8").splitlines():
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
    a = ap.parse_args(argv)
    diffs = compare_dirs(a.old, a.new)
    issues = problems(diffs, a.old, a.new, a.max_loss)
    if a.pack is not None and a.pack.is_file() and a.pack.stat().st_size > BUDGET:
        issues.append(f"the pack is {a.pack.stat().st_size:,} B, over its budget")
    sys.stdout.write(render(diffs, a.pack, issues))
    return 1 if issues else 0


if __name__ == "__main__":
    sys.exit(main())
