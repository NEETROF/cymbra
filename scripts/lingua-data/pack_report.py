#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""What new tables change, against the committed ones (pin-lingua-pack-sources D6, D7).

    pack_report.py <committed tables> <new tables> [--pack <new pack>] [--max-loss 0.2]
                   [--identical [--pair <pair>]] [--measures <measures.json>]

Prints a Markdown report — per table, the keys added, removed and whose value changed, with
samples — and the new pack's size against its budget. Exits 1 when a table the committed set has
is missing from the new one, or loses more than `--max-loss` of its rows: an upstream format change
shows as a collapse, not as an error, so the monthly check has to look for one.

For a pair's folder whose studied language's folder stands beside it (`freq.tsv`), the report also
gives the gloss coverage — the share of the 5,000, 10,000 and 20,000 commonest lemmas glossed
(`gloss_coverage.measure`), new against committed — and, with `--measures`, what the pair's reducer
measured of its own tables and stores in no pack: en-es's translation-table share, named with the
pair (add-lingua-pack-en-es D4), beside the coverage.

The folders compared are a pair's (tables/<pair>/: glosses, senses, expressions) or a studied
language's (tables/<studied>/, split-lingua-pack-tables-by-language: forms, ranks, levels, readings
and dictionary words), which its reference pair's reduction writes. For a studied language's folder,
the report also names every pair reading it whose pack moves: the pairs beside the new folder whose
`pin.json` records another pack than beside the committed one — and fails, naming the pair and the
table, for a pair left behind: one whose `pin.json` beside the new folder still records a studied
table the new folder no longer holds as it was built on (add-lingua-pack-es-en D3).

With `--identical` (generalise-lingua-gloss-reducer D4), the new tables were reduced again expecting
no change — the rules moved, not what they make — and it also exits 1, naming the pair and the
file, when a table, a kept input (`tags.tsv`, `studied.json`), the dictionary words (`lexical.tsv`)
or NOTICE differs by one byte, when `manifest.json` differs anywhere but `meta.pack_version`, or when
`pin.json` differs anywhere but the record of the pack's sha256 and of the rules (`reducer`). The
pair is the new tables' folder, or for a studied language's folder its reference pair, the file
then named with its folder (`es-fr: es/level.tsv differs`).
"""

from __future__ import annotations

import argparse
import json
import sys
from dataclasses import dataclass, field
from pathlib import Path

from gloss_coverage import TOPS, measure
from pack_sources import (
    KEPT_INPUTS,
    LEXICAL,
    RECORDED_STUDIED,
    STUDIED_RECORD,
    PinError,
    get,
    is_pair_name,
    load,
    reference_of,
    sha256,
    studied_of,
)

# What each table maps, as the reader of the report thinks of it.
TABLES = {
    "forms.tsv": "form → lemma",
    "freq.tsv": "lemma → frequency rank",
    "gloss.tsv": "lemma → gloss",
    "level.tsv": "lemma → CEFR level",
    "mwe.tsv": "expression → gloss",
    "grammar.tsv": "form lemma reading → may be named as another word",
    "senses.tsv": "lemma → parts of speech of its senses",
    LEXICAL: "dictionary word",
}
# Tables with several lines per first field: the key is every field but the last.
MULTI_KEYED = {"grammar.tsv"}
# Tables of one column: each line is a key.
KEYS_ONLY = {LEXICAL}
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
        if path.name in KEYS_ONLY:
            if line.strip():
                rows[line.strip()] = ""
            continue
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


def not_identical(pair: str, old: Path, new: Path, folder: str = "") -> list[str]:
    """What moved, in tables reduced again expecting no change: one line per file, naming the pair,
    and the file with `folder` before it (a studied language's folder, which `pair` writes)."""
    out = []
    for name in IDENTICAL:
        before, after = old / name, new / name
        if not before.is_file() and not after.is_file():
            continue
        if not (before.is_file() and after.is_file()) or before.read_bytes() != after.read_bytes():
            out.append(f"{pair}: {folder}{name} differs")
    for name, may in MAY_MOVE.items():
        before, after = old / name, new / name
        if not before.is_file() and not after.is_file():
            continue
        if not (before.is_file() and after.is_file()):
            out.append(f"{pair}: {folder}{name} differs")
            continue
        paths = moved(*(json.loads(path.read_text(encoding="utf-8")) for path in (before, after))) - may
        if paths:
            out.append(f"{pair}: {folder}{name} differs at {', '.join(sorted('.'.join(map(str, p)) for p in paths))}")
    return out


def is_studied(folder: Path) -> bool:
    """A studied language's folder: it names its reference pair."""
    return (folder / STUDIED_RECORD).is_file()


def packs_moved(old: Path, new: Path) -> list[str]:
    """The pairs reading a studied language's folder whose pack moves: the pairs beside `new`
    studying its language whose `pin.json` records another pack than the same pair's beside `old`.
    A pair with no committed pin beside `old` was not part of the run, and is not named."""
    language = new.resolve().name
    out = []
    for folder in sorted(new.parent.iterdir()):
        if not folder.is_dir() or "-" not in folder.name or studied_of(folder.name) != language:
            continue
        before, after = old.parent / folder.name / "pin.json", folder / "pin.json"
        if not (before.is_file() and after.is_file()):
            continue
        try:
            shas = [get(load(pin), "pack.sha256") for pin in (before, after)]
        except PinError:
            continue
        if shas[0] != shas[1]:
            out.append(folder.name)
    return out


def left_behind(new: Path) -> list[str]:
    """The pairs reading a studied language's folder that its new tables left behind, each with the
    table (add-lingua-pack-es-en D3): a pair beside `new` studying its language whose `pin.json`
    records, as what its build read, another sha256 than the new folder's table — or a table one
    side lacks. `<pair>: <language>/<table>`, by pair then table. The reference, and a pair whose
    pin records nothing of the studied tables, are not named: their checks say so elsewhere."""
    language = new.resolve().name
    out = []
    for folder in sorted(new.parent.iterdir()):
        if not folder.is_dir() or "-" not in folder.name or studied_of(folder.name) != language:
            continue
        recorded = load(folder / "pin.json").get("studied")
        if not isinstance(recorded, dict) or not isinstance(recorded.get("tables"), dict):
            continue
        for name in RECORDED_STUDIED:
            got = sha256(new / name) if (new / name).is_file() else None
            if got != recorded["tables"].get(name):
                out.append(f"{folder.name}: {language}/{name}")
    return out


def coverage_of(folder: Path) -> list[float] | None:
    """A pair folder's gloss coverage, its ranks read from the studied language's folder beside it
    (`<root>/<studied>/freq.tsv`, as tables/ and a dry run's root lay them out); None when the
    folder is no pair's, or has no glosses or no studied folder beside it to rank them by."""
    name = folder.resolve().name
    if not is_pair_name(name) or is_studied(folder):
        return None
    studied = folder.resolve().parent / studied_of(name)
    if not (folder / "gloss.tsv").is_file() or not (studied / "freq.tsv").is_file():
        return None
    return measure(folder, studied=studied)


def coverage_line(old: Path, new: Path) -> str | None:
    """The new tables' gloss coverage of the commonest lemmas, against the committed tables'."""
    after = coverage_of(new)
    if after is None:
        return None
    tops = " / ".join(f"{top:,}" for top in TOPS)
    shares = " / ".join(f"{share} %" for share in after)
    before = coverage_of(old)
    was = f" (committed: {' / '.join(f'{share} %' for share in before)})" if before is not None else ""
    return f"Glossed, of the {tops} commonest lemmas: {shares}{was}."


def share_line(pair: str, measures: Path) -> str:
    """What the reducer measured of the glosses' sources (`measures.json`, add-lingua-pack-en-es
    D4): among the glossed lemmas of the `top` commonest, the share from a translation table —
    the direct, then the inverted — rather than from an entry; the pair named."""
    got = json.loads(measures.read_text(encoding="utf-8"))
    tables = ", ".join(f"{name} {len(got.get(name) or ())}" for name in ("direct", "inverted"))
    return (
        f"`{pair}`: of the {got['glossed']:,} glossed lemmas among the {got['top']:,} commonest, "
        f"**{got['share']} %** come from a translation table ({tables}) rather than from an entry."
    )


def render(
    diffs: list[TableDiff],
    pack: Path | None,
    issues: list[str],
    readers: list[str] | None = None,
    behind: list[str] | None = None,
    measured: list[str] = (),
) -> str:
    lines = ["## Dictionary update — what the new tables change", ""]
    lines.append("| Table | Rows before → after | Added | Removed | Changed |")
    lines.append("|---|---|---|---|---|")
    for d in diffs:
        lines.append(
            f"| `{d.name}` ({TABLES[d.name]}) | {d.before} → {d.after} | {len(d.added)} | {len(d.removed)} | {len(d.changed)} |"
        )
    if measured:
        lines += ["", *measured]
    if readers is not None:
        lines.append("")
        if readers:
            lines.append("Pairs reading these tables whose pack moves: " + ", ".join(f"`{r}`" for r in readers) + ".")
        else:
            lines.append("No pack of a pair reading these tables moves.")
    if behind:
        lines.append(
            "Pairs left behind — their pin records another table than the one here: "
            + ", ".join(f"`{b}`" for b in behind)
            + ". Each is reduced again from its own pinned sources after the reference."
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
    ap.add_argument(
        "--pair",
        help="the pair the tables are (default: the new tables' folder, or the reference a studied language's names)",
    )
    ap.add_argument(
        "--measures",
        type=Path,
        help="what the pair's reducer measured of its tables (work/<pair>/measures.json): shown beside the coverage",
    )
    a = ap.parse_args(argv)
    diffs = compare_dirs(a.old, a.new)
    issues = problems(diffs, a.old, a.new, a.max_loss)
    studied = is_studied(a.new)
    pair = a.pair or (reference_of(a.new) if studied else None) or a.new.resolve().name
    folder = f"{a.new.resolve().name}/" if studied else ""
    moved_files = not_identical(pair, a.old, a.new, folder) if a.identical else []
    issues += moved_files
    if a.pack is not None and a.pack.is_file() and a.pack.stat().st_size > BUDGET:
        issues.append(f"the pack is {a.pack.stat().st_size:,} B, over its budget")
    behind = left_behind(a.new) if studied else []
    issues += [f"{line} moved, and that pair's pack was not recorded again" for line in behind]
    measured = [line for line in (coverage_line(a.old, a.new),) if line]
    if a.measures is not None:
        measured.append(share_line(pair, a.measures))
    sys.stdout.write(render(diffs, a.pack, issues, packs_moved(a.old, a.new) if studied else None, behind, measured))
    if a.identical and not moved_files:
        sys.stdout.write(
            "\nReduced again expecting no change: every table, kept input and NOTICE is byte for byte as "
            "committed; `manifest.json` moves only in `pack_version`, `pin.json` only in the pack's sha256 "
            "and the rules' record.\n"
        )
    return 1 if issues else 0


if __name__ == "__main__":
    sys.exit(main())
