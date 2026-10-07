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
- `reducer`: sha256 of the `reduce-<pair>.py` the tables were reduced with.
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
    pack_sources.py keep          --from D --to T                  # the kept inputs of D, into T

Beside the tables a reducer writes, a pair's folder holds inputs no reducer writes, which reducing
the pair again must keep (`KEPT_INPUTS`, add-lingua-pack-lexical-layer).
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
# The inputs of a pair's tables that no reducer writes, so that no rule digest moves with them
# (add-lingua-pack-lexical-layer D1, D4): the studied language's pinned tag pool, which every
# committed pair carries, and the dictionary words of a pair glossed in another native language
# than its studied language's reference pair. Reducing a pair again keeps them as they are.
KEPT_INPUTS = ("tags.tsv", "lexical.tsv")
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


def derive(dump: Path, files: dict, work: Path) -> None:
    """The files a pair takes from an edition's dump (`DUMPS`), in one pass, in the dump's order.

    An `entries` file holds each line as the dump writes it, so the shared rules read it as they
    read a per-language extract. A `translations` file holds, per entry, its word, its part of
    speech and its translations into one language (the word, and the sense when the table names
    one), as sorted JSON: the rest of the entry is not read, and would weigh down the release.
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
                        for t in entry.get("translations") or ()
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


def rule_files(reducer: Path) -> list[Path]:
    """The files a pair's tables are reduced by: its own `reduce-<pair>.py` and every shared
    `reduce_*.py` beside it (generalise-lingua-pack-reducer). Shared rules are named with an
    underscore, a pair's with a hyphen, so a new pair never enters another pair's rule set."""
    return [reducer, *sorted(p for p in reducer.parent.glob("reduce_*.py") if p != reducer)]


def rules_sha256(reducer: Path) -> str:
    """One digest for a pair's reduction rules: every rule file's name and sha256, in order. A
    change to a shared module moves every pair's digest, so each is asked to reduce again."""
    digest = hashlib.sha256()
    for path in rule_files(reducer):
        digest.update(f"{path.name}\0{sha256(path)}\n".encode())
    return digest.hexdigest()


def keep(committed: Path, tables: Path) -> list[str]:
    """Copy the kept inputs `committed` holds into `tables`, a folder the pair's tables are reduced
    into anew (a dry run's scratch copy); answers the names copied."""
    kept = []
    tables.mkdir(parents=True, exist_ok=True)
    for name in KEPT_INPUTS:
        source = committed / name
        if source.is_file() and source.resolve() != (tables / name).resolve():
            shutil.copyfile(source, tables / name)
            kept.append(name)
    return kept


def record_build(pin: Path, pack: Path, reducer: Path) -> None:
    if not (pin.parent / PINNED_POOL).is_file():
        raise PinError(
            f"{pin.parent / PINNED_POOL} is missing: the studied language's pinned tag pool is an input no "
            "reducer writes, which the tables must keep (scripts/lingua-data/SOURCES.md)"
        )
    record = load(pin)
    record["pack"] = {"sha256": sha256(pack), "size": pack.stat().st_size}
    record["reducer"] = {"sha256": rules_sha256(reducer), "files": [p.name for p in rule_files(reducer)]}
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
    want = get(load(pin), "reducer.sha256")
    got = rules_sha256(reducer)
    if got != want:
        names = ", ".join(p.name for p in rule_files(reducer))
        raise PinError(
            f"The reduction rules ({names}) changed since the committed tables were reduced "
            f"(sha256 {got}, pin.json has {want}). "
            "Reduce them again from the pinned sources: scripts/lingua-data/build.sh --reduce "
            f"{pair_of(pin)} <out> (or dispatch lingua-pack-update with mode=reduce)."
        )


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    sub = ap.add_subparsers(dest="cmd", required=True)
    for name in ("fetch-pinned", "fetch-live", "record-build", "check-pack", "check-reducer", "rules", "get", "assets", "keep"):
        p = sub.add_parser(name)
        if name == "keep":
            p.add_argument("--from", dest="source", type=Path, required=True)
            p.add_argument("--to", dest="dest", type=Path, required=True)
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
    except (PinError, subprocess.CalledProcessError) as e:
        print(f"error: {e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
