"""Tests for openspec_archive_order: a delta that waits for another open change is told apart
from one that names a requirement the spec has renamed."""

import sys
import tempfile
import unittest
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent))

from openspec_archive_order import READY, STALE, WAITS, check  # noqa: E402


def write(root: Path, path: str, text: str) -> None:
    target = root / path
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_text(text, encoding="utf-8")


SPEC = """# Spec

## Requirements

### Requirement: Existing rule
The system SHALL do the thing.
"""


class ArchiveOrder(unittest.TestCase):
    def setUp(self) -> None:
        self.tmp = tempfile.TemporaryDirectory()
        self.root = Path(self.tmp.name)
        write(self.root, "openspec/specs/cap/spec.md", SPEC)

    def tearDown(self) -> None:
        self.tmp.cleanup()

    def delta(self, change: str, text: str) -> None:
        write(self.root, f"openspec/changes/{change}/specs/cap/spec.md", text)

    def test_ready_when_every_modified_requirement_is_in_the_spec(self) -> None:
        self.delta("tweak", "## MODIFIED Requirements\n\n### Requirement: Existing rule\nText.\n")
        self.assertEqual(check(self.root, "tweak"), (READY, []))

    def test_waits_for_the_open_change_that_adds_the_requirement(self) -> None:
        self.delta("first", "## ADDED Requirements\n\n### Requirement: New rule\nText.\n")
        self.delta("second", "## MODIFIED Requirements\n\n### Requirement: New rule\nOther text.\n")
        self.assertEqual(check(self.root, "second"), (WAITS, ["first"]))
        self.assertEqual(check(self.root, "first"), (READY, []))

    def test_stale_when_no_spec_and_no_open_change_holds_the_requirement(self) -> None:
        self.delta("stale", "## MODIFIED Requirements\n\n### Requirement: Renamed away\nText.\n")
        self.assertEqual(check(self.root, "stale"), (STALE, ["cap: Renamed away"]))

    def test_an_archived_change_provides_nothing(self) -> None:
        write(
            self.root,
            "openspec/changes/archive/2026-01-01-old/specs/cap/spec.md",
            "## ADDED Requirements\n\n### Requirement: Gone rule\nText.\n",
        )
        self.delta("late", "## REMOVED Requirements\n\n### Requirement: Gone rule\n")
        self.assertEqual(check(self.root, "late"), (STALE, ["cap: Gone rule"]))

    def test_a_delta_may_modify_a_requirement_under_the_name_it_renames_it_to(self) -> None:
        self.delta(
            "rename",
            "## RENAMED Requirements\n\n"
            "- FROM: `### Requirement: Existing rule`\n"
            "  TO: `### Requirement: Better rule`\n\n"
            "## MODIFIED Requirements\n\n### Requirement: Better rule\nText.\n",
        )
        self.assertEqual(check(self.root, "rename"), (READY, []))

    def test_a_rename_from_a_missing_requirement_is_stale(self) -> None:
        self.delta(
            "rename",
            "## RENAMED Requirements\n\n- FROM: `### Requirement: Nowhere`\n- TO: `### Requirement: Somewhere`\n",
        )
        self.assertEqual(check(self.root, "rename"), (STALE, ["cap: Nowhere"]))


if __name__ == "__main__":
    unittest.main()
