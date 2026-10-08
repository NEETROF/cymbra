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


class OwnExtracts(unittest.TestCase):
    """Each pair pins its own extract (add-lingua-pack-es-en D2): es-en reads the address es-fr
    reads, fetched live when es-en is updated and published under es-en's own release; a pair an
    update brings along is reduced from its own pin; the asset cache fetches an asset once when two
    pins name it, and keeps an entry whole or not at all."""

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
        self.serve_extract("house")
        self.served[ps.DUMPS["es-fr"]["kaikki-fr"]["url"]] = Dumps.dump(self.FR)
        self.served[ps.DUMPS["es-fr"]["kaikki-es"]["url"]] = Dumps.dump(self.ES)
        self.released = self.root / "released"
        self.released.mkdir()
        self.fetched = []

    def tearDown(self):
        self._tmp.cleanup()

    def serve_extract(self, gloss):
        # kaikki regenerates its extract every day: what the address serves today.
        line = {"word": "casa", "senses": [{"glosses": [gloss]}]}
        self.served[ps.KAIKKI["es-fr"]["url"]] = (json.dumps(line) + "\n").encode()

    def fetch(self, url, dest, compressed=False):
        self.fetched.append(url)
        dest.parent.mkdir(parents=True, exist_ok=True)
        if url in self.served:
            dest.write_bytes(self.served[url])
            return {"last-modified": "Sat, 03 Oct 2026 00:00:00 GMT"}
        tag, asset = url.rsplit("/", 2)[-2:]
        shutil.copy(self.released / tag / asset, dest)
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

    def publish(self, pair):
        """What the update's release step publishes: the pair's own assets, under its own tag."""
        record = ps.load(self.pin(pair))
        tag = ps.release_tag(pair, record["snapshot"])
        (self.released / tag).mkdir()
        for asset in ps.assets(record, tag):
            shutil.copy(self.work(pair) / asset, self.released / tag / asset)
        return sorted(p.name for p in (self.released / tag).iterdir())

    def fetch_pinned(self, pair):
        import unittest.mock as mock

        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            ps.fetch_pinned(self.pin(pair), self.work(pair), fetch=self.fetch, cache=self.cache)

    def test_the_two_pairs_name_one_address_and_their_own_derived_files(self):
        self.assertEqual(ps.KAIKKI["es-en"], ps.KAIKKI["es-fr"], "one address, one file name")
        self.assertEqual(ps.DUMPS["es-en"]["kaikki-es"]["url"], ps.DUMPS["es-fr"]["kaikki-es"]["url"])
        self.assertEqual(
            ps.DUMPS["es-en"]["kaikki-es"]["files"], {"kaikki-es-traductions-en.jsonl": ("translations", "es", "en")}
        )
        self.assertEqual(ps.release_tag("es-en", "2026.10.08"), "lingua-pack-sources-es-en-2026.10.08")
        self.assertFalse(hasattr(ps, "shared_extract"), "no pair records another pair's fetch")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_updated_a_reader_pair_fetches_and_publishes_its_own_extract(self):
        # es-fr updated, then es-en the same day: es-en fetches the extract again and publishes it,
        # with its derived file, under its own release — never under es-fr's.
        self.update("es-fr")
        es_fr = self.publish("es-fr")
        reader = self.update("es-en")
        self.assertEqual(self.fetched.count(ps.KAIKKI["es-en"]["url"]), 2, "each pair fetches its own")
        own = ps.release_tag("es-en", "2026.10.08")
        self.assertEqual(reader["sources"]["kaikki"]["release"], own)
        self.assertEqual(reader["sources"]["kaikki-es"]["release"], own)
        self.assertEqual(set(reader["sources"]), {"kaikki", "kaikki-es", "wordfreq"})
        line = json.loads((self.work("es-en") / "kaikki-es-traductions-en.jsonl").read_text(encoding="utf-8"))
        self.assertEqual(line, {"pos": "noun", "translations": [{"word": "sector"}], "word": "sector"}, "English words alone")
        self.assertEqual(self.publish("es-en"), ["kaikki-Spanish.jsonl.zst", "kaikki-es-traductions-en.jsonl.zst"])
        self.assertNotIn("kaikki-es-traductions-en.jsonl.zst", es_fr, "nothing of es-en under es-fr's release")
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["assets", "--pin", str(self.pin("es-en")), "--release", own]), 0)
            self.assertEqual(ps.main(["release-tag", "--pin", str(self.pin("es-en"))]), 0)
        self.assertEqual(out.getvalue(), f"kaikki-Spanish.jsonl.zst\nkaikki-es-traductions-en.jsonl.zst\n{own}\n")
        # Without a cache too: the fetch does not depend on it.
        self.fetched.clear()
        self.update("es-en", snapshot="2026.10.09", cache=False)
        self.assertEqual(self.fetched.count(ps.KAIKKI["es-en"]["url"]), 1)

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_an_update_carries_a_reader_pair_s_studied_record_over(self):
        # fetch-live rewrites the pin's sources; the studied tables the pair was built on stay
        # recorded until record-build writes them anew.
        studied = {"reference": "es-fr", "tables": {"forms.tsv": "0" * 64}}
        ps.save(self.pin("es-en"), {"snapshot": "2026.10.07", "pack": {}, "reducer": {}, "studied": studied, "sources": {}})
        record = self.update("es-en")
        self.assertEqual(record["studied"], studied)
        self.assertEqual(list(ps.load(self.pin("es-en"))), ["snapshot", "pack", "reducer", "studied", "sources"])
        self.assertNotIn("studied", self.update("es-fr"), "the reference records none")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_spec_scenario_each_pair_s_own_extract(self):
        # es-en's pin names its own release (its first update, dispatched alone). es-fr's update
        # brings es-en along: es-en is reduced from its own pin — its own extract, though kaikki
        # serves another one today — nothing of es-en is published, and its pin still names its
        # own release.
        self.update("es-en", snapshot="2026.10.07")
        self.publish("es-en")
        pinned = self.pin("es-en").read_bytes()
        shutil.rmtree(self.root / "work")
        self.fetched.clear()
        self.serve_extract("home")
        self.update("es-fr")
        es_fr = self.publish("es-fr")
        self.fetch_pinned("es-en")
        self.assertEqual(self.pin("es-en").read_bytes(), pinned, "es-en's pin names its own release still")
        own = "lingua-pack-sources-es-en-2026.10.07"
        self.assertEqual(
            [url for url in self.fetched if "releases/download" in url],
            [ps.release_url(own, "kaikki-Spanish.jsonl.zst"), ps.release_url(own, "kaikki-es-traductions-en.jsonl.zst")],
        )
        self.assertIn(b'"house"', (self.work("es-en") / "kaikki-Spanish.jsonl").read_bytes(), "its own pinned bytes")
        self.assertIn(b'"home"', (self.work("es-fr") / "kaikki-Spanish.jsonl").read_bytes(), "es-fr's are today's")
        self.assertEqual(
            es_fr,
            [
                "kaikki-Spanish.jsonl.zst",
                "kaikki-es-traductions.jsonl.zst",
                "kaikki-fr-Espagnol.jsonl.zst",
                "kaikki-fr-traductions.jsonl.zst",
            ],
            "es-fr's release holds es-fr's assets alone",
        )
        self.assertEqual(sorted(p.name for p in self.released.iterdir()), [own, ps.release_tag("es-fr", "2026.10.08")])

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_the_cache_fetches_an_asset_once_and_a_reduction_again_nothing(self):
        # Two pins naming the same asset — es-en's extract record copied from es-fr's (made up):
        # the reduce job fetches it once. Reduced again on the same machine, nothing is fetched.
        self.update("es-fr")
        self.publish("es-fr")
        self.update("es-en")
        self.publish("es-en")
        record = ps.load(self.pin("es-en"))
        record["sources"]["kaikki"] = ps.load(self.pin("es-fr"))["sources"]["kaikki"]
        ps.save(self.pin("es-en"), record)
        shutil.rmtree(self.root / "work")
        self.fetched.clear()
        self.fetch_pinned("es-fr")
        self.fetch_pinned("es-en")
        extract = ps.release_url(ps.release_tag("es-fr", "2026.10.08"), "kaikki-Spanish.jsonl.zst")
        self.assertEqual(self.fetched.count(extract), 1, "fetched once, for both pins")
        self.assertEqual(
            (self.work("es-en") / "kaikki-Spanish.jsonl").read_bytes(), (self.work("es-fr") / "kaikki-Spanish.jsonl").read_bytes()
        )
        shutil.rmtree(self.work("es-en"))
        self.fetched.clear()
        self.fetch_pinned("es-en")
        self.assertEqual(self.fetched, [], "every asset read from the cache")
        self.assertTrue((self.work("es-en") / "kaikki-es-traductions-en.jsonl").is_file())
        self.assertEqual([p.name for p in self.cache.iterdir() if p.name.endswith(".part")], [])

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_a_cache_entry_that_is_not_its_bytes_is_deleted_and_named(self):
        self.update("es-en")
        self.publish("es-en")
        sha = ps.load(self.pin("es-en"))["sources"]["kaikki"]["sha256"]
        entry = self.cache / sha
        # Other bytes under its name: deleted, and the error names the cache path.
        subprocess.run(["zstd", "-q", "-f", "-o", str(entry), "-"], input=b"{}\n", check=True)
        with self.assertRaisesRegex(ps.PinError, rf"kaikki: the snapshot decompresses .*the cached {re.escape(str(entry))} is deleted"):
            self.fetch_pinned("es-en")
        self.assertFalse(entry.exists(), "deleted")
        # Bytes that do not decompress at all (a truncated copy): deleted, and named.
        entry.write_bytes(b"not zstd")
        with self.assertRaisesRegex(ps.PinError, rf"the cached {re.escape(str(entry))} does not decompress .*deleted"):
            self.fetch_pinned("es-en")
        self.assertFalse(entry.exists(), "deleted")
        # The next fetch reads the release again.
        self.fetched.clear()
        self.fetch_pinned("es-en")
        self.assertIn(ps.release_url(ps.release_tag("es-en", "2026.10.08"), "kaikki-Spanish.jsonl.zst"), self.fetched)
        self.assertTrue(entry.is_file(), "fetched again")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_a_cache_entry_is_written_whole_or_not_at_all(self):
        import unittest.mock as mock

        # A copy into the cache that stops half way (a full disk, a killed job) leaves no entry: a
        # later fetch reads the release, not half an asset.
        def interrupted(src, dst):
            Path(dst).write_bytes(Path(src).read_bytes()[:10])
            raise OSError("No space left on device")

        with mock.patch.object(ps.shutil, "copyfile", interrupted), self.assertRaises(OSError):
            self.update("es-en")
        self.assertEqual([p.name for p in self.cache.iterdir() if not p.name.endswith(".part")], [])
        self.update("es-en", snapshot="2026.10.09")
        self.publish("es-en")
        shutil.rmtree(self.work("es-en"))
        self.fetched.clear()
        self.fetch_pinned("es-en")
        self.assertEqual(self.fetched, [], "the whole entries a complete update kept")
        # A fetch into the cache goes through `<sha256>.part` too: a fetch that fails leaves none.
        sha = ps.load(self.pin("es-en"))["sources"]["kaikki"]["sha256"]
        (self.cache / sha).unlink()

        def failing(url, dest, compressed=False):
            Path(dest).write_bytes(b"half")
            raise subprocess.CalledProcessError(56, "curl")

        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ), self.assertRaises(
            subprocess.CalledProcessError
        ):
            ps.fetch_pinned(self.pin("es-en"), self.work("es-en"), fetch=failing, cache=self.cache)
        self.assertFalse((self.cache / sha).exists())

    def test_build_sh_keeps_the_cache_outside_a_pair_s_work_folder(self):
        script = (HERE / "build.sh").read_text()
        self.assertIn('cache="${LINGUA_CACHE:-$here/work/cache}"', script)
        self.assertIn('fetch-pinned --pin "$pin" --work "$work" --cache "$cache"', script)
        self.assertIn('fetch-live --pin "$pin" --work "$work" --snapshot "$snapshot" --cache "$cache"', script)
        self.assertIn('LINGUA_STUDIED="$studied"', script, "a reader pair reads this run's studied folder")
        self.assertIn("es-*) echo 60000", script, "every Spanish pair keeps 60,000 lemmas")
        # The version, in both modes, from `pack_sources.py version` (add-lingua-pack-es-en D3).
        self.assertIn('version --pin "$pin" --reducer "$here/reduce-$pair.py")"', script)
        self.assertIn('version --pin "$pin" --reducer "$here/reduce-$pair.py" --live)"', script)
        self.assertEqual(script.count('reduce "$pair" "$work" "$snapshot" "$version"'), 2)


class DumpsOnly(unittest.TestCase):
    """A pair whose sources are dumps alone (add-lingua-pack-en-es D2): en-es has no extract of its
    own — no `KAIKKI` entry, no `sources.kaikki` — and reads the Spanish Wiktionary's dump and the
    English Wiktionary's English extract, served plain, for the files it derives from them."""

    # The Spanish Wiktionary's dump: an English entry (en-es's entries), a Spanish entry listing an
    # English translation (es-en's and en-es's inverted table) and a French one (es-fr's).
    ES = [
        {"word": "house", "lang_code": "en", "pos": "noun", "senses": [{"glosses": ["Casa."]}]},
        {
            "word": "sector",
            "lang_code": "es",
            "pos": "noun",
            "translations": [{"lang_code": "fr", "word": "secteur"}, {"lang_code": "en", "word": "sector"}],
        },
    ]
    # The English Wiktionary's English extract: its tables under its senses.
    EN = [
        {
            "word": "house",
            "lang_code": "en",
            "pos": "noun",
            "senses": [
                {
                    "glosses": ["A structure serving as an abode of human beings."],
                    "translations": [
                        {"lang_code": "fr", "word": "maison", "sense": "abode"},
                        {"lang_code": "es", "word": "casa", "sense": "abode"},
                    ],
                }
            ],
        },
        {"word": "home", "lang_code": "en", "pos": "noun", "senses": [{"glosses": ["A dwelling."]}]},
    ]

    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.tables = self.root / "tables"
        (self.tables / "en").mkdir(parents=True)
        (self.tables / "en" / "studied.json").write_text('{"reference": "en-fr"}\n')
        (self.tables / "en" / "tags.tsv").write_text("NOUN\n")
        self.pin = self.tables / "en-es" / "pin.json"
        self.work = self.root / "work" / "en-es"
        self.cache = self.root / "work" / "cache"
        self.released = self.root / "released"
        self.released.mkdir()
        dumps = ps.DUMPS["en-es"]
        self.served = {
            dumps["kaikki-es"]["url"]: Dumps.dump(self.ES),
            # Served uncompressed, as kaikki serves a language's extract.
            dumps["kaikki-en"]["url"]: "".join(json.dumps(e, ensure_ascii=False) + "\n" for e in self.EN).encode(),
        }
        self.fetched = []

    def tearDown(self):
        self._tmp.cleanup()

    def fetch(self, url, dest, compressed=False):
        self.fetched.append(url)
        dest.parent.mkdir(parents=True, exist_ok=True)
        if url in self.served:
            dest.write_bytes(self.served[url])
            return {"last-modified": "Sat, 03 Oct 2026 11:09:08 GMT"}
        tag, asset = url.rsplit("/", 2)[-2:]
        shutil.copy(self.released / tag / asset, dest)
        return {}

    def update(self, snapshot="2026.10.09"):
        import unittest.mock as mock

        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            with contextlib.redirect_stderr(io.StringIO()) as err:
                record = ps.fetch_live(
                    self.pin, self.work, snapshot, fetch=self.fetch, today=datetime.date(2026, 10, 9), cache=self.cache
                )
        self.notes = err.getvalue()
        return record

    def publish(self):
        record = ps.load(self.pin)
        tag = ps.release_tag("en-es", record["snapshot"])
        (self.released / tag).mkdir()
        for asset in ps.assets(record, tag):
            shutil.copy(self.work / asset, self.released / tag / asset)
        return sorted(p.name for p in (self.released / tag).iterdir())

    def test_a_pair_whose_sources_are_dumps_alone_is_registered_by_them(self):
        self.assertNotIn("en-es", ps.KAIKKI, "no extract of its own")
        self.assertEqual(
            ps.DUMPS["en-es"]["kaikki-es"]["files"],
            {"kaikki-es-English.jsonl": ("entries", "en"), "kaikki-es-traductions-en.jsonl": ("translations", "es", "en")},
            "the entries and the inverted table, in one pass over en-es's own snapshot of the dump",
        )
        self.assertEqual(ps.DUMPS["en-es"]["kaikki-es"]["url"], ps.DUMPS["es-en"]["kaikki-es"]["url"])
        self.assertEqual(ps.DUMPS["en-es"]["kaikki-en"]["files"], {"kaikki-en-traductions-es.jsonl": ("translations", "en", "es")})
        self.assertEqual(
            ps.DUMPS["en-es"]["kaikki-en"]["url"], "https://kaikki.org/dictionary/English/kaikki.org-dictionary-English.jsonl"
        )
        ps.check_registered("en-es")
        ps.check_registered("es-en")
        with self.assertRaisesRegex(ps.PinError, r"no source registry for de-en: add it to PINNED / ESDB / KAIKKI / DUMPS"):
            ps.check_registered("de-en")

    def test_derive_reads_a_plain_or_a_gzipped_dump_alike(self):
        # Told apart by the gzip magic, not the name: the same lines, whatever the encoding.
        self.work.mkdir(parents=True)
        plain, gzipped = self.work / "plain.dump.jsonl.gz", self.work / "gzipped.jsonl"
        plain.write_bytes(self.served[ps.DUMPS["en-es"]["kaikki-en"]["url"]])
        gzipped.write_bytes(gzip.compress(plain.read_bytes()))
        files = ps.DUMPS["en-es"]["kaikki-en"]["files"]
        out = {}
        for dump in (plain, gzipped):
            ps.derive(dump, files, self.work)
            out[dump.name] = (self.work / "kaikki-en-traductions-es.jsonl").read_text(encoding="utf-8")
        self.assertEqual(out["plain.dump.jsonl.gz"], out["gzipped.jsonl"])
        self.assertEqual(
            json.loads(out["gzipped.jsonl"]),
            {"pos": "noun", "translations": [{"sense": "abode", "word": "casa"}], "word": "house"},
            "the Spanish translation under the sense; no French word, and `home`, which lists none",
        )

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_spec_scenario_an_extract_served_plain(self):
        # lingua-pack-update reads the English Wiktionary's English extract for en-es: the Spanish
        # translation tables are derived from it in one pass, published with the snapshot, and the
        # extract is not kept — nor the Spanish Wiktionary's dump, from which the English entries
        # and the English translations of Spanish entries come in one pass too.
        record = self.update()
        self.assertEqual(set(record["sources"]), {"kaikki-es", "kaikki-en", "wordfreq"}, "no extract of its own")
        self.assertNotIn("kaikki", record["sources"])
        own = ps.release_tag("en-es", "2026.10.09")
        for name in ("kaikki-es", "kaikki-en"):
            self.assertEqual(record["sources"][name]["release"], own)
            self.assertEqual(record["sources"][name]["url"], ps.DUMPS["en-es"][name]["url"])
            self.assertEqual(record["sources"][name]["last_modified"], "Sat, 03 Oct 2026 11:09:08 GMT")
        entries = (self.work / "kaikki-es-English.jsonl").read_text(encoding="utf-8")
        self.assertEqual(entries, json.dumps(self.ES[0], ensure_ascii=False) + "\n", "the English entry as the dump writes it")
        inverted = json.loads((self.work / "kaikki-es-traductions-en.jsonl").read_text(encoding="utf-8"))
        self.assertEqual(inverted, {"pos": "noun", "translations": [{"word": "sector"}], "word": "sector"})
        direct = json.loads((self.work / "kaikki-en-traductions-es.jsonl").read_text(encoding="utf-8"))
        self.assertEqual(direct, {"pos": "noun", "translations": [{"sense": "abode", "word": "casa"}], "word": "house"})
        self.assertFalse(list(self.work.glob("*.dump*")), "neither the dump nor the extract is kept")
        self.assertEqual(
            ps.assets(record),
            ["kaikki-es-English.jsonl.zst", "kaikki-es-traductions-en.jsonl.zst", "kaikki-en-traductions-es.jsonl.zst"],
            "the derived files alone, no extract",
        )
        self.assertEqual(ps.assets(record, own), ps.assets(record))
        self.assertEqual(ps.assets(record, "lingua-pack-sources-es-en-2026.10.09"), [])
        # Its size as served is measured at the update, since the repository keeps none.
        served = len(self.served[ps.DUMPS["en-es"]["kaikki-en"]["url"]])
        self.assertIn(f"note: kaikki-en: {served:,} B as served, derived and not kept", self.notes)
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["assets", "--pin", str(self.pin), "--release", own]), 0)
        self.assertEqual(out.getvalue(), "\n".join(ps.assets(record)) + "\n")

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_a_pinned_reduction_fetches_the_derived_files_and_nothing_larger(self):
        import unittest.mock as mock

        self.update()
        self.assertEqual(self.publish(), sorted(ps.assets(ps.load(self.pin))))
        pinned = self.pin.read_bytes()
        # On another machine: nothing in the cache, the release read.
        shutil.rmtree(self.work)
        shutil.rmtree(self.cache)
        self.fetched.clear()
        with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
            ps.fetch_pinned(self.pin, self.work, fetch=self.fetch, cache=self.cache)
        own = ps.release_tag("en-es", "2026.10.09")
        self.assertEqual(
            self.fetched,
            [ps.release_url(own, asset) for asset in ps.assets(ps.load(self.pin))],
            "the three derived files from en-es's own release; no dump, no extract",
        )
        for name in ("kaikki-es-English.jsonl", "kaikki-es-traductions-en.jsonl", "kaikki-en-traductions-es.jsonl"):
            self.assertTrue((self.work / name).is_file(), name)
        self.assertEqual(self.pin.read_bytes(), pinned, "the record stands")
        # A derived file whose bytes differ from the record is refused, naming the source and the file.
        subprocess.run(
            ["zstd", "-q", "-f", "-o", str(self.released / own / "kaikki-en-traductions-es.jsonl.zst"), "-"],
            input=b"{}\n",
            check=True,
        )
        shutil.rmtree(self.cache)
        with self.assertRaisesRegex(ps.PinError, "kaikki-en: kaikki-en-traductions-es.jsonl"):
            with mock.patch.object(ps, "wordfreq_version", return_value=ps.WORDFREQ):
                ps.fetch_pinned(self.pin, self.work, fetch=self.fetch, cache=self.cache)

    def test_a_record_without_an_extract_lists_its_derived_assets(self):
        record = {
            "sources": {
                "kaikki-es": {"release": "r", "files": {"a.jsonl": {"asset": "a.jsonl.zst"}}},
                "kaikki-en": {"release": "r", "files": {"b.jsonl": {"asset": "b.jsonl.zst"}}},
                "wordfreq": {"version": "3.1.1"},
            }
        }
        self.assertEqual(ps.assets(record), ["a.jsonl.zst", "b.jsonl.zst"])
        self.assertEqual(ps.assets(record, "r"), ["a.jsonl.zst", "b.jsonl.zst"])
        self.assertEqual(ps.assets(record, "other"), [])


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

    def test_an_edited_tag_pool_is_named_without_blaming_a_reduction(self):
        # No reduction writes the pinned tag pool: a person edits it. The pair is named as left
        # behind, and told how to catch up, as for any other table.
        (self.es / "tags.tsv").write_text("NOUN\nVERB\n")
        with self.assertRaises(ps.PinError) as caught:
            ps.check_studied_tables(self.pins["es-en"])
        message = str(caught.exception)
        self.assertRegex(message, r"^es-en: es/tags\.tsv is not what es-en's tables were built on")
        self.assertIn("it was edited, and es-en was left behind: reduce es-en again from its pinned sources", message)
        self.assertNotIn("reduction moved it", message)

    def test_spec_scenario_an_updated_dictionary_for_a_reader_pair(self):
        # A reader pair's pack_version names the studied tables it read, so it moves when one of
        # them does though nothing of the pair's own moved (lingua-data-packs, *An updated
        # dictionary*); in either mode, so a re-reduction of an update gives its version again.
        reducer = self.root / "reduce-es-en.py"
        rules = ps.rules_sha256(reducer)[:7]
        digest = ps.studied_digest(ps.load(self.pins["es-en"])["studied"]["tables"])[:7]
        version = ps.pack_version(self.pins["es-en"], reducer)
        self.assertEqual(version, f"2026.10.08+{rules}.{digest}")
        self.assertEqual(ps.pack_version(self.pins["es-en"], reducer, live=True), version)
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["version", "--pin", str(self.pins["es-en"]), "--reducer", str(reducer)]), 0)
        self.assertEqual(out.getvalue(), f"{version}\n")
        # The reference's is its snapshot for an update, its snapshot and rules after a re-reduction.
        fr = self.root / "reduce-es-fr.py"
        self.assertEqual(ps.pack_version(self.pins["es-fr"], fr, live=True), "2026.10.08")
        self.assertEqual(ps.pack_version(self.pins["es-fr"], fr), f"2026.10.08+{ps.rules_sha256(fr)[:7]}")
        # es-fr reduced again moves Spanish's levels: es-en's version moves with them, its
        # snapshot and rules unchanged; recorded again, the digest is the record's.
        (self.es / "level.tsv").write_text("casa\tA2\n")
        moved = ps.pack_version(self.pins["es-en"], reducer)
        self.assertNotEqual(moved, version)
        self.assertTrue(moved.startswith(f"2026.10.08+{rules}."), moved)
        self.record("es-en")
        self.assertEqual(moved.rsplit(".", 1)[1], ps.studied_digest(ps.load(self.pins["es-en"])["studied"]["tables"])[:7])
        # The digest reads every recorded table by name: a table that appears or goes moves it,
        # and so do two tables trading their bytes.
        self.assertNotEqual(ps.studied_digest({}), ps.studied_digest({"tags.tsv": "a"}))
        self.assertNotEqual(
            ps.studied_digest({"forms.tsv": "a", "freq.tsv": "b"}), ps.studied_digest({"forms.tsv": "b", "freq.tsv": "a"})
        )


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
            # snapshot and the rules for tables a re-reduction made from the same sources — and,
            # for a pair that is not its studied language's reference, the studied tables it read.
            self.assertIn(
                manifest["meta"]["pack_version"],
                (ps.pack_version(pin, reducer, live=True), ps.pack_version(pin, reducer)),
                f"{pair}: the pack says which dictionary it is",
            )
            if "studied" in record:
                digest = ps.studied_digest(record["studied"]["tables"])[:7]
                self.assertTrue(manifest["meta"]["pack_version"].endswith(f".{digest}"), pair)


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
