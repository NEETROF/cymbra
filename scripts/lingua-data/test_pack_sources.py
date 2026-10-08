# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the pinned sources and the record (pin-lingua-pack-sources), with no download.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import ast
import contextlib
import datetime
import gzip
import hashlib
import io
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile
import unittest
from pathlib import Path

HERE = Path(__file__).resolve().parent
sys.path.insert(0, str(HERE))

import pack_sources as ps  # noqa: E402

HAS_ZSTD = shutil.which("zstd") is not None


# What makes a module load code other than by an import statement: `rule_files` reads `sys.modules`
# once the reducer is imported, and the import-statement test reads the source, so both miss it.
_DYNAMIC_MODULES = {"importlib", "runpy", "imp", "pkgutil"}
_DYNAMIC_NAMES = {"__import__", "exec", "eval"}


def _dynamic_loads(source):
    """Where `source` loads code by name or by path rather than by an import statement."""
    found = []
    for node in ast.walk(ast.parse(source)):
        modules = [a.name for a in node.names] if isinstance(node, ast.Import) else []
        if isinstance(node, ast.ImportFrom):
            modules = [node.module or ""]
        found += [f"{node.lineno}: imports {m}" for m in modules if m.split(".")[0] in _DYNAMIC_MODULES]
        name = node.id if isinstance(node, ast.Name) else node.attr if isinstance(node, ast.Attribute) else None
        if name in _DYNAMIC_NAMES:
            found.append(f"{node.lineno}: {name}")
    return found


class Pinned(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.pin = self.root / "tables" / "en-fr" / "pin.json"
        self.work = self.root / "work"
        # What "upstream" serves, by URL.
        self.served = {spec["url"]: f"{name} bytes\n".encode() for name, spec in ps.PINNED["en-fr"].items()}
        self.served[ps.KAIKKI["en-fr"]["url"]] = b'{"word": "harbour"}\n'
        self.esdb_export = "35: run <n_v>: ran, run, running, runs, run's\n"

    def tearDown(self):
        self._tmp.cleanup()

    def build(self, spec, work):
        """ESDB's export, as its pinned commit gives it (no git, no build here)."""
        out = Path(work) / spec["file"]
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text(self.esdb_export, encoding="utf-8")
        return out

    def fetch(self, url, dest, compressed=False):
        dest.parent.mkdir(parents=True, exist_ok=True)
        if url in self.served:
            dest.write_bytes(self.served[url])
            return {"last-modified": "Thu, 24 Sep 2026 00:04:03 GMT"}
        released = self.work.parent / "released" / Path(url).name
        shutil.copy(released, dest)
        return {}

    def update(self, snapshot="2026.09.26"):
        import unittest.mock as mock

        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            return ps.fetch_live(
                self.pin, self.work, snapshot, fetch=self.fetch, build=self.build, today=datetime.date(2026, 9, 26)
            )

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_update_records_every_source_and_keeps_kaikki(self):
        record = self.update()
        self.assertEqual(record["snapshot"], "2026.09.26")
        sources = record["sources"]
        self.assertEqual(set(sources), {"esdb", "cefrj", "octanove", "kaikki", "wordfreq"})
        self.assertEqual(sources["esdb"]["commit"], ps.ESDB["en-fr"]["commit"])
        self.assertRegex(sources["esdb"]["sha256"], r"^[0-9a-f]{64}$")
        for name in ("cefrj", "octanove"):
            self.assertRegex(sources[name]["url"], r"/[0-9a-f]{40}/", f"{name} must be read at a commit")
            self.assertRegex(sources[name]["sha256"], r"^[0-9a-f]{64}$")
        kaikki = sources["kaikki"]
        self.assertEqual(kaikki["release"], "lingua-pack-sources-en-fr-2026.09.26")
        self.assertEqual(kaikki["asset"], "kaikki-Anglais.jsonl.zst")
        self.assertTrue((self.work / kaikki["asset"]).is_file(), "the snapshot to publish is left beside the source")
        self.assertEqual(kaikki["last_modified"], "Thu, 24 Sep 2026 00:04:03 GMT")
        self.assertEqual(sources["wordfreq"], {"version": "3.1.1"})
        self.assertEqual(json.loads(self.pin.read_text()), record, "the record round-trips")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_reduce_fetches_the_recorded_bytes_and_refuses_others(self):
        import unittest.mock as mock

        self.update()
        released = self.root / "released"
        released.mkdir()
        shutil.copy(self.work / "kaikki-Anglais.jsonl.zst", released / "kaikki-Anglais.jsonl.zst")
        again = self.root / "again"
        self.work = again
        with mock.patch.object(ps, "wordfreq_version", return_value="3.1.1"):
            ps.fetch_pinned(self.pin, again, fetch=self.fetch, build=self.build)
            self.assertEqual((again / "kaikki-Anglais.jsonl").read_bytes(), b'{"word": "harbour"}\n')
            # Upstream changed the bytes behind a pinned address.
            self.served[ps.PINNED["en-fr"]["cefrj"]["url"]] = b"something else\n"
            with self.assertRaisesRegex(ps.PinError, "cefrj"):
                ps.fetch_pinned(self.pin, again, fetch=self.fetch, build=self.build)
            # ESDB's export changed at the same commit.
            self.served[ps.PINNED["en-fr"]["cefrj"]["url"]] = b"cefrj bytes\n"
            self.esdb_export += "35: go <v>: went, gone, going, goes\n"
            with self.assertRaisesRegex(ps.PinError, "esdb"):
                ps.fetch_pinned(self.pin, again, fetch=self.fetch, build=self.build)

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_reduce_refuses_another_wordfreq(self):
        import unittest.mock as mock

        self.update()
        released = self.root / "released"
        released.mkdir()
        shutil.copy(self.work / "kaikki-Anglais.jsonl.zst", released / "kaikki-Anglais.jsonl.zst")
        with mock.patch.object(ps, "wordfreq_version", return_value="3.2.0"):
            with self.assertRaisesRegex(ps.PinError, "wordfreq"):
                ps.fetch_pinned(self.pin, self.root / "again", fetch=self.fetch, build=self.build)


    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_a_record_from_before_esdb_takes_it_in_and_drops_agid(self):
        import unittest.mock as mock

        self.update()
        record = ps.load(self.pin)
        del record["sources"]["esdb"]
        record["sources"]["agid"] = {"url": "https://raw.githubusercontent.com/en-wl/wordlist/464bea8c/agid/infl.txt", "sha256": "0" * 64}
        ps.save(self.pin, record)
        released = self.root / "released"
        released.mkdir()
        shutil.copy(self.work / "kaikki-Anglais.jsonl.zst", released / "kaikki-Anglais.jsonl.zst")
        with mock.patch.object(ps, "wordfreq_version", return_value="3.1.1"):
            ps.fetch_pinned(self.pin, self.root / "again", fetch=self.fetch, build=self.build)
        sources = ps.load(self.pin)["sources"]
        self.assertNotIn("agid", sources)
        self.assertEqual(sources["esdb"]["tag"], "rel-2026.02.25")
        self.assertEqual(sources["esdb"]["sha256"], ps.sha256(self.root / "again" / "scowl.txt"))


class Dumps(unittest.TestCase):
    """es-fr reads kaikki's dumps of whole Wiktionary editions, kept as the files it derives."""

    FR = [
        {"word": "casa", "lang_code": "es", "pos": "noun", "senses": [{"glosses": ["Maison."]}]},
        {
            "word": "maison",
            "lang_code": "fr",
            "pos": "noun",
            "translations": [
                {"lang_code": "es", "word": "casa", "sense": "Bâtiment"},
                {"lang_code": "it", "word": "casa"},
            ],
        },
        {"word": "chat", "lang_code": "fr", "pos": "noun", "translations": [{"lang_code": "en", "word": "cat"}]},
        {"word": "Haus", "lang_code": "de", "pos": "noun"},
    ]
    ES = [
        {
            "word": "sector",
            "lang_code": "es",
            "pos": "noun",
            "translations": [{"lang_code": "fr", "word": "secteur"}, {"lang_code": "en", "word": "sector"}],
        },
        {"word": "casa", "lang_code": "es", "pos": "noun"},
    ]

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.pin = self.root / "tables" / "es-fr" / "pin.json"
        self.work = self.root / "work"
        self.served = {spec["url"]: f"{name} bytes\n".encode() for name, spec in ps.PINNED["es-fr"].items()}
        self.served[ps.KAIKKI["es-fr"]["url"]] = b'{"word": "casa"}\n'
        dumps = ps.DUMPS["es-fr"]
        self.served[dumps["kaikki-fr"]["url"]] = self.dump(self.FR)
        self.served[dumps["kaikki-es"]["url"]] = self.dump(self.ES)

    def tearDown(self):
        self._tmp.cleanup()

    @staticmethod
    def dump(entries):
        return gzip.compress("".join(json.dumps(e, ensure_ascii=False) + "\n" for e in entries).encode())

    def fetch(self, url, dest, compressed=False):
        dest.parent.mkdir(parents=True, exist_ok=True)
        if url in self.served:
            dest.write_bytes(self.served[url])
            return {"last-modified": "Fri, 02 Oct 2026 00:10:16 GMT"}
        shutil.copy(self.root / "released" / Path(url).name, dest)
        return {}

    def update(self):
        import unittest.mock as mock

        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            return ps.fetch_live(self.pin, self.work, "2026.10.04", fetch=self.fetch, today=datetime.date(2026, 10, 4))

    def test_derive_keeps_a_language_s_entries_and_cuts_tables_down_to_translations(self):
        self.work.mkdir()
        dump = self.work / "fr.jsonl.gz"
        dump.write_bytes(self.dump(self.FR))
        ps.derive(dump, ps.DUMPS["es-fr"]["kaikki-fr"]["files"], self.work)
        entries = (self.work / "kaikki-fr-Espagnol.jsonl").read_text(encoding="utf-8")
        self.assertEqual(entries, json.dumps(self.FR[0], ensure_ascii=False) + "\n", "the line as the dump writes it")
        tables = [json.loads(line) for line in (self.work / "kaikki-fr-traductions.jsonl").read_text().splitlines()]
        # `chat` lists no Spanish word; the Italian translation of `maison` is not read.
        self.assertEqual(tables, [{"pos": "noun", "translations": [{"sense": "Bâtiment", "word": "casa"}], "word": "maison"}])

    def test_an_entry_level_table_derives_byte_for_byte_as_before(self):
        # generalise-lingua-gloss-reducer D3: the French and Spanish Wiktionaries write their tables per
        # entry, and es-fr's pinned derived files must re-derive to their sha256.
        self.work.mkdir()
        dump = self.work / "fr.jsonl.gz"
        dump.write_bytes(self.dump(self.FR))
        ps.derive(dump, {"t.jsonl": ("translations", "fr", "es")}, self.work)
        self.assertEqual(
            (self.work / "t.jsonl").read_text(encoding="utf-8"),
            '{"pos": "noun", "translations": [{"sense": "Bâtiment", "word": "casa"}], "word": "maison"}\n',
        )

    def test_a_table_under_a_sense_is_derived_with_the_sense_it_names(self):
        # The English Wiktionary writes its tables under its senses (68,579 English entries list
        # Spanish translations under a sense, 5,080 for the whole entry).
        house = {
            "word": "house",
            "lang_code": "en",
            "pos": "noun",
            "translations": [{"lang_code": "es", "word": "hogar", "sense": "home"}],
            "senses": [
                {
                    "glosses": ["A structure serving as an abode of human beings."],
                    "translations": [
                        {"lang_code": "fr", "word": "maison", "sense": "abode of a human being"},
                        {"lang_code": "es", "word": "casa", "sense": "abode of a human being"},
                        {"lang_code": "es", "word": "vivienda", "sense": "abode of a human being"},
                    ],
                },
                {"glosses": ["A dynasty."], "translations": [{"lang_code": "es", "word": "casa"}]},
                {"glosses": ["An audience."]},
            ],
        }
        self.work.mkdir()
        dump = self.work / "en.jsonl.gz"
        dump.write_bytes(self.dump([house, {"word": "home", "lang_code": "en", "pos": "noun", "senses": [{}]}]))
        ps.derive(dump, {"en-es.jsonl": ("translations", "en", "es")}, self.work)
        lines = (self.work / "en-es.jsonl").read_text(encoding="utf-8").splitlines()
        self.assertEqual(
            [json.loads(line) for line in lines],
            [
                {
                    "word": "house",
                    "pos": "noun",
                    "translations": [
                        {"word": "hogar", "sense": "home"},
                        {"word": "casa", "sense": "abode of a human being"},
                        {"word": "vivienda", "sense": "abode of a human being"},
                        {"word": "casa"},
                    ],
                }
            ],
            "the entry's table first, then each sense's, with the sense a table names; no French word",
        )

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_update_keeps_each_derived_file_as_an_asset_of_the_snapshot(self):
        record = self.update()
        sources = record["sources"]
        self.assertEqual(set(sources), {"gsd-train", "gsd-dev", "kaikki", "kaikki-fr", "kaikki-es", "wordfreq"})
        fr = sources["kaikki-fr"]
        self.assertEqual(fr["release"], "lingua-pack-sources-es-fr-2026.10.04")
        self.assertEqual(fr["last_modified"], "Fri, 02 Oct 2026 00:10:16 GMT")
        self.assertEqual(set(fr["files"]), {"kaikki-fr-Espagnol.jsonl", "kaikki-fr-traductions.jsonl"})
        for name, file in {**fr["files"], **sources["kaikki-es"]["files"]}.items():
            self.assertEqual(file["asset"], name + ".zst")
            self.assertEqual(file["sha256"], ps.sha256(self.work / name))
            self.assertTrue((self.work / file["asset"]).is_file(), "the asset to publish is left beside it")
        self.assertFalse(list(self.work.glob("*.dump.jsonl.gz")), "a dump is never kept whole")
        self.assertEqual(
            ps.assets(record),
            [
                "kaikki-Spanish.jsonl.zst",
                "kaikki-fr-Espagnol.jsonl.zst",
                "kaikki-fr-traductions.jsonl.zst",
                "kaikki-es-traductions.jsonl.zst",
            ],
        )

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_reduce_fetches_each_derived_file_and_refuses_other_bytes(self):
        import unittest.mock as mock

        record = self.update()
        released = self.root / "released"
        released.mkdir()
        for asset in ps.assets(record):
            shutil.copy(self.work / asset, released / asset)
        again = self.root / "again"
        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            ps.fetch_pinned(self.pin, again, fetch=self.fetch)
            self.assertEqual(
                (again / "kaikki-es-traductions.jsonl").read_bytes(), (self.work / "kaikki-es-traductions.jsonl").read_bytes()
            )
            self.assertIn("kaikki-fr", ps.load(self.pin)["sources"], "a derived source is no retired one")
            subprocess.run(["zstd", "-q", "-f", "-o", str(released / "kaikki-fr-traductions.jsonl.zst"), "-"], input=b"{}\n", check=True)
            with self.assertRaisesRegex(ps.PinError, "kaikki-fr: kaikki-fr-traductions.jsonl"):
                ps.fetch_pinned(self.pin, again, fetch=self.fetch)


class SharedSources(unittest.TestCase):
    """Two pairs share a source (add-lingua-pack-es-en D2): es-en reads the extract es-fr reads,
    through one asset cache, and each update publishes a pair's own assets only."""

    ES = [
        {
            "word": "sector",
            "lang_code": "es",
            "pos": "noun",
            "translations": [{"lang_code": "fr", "word": "secteur"}, {"lang_code": "en", "word": "sector"}],
        },
    ]
    FR = [{"word": "casa", "lang_code": "es", "pos": "noun", "senses": [{"glosses": ["Maison."]}]}]

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.tables = self.root / "tables"
        (self.tables / "es").mkdir(parents=True)
        (self.tables / "es" / "studied.json").write_text('{"reference": "es-fr"}\n')
        (self.tables / "es" / "tags.tsv").write_text("NOUN\n")
        self.cache = self.root / "work" / "cache"
        self.served = {spec["url"]: f"{name} bytes\n".encode() for name, spec in ps.PINNED["es-fr"].items()}
        self.served[ps.KAIKKI["es-fr"]["url"]] = b'{"word": "casa", "senses": [{"glosses": ["house"]}]}\n'
        self.served[ps.DUMPS["es-fr"]["kaikki-fr"]["url"]] = Dumps.dump(self.FR)
        self.served[ps.DUMPS["es-fr"]["kaikki-es"]["url"]] = Dumps.dump(self.ES)
        self.fetched = []

    def tearDown(self):
        self._tmp.cleanup()

    def fetch(self, url, dest, compressed=False):
        self.fetched.append(url)
        dest.parent.mkdir(parents=True, exist_ok=True)
        if url in self.served:
            dest.write_bytes(self.served[url])
            return {"last-modified": "Sat, 03 Oct 2026 00:00:00 GMT"}
        shutil.copy(self.root / "released" / Path(url).name, dest)
        return {}

    def pin(self, pair):
        return self.tables / pair / "pin.json"

    def work(self, pair):
        return self.root / "work" / pair

    def update(self, pair, snapshot="2026.10.08", cache=True):
        import unittest.mock as mock

        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            return ps.fetch_live(
                self.pin(pair),
                self.work(pair),
                snapshot,
                fetch=self.fetch,
                today=datetime.date(2026, 10, 8),
                cache=self.cache if cache else None,
            )

    def test_the_two_pairs_name_one_extract_and_their_own_derived_files(self):
        self.assertEqual(ps.KAIKKI["es-en"], ps.KAIKKI["es-fr"], "one address, one file")
        self.assertEqual(ps.DUMPS["es-en"]["kaikki-es"]["url"], ps.DUMPS["es-fr"]["kaikki-es"]["url"])
        self.assertEqual(
            ps.DUMPS["es-en"]["kaikki-es"]["files"], {"kaikki-es-traductions-en.jsonl": ("translations", "es", "en")}
        )
        self.assertEqual(ps.release_tag("es-en", "2026.10.08"), "lingua-pack-sources-es-en-2026.10.08")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_spec_scenario_a_shared_extract(self):
        # es-fr's update, then es-en's in the same run: es-en downloads no extract, its pin names
        # es-fr's release for it, and es-en's own release holds its derived file alone.
        reference = self.update("es-fr")
        extract = ps.KAIKKI["es-fr"]["url"]
        self.assertEqual(self.fetched.count(extract), 1)
        self.assertTrue((self.cache / reference["sources"]["kaikki"]["sha256"]).is_file(), "kept in the cache")
        self.assertEqual(ps.assets(reference, ps.release_tag("es-fr", "2026.10.08")), ps.assets(reference))

        with contextlib.redirect_stderr(io.StringIO()) as err:
            reader = self.update("es-en")
        self.assertEqual(self.fetched.count(extract), 1, "es-en's update downloads no extract")
        self.assertIn("read from it", err.getvalue())
        kaikki = reader["sources"]["kaikki"]
        self.assertEqual(kaikki["release"], "lingua-pack-sources-es-fr-2026.10.08", "es-fr's release for the extract")
        self.assertEqual(kaikki, reference["sources"]["kaikki"])
        self.assertEqual(
            (self.work("es-en") / "kaikki-Spanish.jsonl").read_bytes(), (self.work("es-fr") / "kaikki-Spanish.jsonl").read_bytes()
        )
        self.assertFalse((self.work("es-en") / "kaikki-Spanish.jsonl.zst").exists(), "nothing of it to publish")
        derived = reader["sources"]["kaikki-es"]
        self.assertEqual(derived["release"], "lingua-pack-sources-es-en-2026.10.08", "its own release")
        self.assertEqual(list(derived["files"]), ["kaikki-es-traductions-en.jsonl"])
        line = json.loads((self.work("es-en") / "kaikki-es-traductions-en.jsonl").read_text(encoding="utf-8"))
        self.assertEqual(line, {"pos": "noun", "translations": [{"word": "sector"}], "word": "sector"}, "English words alone")
        self.assertEqual(set(reader["sources"]), {"kaikki", "kaikki-es", "wordfreq"})
        # The update publishes a pair's own assets only: the derived file, under es-en's tag.
        own = ps.release_tag("es-en", "2026.10.08")
        self.assertEqual(ps.assets(reader, own), ["kaikki-es-traductions-en.jsonl.zst"])
        self.assertEqual(ps.assets(reader), ["kaikki-Spanish.jsonl.zst", "kaikki-es-traductions-en.jsonl.zst"])
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["assets", "--pin", str(self.pin("es-en")), "--release", own]), 0)
            self.assertEqual(ps.main(["release-tag", "--pin", str(self.pin("es-en"))]), 0)
        self.assertEqual(out.getvalue(), f"kaikki-es-traductions-en.jsonl.zst\n{own}\n")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_updated_alone_a_reader_pair_fetches_and_publishes_its_own_extract(self):
        # Dispatched alone (the bootstrap), or a day after its reference: its own release holds
        # the extract and the derived file.
        self.update("es-fr", snapshot="2026.10.07")
        reader = self.update("es-en")
        self.assertEqual(self.fetched.count(ps.KAIKKI["es-en"]["url"]), 2)
        self.assertEqual(reader["sources"]["kaikki"]["release"], "lingua-pack-sources-es-en-2026.10.08")
        self.assertTrue((self.work("es-en") / "kaikki-Spanish.jsonl.zst").is_file(), "to publish")
        self.assertEqual(
            ps.assets(reader, ps.release_tag("es-en", "2026.10.08")),
            ["kaikki-Spanish.jsonl.zst", "kaikki-es-traductions-en.jsonl.zst"],
        )
        # Without a cache, nothing is shared: the reference's fetch is not known.
        self.fetched.clear()
        self.update("es-fr")
        self.update("es-en", cache=False)
        self.assertEqual(self.fetched.count(ps.KAIKKI["es-en"]["url"]), 2)

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_a_pinned_fetch_follows_another_pair_s_release_through_the_cache(self):
        # The reduce job: es-fr reduced, then es-en, each from its pinned sources, the shared
        # extract fetched once — and a cache entry that is not what its name says is dropped.
        import unittest.mock as mock

        self.update("es-fr")
        reader = self.update("es-en")
        released = self.root / "released"
        released.mkdir()
        for pair, record in (("es-fr", ps.load(self.pin("es-fr"))), ("es-en", reader)):
            for asset in ps.assets(record, ps.release_tag(pair, "2026.10.08")):
                shutil.copy(self.work(pair) / asset, released / asset)
        shutil.rmtree(self.root / "work")
        self.fetched.clear()
        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            ps.fetch_pinned(self.pin("es-fr"), self.work("es-fr"), fetch=self.fetch, cache=self.cache)
            ps.fetch_pinned(self.pin("es-en"), self.work("es-en"), fetch=self.fetch, cache=self.cache)
        extract = ps.release_url("lingua-pack-sources-es-fr-2026.10.08", "kaikki-Spanish.jsonl.zst")
        self.assertEqual(self.fetched.count(extract), 1, "fetched once, for both pairs")
        self.assertIn(ps.release_url("lingua-pack-sources-es-en-2026.10.08", "kaikki-es-traductions-en.jsonl.zst"), self.fetched)
        self.assertEqual(
            (self.work("es-en") / "kaikki-Spanish.jsonl").read_bytes(), (self.work("es-fr") / "kaikki-Spanish.jsonl").read_bytes()
        )
        self.assertTrue((self.work("es-en") / "kaikki-es-traductions-en.jsonl").is_file())
        # A stale cache entry: dropped, and the fetch fails naming the source.
        sha = reader["sources"]["kaikki"]["sha256"]
        subprocess.run(["zstd", "-q", "-f", "-o", str(self.cache / sha), "-"], input=b"{}\n", check=True)
        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            with self.assertRaisesRegex(ps.PinError, "kaikki: the snapshot decompresses"):
                ps.fetch_pinned(self.pin("es-en"), self.work("es-en"), fetch=self.fetch, cache=self.cache)
        self.assertFalse((self.cache / sha).exists(), "dropped")
        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            ps.fetch_pinned(self.pin("es-en"), self.work("es-en"), fetch=self.fetch, cache=self.cache)
        self.assertTrue((self.cache / sha).is_file(), "fetched again")

    def test_build_sh_keeps_the_cache_outside_a_pair_s_work_folder(self):
        script = (HERE / "build.sh").read_text()
        self.assertIn('cache="${LINGUA_CACHE:-$here/work/cache}"', script)
        self.assertIn('fetch-pinned --pin "$pin" --work "$work" --cache "$cache"', script)
        self.assertIn('fetch-live --pin "$pin" --work "$work" --snapshot "$snapshot" --cache "$cache"', script)
        self.assertIn('LINGUA_STUDIED="$studied"', script, "a reader pair reads this run's studied folder")
        self.assertIn("es-*) echo 60000", script, "every Spanish pair keeps 60,000 lemmas")


class ReaderPair(unittest.TestCase):
    """A pair that is not its studied language's reference records the studied tables its build
    read (add-lingua-pack-es-en D3): es-en, beside es-fr, which writes tables/es/."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.es = self.root / "es"
        self.es.mkdir()
        (self.es / "studied.json").write_text('{"reference": "es-fr"}\n')
        tables = {
            "forms.tsv": "casa\tcasa\ncasas\tcasa\n",
            "freq.tsv": "casa\t1\n",
            "grammar.tsv": "casa\tcasa\tNOUN|Gender=Fem|Number=Sing\t-\n",
            "level.tsv": "casa\tA1\n",
            "lexical.tsv": "casa\n",
            "tags.tsv": "NOUN\n",
        }
        for name, text in tables.items():
            (self.es / name).write_text(text)
        self.pins = {}
        for pair in ("es-fr", "es-en"):
            self.pins[pair] = self.root / pair / "pin.json"
            ps.save(self.pins[pair], {"snapshot": "2026.10.08", "sources": {"wordfreq": {"version": "3.1.1"}}})
            (self.root / f"reduce-{pair}.py").write_text(f"# {pair}'s rules\n")
        self.pack = self.root / "pack.lingua"
        self.pack.write_bytes(b"LINGUA pack")
        for pair in self.pins:
            self.record(pair)

    def tearDown(self):
        self._tmp.cleanup()

    def record(self, pair):
        ps.record_build(self.pins[pair], self.pack, self.root / f"reduce-{pair}.py")

    def check(self, pair):
        ps.check_reducer(self.pins[pair], self.root / f"reduce-{pair}.py")

    def test_the_reader_s_pin_records_the_reference_and_the_six_studied_tables(self):
        record = ps.load(self.pins["es-en"])
        self.assertEqual(list(record), ["snapshot", "pack", "reducer", "studied", "sources"])
        self.assertEqual(record["studied"]["reference"], "es-fr")
        self.assertEqual(
            record["studied"]["tables"], {name: ps.sha256(self.es / name) for name in ps.RECORDED_STUDIED}
        )
        self.assertEqual(
            sorted(ps.RECORDED_STUDIED),
            ["forms.tsv", "freq.tsv", "grammar.tsv", "level.tsv", "lexical.tsv", "tags.tsv"],
            "the six studied tables; studied.json is the record, not a table",
        )
        # The reference writes them: its pin records nothing of its own studied folder.
        self.assertNotIn("studied", ps.load(self.pins["es-fr"]))
        self.check("es-fr")
        self.check("es-en")
        # A fetch of the pinned sources saves the record as it is, `studied` kept.
        record = ps.load(self.pins["es-en"])
        ps.save(self.pins["es-en"], record)
        self.assertEqual(ps.load(self.pins["es-en"]), record)

    def test_spec_scenario_the_studied_side_moves(self):
        # es-fr is reduced again and Spanish's levels move; es-en is not recorded again.
        (self.es / "level.tsv").write_text("casa\tA2\n")
        self.record("es-fr")
        self.check("es-fr")
        with self.assertRaisesRegex(ps.PinError, r"^es-en: es/level\.tsv is not what es-en's tables were built on"):
            self.check("es-en")
        with contextlib.redirect_stderr(io.StringIO()) as err:
            code = ps.main(
                ["check-reducer", "--pin", str(self.pins["es-en"]), "--reducer", str(self.root / "reduce-es-en.py")]
            )
        self.assertEqual(code, 1)
        self.assertIn("es-en: es/level.tsv", err.getvalue())
        self.assertIn("es-fr's reduction moved it, and es-en was left behind", err.getvalue())
        # Reduced again after es-fr, es-en records the new table, and passes.
        self.record("es-en")
        self.check("es-en")

    def test_spec_scenario_a_rules_only_change_of_the_reference(self):
        # es-fr's rules change and es-fr is reduced again, no studied table moving: es-en's pin is
        # byte for byte what it was, and its checks pass.
        before = self.pins["es-en"].read_bytes()
        (self.root / "reduce-es-fr.py").write_text("# es-fr's rules, edited\n")
        with self.assertRaisesRegex(ps.PinError, "es-en reads es/, which es-fr's reduction writes"):
            self.check("es-en")
        self.record("es-fr")
        self.check("es-en")
        self.assertEqual(self.pins["es-en"].read_bytes(), before)

    def test_a_reader_s_pin_that_records_no_studied_tables_fails_naming_the_pair(self):
        record = ps.load(self.pins["es-en"])
        del record["studied"]
        ps.save(self.pins["es-en"], record)
        with self.assertRaisesRegex(ps.PinError, r"es-en reads es/, and its pin.json records nothing"):
            self.check("es-en")
        record["studied"] = {"reference": "es-xx", "tables": {}}
        ps.save(self.pins["es-en"], record)
        with self.assertRaisesRegex(ps.PinError, r"records 'es-xx' as the pair whose reduction writes es/"):
            self.check("es-en")

    def test_a_studied_table_one_side_lacks_is_named(self):
        (self.es / "lexical.tsv").unlink()
        with self.assertRaisesRegex(ps.PinError, r"es-en: es/lexical\.tsv .*sha256 no file"):
            ps.check_studied_tables(self.pins["es-en"])


class Record(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.pin = self.root / "en-fr" / "pin.json"
        ps.save(self.pin, {"snapshot": "2026.09.26", "sources": {"wordfreq": {"version": "3.1.1"}}})
        # The studied language's folder beside the pair's (split-lingua-pack-tables-by-language).
        self.studied = self.root / "en"
        self.studied.mkdir()
        (self.studied / "tags.tsv").write_text("NOUN\nVERB\n")
        self.pack = self.root / "pack.lingua"
        self.pack.write_bytes(b"LINGUA pack")
        self.reducer = self.root / "reduce-en-fr.py"
        self.reducer.write_text("# rules\n")

    def tearDown(self):
        self._tmp.cleanup()

    def test_a_shared_rule_module_is_part_of_every_pair_s_rules(self):
        shared = self.root / "reduce_common.py"
        shared.write_text("# shared rules\n")
        self.reducer.write_text("import reduce_common  # noqa: F401\n")
        ps.record_build(self.pin, self.pack, self.reducer)
        self.assertEqual(ps.load(self.pin)["reducer"]["files"], ["reduce-en-fr.py", "reduce_common.py"])
        ps.check_reducer(self.pin, self.reducer)
        shared.write_text("# shared rules, edited\n")
        with self.assertRaisesRegex(ps.PinError, "reduce-en-fr.py, reduce_common.py"):
            ps.check_reducer(self.pin, self.reducer)
        ps.record_build(self.pin, self.pack, self.reducer)
        ps.check_reducer(self.pin, self.reducer)

    def test_a_rule_module_the_reducer_does_not_load_is_not_part_of_its_rules(self):
        # generalise-lingua-gloss-reducer D2: the English Wiktionary's rules are no French-native
        # pair's, so tuning them re-pins none.
        (self.root / "reduce_common.py").write_text("# shared rules\n")
        (self.root / "reduce_edition_fr.py").write_text("import reduce_common  # noqa: F401\n")
        (self.root / "reduce_edition_en.py").write_text("# the English Wiktionary's rules\n")
        self.reducer.write_text("import reduce_edition_fr  # noqa: F401\n")
        ps.record_build(self.pin, self.pack, self.reducer)
        self.assertEqual(
            ps.load(self.pin)["reducer"]["files"], ["reduce-en-fr.py", "reduce_common.py", "reduce_edition_fr.py"]
        )
        (self.root / "reduce_edition_en.py").write_text("# the English Wiktionary's rules, tuned\n")
        ps.check_reducer(self.pin, self.reducer)
        (self.root / "reduce_edition_fr.py").write_text("import reduce_common  # noqa: F401\n# tuned\n")
        with self.assertRaisesRegex(ps.PinError, "reduce_edition_fr.py"):
            ps.check_reducer(self.pin, self.reducer)

    def test_a_reducer_loading_a_module_its_record_does_not_name_fails_naming_it(self):
        (self.root / "reduce_common.py").write_text("# shared rules\n")
        self.reducer.write_text("import reduce_common  # noqa: F401\n")
        ps.record_build(self.pin, self.pack, self.reducer)
        (self.root / "reduce_edition_es.py").write_text("# the Spanish Wiktionary's rules\n")
        self.reducer.write_text("import reduce_common  # noqa: F401\nimport reduce_edition_es  # noqa: F401\n")
        with self.assertRaisesRegex(ps.PinError, r"loads reduce_edition_es\.py, which the rules recorded in pin\.json"):
            ps.check_reducer(self.pin, self.reducer)
        # Even with the digest written by hand: the record still names the files it was reduced by.
        record = ps.load(self.pin)
        record["reducer"]["sha256"] = ps.rules_sha256(self.reducer)
        ps.save(self.pin, record)
        with self.assertRaisesRegex(ps.PinError, "reduce_edition_es.py"):
            ps.check_reducer(self.pin, self.reducer)

    def test_the_caller_s_imports_are_no_rules_of_the_reducer(self):
        # rule_files imports the reducer in an interpreter of its own: what this process has loaded
        # (the test runner holds reduce_common from the reducers' tests) does not count.
        (self.root / "reduce_common.py").write_text("# shared rules\n")
        self.assertEqual(ps.rule_files(self.reducer), [self.reducer])

    def test_a_reducer_that_cannot_be_imported_is_refused(self):
        self.reducer.write_text("import reduce_missing  # noqa: F401\n")
        with self.assertRaisesRegex(ps.PinError, "importing reduce-en-fr.py failed: .*reduce_missing"):
            ps.rule_files(self.reducer)

    def test_another_pair_s_reducer_is_not_part_of_this_pair_s_rules(self):
        ps.record_build(self.pin, self.pack, self.reducer)
        (self.root / "reduce-es-fr.py").write_text("# another pair\n")
        ps.check_reducer(self.pin, self.reducer)

    def test_record_build_then_check(self):
        ps.record_build(self.pin, self.pack, self.reducer)
        record = ps.load(self.pin)
        self.assertEqual(record["pack"]["size"], len(b"LINGUA pack"))
        self.assertEqual(list(record), ["snapshot", "pack", "reducer", "sources"])
        ps.check_pack(self.pin, self.pack)
        ps.check_reducer(self.pin, self.reducer)

    def test_a_pack_that_differs_is_refused(self):
        ps.record_build(self.pin, self.pack, self.reducer)
        self.pack.write_bytes(b"LINGUA pack, rebuilt by another zstd")
        with self.assertRaisesRegex(ps.PinError, "pack.sha256"):
            ps.check_pack(self.pin, self.pack)

    def test_edited_rules_ask_for_a_re_reduce(self):
        ps.record_build(self.pin, self.pack, self.reducer)
        self.reducer.write_text("# rules, edited\n")
        with self.assertRaisesRegex(ps.PinError, "--reduce"):
            ps.check_reducer(self.pin, self.reducer)

    def test_a_reference_s_rules_changed_fail_every_pair_of_its_language(self):
        # spec: *A reference pair's rules changed and not applied* — es-fr's and es-en's checks fail,
        # naming es-fr's rule files; en-fr's still pass.
        (self.studied / "studied.json").write_text('{"reference": "en-fr"}\n')
        spanish = self.root / "es"
        spanish.mkdir()
        (spanish / "studied.json").write_text('{"reference": "es-fr"}\n')
        (spanish / "tags.tsv").write_text("NOUN\n")
        pins = {"en-fr": self.pin}
        for pair in ("es-fr", "es-en"):
            pins[pair] = self.root / pair / "pin.json"
            ps.save(pins[pair], {"snapshot": "2026.10.03", "sources": {}})
            (self.root / f"reduce-{pair}.py").write_text(f"# {pair}'s rules\n")
        for pair, pin in pins.items():
            ps.record_build(pin, self.pack, self.root / f"reduce-{pair}.py")
            ps.check_reducer(pin, self.root / f"reduce-{pair}.py")
        (self.root / "reduce-es-fr.py").write_text("# es-fr's rules, edited\n")
        with self.assertRaisesRegex(ps.PinError, r"reduce-es-fr\.py.*--reduce es-fr"):
            ps.check_reducer(pins["es-fr"], self.root / "reduce-es-fr.py")
        with self.assertRaisesRegex(ps.PinError, r"es-en reads es/, which es-fr's reduction writes: .*reduce-es-fr\.py"):
            ps.check_reducer(pins["es-en"], self.root / "reduce-es-en.py")
        ps.check_reducer(pins["en-fr"], self.reducer)
        # Reduced again (recorded), every Spanish pair passes.
        ps.record_build(pins["es-fr"], self.pack, self.root / "reduce-es-fr.py")
        ps.check_reducer(pins["es-en"], self.root / "reduce-es-en.py")

    def test_build_sh_names_the_studied_folder_a_listed_pair_lacks(self):
        # spec: *A listed pair whose studied language has no tables* — before any cargo run.
        scratch = self.root / "data"
        (scratch / "tables" / "de-fr").mkdir(parents=True)
        shutil.copy(HERE / "build.sh", scratch / "build.sh")
        ps.save(scratch / "tables" / "de-fr" / "pin.json", {"pack": {"sha256": "0" * 64}})
        done = subprocess.run(
            ["bash", str(scratch / "build.sh"), "de-fr", str(self.root / "de-fr.lingua")],
            capture_output=True,
            text=True,
            check=False,
        )
        self.assertEqual(done.returncode, 2, done.stderr)
        self.assertIn("de-fr studies de", done.stderr)
        self.assertIn("tables/de/", done.stderr)

    def test_build_sh_names_a_pair_left_behind_by_its_studied_tables(self):
        # spec: *A pair left behind* and *A rule of the reference's own edition* — a change under
        # tables/es/ (a level, or a lemma es-fr comes to gloss) that does not record a second
        # Spanish pair's new pack fails that pair's build, naming it and its pin. The builder is
        # doubled: its pack is the tables it reads (that the real one's bytes move with them is
        # tests/committed_tables.rs's, *A pair left behind*).
        scratch = self.root / "data"
        es, es_en = scratch / "tables" / "es", scratch / "tables" / "es-en"
        es.mkdir(parents=True)
        es_en.mkdir()
        shutil.copy(HERE / "build.sh", scratch / "build.sh")
        (es / "studied.json").write_text('{"reference": "es-fr"}\n')
        (es / "level.tsv").write_text("casa\tA1\n")
        (es / "lexical.tsv").write_text("casa\n")
        (es_en / "gloss.tsv").write_text("casa\tHouse\n")
        bin_ = self.root / "bin"
        bin_.mkdir()
        (bin_ / "cargo").write_text(
            "#!/usr/bin/env bash\n"
            "# lingua-pack-build, doubled: `run … -- --studied S P OUT` packs what it reads.\n"
            'while [ "$1" != --studied ]; do shift; done\n'
            'cat "$2/level.tsv" "$2/lexical.tsv" "$3/gloss.tsv" > "$4"\n'
        )
        (bin_ / "cargo").chmod(0o755)
        packed = b"casa\tA1\ncasa\ncasa\tHouse\n"
        ps.save(es_en / "pin.json", {"pack": {"sha256": hashlib.sha256(packed).hexdigest()}})

        def build():
            return subprocess.run(
                ["bash", str(scratch / "build.sh"), "es-en", str(self.root / "es-en.lingua")],
                capture_output=True,
                text=True,
                check=False,
                env={**os.environ, "PATH": f"{bin_}{os.pathsep}{os.environ['PATH']}"},
            )

        done = build()
        self.assertEqual(done.returncode, 0, done.stderr)
        self.assertEqual((self.root / "es-en.lingua").read_bytes(), packed)
        for table, text in (("level.tsv", "casa\tA2\n"), ("lexical.tsv", "augusto\ncasa\n")):
            before = (es / table).read_text()
            (es / table).write_text(text)
            done = build()
            self.assertEqual(done.returncode, 1, f"{table}: {done.stderr}")
            self.assertIn("error: es-en:", done.stderr)
            self.assertIn("(tables/es-en/pin.json)", done.stderr)
            self.assertIn("tables/es/", done.stderr)
            (es / table).write_text(before)
        self.assertEqual(build().returncode, 0, "recorded against the committed tables again")

    def test_get_reads_a_dotted_key(self):
        self.assertEqual(ps.get(ps.load(self.pin), "sources.wordfreq.version"), "3.1.1")
        with self.assertRaises(ps.PinError):
            ps.get(ps.load(self.pin), "pack.sha256")

    def test_build_sh_reads_the_pack_hash_without_python(self):
        # build.sh's Build mode reads pack.sha256 with sed, from the file save() writes.
        ps.record_build(self.pin, self.pack, self.reducer)
        script = (HERE / "build.sh").read_text()
        body = re.search(r"recorded_pack_sha256\(\) \{\n(.*?)\n\}", script, re.S).group(1)
        got = subprocess.run(
            ["bash", "-c", f"recorded_pack_sha256() {{\n{body}\n}}\nrecorded_pack_sha256 {self.pin}"],
            check=True,
            capture_output=True,
            text=True,
        ).stdout.strip()
        self.assertEqual(got, ps.sha256(self.pack))

    def test_a_build_is_recorded_only_beside_the_pinned_tag_pool(self):
        # add-lingua-pack-lexical-layer D4: tags.tsv is an input no reducer writes; tables reduced
        # again without it would build a pack whose readings another pair stores differently.
        (self.studied / "tags.tsv").unlink()
        with self.assertRaisesRegex(ps.PinError, "tags.tsv"):
            ps.record_build(self.pin, self.pack, self.reducer)
        self.assertNotIn("pack", ps.load(self.pin))

    def test_keep_copies_a_studied_folder_into_a_dry_run_s_root(self):
        (self.studied / "studied.json").write_text('{"reference": "en-fr"}\n')
        (self.studied / "forms.tsv").write_text("ran\trun\n")
        scratch = self.root / "dry" / "en"
        self.assertEqual(ps.keep(self.studied, scratch), ["forms.tsv", "studied.json", "tags.tsv"])
        self.assertEqual((scratch / "tags.tsv").read_text(), "NOUN\nVERB\n")
        # Into the folder itself, nothing is copied; a language with no folder yet keeps nothing.
        self.assertEqual(ps.keep(self.studied, self.studied), [])
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["keep", "--from", str(self.root / "fr"), "--to", str(self.root / "other")]), 0)
        self.assertEqual(out.getvalue(), "")

    def test_build_sh_files_a_reduction_by_side_and_never_writes_a_kept_input(self):
        # A reducer writes both sides; split files them, and touches no input a person writes.
        self.assertFalse(set(ps.STUDIED_TABLES) & set(ps.PAIR_TABLES))
        self.assertFalse((set(ps.STUDIED_TABLES) | set(ps.PAIR_TABLES) | {ps.LEXICAL}) & set(ps.KEPT_INPUTS))
        script = (HERE / "build.sh").read_text()
        self.assertIn('pack_sources.py" split --work', script)
        self.assertIn('pack_sources.py" keep --from', script)
        self.assertNotIn("TABLE_FILES", script)

    def test_every_committed_pair_reads_a_studied_folder_that_pins_its_tag_pool(self):
        for pin in sorted((HERE / "tables").glob("*/pin.json")):
            studied = ps.studied_dir(pin)
            self.assertTrue((studied / ps.PINNED_POOL).is_file(), studied.name)
            self.assertIsNotNone(ps.reference_of(studied), studied.name)
            for name in ps.STUDIED_TABLES + (ps.PINNED_POOL, ps.LEXICAL):
                self.assertFalse((pin.parent / name).exists(), f"{pin.parent.name}/{name}")

    def test_every_folder_of_the_tables_is_a_pair_or_a_studied_language(self):
        for folder in ps.folders(HERE / "tables"):
            if ps.is_pair(folder):
                self.assertTrue((folder / "pin.json").is_file(), folder.name)
            else:
                self.assertTrue((folder / ps.STUDIED_RECORD).is_file(), folder.name)
        # The reduce job's order: each reference before the other pairs of its language — es-en,
        # which reads tables/es, after es-fr, which writes it (add-lingua-pack-es-en, *The reduce job*).
        self.assertEqual(ps.pairs(HERE / "tables"), ["en-fr", "es-fr", "es-en"])
        self.assertEqual(ps.pairs(HERE / "tables", after="es-fr"), ["es-fr", "es-en"])
        self.assertEqual(ps.pairs(HERE / "tables", after="es-en"), ["es-en"])

    def test_the_committed_dictionary_words_are_the_reference_s_glossed_lemmas(self):
        for language in ("en", "es"):
            studied = HERE / "tables" / language
            reference = ps.reference_of(studied)
            words = (studied / ps.LEXICAL).read_text(encoding="utf-8")
            self.assertEqual(
                words, "".join(f"{w}\n" for w in ps.glossed_lemmas(HERE / "tables" / reference / "gloss.tsv")), language
            )

    def test_the_pairs_glossed_in_french_read_the_french_wiktionary_s_rules_alone(self):
        for pair in ("en-fr", "es-fr"):
            reducer = HERE / f"reduce-{pair}.py"
            self.assertEqual(
                [p.name for p in ps.rule_files(reducer)],
                [reducer.name, "reduce_common.py", "reduce_edition_fr.py"],
                f"{pair} loads the shared rules and the French Wiktionary's, not another edition's",
            )

    def test_no_rule_module_a_reducer_imports_escapes_its_rules(self):
        # rule_files reads sys.modules after importing the reducer; an import made later — inside a
        # function — would escape it. Every `reduce_*` import anywhere in a reducer or in a module it
        # loads must be one of its rule files.
        for pin in sorted((HERE / "tables").glob("*/pin.json")):
            reducer = HERE / f"reduce-{pin.parent.name}.py"
            files = ps.rule_files(reducer)
            names = {p.stem for p in files}
            for path in files:
                for node in ast.walk(ast.parse(path.read_text(encoding="utf-8"))):
                    imported = [a.name for a in node.names] if isinstance(node, ast.Import) else []
                    if isinstance(node, ast.ImportFrom) and node.module:
                        imported = [node.module]
                    for module in imported:
                        if module.startswith("reduce_"):
                            self.assertIn(module, names, f"{path.name} imports {module}, outside {reducer.name}'s rules")

    def test_a_reducer_and_its_rules_load_code_by_import_statements_alone(self):
        # The test above reads import statements: a module loaded by name or by path at run time
        # (`importlib`, `__import__`, `exec`) would escape it, and `sys.modules` too when it runs
        # inside a function. No reducer or rule module loads code that way, so none may.
        modules = sorted(HERE.glob("reduce[-_]*.py"))
        self.assertGreaterEqual(len(modules), 5)
        for path in modules:
            self.assertEqual(_dynamic_loads(path.read_text(encoding="utf-8")), [], path.name)

    def test_a_dynamic_load_is_seen_wherever_it_sits(self):
        # Made up: a rule module loading another edition inside a function, four ways.
        for body in (
            "import importlib\n    return importlib.import_module('reduce_edition_en')",
            "from importlib import import_module\n    return import_module('reduce_edition_en')",
            "return __import__('reduce_edition_en')",
            "exec(open('reduce_edition_en.py').read())",
        ):
            self.assertNotEqual(_dynamic_loads(f"import re\n\ndef gloss():\n    {body}\n"), [], body)
        self.assertEqual(_dynamic_loads("import re\nimport reduce_common as common\nre.compile('x')\n"), [])

    def test_the_committed_record_is_complete(self):
        for pin in sorted((HERE / "tables").glob("*/pin.json")):
            pair = pin.parent.name
            record = ps.load(pin)
            self.assertRegex(record["snapshot"], r"^\d{4}\.\d{2}\.\d{2}$", pair)
            self.assertRegex(record["pack"]["sha256"], r"^[0-9a-f]{64}$", pair)
            reducer = HERE / f"reduce-{pair}.py"
            self.assertEqual(record["reducer"]["sha256"], ps.rules_sha256(reducer), f"{pair}: tables reduced by these rules")
            self.assertEqual(record["reducer"]["files"], [p.name for p in ps.rule_files(reducer)], pair)
            ps.check_reducer(pin, reducer)
            manifest = json.loads((pin.parent / "manifest.json").read_text())
            # The pack says which dictionary it is: the snapshot for tables an update reduced, the
            # snapshot and the rules for tables a re-reduction made from the same sources.
            self.assertIn(
                manifest["meta"]["pack_version"],
                (record["snapshot"], f"{record['snapshot']}+{record['reducer']['sha256'][:7]}"),
                f"{pair}: the pack says which dictionary it is",
            )


class Split(unittest.TestCase):
    """A reduction filed by side (split-lingua-pack-tables-by-language, M24)."""

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.tables = self.root / "tables"
        self.work = self.root / "work"
        self.work.mkdir()
        for name, text in {
            "forms.tsv": "casas\tcasa\n",
            "freq.tsv": "casa\t1\ndios\t2\n",
            "grammar.tsv": "casas\tcasa\tNOUN\t-\n",
            "gloss.tsv": "dios\tDieu\ncasa\tMaison\n \tnothing\nno tab\n",
            "senses.tsv": "casa\tNOUN:1\n",
            "NOTICE": "notice",
            "manifest.json": "{}",
        }.items():
            (self.work / name).write_text(text, encoding="utf-8")

    def tearDown(self):
        self._tmp.cleanup()

    def test_the_first_pair_of_a_language_is_its_reference_and_writes_both_sides(self):
        written = ps.split(self.work, self.tables, "es-fr")
        self.assertEqual(ps.reference_of(self.tables / "es"), "es-fr")
        self.assertIn("es/forms.tsv", written)
        self.assertIn("es-fr/gloss.tsv", written)
        self.assertEqual(sorted(p.name for p in (self.tables / "es-fr").iterdir()),
                         ["NOTICE", "gloss.tsv", "manifest.json", "senses.tsv"])
        self.assertEqual(sorted(p.name for p in (self.tables / "es").iterdir()),
                         ["forms.tsv", "freq.tsv", "grammar.tsv", "lexical.tsv", "studied.json"])
        # The dictionary words: the glossed lemmas, byte-sorted, as the builder reads them.
        self.assertEqual((self.tables / "es" / "lexical.tsv").read_text(), "casa\ndios\n")

    def test_a_reference_reduced_again_writes_both_folders_and_keeps_the_pinned_pool(self):
        # spec: *A reference pair reduced again* — both folders as before, tags.tsv kept.
        ps.split(self.work, self.tables, "es-fr")
        (self.tables / "es" / "tags.tsv").write_text("NOUN\nVERB\n")
        before = {p.relative_to(self.tables): p.read_bytes() for p in self.tables.rglob("*") if p.is_file()}
        written = ps.split(self.work, self.tables, "es-fr")
        self.assertIn("es/lexical.tsv", written)
        self.assertNotIn("es/tags.tsv", written)
        self.assertNotIn("es/studied.json", written, "the record is written once, by the first pair")
        after = {p.relative_to(self.tables): p.read_bytes() for p in self.tables.rglob("*") if p.is_file()}
        self.assertEqual(after, before)
        # A new lemma glossed by the reference reaches the dictionary words.
        (self.work / "gloss.tsv").write_text("dios\tDieu\ncasa\tMaison\nárbol\tArbre\n", encoding="utf-8")
        ps.split(self.work, self.tables, "es-fr")
        self.assertEqual((self.tables / "es" / "lexical.tsv").read_text(encoding="utf-8"), "casa\ndios\nárbol\n")
        self.assertEqual((self.tables / "es" / "tags.tsv").read_text(), "NOUN\nVERB\n")

    def test_the_dictionary_words_are_read_as_the_builder_reads_a_gloss_table(self):
        # lingua_pack::tsv_pairs: lines split on \n alone, a trailing \r dropped, the key before the
        # first tab, trimmed; a line without a tab or with a blank key is no gloss. A lone \r is no
        # line break for the builder, so it is none here either: `ca\rsa` is one key.
        gloss = self.root / "gloss.tsv"
        gloss.write_bytes("b\tB\r\n  a \tA\n\t\n   \tblank\nno tab\nz\tZ\tmore\né\tE\nb\tB again\nca\rsa\tX\n".encode())
        self.assertEqual(ps.glossed_lemmas(gloss), ["a", "b", "ca\rsa", "z", "é"])
        with self.assertRaisesRegex(ps.PinError, "es-en/gloss.tsv is missing"):
            ps.glossed_lemmas(self.root / "es-en" / "gloss.tsv")

    def test_a_record_that_cannot_be_read_fails_before_anything_is_written(self):
        # A malformed studied.json is an error line, and the pair folder is left as it was.
        (self.tables / "es").mkdir(parents=True)
        (self.tables / "es" / "studied.json").write_text("{", encoding="utf-8")
        with self.assertRaisesRegex(ps.PinError, "es/studied.json"):
            ps.split(self.work, self.tables, "es-fr")
        self.assertFalse((self.tables / "es-fr").exists())
        (self.tables / "es" / "studied.json").write_text('{"reference": "es"}\n', encoding="utf-8")
        with self.assertRaisesRegex(ps.PinError, "names 'es', which is no pair"):
            ps.reference_of(self.tables / "es")
        with contextlib.redirect_stderr(io.StringIO()) as err:
            self.assertEqual(ps.main(["split", "--work", str(self.work), "--tables", str(self.tables), "--pair", "es-fr"]), 1)
        self.assertIn("error: es/studied.json", err.getvalue())

    def test_after_names_a_pair(self):
        ps.split(self.work, self.tables, "es-fr")
        with self.assertRaisesRegex(ps.PinError, "es is no pair of"):
            ps.pairs(self.tables, after="es")
        with self.assertRaisesRegex(ps.PinError, "en-fr is no pair of"):
            ps.pairs(self.tables, after="en-fr")

    def test_another_pair_reads_the_studied_folder_and_never_writes_it(self):
        ps.split(self.work, self.tables, "es-fr")
        (self.tables / "es" / "tags.tsv").write_text("NOUN\n")
        before = {p.name: p.read_bytes() for p in (self.tables / "es").iterdir()}
        (self.work / "forms.tsv").write_text("casas\tcasar\n")
        (self.work / "gloss.tsv").write_text("casa\tHouse\naugusto\tAugust\n")
        written = ps.split(self.work, self.tables, "es-en")
        self.assertTrue(all(w.startswith("es-en/") for w in written), written)
        self.assertEqual({p.name: p.read_bytes() for p in (self.tables / "es").iterdir()}, before)
        self.assertFalse((self.tables / "es-en" / "forms.tsv").exists())

    def test_a_table_the_reduction_no_longer_writes_is_removed(self):
        ps.split(self.work, self.tables, "es-fr")
        (self.work / "senses.tsv").unlink()
        (self.work / "grammar.tsv").unlink()
        ps.split(self.work, self.tables, "es-fr")
        self.assertFalse((self.tables / "es-fr" / "senses.tsv").exists())
        self.assertFalse((self.tables / "es" / "grammar.tsv").exists())

    def test_pairs_come_references_first_and_a_reference_brings_its_readers(self):
        for pair in ("en-fr", "es-en", "es-fr", "en-es"):
            ps.split(self.work, self.tables, pair)
        (self.tables / "es" / "studied.json").write_text('{"reference": "es-fr"}\n')
        self.assertEqual(ps.reference_of(self.tables / "en"), "en-fr")
        self.assertEqual(ps.pairs(self.tables), ["en-fr", "es-fr", "en-es", "es-en"])
        self.assertEqual(ps.pairs(self.tables, after="es-fr"), ["es-fr", "es-en"])
        self.assertEqual(ps.pairs(self.tables, after="es-en"), ["es-en"])
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["pairs", "--tables", str(self.tables), "--after", "en-fr"]), 0)
        self.assertEqual(out.getvalue(), "en-fr\nen-es\n")
        # The studied folders are no pairs; a hidden folder is skipped.
        (self.tables / ".cache").mkdir()
        self.assertNotIn("es", ps.pairs(self.tables))
        self.assertEqual(len(ps.pairs(self.tables)), 4)

    def test_a_folder_that_is_neither_a_pair_nor_a_studied_language_fails(self):
        ps.split(self.work, self.tables, "es-fr")
        (self.tables / "work").mkdir()
        with self.assertRaisesRegex(ps.PinError, r"work is neither a pair .* nor a studied language"):
            ps.pairs(self.tables)
        with contextlib.redirect_stderr(io.StringIO()) as err:
            self.assertEqual(ps.main(["pairs", "--tables", str(self.tables)]), 1)
        self.assertIn("work/studied.json", err.getvalue())

    def test_tables_holding_no_pair_fail(self):
        # A loop over no pair would reduce nothing and pass.
        (self.tables / "es").mkdir(parents=True)
        (self.tables / "es" / "studied.json").write_text('{"reference": "es-fr"}\n')
        with self.assertRaisesRegex(ps.PinError, "holds no pair"):
            ps.pairs(self.tables)
        with contextlib.redirect_stderr(io.StringIO()) as err:
            self.assertEqual(ps.main(["pairs", "--tables", str(self.tables)]), 1)
        self.assertIn("holds no pair", err.getvalue())

    def test_a_moved_file_names_the_pair_whose_reduction_writes_it(self):
        # spec: *A studied table edited by hand* — the reduce job names en-fr and en/forms.tsv.
        for pair in ("en-fr", "es-fr"):
            ps.split(self.work, self.tables, pair)
        status = [
            " M scripts/lingua-data/tables/en/forms.tsv",
            " M scripts/lingua-data/tables/es-fr/gloss.tsv",
            "?? scripts/lingua-data/tables/es/stray.tsv",
            "?? scripts/lingua-data/tables/es-en/",
            "",
        ]
        self.assertEqual(
            ps.moved(self.tables, status),
            ["en-fr: en/forms.tsv", "es-fr: es-fr/gloss.tsv", "es-fr: es/stray.tsv", "es-en: es-en/"],
        )
        with contextlib.redirect_stdout(io.StringIO()) as out:
            with unittest.mock.patch("sys.stdin", io.StringIO(" M scripts/lingua-data/tables/es/level.tsv\n")):
                self.assertEqual(ps.main(["moved", "--tables", str(self.tables)]), 0)
        self.assertEqual(out.getvalue(), "es-fr: es/level.tsv\n")


if __name__ == "__main__":
    unittest.main()
