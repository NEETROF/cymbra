#!/usr/bin/env python3
# Copyright 2026 NEETROF
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""How much of each shipped pair's commonest vocabulary has a gloss (add-site-lingua-spanish-pages D2, D3).

    gloss_coverage.py            # print the figures
    gloss_coverage.py --write    # write them where the site's Lingua pages read them
    gloss_coverage.py --check    # exit 1 when the written figures no longer match the tables
    gloss_coverage.py --pair es-en [--floor 87.6 77.2 63.7]   # one pair, shipped or not; exit 1 under its floor

For each pair `apps/lingua-extension/packs.json` ships: of the N lemmas its `freq.tsv` ranks
commonest, the share its `gloss.tsv` glosses, for N = 5,000, 10,000 and 20,000, in percent to one
decimal. The same measure for every pair, from the committed tables, so the site compares them like
for like. Stdlib only: it runs on the Python of the lingua-data unit tests.

`--pair` measures one committed pair whether or not it ships, and prints nothing the site reads;
it fails under the pair's floor — the share each of the three tops must reach, `FLOORS`, or
`--floor` given (add-lingua-pack-es-en D6: es-en is held to es-fr's published figures by the reduce
job, before it ships and is published; add-lingua-pack-en-es D3: en-es to a floor the owner sets;
add-lingua-pack-fr-en D4: fr-en likewise).
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

# The share of each of `TOPS` a pair not yet shipped must gloss, held by the reduce job
# (add-lingua-pack-es-en D6): es-en, es-fr's published figures when es-en was proposed. Fixed here,
# not read from the site's file, so that es-fr's next update does not move es-en's floor.
# en-es (add-lingua-pack-en-es D3, the programme's risk 5): the study's figures less two points —
# proposed, owner settles (task 5.1) here, the one place: the reduce job passes no `--floor`, and
# the tests hold the committed tables to this entry.
# fr-en (add-lingua-pack-fr-en D4): en-es's rule, the study's figures (93.9 / 87.1 / 76.4 %, the
# programme's) less two points — no French pair is published to hold it to; proposed, the owner
# settles (task 6.1) here, the one place.
FLOORS = {"es-en": (87.6, 77.2, 63.7), "en-es": (91.4, 83.2, 69.9), "fr-en": (91.9, 85.1, 74.4)}


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


def measure(tables: Path, tops: tuple[int, ...] = TOPS, studied: Path | None = None) -> list[float]:
    """One pair's glossed share of its `tops` commonest lemmas, in percent to one decimal: the
    ranks from its studied language's folder (split-lingua-pack-tables-by-language), the glosses
    from its own."""
    ranks = read_keys((studied or tables) / "freq.tsv")
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
    return {
        "tops": list(TOPS),
        # A pair's ranks are its studied language's, kept once in tables/<studied>/; its glosses its own.
        "glossed": {pair: measure(tables / pair, studied=tables / pair.split("-")[0]) for pair in shipped_pairs(packs)},
    }


def render(data: dict) -> str:
    return json.dumps(data, indent=2) + "\n"


def shown(path: Path) -> str:
    """A path as the messages name it: from the repository root when it is inside it."""
    try:
        return str(path.relative_to(ROOT))
    except ValueError:
        return str(path)


def pair_figures(pair: str, tables: Path) -> dict:
    """One committed pair's figures, shipped or not, in the shape the site reads."""
    if not (tables / pair / "gloss.tsv").is_file():
        raise SystemExit(f"error: {shown(tables / pair)} holds no committed tables (gloss.tsv)")
    return {"tops": list(TOPS), "glossed": {pair: measure(tables / pair, studied=tables / pair.split("-")[0])}}


def under_floor(pair: str, shares: list[float], floor: list[float], tops: tuple[int, ...] = TOPS) -> list[str]:
    """What falls short: one line per top whose share is under its floor."""
    return [
        f"{pair}: {share} % of the {top:,} commonest lemmas are glossed, under the floor of {least} %"
        for top, share, least in zip(tops, shares, floor)
        if share < least
    ]


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    mode = parser.add_mutually_exclusive_group()
    mode.add_argument("--write", action="store_true", help=f"write {shown(SITE_DATA)}")
    mode.add_argument("--check", action="store_true", help="exit 1 when the written figures are stale")
    mode.add_argument("--pair", help="measure this committed pair alone, shipped or not")
    parser.add_argument(
        "--floor",
        nargs=len(TOPS),
        type=float,
        metavar="PERCENT",
        help=f"with --pair: the share each of the {len(TOPS)} tops must reach (default: the pair's FLOORS); exit 1 under it",
    )
    args = parser.parse_args(argv)
    if args.floor is not None and args.pair is None:
        parser.error("--floor needs --pair")
    if args.pair is not None:
        data = pair_figures(args.pair, TABLES)
        sys.stdout.write(render(data))
        floor = args.floor if args.floor is not None else FLOORS.get(args.pair)
        short = under_floor(args.pair, data["glossed"][args.pair], list(floor)) if floor is not None else []
        for line in short:
            print(line, file=sys.stderr)
        return 1 if short else 0
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
