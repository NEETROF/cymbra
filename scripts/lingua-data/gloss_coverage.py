#!/usr/bin/env python3
# Copyright 2026 NEETROF
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""How much of each shipped pair's commonest vocabulary has a French gloss (add-site-lingua-spanish-pages D2, D3).

    gloss_coverage.py            # print the figures
    gloss_coverage.py --write    # write them where the site's Lingua pages read them
    gloss_coverage.py --check    # exit 1 when the written figures no longer match the tables

For each pair `apps/lingua-extension/packs.json` ships: of the N lemmas its `freq.tsv` ranks
commonest, the share its `gloss.tsv` glosses, for N = 5,000, 10,000 and 20,000, in percent to one
decimal. The same measure for every pair, from the committed tables, so the site compares them like
for like. Stdlib only: it runs on the Python of the lingua-data unit tests.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

HERE = Path(__file__).resolve().parent
ROOT = HERE.parent.parent
PACKS = ROOT / "apps/lingua-extension/packs.json"
TABLES = HERE / "tables"
SITE_DATA = ROOT / "apps/site/src/data/lingua-coverage.json"

# Not 1,000: each frequency list's first thousand carries noise of its own — English contractions
# (« it's »), Spanish corpora's English words and abbreviations (« the », « etc ») — so that row
# would compare the lists, not the dictionaries (D2).
TOPS = (5_000, 10_000, 20_000)


def read_keys(path: Path) -> dict[str, str]:
    """A table's first column mapped to its second, skipping blank and comment lines."""
    rows: dict[str, str] = {}
    with path.open(encoding="utf-8") as f:
        for line in f:
            if not line.strip() or line.startswith("#"):
                continue
            key, _, rest = line.rstrip("\n").partition("\t")
            rows[key] = rest
    return rows


def measure(tables: Path, tops: tuple[int, ...] = TOPS) -> list[float]:
    """One pair's glossed share of its `tops` commonest lemmas, in percent to one decimal."""
    ranks = read_keys(tables / "freq.tsv")
    glossed = read_keys(tables / "gloss.tsv")
    ranked = sorted(ranks, key=lambda lemma: int(ranks[lemma]))
    shares = []
    for top in tops:
        commonest = ranked[:top]
        shares.append(round(100 * sum(1 for lemma in commonest if lemma in glossed) / len(commonest), 1))
    return shares


def shipped_pairs(packs: Path = PACKS) -> list[str]:
    return json.loads(packs.read_text(encoding="utf-8"))["pairs"]


def figures(tables: Path = TABLES, packs: Path = PACKS) -> dict:
    """Every shipped pair's figures, as the site reads them."""
    return {"tops": list(TOPS), "glossed": {pair: measure(tables / pair) for pair in shipped_pairs(packs)}}


def render(data: dict) -> str:
    return json.dumps(data, indent=2) + "\n"


def shown(path: Path) -> str:
    """A path as the messages name it: from the repository root when it is inside it."""
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true", help=f"write {shown(SITE_DATA)}")
    mode.add_argument("--check", action="store_true", help="exit 1 when the written figures are stale")
    args = parser.parse_args(argv)
    text = render(figures())
    if args.write:
        SITE_DATA.parent.mkdir(parents=True, exist_ok=True)
        SITE_DATA.write_text(text, encoding="utf-8")
        return 0
    if args.check:
        written = SITE_DATA.read_text(encoding="utf-8") if SITE_DATA.exists() else ""
        if written != text:
            print(
                f"{shown(SITE_DATA)} no longer matches the tables: run scripts/lingua-data/gloss_coverage.py --write",
                file=sys.stderr,
            )
            return 1
        return 0
    sys.stdout.write(text)
    return 0


if __name__ == "__main__":
    sys.exit(main())
