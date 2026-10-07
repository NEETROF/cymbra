# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the pinned sources and the record (pin-lingua-pack-sources), with no download.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import contextlib
import datetime
import gzip
import io
import json
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


class Record(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.pin = self.root / "en-fr" / "pin.json"
        ps.save(self.pin, {"snapshot": "2026.09.26", "sources": {"wordfreq": {"version": "3.1.1"}}})
        (self.pin.parent / "tags.tsv").write_text("NOUN\nVERB\n")
        self.pack = self.root / "pack.lingua"
        self.pack.write_bytes(b"LINGUA pack")
        self.reducer = self.root / "reduce-en-fr.py"
        self.reducer.write_text("# rules\n")

    def tearDown(self):
        self._tmp.cleanup()

    def test_a_shared_rule_module_is_part_of_every_pair_s_rules(self):
        ps.record_build(self.pin, self.pack, self.reducer)
        shared = self.root / "reduce_common.py"
        shared.write_text("# shared rules\n")
        with self.assertRaises(ps.PinError):
            ps.check_reducer(self.pin, self.reducer)
        ps.record_build(self.pin, self.pack, self.reducer)
        self.assertEqual(ps.load(self.pin)["reducer"]["files"], ["reduce-en-fr.py", "reduce_common.py"])
        ps.check_reducer(self.pin, self.reducer)

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
        (self.pin.parent / "tags.tsv").unlink()
        with self.assertRaisesRegex(ps.PinError, "tags.tsv"):
            ps.record_build(self.pin, self.pack, self.reducer)
        self.assertNotIn("pack", ps.load(self.pin))

    def test_keep_copies_the_inputs_no_reducer_writes(self):
        committed = self.pin.parent
        (committed / "lexical.tsv").write_text("casa\n")
        (committed / "gloss.tsv").write_text("casa\tMaison\n")
        scratch = self.root / "dry" / "en-fr"
        self.assertEqual(ps.keep(committed, scratch), ["tags.tsv", "lexical.tsv"])
        self.assertEqual((scratch / "tags.tsv").read_text(), "NOUN\nVERB\n")
        self.assertEqual((scratch / "lexical.tsv").read_text(), "casa\n")
        self.assertFalse((scratch / "gloss.tsv").exists(), "a reducer's table is not kept")
        # Into the folder itself, nothing is copied.
        self.assertEqual(ps.keep(committed, committed), [])
        # A pair without a lexical table keeps its pool alone.
        (committed / "lexical.tsv").unlink()
        with contextlib.redirect_stdout(io.StringIO()) as out:
            self.assertEqual(ps.main(["keep", "--from", str(committed), "--to", str(self.root / "other")]), 0)
        self.assertEqual(out.getvalue(), "kept tags.tsv\n")
        self.assertEqual(sorted(p.name for p in (self.root / "other").iterdir()), ["tags.tsv"])

    def test_build_sh_never_overwrites_a_kept_input(self):
        # copy_tables touches the tables a reducer writes alone, so reducing a pair again keeps
        # tags.tsv and lexical.tsv; a dry run copies them into its scratch folder.
        script = (HERE / "build.sh").read_text()
        written = re.search(r"^TABLE_FILES=\((.*?)\)$", script, re.M).group(1).split()
        self.assertTrue(written)
        self.assertFalse(set(written) & set(ps.KEPT_INPUTS))
        self.assertIn('pack_sources.py" keep --from', script)

    def test_every_committed_pair_keeps_its_pinned_tag_pool(self):
        for pin in sorted((HERE / "tables").glob("*/pin.json")):
            self.assertTrue((pin.parent / ps.PINNED_POOL).is_file(), pin.parent.name)

    def test_the_committed_record_is_complete(self):
        pin = HERE / "tables" / "en-fr" / "pin.json"
        record = ps.load(pin)
        self.assertRegex(record["snapshot"], r"^\d{4}\.\d{2}\.\d{2}$")
        self.assertRegex(record["pack"]["sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual(record["reducer"]["sha256"], ps.rules_sha256(HERE / "reduce-en-fr.py"), "tables reduced by these rules")
        self.assertEqual(record["reducer"]["files"], [p.name for p in ps.rule_files(HERE / "reduce-en-fr.py")])
        manifest = json.loads((pin.parent / "manifest.json").read_text())
        # The pack says which dictionary it is: the snapshot for tables an update reduced, the
        # snapshot and the rules for tables a re-reduction made from the same sources.
        self.assertIn(
            manifest["meta"]["pack_version"],
            (record["snapshot"], f"{record['snapshot']}+{record['reducer']['sha256'][:7]}"),
            "the pack says which dictionary it is",
        )


if __name__ == "__main__":
    unittest.main()
