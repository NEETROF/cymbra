#!/usr/bin/env python3
# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not
# use this file except in compliance with the License. You may obtain a copy of
# the License at http://www.apache.org/licenses/LICENSE-2.0
"""The raw sources of a data pack, pinned (pin-lingua-pack-sources D2, D3).

`tables/<pair>/pin.json` records where the committed tables came from and what they build:

- `snapshot`: the day the sources were read (`2026.09.26`); it is also the pack's `pack_version`
  when the tables come from an update, and names it with the reducer's rules when they come from a
  re-reduction of the same sources (`2026.09.26+1dff19c`).
- `pack`: sha256 and size of the pack the tables build — every lane must obtain exactly that.
- `reducer`: the files the tables were reduced by — the pair's `reduce-<pair>.py` and the shared
  `reduce_*.py` modules importing it loads (`rule_files`) — and one sha256 over all of them.
- `sources`: each raw source, pinned — CEFR-J and Octanove at a commit of their own repository
  and by sha256; ESDB (the inflections) built at a commit of en-wl/wordlist, by the sha256 of its
  exported `scowl.txt`; kaikki (regenerated upstream every day) as our own snapshot, a
  zstd-compressed GitHub Release asset, checked by the sha256 of its decompressed bytes; the files
  a pair derives from kaikki's dumps of whole Wiktionary editions, kept the same way beside it;
  wordfreq by version (it is its own snapshot; requirements-reduce.txt pins it by hash).

Stdlib only: the build mode reads this record too, and needs no Python package.

    pack_sources.py fetch-pinned  --pin P --work W    # the recorded bytes, checked, into W
    pack_sources.py fetch-live    --pin P --work W [--snapshot D]  # today's bytes, recorded in P
    pack_sources.py record-build  --pin P --pack F --reducer R     # what the tables build
    pack_sources.py check-pack    --pin P --pack F                 # a pack, against the record
    pack_sources.py check-reducer --pin P --reducer R              # the rules, against the record
    pack_sources.py get           --pin P KEY                      # e.g. snapshot, pack.sha256
    pack_sources.py assets        --pin P                          # the snapshot's release assets
    pack_sources.py keep          --from D --to T                  # a studied folder, into T
    pack_sources.py split         --work W --tables T --pair P     # a reduction, into T's folders
    pack_sources.py pairs         --tables T [--after P]           # the pairs, references first
    pack_sources.py moved         --tables T < git-status          # each moved file, by its writer

The tables are kept once per studied language (split-lingua-pack-tables-by-language, M24):
`tables/<studied>/` holds the studied side (`STUDIED_TABLES`, the pinned tag pool `tags.tsv` and
the dictionary words `lexical.tsv`) and `studied.json`, which names the language's reference pair;
`tables/<pair>/` holds the native side (`PAIR_TABLES`), `pin.json` and README.md. A pair's reducer
writes both sides into its work folder; `split` files them, and only the reference pair's reduction
writes the studied folder. The reference's `pin.json` is the studied tables' provenance.
"""

from __future__ import annotations

import argparse
import datetime
import gzip
import hashlib
import json
import shutil
import subprocess
import sys
import urllib.request
from pathlib import Path

REPOSITORY = "NEETROF/cymbra"

# The sources each pair reads at a commit — en-fr's read on 2026-09-25/26. The URLs name a commit,
# never a branch: the AGID URL the pipeline once used named `master`, a branch en-wl/wordlist no
# longer has, and worked through a leftover redirect. At a commit, a URL means the same bytes for
# as long as the repository exists.
PINNED = {
    "en-fr": {
        "cefrj": {
            "file": "cefrj-vocabulary-profile-1.5.csv",
            "url": "https://raw.githubusercontent.com/openlanguageprofiles/olp-en-cefrj/"
            "d4e45b75b38f27b30dfc5c44d8c571aec7e7092f/cefrj-vocabulary-profile-1.5.csv",
        },
        "octanove": {
            "file": "octanove-vocabulary-profile-c1c2-1.0.csv",
            "url": "https://raw.githubusercontent.com/openlanguageprofiles/olp-en-cefrj/"
            "d4e45b75b38f27b30dfc5c44d8c571aec7e7092f/octanove-vocabulary-profile-c1c2-1.0.csv",
        },
    },
    # UD Spanish-GSD, read for its counts of each form under each lemma (add-lingua-spanish-forms-
    # tables D2), at the commit read on 2026-10-03.
    "es-fr": {
        "gsd-train": {
            "file": "es_gsd-ud-train.conllu",
            "url": "https://raw.githubusercontent.com/UniversalDependencies/UD_Spanish-GSD/"
            "267f3530d4f122ee85d1891800211a06dfb79347/es_gsd-ud-train.conllu",
        },
        "gsd-dev": {
            "file": "es_gsd-ud-dev.conllu",
            "url": "https://raw.githubusercontent.com/UniversalDependencies/UD_Spanish-GSD/"
            "267f3530d4f122ee85d1891800211a06dfb79347/es_gsd-ud-dev.conllu",
        },
    },
}
# ESDB, the English Speller Database (switch-lingua-inflections-to-esdb): not a file but a database
# its repository builds; `scowl.txt` is its export. Built at the commit of `rel-2026.02.25`, with
# the pinned interpreter — the repository's Makefile would call /usr/bin/python3, whatever it is.
ESDB = {
    "en-fr": {
        "file": "scowl.txt",
        "repository": "https://github.com/en-wl/wordlist.git",
        "tag": "rel-2026.02.25",
        "commit": "7e99edab8e32f9f9ea2b15f249ca8d4d67237410",
    }
}
KAIKKI = {
    "en-fr": {
        "file": "kaikki-Anglais.jsonl",
        "url": "https://kaikki.org/frwiktionary/Anglais/kaikki.org-dictionary-Anglais.jsonl",
    },
    # The English Wiktionary's Spanish section: its inflections are tagged, the French one's are not
    # (add-lingua-spanish-forms-tables).
    "es-fr": {
        "file": "kaikki-Spanish.jsonl",
        "url": "https://kaikki.org/dictionary/Spanish/kaikki.org-dictionary-Spanish.jsonl",
    },
}
# kaikki's dumps of whole Wiktionary editions, for what a pair reads beyond its extract
# (add-lingua-spanish-gloss-tables). kaikki is retiring its per-language files; an edition's dump
# stays. A dump is never kept whole: each file a pair derives from it is, as a zstd-compressed asset
# of the snapshot's release beside the extract, checked by the sha256 of its decompressed bytes.
# A file is `("entries", lang)`, the entries of one language as the dump writes them, or
# `("translations", lang, into)`, the entries of `lang` that list translations into `into`, cut down
# to those (`derive`).
DUMPS = {
    "es-fr": {
        # The French Wiktionary: its Spanish entries gloss the words and expressions; its French
        # entries' translation tables, read backwards, gloss what those leave out.
        "kaikki-fr": {
            "url": "https://kaikki.org/frwiktionary/raw-wiktextract-data.jsonl.gz",
            "files": {
                "kaikki-fr-Espagnol.jsonl": ("entries", "es"),
                "kaikki-fr-traductions.jsonl": ("translations", "fr", "es"),
            },
        },
        # The Spanish Wiktionary: the French translations its Spanish entries list.
        "kaikki-es": {
            "url": "https://kaikki.org/eswiktionary/raw-wiktextract-data.jsonl.gz",
            "files": {"kaikki-es-traductions.jsonl": ("translations", "es", "fr")},
        },
    },
}
# What a reducer writes, by side (split-lingua-pack-tables-by-language, M24). The studied side
# belongs to the studied language and is kept once, in tables/<studied>/; the native side, with the
# pack's manifest and attribution, in tables/<pair>/.
STUDIED_TABLES = ("forms.tsv", "freq.tsv", "grammar.tsv", "level.tsv")
PAIR_TABLES = ("gloss.tsv", "mwe.tsv", "senses.tsv", "NOTICE", "manifest.json")
# The file of a studied folder that names its reference pair: the one pair whose reduction writes
# the folder, and whose glossed lemmas are the language's dictionary words.
STUDIED_RECORD = "studied.json"
# The dictionary words (add-lingua-pack-lexical-layer D1, D3): the reference's glossed lemmas,
# written by `split` beside the studied tables — no reducer writes them, so no digest moves.
LEXICAL = "lexical.tsv"
# The inputs of a studied folder that nothing writes but a person, so that no rule digest moves with
# them (add-lingua-pack-lexical-layer D4): the pinned tag pool, and the record naming the reference.
KEPT_INPUTS = ("tags.tsv", STUDIED_RECORD)
PINNED_POOL = KEPT_INPUTS[0]
WORDFREQ = "3.1.1"
PYTHON = (3, 12)
ZSTD_LEVEL = 19


class PinError(Exception):
    """A source or a pack that is not what the record says."""


# — the record —


def load(pin: Path) -> dict:
    return json.loads(pin.read_text(encoding="utf-8")) if pin.is_file() else {}


def save(pin: Path, record: dict) -> None:
    pin.parent.mkdir(parents=True, exist_ok=True)
    pin.write_text(json.dumps(record, indent=2, ensure_ascii=False, sort_keys=False) + "\n", encoding="utf-8")


def get(record: dict, key: str):
    value = record
    for part in key.split("."):
        if not isinstance(value, dict) or part not in value:
            raise PinError(f"pin.json has no {key}")
        value = value[part]
    return value


def sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with open(path, "rb") as f:
        for block in iter(lambda: f.read(1 << 20), b""):
            digest.update(block)
    return digest.hexdigest()


def pair_of(pin: Path) -> str:
    return pin.parent.name


def studied_of(pair: str) -> str:
    """A pair's studied language, as its name says: `es-fr` studies `es`."""
    return pair.split("-", 1)[0]


def studied_dir(pin: Path) -> Path:
    """The studied folder beside a pair's folder: tables/es for tables/es-fr/pin.json."""
    return pin.parent.parent / studied_of(pair_of(pin))


def reference_of(studied: Path) -> str | None:
    """The pair a studied folder names as its reference, or None when it names none yet."""
    record = studied / STUDIED_RECORD
    return json.loads(record.read_text(encoding="utf-8"))["reference"] if record.is_file() else None


def is_pair(folder: Path) -> bool:
    """A folder of tables/ is a pair's when it is named `<studied>-<native>`."""
    return folder.is_dir() and "-" in folder.name


def folders(tables: Path) -> list[Path]:
    """The folders of `tables`, but hidden ones; each is a pair's or a studied language's, which
    names its reference pair (`studied.json`). Any other fails: a loop would take it for a pair."""
    found = sorted(f for f in tables.iterdir() if f.is_dir() and not f.name.startswith("."))
    for folder in found:
        if not is_pair(folder) and not (folder / STUDIED_RECORD).is_file():
            raise PinError(
                f"{folder} is neither a pair (a folder named <studied>-<native>) nor a studied "
                f"language ({folder.name}/{STUDIED_RECORD} names the pair whose reduction writes it)"
            )
    return found


def pairs(tables: Path, after: str | None = None) -> list[str]:
    """The pairs of `tables`, each studied language's reference first, then its other pairs, by
    name; with `after`, that pair, then — when it is its language's reference — the pairs that read
    its studied folder, which must be reduced again after it."""
    names = [folder.name for folder in folders(tables) if is_pair(folder)]
    refs = {lang: reference_of(tables / lang) for lang in {studied_of(n) for n in names}}
    ordered = sorted(names, key=lambda n: (refs[studied_of(n)] != n, n))
    if after is None:
        return ordered
    if refs.get(studied_of(after)) != after:
        return [after]
    return [after, *(n for n in ordered if n != after and studied_of(n) == studied_of(after))]


def glossed_lemmas(gloss: Path) -> list[str]:
    """The lemmas a gloss table glosses, byte-sorted, each once — as the builder reads them
    (lingua_pack::tsv_pairs: the text before the first tab, trimmed, non-empty)."""
    words = set()
    for line in gloss.read_text(encoding="utf-8").split("\n"):
        key, sep, _ = line.removesuffix("\r").partition("\t")
        if sep and key.strip():
            words.add(key.strip())
    # Code-point order is UTF-8 byte order.
    return sorted(words)


def split(work: Path, tables: Path, pair: str) -> list[str]:
    """File a reduction's tables, left in `work` by the pair's reducer, into `tables`: the native
    side into tables/<pair>/, and — when `pair` is its studied language's reference, or the first
    pair reduced for a language — the studied side into tables/<studied>/, with the dictionary words
    (`lexical.tsv`, the pair's glossed lemmas). Any other pair reads the studied folder and never
    writes it: what its reducer wrote of the studied side is left in `work`. Answers the files
    written, as `<folder>/<file>`."""
    lang = studied_of(pair)
    written = []

    def file(names: tuple[str, ...], to: Path) -> None:
        to.mkdir(parents=True, exist_ok=True)
        for name in names:
            if (work / name).is_file():
                shutil.copyfile(work / name, to / name)
                written.append(f"{to.name}/{name}")
            else:
                (to / name).unlink(missing_ok=True)

    file(PAIR_TABLES, tables / pair)
    studied = tables / lang
    reference = reference_of(studied)
    if reference is None:
        studied.mkdir(parents=True, exist_ok=True)
        (studied / STUDIED_RECORD).write_text(json.dumps({"reference": pair}, indent=2) + "\n", encoding="utf-8")
        written.append(f"{lang}/{STUDIED_RECORD}")
        reference = pair
    if reference == pair:
        file(STUDIED_TABLES, studied)
        (studied / LEXICAL).write_text(
            "".join(f"{w}\n" for w in glossed_lemmas(tables / pair / "gloss.tsv")), encoding="utf-8"
        )
        written.append(f"{lang}/{LEXICAL}")
    return written


def writer_of(tables: Path, folder: str) -> str:
    """The pair whose reduction writes a folder of `tables`: a pair's own, or the reference a
    studied language's folder names."""
    if "-" in folder:
        return folder
    return reference_of(tables / folder) or f"{folder}'s reference (no {STUDIED_RECORD})"


def moved(tables: Path, status: list[str]) -> list[str]:
    """`git status --porcelain` lines under `tables`, as `<pair>: <folder>/<file>`, the pair being
    the one whose reduction writes the file — a studied table names its language's reference."""
    out = []
    for line in status:
        path = line[3:].strip() if len(line) > 3 and line[2] == " " else line.strip()
        if not path:
            continue
        path = path.split(" -> ")[-1].strip('"')
        parts = Path(path).parts
        if tables.name in parts:
            parts = parts[parts.index(tables.name) + 1 :]
        if not parts:
            out.append(f"?: {path}")
            continue
        out.append(f"{writer_of(tables, parts[0])}: {'/'.join(parts)}{'/' if len(parts) == 1 else ''}")
    return out


def keep(committed: Path, tables: Path) -> list[str]:
    """Copy a committed studied folder into `tables`, the studied folder a dry run reduces into
    anew: its kept inputs, and the studied tables a pair that is not the reference builds on.
    Answers the names copied."""
    kept = []
    tables.mkdir(parents=True, exist_ok=True)
    for source in sorted(committed.iterdir()) if committed.is_dir() else ():
        if source.is_file() and source.resolve() != (tables / source.name).resolve():
            shutil.copyfile(source, tables / source.name)
            kept.append(source.name)
    return kept


def release_tag(pair: str, snapshot: str) -> str:
    return f"lingua-pack-sources-{pair}-{snapshot}"


def release_url(tag: str, asset: str) -> str:
    return f"https://github.com/{REPOSITORY}/releases/download/{tag}/{asset}"


# — fetching —


def download(url: str, dest: Path, *, compressed: bool = False) -> dict[str, str]:
    """Fetch `url` into `dest`; returns the response headers that date it."""
    dest.parent.mkdir(parents=True, exist_ok=True)
    tmp = dest.with_name(dest.name + ".part")
    cmd = ["curl", "-sSL", "--fail", "--retry", "3", "-D", "-", "-o", str(tmp), url]
    if compressed:
        cmd.insert(1, "--compressed")
    out = subprocess.run(cmd, check=True, capture_output=True, text=True).stdout
    tmp.replace(dest)
    headers = {}
    for line in out.splitlines():
        if ":" in line:
            name, _, value = line.partition(":")
            headers[name.strip().lower()] = value.strip()
    return headers


def translations_of(entry: dict):
    """An entry's translations wherever its edition writes them (generalise-lingua-gloss-reducer D3):
    the table of the whole entry, then each sense's, in source order. The French and Spanish
    Wiktionaries write one table per entry; the English one writes them under its senses — 68,579
    of its English entries list Spanish translations under a sense, against 5,080 for the whole
    entry."""
    yield from entry.get("translations") or ()
    for sense in entry.get("senses") or ():
        if isinstance(sense, dict):
            yield from sense.get("translations") or ()


def derive(dump: Path, files: dict, work: Path) -> None:
    """The files a pair takes from an edition's dump (`DUMPS`), in one pass, in the dump's order.

    An `entries` file holds each line as the dump writes it, so the shared rules read it as they
    read a per-language extract. A `translations` file holds, per entry, its word, its part of
    speech and its translations into one language (the word, and the sense when the table names
    one), wherever the entry lists them (`translations_of`), as sorted JSON: the rest of the entry
    is not read, and would weigh down the release.
    """
    # A line cannot belong to a file unless it names the languages that file reads, as the dump
    # writes them; most of a dump's millions of lines are then never parsed.
    marks = {
        name: [f'"lang_code": "{code}"' for code in (lang, *into)] for name, (_, lang, *into) in files.items()
    }
    outs = {name: open(work / name, "w", encoding="utf-8") for name in files}
    try:
        with gzip.open(dump, "rt", encoding="utf-8") as f:
            for line in f:
                if not any(all(mark in line for mark in need) for need in marks.values()):
                    continue
                try:
                    entry = json.loads(line)
                except json.JSONDecodeError:
                    continue
                for name, (kind, lang, *into) in files.items():
                    if entry.get("lang_code") != lang:
                        continue
                    if kind == "entries":
                        outs[name].write(line if line.endswith("\n") else line + "\n")
                        continue
                    found = [
                        {"word": t["word"], **({"sense": t["sense"]} if t.get("sense") else {})}
                        for t in translations_of(entry)
                        if t.get("lang_code") == into[0] and t.get("word")
                    ]
                    if found:
                        cut = {"word": entry.get("word"), "pos": entry.get("pos"), "translations": found}
                        outs[name].write(json.dumps(cut, ensure_ascii=False, sort_keys=True) + "\n")
    finally:
        for out in outs.values():
            out.close()


def build_esdb(spec: dict, work: Path) -> Path:
    """Export ESDB's `scowl.txt` from its repository at the pinned commit, into `work`."""
    repo = work / "esdb-wordlist"
    if repo.exists():
        subprocess.run(["rm", "-rf", str(repo)], check=True)
    repo.mkdir(parents=True)
    git = ["git", "-C", str(repo)]
    subprocess.run([*git, "init", "-q"], check=True)
    subprocess.run([*git, "fetch", "-q", "--depth", "1", spec["repository"], spec["commit"]], check=True)
    subprocess.run([*git, "checkout", "-q", "FETCH_HEAD"], check=True)
    subprocess.run([sys.executable, "combine.py", "create-db", "scowl.db"], cwd=repo, check=True, capture_output=True)
    out = work / spec["file"]
    with open(out, "wb") as f:
        subprocess.run([sys.executable, "scowl", "export", "--db", "scowl.db"], cwd=repo, check=True, stdout=f)
    return out


def check_python() -> None:
    if sys.version_info[:2] != PYTHON:
        raise PinError(
            f"the reducer runs on Python {PYTHON[0]}.{PYTHON[1]} (this is {sys.version.split()[0]}): "
            "the same tables need the same interpreter"
        )


def wordfreq_version() -> str:
    try:
        from importlib.metadata import version

        return version("wordfreq")
    except Exception as e:  # noqa: BLE001 — not installed, or no metadata
        raise PinError(f"wordfreq is not installed: {e}") from e


def fetch_pinned(pin: Path, work: Path, *, fetch=download, build=build_esdb) -> None:
    """Every raw source as recorded, into `work`, each checked by sha256 (re-reduce mode)."""
    record = load(pin)
    pair = pair_of(pin)
    sources = get(record, "sources")
    if pair not in KAIKKI:
        raise PinError(f"no source registry for {pair}: add it to PINNED / ESDB / KAIKKI in pack_sources.py")
    esdb = ESDB.get(pair)
    if esdb is None:
        pass  # a pair whose inflections come from another source (ESDB is English)
    elif "esdb" not in sources:
        # A source the code now reads and the record does not know yet: it is pinned at a commit,
        # so recording what that commit exports is the same act as reading it.
        sources["esdb"] = {k: esdb[k] for k in ("repository", "tag", "commit")}
        sources["esdb"]["sha256"] = sha256(build(esdb, work))
        print(f"note: ESDB recorded in pin.json at {esdb['tag']}", file=sys.stderr)
    else:
        got = sha256(build({**esdb, **sources["esdb"]}, work))
        if got != sources["esdb"]["sha256"]:
            raise PinError(f"esdb: scowl.txt has sha256 {got}, pin.json records {sources['esdb']['sha256']}")
    read = (*PINNED.get(pair, {}), *(("esdb",) if esdb else ()), "kaikki", *DUMPS.get(pair, {}), "wordfreq")
    for retired in [name for name in sources if name not in read]:
        del sources[retired]
        print(f"note: {retired} is no longer read; removed from pin.json", file=sys.stderr)
    save(pin, record)
    for name, spec in PINNED.get(pair, {}).items():
        entry = sources.get(name) or {}
        dest = work / spec["file"]
        fetch(entry.get("url") or spec["url"], dest)
        got = sha256(dest)
        if got != entry.get("sha256"):
            raise PinError(f"{name}: {dest.name} has sha256 {got}, pin.json records {entry.get('sha256')}")
    kaikki = get(record, "sources.kaikki")
    raw = work / KAIKKI[pair]["file"]
    packed = raw.with_name(kaikki["asset"])
    fetch(release_url(kaikki["release"], kaikki["asset"]), packed)
    subprocess.run(["zstd", "-q", "-d", "-f", str(packed), "-o", str(raw)], check=True)
    got = sha256(raw)
    if got != kaikki["sha256"]:
        raise PinError(f"kaikki: the snapshot decompresses to sha256 {got}, pin.json records {kaikki['sha256']}")
    for name in DUMPS.get(pair, {}):
        dumped = get(record, f"sources.{name}")
        for file, spec in get(record, f"sources.{name}.files").items():
            raw = work / file
            packed = raw.with_name(spec["asset"])
            fetch(release_url(dumped["release"], spec["asset"]), packed)
            subprocess.run(["zstd", "-q", "-d", "-f", str(packed), "-o", str(raw)], check=True)
            got = sha256(raw)
            if got != spec["sha256"]:
                raise PinError(f"{name}: {file} decompresses to sha256 {got}, pin.json records {spec['sha256']}")
    installed = wordfreq_version()
    if installed != get(record, "sources.wordfreq.version"):
        raise PinError(f"wordfreq {installed} is installed, pin.json records {get(record, 'sources.wordfreq.version')}")


def fetch_live(pin: Path, work: Path, snapshot: str, *, fetch=download, build=build_esdb, today=None) -> dict:
    """Today's raw sources into `work`, recorded in `pin` as a new snapshot (update mode).

    CEFR-J, Octanove and ESDB stay at their pinned commits — a newer commit is a deliberate edit
    of PINNED or ESDB. kaikki is read live, recorded by the sha256 of its bytes, and compressed beside
    them for the release that keeps it; so is each file derived from a dump (`DUMPS`).
    """
    record = load(pin)
    pair = pair_of(pin)
    if pair not in KAIKKI:
        raise PinError(f"no source registry for {pair}: add it to PINNED / ESDB / KAIKKI in pack_sources.py")
    sources: dict = {}
    for name, spec in PINNED.get(pair, {}).items():
        dest = work / spec["file"]
        fetch(spec["url"], dest)
        sources[name] = {"url": spec["url"], "sha256": sha256(dest)}
    esdb = ESDB.get(pair)
    if esdb is not None:
        sources["esdb"] = {k: esdb[k] for k in ("repository", "tag", "commit")}
        sources["esdb"]["sha256"] = sha256(build(esdb, work))
    raw = work / KAIKKI[pair]["file"]
    headers = fetch(KAIKKI[pair]["url"], raw, compressed=True) or {}
    asset = raw.name + ".zst"
    subprocess.run(
        ["zstd", "-q", f"-{ZSTD_LEVEL}", "-T0", "-f", str(raw), "-o", str(raw.with_name(asset))], check=True
    )
    sources["kaikki"] = {
        "release": release_tag(pair, snapshot),
        "asset": asset,
        "sha256": sha256(raw),
        "size": raw.stat().st_size,
        "fetched": (today or datetime.date.today()).isoformat(),
        "last_modified": headers.get("last-modified", ""),
        "url": KAIKKI[pair]["url"],
    }
    for name, spec in DUMPS.get(pair, {}).items():
        # The dump itself is never kept: only what the pair derives from it.
        dump = work / f"{name}.dump.jsonl.gz"
        headers = fetch(spec["url"], dump) or {}
        derive(dump, spec["files"], work)
        dump.unlink()
        files = {}
        for file in spec["files"]:
            raw = work / file
            asset = file + ".zst"
            subprocess.run(
                ["zstd", "-q", f"-{ZSTD_LEVEL}", "-T0", "-f", str(raw), "-o", str(raw.with_name(asset))], check=True
            )
            files[file] = {"asset": asset, "sha256": sha256(raw), "size": raw.stat().st_size}
        sources[name] = {
            "release": release_tag(pair, snapshot),
            "url": spec["url"],
            "fetched": (today or datetime.date.today()).isoformat(),
            "last_modified": headers.get("last-modified", ""),
            "files": files,
        }
    sources["wordfreq"] = {"version": wordfreq_version()}
    if sources["wordfreq"]["version"] != WORDFREQ:
        raise PinError(f"wordfreq {sources['wordfreq']['version']} is installed; the pipeline pins {WORDFREQ}")
    record = {"snapshot": snapshot, "pack": record.get("pack", {}), "reducer": record.get("reducer", {}), "sources": sources}
    save(pin, record)
    return record


# — what the tables build —


# What importing a reducer loads, run in an interpreter of its own: the caller's imports never
# count, and the reducer's import touches nothing of the caller's. No bytecode is written, and the
# working directory is not on the path: a module is found beside the reducer, wherever it runs from.
_LOADED_RULES = r"""
import importlib.util, json, sys
from pathlib import Path

reducer = Path(sys.argv[1]).resolve()
sys.path[:] = [str(reducer.parent), *(p for p in sys.path if p)]
spec = importlib.util.spec_from_file_location("_lingua_reducer", reducer)
spec.loader.exec_module(importlib.util.module_from_spec(spec))
loaded = set()
for name, module in list(sys.modules.items()):
    path = getattr(module, "__file__", None)
    if name.startswith("reduce_") and path and Path(path).resolve().parent == reducer.parent:
        loaded.add(Path(path).name)
print(json.dumps(sorted(loaded)))
"""


def loaded_rules(reducer: Path) -> list[str]:
    """The shared rule modules importing `reducer` loads — the `reduce_*.py` beside it that are in
    `sys.modules` once it is imported — by file name, sorted."""
    done = subprocess.run(
        [sys.executable, "-B", "-c", _LOADED_RULES, str(reducer)], capture_output=True, text=True, check=False
    )
    if done.returncode != 0:
        lines = done.stderr.strip().splitlines()
        raise PinError(f"importing {reducer.name} failed: {lines[-1] if lines else done.returncode}")
    return json.loads(done.stdout)


def rule_files(reducer: Path) -> list[Path]:
    """The files a pair's tables are reduced by (generalise-lingua-gloss-reducer D2): its own
    `reduce-<pair>.py` and every shared `reduce_*.py` module importing it loads — the rules every
    pair shares and the rules of the Wiktionary edition its glosses come from. A rule module the
    reducer loads never escapes the digest, and the rules of an edition it does not read are not
    part of it: tuning the English Wiktionary's rules never asks a pair glossed in French to reduce
    again. Shared rules are named with an underscore, a pair's with a hyphen, so a pair's reducer
    never enters another pair's rule set."""
    return [reducer, *(reducer.parent / name for name in loaded_rules(reducer))]


def files_sha256(files: list[Path]) -> str:
    """One digest over rule files: every file's name and sha256, in order."""
    digest = hashlib.sha256()
    for path in files:
        digest.update(f"{path.name}\0{sha256(path)}\n".encode())
    return digest.hexdigest()


def rules_sha256(reducer: Path) -> str:
    """One digest for a pair's reduction rules (`rule_files`). A change to a shared module moves the
    digest of every pair that loads it, so each is asked to reduce again."""
    return files_sha256(rule_files(reducer))


def record_build(pin: Path, pack: Path, reducer: Path) -> None:
    pool = studied_dir(pin) / PINNED_POOL
    if not pool.is_file():
        raise PinError(
            f"{pool} is missing: the studied language's pinned tag pool is an input no "
            "reducer writes, which the tables must keep (scripts/lingua-data/SOURCES.md)"
        )
    record = load(pin)
    record["pack"] = {"sha256": sha256(pack), "size": pack.stat().st_size}
    files = rule_files(reducer)
    record["reducer"] = {"sha256": files_sha256(files), "files": [p.name for p in files]}
    save(pin, {k: record[k] for k in ("snapshot", "pack", "reducer", "sources") if k in record})


def assets(record: dict) -> list[str]:
    """The files the snapshot's release holds: the extract's, then each derived file's, in the
    record's order."""
    sources = get(record, "sources")
    out = [get(record, "sources.kaikki.asset")]
    for spec in sources.values():
        if isinstance(spec, dict):
            out.extend(file["asset"] for file in (spec.get("files") or {}).values())
    return out


def check_pack(pin: Path, pack: Path) -> None:
    want = get(load(pin), "pack.sha256")
    got = sha256(pack)
    if got != want:
        raise PinError(
            f"{pack} has sha256 {got}, but the committed tables build {want} (pin.json). The builder or a "
            "dependency changed the pack: if that is intended, update pack.sha256 in the same pull request."
        )


def check_reducer(pin: Path, reducer: Path) -> None:
    """The rules a pair's tables were reduced by, against its record — and, for a pair that is not
    its studied language's reference, its reference's too: that pair's reduction writes the studied
    tables every pair of the language reads (split-lingua-pack-tables-by-language)."""
    check_own_reducer(pin, reducer)
    pair, studied = pair_of(pin), studied_dir(pin)
    reference = reference_of(studied)
    if reference is not None and reference != pair:
        try:
            check_own_reducer(studied.parent / reference / "pin.json", reducer.parent / f"reduce-{reference}.py")
        except PinError as e:
            raise PinError(f"{pair} reads {studied.name}/, which {reference}'s reduction writes: {e}") from e


def check_own_reducer(pin: Path, reducer: Path) -> None:
    record = load(pin)
    want = get(record, "reducer.sha256")
    files = rule_files(reducer)
    named = set(record["reducer"].get("files") or ())
    unnamed = [p.name for p in files if p.name not in named]
    if unnamed:
        raise PinError(
            f"{reducer.name} loads {', '.join(unnamed)}, which the rules recorded in pin.json (reducer.files) "
            "do not name: the committed tables were reduced without it. Reduce them again from the pinned "
            f"sources: scripts/lingua-data/build.sh --reduce {pair_of(pin)} <out> (or dispatch "
            "lingua-pack-update with mode=reduce)."
        )
    got = files_sha256(files)
    if got != want:
        names = ", ".join(p.name for p in files)
        raise PinError(
            f"The reduction rules ({names}) changed since the committed tables were reduced "
            f"(sha256 {got}, pin.json has {want}). "
            "Reduce them again from the pinned sources: scripts/lingua-data/build.sh --reduce "
            f"{pair_of(pin)} <out> (or dispatch lingua-pack-update with mode=reduce)."
        )


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    commands = ("fetch-pinned", "fetch-live", "record-build", "check-pack", "check-reducer", "rules", "get", "assets")
    for name in (*commands, "keep", "split", "pairs", "moved"):
        p = sub.add_parser(name)
        if name == "keep":
            p.add_argument("--from", dest="source", type=Path, required=True)
            p.add_argument("--to", dest="dest", type=Path, required=True)
            continue
        if name in ("split", "pairs", "moved"):
            p.add_argument("--tables", type=Path, required=True)
            if name == "split":
                p.add_argument("--work", type=Path, required=True)
                p.add_argument("--pair", required=True)
            elif name == "pairs":
                p.add_argument("--after")
            continue
        p.add_argument("--pin", type=Path, required=name != "rules")
        if name in ("fetch-pinned", "fetch-live"):
            p.add_argument("--work", type=Path, required=True)
        if name == "fetch-live":
            p.add_argument("--snapshot", default=datetime.date.today().strftime("%Y.%m.%d"))
        if name in ("record-build", "check-pack"):
            p.add_argument("--pack", type=Path, required=True)
        if name in ("record-build", "check-reducer", "rules"):
            p.add_argument("--reducer", type=Path, required=True)
        if name == "get":
            p.add_argument("key")
    a = ap.parse_args(argv)
    try:
        if a.cmd == "fetch-pinned":
            check_python()
            fetch_pinned(a.pin, a.work)
        elif a.cmd == "fetch-live":
            check_python()
            fetch_live(a.pin, a.work, a.snapshot)
        elif a.cmd == "record-build":
            record_build(a.pin, a.pack, a.reducer)
        elif a.cmd == "check-pack":
            check_pack(a.pin, a.pack)
        elif a.cmd == "check-reducer":
            check_reducer(a.pin, a.reducer)
        elif a.cmd == "rules":
            print(rules_sha256(a.reducer))
        elif a.cmd == "get":
            print(get(load(a.pin), a.key))
        elif a.cmd == "assets":
            print("\n".join(assets(load(a.pin))))
        elif a.cmd == "keep":
            for name in keep(a.source, a.dest):
                print(f"kept {name}")
        elif a.cmd == "split":
            for name in split(a.work, a.tables, a.pair):
                print(f"wrote {name}")
        elif a.cmd == "pairs":
            print("\n".join(pairs(a.tables, a.after)))
        elif a.cmd == "moved":
            print("\n".join(moved(a.tables, sys.stdin.read().splitlines())))
    except (PinError, subprocess.CalledProcessError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
