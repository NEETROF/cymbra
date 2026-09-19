"""Tests for release_version.py, including the committed version.txt (run: python3 -m unittest)."""

import unittest
from pathlib import Path

import release_version

VERSION_FILE = Path(__file__).parent.parent / "version.txt"


class ResolveTest(unittest.TestCase):
    def test_a_dispatch_takes_the_file_as_it_is(self):
        self.assertEqual(release_version.resolve("1.2.3\n", "branch", "main"), "1.2.3")

    def test_a_matching_tag_passes(self):
        self.assertEqual(release_version.resolve("1.2.3\n", "tag", "lingua-apple-v1.2.3"), "1.2.3")

    def test_a_tag_that_disagrees_stops_the_build(self):
        # A tag pushed by hand: shipping would put a version in App Store Connect that no
        # changelog describes.
        with self.assertRaises(release_version.VersionError) as caught:
            release_version.resolve("1.2.3\n", "tag", "lingua-apple-v1.3.0")
        self.assertIn("1.3.0", str(caught.exception))
        self.assertIn("1.2.3", str(caught.exception))

    def test_a_tag_this_lane_does_not_own(self):
        with self.assertRaises(release_version.VersionError):
            release_version.resolve("1.2.3\n", "tag", "music-v1.2.3")

    def test_a_hand_edited_version_file(self):
        for bad in ("1.2", "1.2.3-rc.1", "v1.2.3", "", "not a version"):
            with self.subTest(bad=bad), self.assertRaises(release_version.VersionError):
                release_version.resolve(bad, "branch", "main")

    def test_the_committed_version_file_is_usable(self):
        # The guard is only worth having if it runs against the real file.
        version = release_version.resolve(VERSION_FILE.read_text(), "branch", "main")
        self.assertRegex(version, r"^\d+\.\d+\.\d+$")


class MainTest(unittest.TestCase):
    def test_prints_the_version_and_succeeds(self):
        code = release_version.main(
            ["--version-file", str(VERSION_FILE), "--ref-type", "branch", "--ref-name", "main"]
        )
        self.assertEqual(code, 0)

    def test_fails_on_a_mismatched_tag(self):
        code = release_version.main(
            [
                "--version-file",
                str(VERSION_FILE),
                "--ref-type",
                "tag",
                "--ref-name",
                "lingua-apple-v99.99.99",
            ]
        )
        self.assertEqual(code, 1)


if __name__ == "__main__":
    unittest.main()
