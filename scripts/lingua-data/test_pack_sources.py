# Copyright 2026 NEETROF
#
# Licensed under the Apache License, Version 2.0 (the "License"); you may not use
# this file except in compliance with the License. You may obtain a copy of the
# License at http://www.apache.org/licenses/LICENSE-2.0

"""Tests for the pinned sources and the record (pin-lingua-pack-sources), with no download.

Run: python3 -m unittest discover -s scripts/lingua-data -p "test_*.py"
"""

import datetime
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

    def tearDown(self):
        self._tmp.cleanup()

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
            return ps.fetch_live(self.pin, self.work, snapshot, fetch=self.fetch, today=datetime.date(2026, 9, 26))

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_update_records_every_source_and_keeps_kaikki(self):
        record = self.update()
        self.assertEqual(record["snapshot"], "2026.09.26")
        sources = record["sources"]
        self.assertEqual(set(sources), {"agid", "cefrj", "octanove", "kaikki", "wordfreq"})
        for name in ("agid", "cefrj", "octanove"):
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
            ps.fetch_pinned(self.pin, again, fetch=self.fetch)
            self.assertEqual((again / "kaikki-Anglais.jsonl").read_bytes(), b'{"word": "harbour"}\n')
            # Upstream changed the bytes behind a pinned address.
            self.served[ps.PINNED["en-fr"]["agid"]["url"]] = b"something else\n"
            with self.assertRaisesRegex(ps.PinError, "agid"):
                ps.fetch_pinned(self.pin, again, fetch=self.fetch)

    @unittest.skipUnless(HAS_ZSTD, "zstd not installed")
    def test_reduce_refuses_another_wordfreq(self):
        import unittest.mock as mock

        self.update()
        released = self.root / "released"
        released.mkdir()
        shutil.copy(self.work / "kaikki-Anglais.jsonl.zst", released / "kaikki-Anglais.jsonl.zst")
        with mock.patch.object(ps, "wordfreq_version", return_value="3.2.0"):
            with self.assertRaisesRegex(ps.PinError, "wordfreq"):
                ps.fetch_pinned(self.pin, self.root / "again", fetch=self.fetch)


class Record(unittest.TestCase):
    def setUp(self):
        self._tmp = tempfile.TemporaryDirectory()
        self.root = Path(self._tmp.name)
        self.pin = self.root / "en-fr" / "pin.json"
        ps.save(self.pin, {"snapshot": "2026.09.26", "sources": {"wordfreq": {"version": "3.1.1"}}})
        self.pack = self.root / "pack.lingua"
        self.pack.write_bytes(b"LINGUA pack")
        self.reducer = self.root / "reduce-en-fr.py"
        self.reducer.write_text("# rules\n")

    def tearDown(self):
        self._tmp.cleanup()

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

    def test_the_committed_record_is_complete(self):
        pin = HERE / "tables" / "en-fr" / "pin.json"
        record = ps.load(pin)
        self.assertRegex(record["snapshot"], r"^\d{4}\.\d{2}\.\d{2}$")
        self.assertRegex(record["pack"]["sha256"], r"^[0-9a-f]{64}$")
        self.assertEqual(record["reducer"]["sha256"], ps.sha256(HERE / "reduce-en-fr.py"), "tables reduced by these rules")
        manifest = json.loads((pin.parent / "manifest.json").read_text())
        self.assertEqual(manifest["meta"]["pack_version"], record["snapshot"], "the pack says which dictionary it is")


if __name__ == "__main__":
    unittest.main()
