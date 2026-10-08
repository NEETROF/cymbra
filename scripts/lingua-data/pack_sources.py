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
  re-reduction of the same sources (`2026.09.26+1dff19c`). A pair that is not its studied
  language's reference names, in either mode, its snapshot, its rules and the studied tables it
  read (`2026.10.08+733f815.5e0c2a1`, `pack_version`): its version moves when they do.
- `pack`: sha256 and size of the pack the tables build — every lane must obtain exactly that.
- `reducer`: the files the tables were reduced by — the pair's `reduce-<pair>.py` and the shared
  `reduce_*.py` modules importing it loads (`rule_files`) — and one sha256 over all of them.
- `sources`: each raw source, pinned — CEFR-J and Octanove at a commit of their own repository
  and by sha256; ESDB (the inflections) built at a commit of en-wl/wordlist, by the sha256 of its
  exported `scowl.txt`; kaikki (regenerated upstream every day) as the files a pair derives from
  its dumps of whole Wiktionary editions — our own snapshot of each, a zstd-compressed GitHub
  Release asset, checked by the sha256 of its decompressed bytes — with each dump named by its
  address, regeneration date and decompressed sha256, never kept (a pin written before
  migrate-lingua-pack-sources-to-raw-dumps names kaikki's per-language extract instead, kept the
  same way); wordfreq by version (it is its own snapshot; requirements-reduce.txt pins it by hash).
- `studied`, for a pair that is not its studied language's reference (add-lingua-pack-es-en D3):
  the reference pair, and the sha256 of each of the six studied tables the build read as committed
  (`RECORDED_STUDIED`). The checks fail, naming the pair and the table, when a committed studied
  table is no longer what the pair's build read: the reference's reduction moved it, and the pair
  was left behind until it is reduced again. The pair's `pack_version` names a digest of the record
  (`studied_digest`).

Stdlib only: the build mode reads this record too, and needs no Python package.

    pack_sources.py fetch-pinned  --pin P --work W [--cache C]     # the recorded bytes, checked, into W
    pack_sources.py fetch-live    --pin P --work W [--snapshot D] [--cache C] [--editions E]  # today's bytes, recorded in P
    pack_sources.py record-build  --pin P --pack F --reducer R     # what the tables build
    pack_sources.py check-pack    --pin P --pack F                 # a pack, against the record
    pack_sources.py check-reducer --pin P --reducer R              # the rules, against the record
    pack_sources.py get           --pin P KEY                      # e.g. snapshot, pack.sha256
    pack_sources.py version       --pin P --reducer R [--live]     # the pack_version to reduce under
    pack_sources.py release-tag   --pin P                          # the pair's own release of its snapshot
    pack_sources.py assets        --pin P [--release TAG]          # the release assets (of TAG alone)
    pack_sources.py dumps         --pin P [--release TAG]          # the dumps they derive from, for the notes
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

kaikki is read at three addresses, one dump per Wiktionary edition (`EDITIONS`,
migrate-lingua-pack-sources-to-raw-dumps D1): every file a pair reads of kaikki — a language's
entries, or the translations an edition's entries list into one language — is derived from the
dump of the edition that writes it, and `DUMPS[pair]` names the files the pair reads, by edition.
A dump is read once per run (D4): its edition's whole catalogue is derived at the first read, into
`--editions E` (`work/editions/<edition>-<snapshot>/`), and a later pair of the run copies what it
reads from there; the dump is deleted after its pass, and recorded in the pin by its address, its
regeneration date and the sha256 and size of its decompressed bytes (D3).

Each pair pins its own fetch (add-lingua-pack-es-en D2): es-en reads the English edition's Spanish
section as es-fr does, recorded and published under es-en's own release, `release_tag(pair,
snapshot)`, which holds the pair's own assets only (`assets --release`). A pair an update brings
along is reduced from its own pin. Each source record names the release that holds its assets, and
`fetch-pinned` follows it, reading a record by its shape — derived `files`, or the `asset` of a
per-language extract a pin written before the dumps names (D5) — so no committed pin moves. With
`--cache C`, a fetched release asset is kept in `C/<sha256>` (the sha256 of its decompressed bytes,
as the record names it) and found there by a later fetch of the same bytes: a run fetches an asset
once when two pins name the same one, and reducing a pair again on the same machine fetches nothing
again.
"""

from __future__ import annotations

import argparse
import datetime
import gzip
import hashlib
import io
import json
import os
import shutil
import subprocess
import sys
import time
import urllib.request
import zlib
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
# kaikki's dumps of whole Wiktionary editions (migrate-lingua-pack-sources-to-raw-dumps D1), the one
# thing kaikki says it will keep serving: its per-language extracts are deprecated. Each edition's
# **catalogue** is every file derivable from its dump that a pair of the programme reads: a file is
# `("entries", lang)`, the entries of one language as the dump writes them, or `("translations",
# lang, into)`, the entries of `lang` that list translations into `into`, cut down to those
# (`derive`). A pair registers its reads in `DUMPS`; a change that needs a file the catalogue lacks
# adds it here, in the edition that writes it, and nowhere else. The existing names are kept — a
# name is a reducer's input — and new ones carry their edition. A dump is read as served, gzipped or
# plain, told apart by the gzip magic (`open_dump`), and never kept: each file a pair derives from
# it is, as a zstd-compressed asset of the pair's release.
EDITIONS = {
    # The English Wiktionary: 2,981,058,381 B gzipped (regenerated 2026-10-03 08:24) — above the
    # 2 GiB a release asset may weigh. Its inflections are tagged, the French one's are not
    # (add-lingua-spanish-forms-tables); it writes its translation tables under its senses.
    "en": {
        "url": "https://kaikki.org/dictionary/raw-wiktextract-data.jsonl.gz",
        "files": {
            # Spanish's studied side (forms, readings, the gender of nouns) and es-en's glosses.
            "kaikki-Spanish.jsonl": ("entries", "es"),
            # French's studied side and fr-en's glosses (changes 43, 45, 48).
            "kaikki-French.jsonl": ("entries", "fr"),
            # en-es's direct fallback (add-lingua-pack-en-es D2).
            "kaikki-en-traductions-es.jsonl": ("translations", "en", "es"),
            # fr-en's inverted fallback (change 48).
            "kaikki-en-traductions-fr.jsonl": ("translations", "en", "fr"),
        },
    },
    # The French Wiktionary: 736,590,407 B gzipped (regenerated 2026-10-02 00:10).
    "fr": {
        "url": "https://kaikki.org/frwiktionary/raw-wiktextract-data.jsonl.gz",
        "files": {
            # en-fr's glosses, expressions and form links.
            "kaikki-Anglais.jsonl": ("entries", "en"),
            # es-fr's glosses.
            "kaikki-fr-Espagnol.jsonl": ("entries", "es"),
            # es-fr's inverted fallback, fr-es's direct one (change 49).
            "kaikki-fr-traductions.jsonl": ("translations", "fr", "es"),
            # fr-en's direct fallback (change 48).
            "kaikki-fr-traductions-en.jsonl": ("translations", "fr", "en"),
        },
    },
    # The Spanish Wiktionary: 103,226,106 B gzipped (regenerated 2026-10-02 12:12).
    "es": {
        "url": "https://kaikki.org/eswiktionary/raw-wiktextract-data.jsonl.gz",
        "files": {
            # en-es's glosses (add-lingua-pack-en-es D2).
            "kaikki-es-English.jsonl": ("entries", "en"),
            # fr-es's glosses (change 49).
            "kaikki-es-Francés.jsonl": ("entries", "fr"),
            # es-fr's direct fallback, fr-es's inverted one.
            "kaikki-es-traductions.jsonl": ("translations", "es", "fr"),
            # es-en's direct fallback, en-es's inverted one (add-lingua-pack-es-en D1).
            "kaikki-es-traductions-en.jsonl": ("translations", "es", "en"),
        },
    },
}
# The files each pair reads of kaikki, by edition, in the order the pin records them (and the
# release lists its assets): each edition the pair reads is one source record, `kaikki-<edition>`
# (`source_name`). Each pair pins its own fetch, published under its own release — es-fr and es-en
# read the same Spanish section, each under its own snapshot (add-lingua-pack-es-en D2).
DUMPS = {
    # The French Wiktionary's English entries (pin-lingua-pack-sources).
    "en-fr": {"fr": ("kaikki-Anglais.jsonl",)},
    # The English Wiktionary's Spanish section for Spanish's forms; the French Wiktionary's Spanish
    # entries gloss the words and expressions, its French entries' translation tables, read
    # backwards, gloss what those leave out; the French translations the Spanish Wiktionary's Spanish
    # entries list (add-lingua-spanish-gloss-tables).
    "es-fr": {
        "en": ("kaikki-Spanish.jsonl",),
        "fr": ("kaikki-fr-Espagnol.jsonl", "kaikki-fr-traductions.jsonl"),
        "es": ("kaikki-es-traductions.jsonl",),
    },
    # The same Spanish section, read for its English glosses, and the English translations the
    # Spanish Wiktionary's Spanish entries list, their direct fallback (add-lingua-pack-es-en D1).
    "es-en": {"en": ("kaikki-Spanish.jsonl",), "es": ("kaikki-es-traductions-en.jsonl",)},
    # The Spanish Wiktionary's English section, whose definitions gloss first, and the English
    # translations its Spanish entries list, read backwards; the Spanish translations the English
    # Wiktionary's English entries list, the direct fallback (add-lingua-pack-en-es D2).
    "en-es": {
        "es": ("kaikki-es-English.jsonl", "kaikki-es-traductions-en.jsonl"),
        "en": ("kaikki-en-traductions-es.jsonl",),
    },
}
# The source record of a pin written before the dumps: kaikki's per-language extract, kept whole as
# a release asset (D5). Read as recorded, never written again.
LEGACY_EXTRACT = "kaikki"
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
# What a pair that is not its studied language's reference records of the studied folder it built
# on (add-lingua-pack-es-en D3): the six studied tables, by sha256. `studied.json` is the record
# naming the reference, not a table.
RECORDED_STUDIED = (*STUDIED_TABLES, LEXICAL, PINNED_POOL)
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
    """The pair a studied folder names as its reference, or None when it names none yet. A record
    that cannot be read fails as an error line, not a traceback."""
    record = studied / STUDIED_RECORD
    if not record.is_file():
        return None
    try:
        reference = json.loads(record.read_text(encoding="utf-8"))["reference"]
    except (ValueError, KeyError, TypeError) as e:
        raise PinError(f"{studied.name}/{STUDIED_RECORD}: {e!r}: expected {{\"reference\": \"<pair>\"}}") from None
    if not isinstance(reference, str) or not is_pair_name(reference):
        raise PinError(f"{studied.name}/{STUDIED_RECORD} names {reference!r}, which is no pair (<studied>-<native>)")
    return reference


def is_pair_name(name: str) -> bool:
    """A pair is named `<studied>-<native>`, both non-empty."""
    studied, sep, native = name.partition("-")
    return bool(sep and studied and native)


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
    its studied folder, which must be reduced again after it. No pair at all fails: a loop over
    none would reduce nothing and pass."""
    names = [folder.name for folder in folders(tables) if is_pair(folder)]
    if not names:
        raise PinError(f"{tables} holds no pair (a folder named <studied>-<native>)")
    refs = {lang: reference_of(tables / lang) for lang in {studied_of(n) for n in names}}
    ordered = sorted(names, key=lambda n: (refs[studied_of(n)] != n, n))
    if after is None:
        return ordered
    if after not in names:
        raise PinError(f"{after} is no pair of {tables} ({', '.join(ordered)})")
    if refs.get(studied_of(after)) != after:
        return [after]
    return [after, *(n for n in ordered if n != after and studied_of(n) == studied_of(after))]


def glossed_lemmas(gloss: Path) -> list[str]:
    """The lemmas a gloss table glosses, byte-sorted, each once — as the builder reads them
    (lingua_pack::tsv_pairs: the text before the first tab, trimmed, non-empty)."""
    words = set()
    # Bytes, not universal newlines: the builder splits on \n alone, so a lone \r stays in its key.
    try:
        text = gloss.read_bytes().decode("utf-8")
    except FileNotFoundError:
        raise PinError(f"{gloss.parent.name}/{gloss.name} is missing: the dictionary words are its lemmas") from None
    for line in text.split("\n"):
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

    studied = tables / lang
    reference = reference_of(studied)  # before anything is written: a bad record fails cleanly
    file(PAIR_TABLES, tables / pair)
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


GZIP_MAGIC = b"\x1f\x8b"


class _Hashing(io.RawIOBase):
    """A binary stream read through, hashing what it yields: the sha256 and size of a dump's
    decompressed bytes, computed in the pass that derives from it (migrate-lingua-pack-sources-to-
    raw-dumps D3)."""

    def __init__(self, stream):
        self.stream = stream
        self.digest = hashlib.sha256()
        self.size = 0

    def readable(self) -> bool:
        return True

    def readinto(self, buffer) -> int:
        n = self.stream.readinto(buffer)
        if n:
            self.digest.update(memoryview(buffer)[:n])
            self.size += n
        return n

    def close(self) -> None:
        self.stream.close()
        super().close()


def dump_bytes(dump: Path):
    """A dump's decompressed bytes, as served: gzipped, as kaikki serves an edition's dump, or
    plain, as it serves a language's extract (add-lingua-pack-en-es D2) and could serve a dump —
    told apart by the gzip magic. The address says nothing: a plain file under a `.gz` name, or
    the reverse, reads the same."""
    with open(dump, "rb") as f:
        gzipped = f.read(len(GZIP_MAGIC)) == GZIP_MAGIC
    return gzip.open(dump, "rb") if gzipped else open(dump, "rb", buffering=0)


def dump_lines(raw) -> io.TextIOWrapper:
    """A dump's decompressed bytes (`dump_bytes`), read as lines of UTF-8 text with universal
    newlines — as `open(…, encoding="utf-8")` and `gzip.open(…, "rt", encoding="utf-8")` read
    them, so a derived file is the same bytes whichever opened it."""
    return io.TextIOWrapper(io.BufferedReader(raw, buffer_size=1 << 20), encoding="utf-8")


def open_dump(dump: Path) -> io.TextIOWrapper:
    """A dump's lines, gzipped or plain alike (`dump_bytes`)."""
    return dump_lines(dump_bytes(dump))


def derive(dump: Path, files: dict, work: Path) -> dict:
    """The files taken from an edition's dump (`EDITIONS`), in one pass, in the dump's order.
    Answers the dump's identity, measured in the same pass: the sha256 and size of its decompressed
    bytes (migrate-lingua-pack-sources-to-raw-dumps D3).

    An `entries` file holds each line as the dump writes it, so the shared rules read it as they
    read a per-language extract. A `translations` file holds, per entry, its word, its part of
    speech and its translations into one language (the word, and the sense when the table names
    one), wherever the entry lists them (`translations_of`), as sorted JSON: the rest of the entry
    is not read, and would weigh down the release. The dump is read gzipped or plain (`open_dump`).

    Each file is written as `<name>.part` and renamed whole at the end, as `download` and the
    asset cache write theirs: a dump that cannot be read whole — a transfer cut short (the gzip
    stream ends before its marker), a stream that is not gzip after its magic, a body that is not
    UTF-8 — is a `PinError` naming it, and leaves no derived file behind, partial or not, for an
    update to compress and publish as the snapshot's.
    """
    # A line cannot belong to a file unless it names the languages that file reads, as the dump
    # writes them; most of a dump's millions of lines are then never parsed.
    marks = {
        name: [f'"lang_code": "{code}"' for code in (lang, *into)] for name, (_, lang, *into) in files.items()
    }
    parts = {name: work / f"{name}.part" for name in files}
    outs = {name: open(part, "w", encoding="utf-8") for name, part in parts.items()}
    whole = False
    try:
        hashing = _Hashing(dump_bytes(dump))
        with dump_lines(hashing) as f:
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
            whole = True
    except (EOFError, gzip.BadGzipFile, zlib.error, UnicodeDecodeError) as why:
        raise PinError(f"{dump}: cannot be read whole ({why}): nothing is derived from it") from why
    finally:
        for out in outs.values():
            out.close()
        for name, part in parts.items():
            if whole:
                os.replace(part, work / name)
            else:
                part.unlink(missing_ok=True)
    return {"sha256": hashing.digest.hexdigest(), "size": hashing.size}


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


# — the asset cache (add-lingua-pack-es-en D2) —


def release_asset(fetch, url: str, dest: Path, want: str, cache: Path | None) -> Path:
    """A release asset — zstd-compressed bytes whose decompressed sha256 the record names `want` —
    fetched to `dest`; or, with a `cache`, kept in `cache/<want>` and found there by a later fetch
    of the same bytes: once when two pins name the same asset, and not again when a pair is reduced
    again on the same machine. The entry appears whole or not at all (`<want>.part`, then renamed).
    Answers where the compressed bytes are."""
    if cache is None:
        fetch(url, dest)
        return dest
    cached = cache / want
    if not cached.is_file():
        part = cache / f"{want}.part"
        fetch(url, part)
        os.replace(part, cached)
    return cached


def unpack(packed: Path, raw: Path, want: str, what: str, *, cached: bool) -> None:
    """`packed` decompressed into `raw`, whose sha256 must be `want`. A cache entry that does not
    decompress, or not to what its name says, is deleted before the error, which names it, so the
    next fetch reads the release again."""
    raw.parent.mkdir(parents=True, exist_ok=True)
    done = subprocess.run(
        ["zstd", "-q", "-d", "-f", str(packed), "-o", str(raw)], capture_output=True, text=True, check=False
    )
    if done.returncode != 0:
        lines = done.stderr.strip().splitlines()
        why = lines[-1] if lines else f"zstd exited {done.returncode}"
        if cached:
            packed.unlink(missing_ok=True)
            raise PinError(f"{what}: the cached {packed} does not decompress ({why}); deleted, fetch it again")
        raise PinError(f"{what}: {packed} does not decompress ({why})")
    got = sha256(raw)
    if got != want:
        if cached:
            packed.unlink(missing_ok=True)
            raise PinError(
                f"{what} decompresses to sha256 {got}, pin.json records {want}: the cached {packed} is deleted, "
                "fetch it again"
            )
        raise PinError(f"{what} decompresses to sha256 {got}, pin.json records {want}")


def pack_asset(raw: Path, asset: str, cache: Path | None) -> str:
    """`raw` zstd-compressed beside it as `asset`, for the release that keeps it; with a `cache`, a
    copy kept under the sha256 of `raw`, where a later fetch of the same bytes finds it — copied to
    `<sha256>.part` and renamed, so an interrupted copy never stands as the entry. Answers the
    sha256."""
    packed = raw.with_name(asset)
    subprocess.run(["zstd", "-q", f"-{ZSTD_LEVEL}", "-T0", "-f", str(raw), "-o", str(packed)], check=True)
    digest = sha256(raw)
    if cache is not None:
        cache.mkdir(parents=True, exist_ok=True)
        if not (cache / digest).is_file():
            part = cache / f"{digest}.part"
            shutil.copyfile(packed, part)
            os.replace(part, cache / digest)
    return digest


def source_name(edition: str) -> str:
    """The pin's record of what a pair read of an edition's dump: `kaikki-en`, `kaikki-fr`, …"""
    return f"kaikki-{edition}"


def check_registered(pair: str) -> None:
    """A pair reads kaikki through the editions' catalogues: each file it names in `DUMPS` is one
    an edition's dump gives (`EDITIONS`)."""
    if pair not in DUMPS:
        raise PinError(f"no source registry for {pair}: add it to PINNED / ESDB / DUMPS in pack_sources.py")
    for edition, files in DUMPS[pair].items():
        catalogue = EDITIONS.get(edition, {}).get("files", {})
        missing = [file for file in files if file not in catalogue]
        if edition not in EDITIONS or missing:
            raise PinError(
                f"{pair} reads {', '.join(missing or files)} of the {edition!r} edition, which its catalogue "
                f"does not list: add it to EDITIONS[{edition!r}] in pack_sources.py"
            )


def kaikki_records(pair: str, sources: dict) -> list[str]:
    """The records of a pin that name kaikki's bytes, in the pin's order: a legacy extract's
    (`LEGACY_EXTRACT`, D5) when the pin has one, and each edition's the pair reads (`DUMPS`)."""
    named = {LEGACY_EXTRACT, *(source_name(edition) for edition in DUMPS.get(pair, {}))}
    return [name for name in sources if name in named]


def fetch_pinned(pin: Path, work: Path, *, fetch=download, build=build_esdb, cache: Path | None = None) -> None:
    """Every raw source as recorded, into `work`, each checked by sha256 (re-reduce mode). A release
    asset is read from `cache` when an earlier fetch kept it there (`release_asset`). A kaikki
    record is read by its shape, from the release it names (migrate-lingua-pack-sources-to-raw-dumps
    D5): derived `files`, or the `asset` of a per-language extract, which a pin written before the
    dumps names — kept in the pin, never pruned, and fetched under the name its asset gives
    (`kaikki-Spanish.jsonl.zst` → `kaikki-Spanish.jsonl`), so the committed pins reproduce as they
    are. No dump is read: the derived files are what the release keeps."""
    record = load(pin)
    pair = pair_of(pin)
    sources = get(record, "sources")
    check_registered(pair)
    work.mkdir(parents=True, exist_ok=True)
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
    kaikki = kaikki_records(pair, sources)
    read = (*PINNED.get(pair, {}), *(("esdb",) if esdb else ()), *kaikki, "wordfreq")
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
    # Each release asset from the release its record names (add-lingua-pack-es-en D2).
    for name in kaikki:
        spec = sources[name]
        if not isinstance(spec, dict) or "release" not in spec:
            raise PinError(f"{name}: pin.json names no release to fetch it from")
        if "asset" in spec:
            # A per-language extract, kept whole (a pin written before the dumps).
            raw = work / spec["asset"].removesuffix(".zst")
            if raw.name == spec["asset"]:
                raise PinError(f"{name}: the asset {spec['asset']} is not zstd-compressed (.zst)")
            packed = release_asset(fetch, release_url(spec["release"], spec["asset"]), raw.with_name(spec["asset"]), spec["sha256"], cache)
            unpack(packed, raw, spec["sha256"], f"{name}: the snapshot", cached=cache is not None)
        elif "files" in spec:
            for file, entry in spec["files"].items():
                raw = work / file
                packed = release_asset(
                    fetch, release_url(spec["release"], entry["asset"]), raw.with_name(entry["asset"]), entry["sha256"], cache
                )
                unpack(packed, raw, entry["sha256"], f"{name}: {file}", cached=cache is not None)
        else:
            raise PinError(f"{name}: pin.json names neither an extract (`asset`) nor derived `files`")
    installed = wordfreq_version()
    if installed != get(record, "sources.wordfreq.version"):
        raise PinError(f"wordfreq {installed} is installed, pin.json records {get(record, 'sources.wordfreq.version')}")


def disk_used(path: Path) -> str:
    """How full the disk holding `path` is, for the notes that measure a run (D8)."""
    try:
        return f"{shutil.disk_usage(path).used / 1e9:.1f} GB of the disk used"
    except OSError:
        return "disk use unknown"


def read_edition(
    edition: str, snapshot: str, editions: Path, *, fetch=download, today=None
) -> tuple[Path, dict]:
    """An edition's whole catalogue, derived from its dump once per run (D4): at the first read of
    the run, the dump is fetched into `<editions>/../dumps/`, every file of the catalogue derived
    into `<editions>/<edition>-<snapshot>/` in one pass, and the dump deleted; a later read finds
    the folder. Answers the folder and what the pin records of the dump — its address, the day it
    was fetched, the date kaikki regenerated it, and the sha256 and size of its decompressed bytes
    with its size as served (D3). The folder is the run's: keyed by edition and snapshot day, never
    meant to be read across runs (a regeneration may fall between two), and removed with the run.
    It counts once it holds `dump.json`, written last: a pass cut short is started again."""
    spec = EDITIONS[edition]
    folder = editions / f"{edition}-{snapshot}"
    done = folder / "dump.json"
    if done.is_file():
        return folder, json.loads(done.read_text(encoding="utf-8"))
    if folder.exists():
        shutil.rmtree(folder)
    folder.mkdir(parents=True)
    dump = editions.parent / "dumps" / f"{edition}-{snapshot}.jsonl.gz"
    try:
        started = time.monotonic()
        headers = fetch(spec["url"], dump) or {}
        fetched = time.monotonic() - started
        served = dump.stat().st_size
        started = time.monotonic()
        identity = derive(dump, spec["files"], folder)
        derived = time.monotonic() - started
        # Measured here, since the repository keeps no dump: its size as served and decompressed,
        # the download and the pass, and the disk at the run's fullest — the dump and its whole
        # catalogue beside each other (migrate-lingua-pack-sources-to-raw-dumps D4, D8).
        print(
            f"note: {source_name(edition)}: {served:,} B as served, fetched in {fetched:.0f} s; "
            f"{identity['size']:,} B decompressed, its catalogue ({len(spec['files'])} files) derived in "
            f"{derived:.0f} s; {disk_used(folder)}; the dump is not kept",
            file=sys.stderr,
        )
    finally:
        dump.unlink(missing_ok=True)
    record = {
        "url": spec["url"],
        "fetched": (today or datetime.date.today()).isoformat(),
        "last_modified": headers.get("last-modified", ""),
        "dump": {**identity, "compressed_size": served},
    }
    part = folder / "dump.json.part"
    part.write_text(json.dumps(record, indent=2) + "\n", encoding="utf-8")
    os.replace(part, done)
    return folder, record


def fetch_live(
    pin: Path,
    work: Path,
    snapshot: str,
    *,
    fetch=download,
    build=build_esdb,
    today=None,
    cache: Path | None = None,
    editions: Path | None = None,
) -> dict:
    """Today's raw sources into `work`, recorded in `pin` as a new snapshot (update mode).

    CEFR-J, Octanove and ESDB stay at their pinned commits — a newer commit is a deliberate edit
    of PINNED or ESDB. kaikki is read as its editions' dumps alone (migrate-lingua-pack-sources-to-
    raw-dumps D1): for each edition the pair reads (`DUMPS`), the files it reads are copied from the
    edition's catalogue, derived at the run's first read of the dump (`read_edition`, into
    `editions`, `work/editions` by default), compressed beside them for the pair's own release and
    recorded by the sha256 of their decompressed bytes, with the dump they came from (D3). Every
    pair records its own fetch, under its own release, even of a file another pair reads
    (add-lingua-pack-es-en D2). No extract is read, and no `kaikki` record is written. A reader
    pair's record of the studied tables it read (`studied`) is carried over until `record-build`
    writes it anew.
    """
    record = load(pin)
    pair = pair_of(pin)
    check_registered(pair)
    work.mkdir(parents=True, exist_ok=True)
    editions = work / "editions" if editions is None else editions
    sources: dict = {}
    for name, spec in PINNED.get(pair, {}).items():
        dest = work / spec["file"]
        fetch(spec["url"], dest)
        sources[name] = {"url": spec["url"], "sha256": sha256(dest)}
    esdb = ESDB.get(pair)
    if esdb is not None:
        sources["esdb"] = {k: esdb[k] for k in ("repository", "tag", "commit")}
        sources["esdb"]["sha256"] = sha256(build(esdb, work))
    for edition, reads in DUMPS[pair].items():
        folder, dumped = read_edition(edition, snapshot, editions, fetch=fetch, today=today)
        files = {}
        for file in reads:
            raw = work / file
            shutil.copyfile(folder / file, raw)
            asset = file + ".zst"
            files[file] = {"asset": asset, "sha256": pack_asset(raw, asset, cache), "size": raw.stat().st_size}
        sources[source_name(edition)] = {
            "release": release_tag(pair, snapshot),
            "url": dumped["url"],
            "fetched": dumped["fetched"],
            "last_modified": dumped["last_modified"],
            "dump": dumped["dump"],
            "files": files,
        }
    sources["wordfreq"] = {"version": wordfreq_version()}
    if sources["wordfreq"]["version"] != WORDFREQ:
        raise PinError(f"wordfreq {sources['wordfreq']['version']} is installed; the pipeline pins {WORDFREQ}")
    print(f"note: {pair}: its sources read and compressed; {disk_used(work)}", file=sys.stderr)
    kept = {"snapshot": snapshot, "pack": record.get("pack", {}), "reducer": record.get("reducer", {})}
    if "studied" in record:
        kept["studied"] = record["studied"]
    record = {**kept, "sources": sources}
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


def studied_record(pin: Path) -> dict | None:
    """What a pair's build read of its studied language's folder (add-lingua-pack-es-en D3): the
    reference pair, and the sha256 of each studied table committed there (`RECORDED_STUDIED`), for
    a pair that is not the reference; None for the reference, whose own reduction writes them."""
    studied = studied_dir(pin)
    reference = reference_of(studied)
    if reference is None or reference == pair_of(pin):
        return None
    tables = {name: sha256(studied / name) for name in RECORDED_STUDIED if (studied / name).is_file()}
    return {"reference": reference, "tables": tables}


def studied_digest(tables: dict) -> str:
    """One digest over a reader pair's record of the studied tables (`studied.tables`): each
    table's name and sha256, in `RECORDED_STUDIED` order, a table the folder lacks as none."""
    digest = hashlib.sha256()
    for name in RECORDED_STUDIED:
        digest.update(f"{name}\0{tables.get(name) or ''}\n".encode())
    return digest.hexdigest()


def pack_version(pin: Path, reducer: Path, *, live: bool = False) -> str:
    """The `pack_version` a pair's tables are reduced under (lingua-data-packs, *A pack says which
    dictionary it is*): its snapshot for an update (`live`), and its snapshot and rules after a
    re-reduction of the same sources (`2026.10.03+0d876dc`). A pair that is not its studied
    language's reference reads tables it does not write, so its version names them too, in either
    mode: its snapshot, its rules and the digest of the studied tables its build reads
    (`2026.10.08+733f815.5e0c2a1`, add-lingua-pack-es-en D3) — the version moves when they do,
    though nothing of the pair's own moved."""
    snapshot = get(load(pin), "snapshot")
    studied = studied_record(pin)
    if studied is None:
        return snapshot if live else f"{snapshot}+{rules_sha256(reducer)[:7]}"
    return f"{snapshot}+{rules_sha256(reducer)[:7]}.{studied_digest(studied['tables'])[:7]}"


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
    studied = studied_record(pin)
    if studied is None:
        record.pop("studied", None)
    else:
        record["studied"] = studied
    save(pin, {k: record[k] for k in ("snapshot", "pack", "reducer", "studied", "sources") if k in record})


def released(record: dict, release: str | None = None) -> list[tuple[str, dict]]:
    """The records of a pin whose bytes a release keeps — each names its `release` — in the pin's
    order; with `release`, those of that release alone. A record of either shape: derived `files`,
    or a legacy extract's `asset` (D5); a record naming neither fails, naming it."""
    out = []
    for name, spec in get(record, "sources").items():
        if not isinstance(spec, dict) or "release" not in spec:
            continue
        if "asset" not in spec and "files" not in spec:
            raise PinError(f"{name}: pin.json names release {spec['release']} but neither an extract (`asset`) nor `files`")
        if release is None or spec["release"] == release:
            out.append((name, spec))
    return out


def assets(record: dict, release: str | None = None) -> list[str]:
    """The files the snapshot's releases hold, in the record's order: a legacy extract's asset
    (D5), each derived file's. With `release`, the files that release holds alone: an update
    publishes a pair's own assets only, under its own release (add-lingua-pack-es-en D2)."""
    out = []
    for _, spec in released(record, release):
        if "asset" in spec:
            out.append(spec["asset"])
        out.extend(file["asset"] for file in (spec.get("files") or {}).values())
    return out


def dumps(record: dict, release: str | None = None) -> list[str]:
    """What a release's notes say of the kaikki bytes its assets come from (D7), one Markdown item
    per record: each edition's dump — its address, the date kaikki regenerated it, the sha256 and
    size of its decompressed bytes — and the files derived from it. A record written before the
    dumps is named as it is recorded: derived files without the dump's identity, or a per-language
    extract kept whole."""
    out = []
    for name, spec in released(record, release):
        url, regenerated = spec.get("url", "?"), spec.get("last_modified") or "on a date not recorded"
        if "asset" in spec:
            out.append(
                f"- `{name}`: kaikki's per-language extract {url}, regenerated {regenerated}, kept whole as "
                f"`{spec['asset']}` (sha256 {spec.get('sha256', '?')}, {spec.get('size', 0):,} B)"
            )
            continue
        derived = ", ".join(f"`{file}`" for file in spec.get("files") or {})
        dump = spec.get("dump")
        if isinstance(dump, dict):
            identity = (
                f"decompressed sha256 {dump.get('sha256', '?')}, {dump.get('size', 0):,} B "
                f"({dump.get('compressed_size', 0):,} B as served)"
            )
        else:
            identity = "the dump's sha256 was not recorded (a pin written before the dumps were)"
        out.append(f"- `{name}`: the dump {url}, regenerated {regenerated}; {identity}; derived: {derived}")
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
        check_studied_tables(pin)


def check_studied_tables(pin: Path) -> None:
    """A reader pair's record of the studied tables its build read (`studied` in pin.json), against
    the committed ones (add-lingua-pack-es-en D3): a table the reference's reduction moved since
    left the pair behind, and the check names the pair and the table. The pack's sha256 catches the
    move too; this says which table."""
    pair, studied = pair_of(pin), studied_dir(pin)
    reference = reference_of(studied)
    if reference is None or reference == pair:
        return
    again = f"reduce {pair} again from its pinned sources (scripts/lingua-data/build.sh --reduce {pair} <out>, or lingua-pack-update with mode=reduce)"
    recorded = load(pin).get("studied")
    if not isinstance(recorded, dict) or not isinstance(recorded.get("tables"), dict):
        raise PinError(
            f"{pair} reads {studied.name}/, and its pin.json records nothing of the studied tables it was built "
            f"on (no `studied`): {again}."
        )
    if recorded.get("reference") != reference:
        raise PinError(
            f"{pair}: pin.json records {recorded.get('reference')!r} as the pair whose reduction writes "
            f"{studied.name}/, and {studied.name}/{STUDIED_RECORD} names {reference}: {again}."
        )
    for name in RECORDED_STUDIED:
        got = sha256(studied / name) if (studied / name).is_file() else None
        want = recorded["tables"].get(name)
        if got != want:
            # The pinned tag pool is an input no reduction writes: a person edits it.
            moved = "it was edited" if name in KEPT_INPUTS else f"{reference}'s reduction moved it"
            raise PinError(
                f"{pair}: {studied.name}/{name} is not what {pair}'s tables were built on (sha256 "
                f"{got or 'no file'}, pin.json records {want or 'no file'}): {moved}, and {pair} was left "
                f"behind: {again}."
            )


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
    commands += ("dumps",)
    commands += ("release-tag", "version")
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
            p.add_argument("--cache", type=Path, help="keep fetched release assets here, by sha256, across pairs")
        if name == "fetch-live":
            p.add_argument("--snapshot", default=datetime.date.today().strftime("%Y.%m.%d"))
            p.add_argument(
                "--editions", type=Path, help="derive each edition's catalogue here, once per run (default: WORK/editions)"
            )
        if name in ("assets", "dumps"):
            p.add_argument("--release", help="the records of this release alone (a pair's own)")
        if name in ("record-build", "check-pack"):
            p.add_argument("--pack", type=Path, required=True)
        if name in ("record-build", "check-reducer", "rules", "version"):
            p.add_argument("--reducer", type=Path, required=True)
        if name == "version":
            p.add_argument("--live", action="store_true", help="an update's version: today's snapshot")
        if name == "get":
            p.add_argument("key")
    a = ap.parse_args(argv)
    try:
        if a.cmd == "fetch-pinned":
            check_python()
            fetch_pinned(a.pin, a.work, cache=a.cache)
        elif a.cmd == "fetch-live":
            check_python()
            fetch_live(a.pin, a.work, a.snapshot, cache=a.cache, editions=a.editions)
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
            print("\n".join(assets(load(a.pin), a.release)))
        elif a.cmd == "dumps":
            print("\n".join(dumps(load(a.pin), a.release)))
        elif a.cmd == "release-tag":
            print(release_tag(pair_of(a.pin), get(load(a.pin), "snapshot")))
        elif a.cmd == "version":
            print(pack_version(a.pin, a.reducer, live=a.live))
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
