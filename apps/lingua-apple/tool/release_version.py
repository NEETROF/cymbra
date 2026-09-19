#!/usr/bin/env python3
"""The version the Cymbra Lingua host app ships, for lingua-apple-release.

It comes from ``apps/lingua-apple/version.txt``, which release-please maintains — not from
``MARKETING_VERSION`` in the Xcode project. CI already rewrites that project file for App
Store signing (``ci_release_signing.py``), and a second CI-owned edit in the same file is a
conflict waiting for the next signing change; the lane passes the version on the
``xcodebuild`` command line instead, beside ``CURRENT_PROJECT_VERSION``.

On a ``lingua-apple-v*`` tag the two must agree. They can only disagree when a tag was
pushed by hand, and shipping then would put a version in App Store Connect that no
changelog describes — so the run stops instead.

Usage (prints the version, or the reason on stderr and exits 1)::

    release_version.py --version-file apps/lingua-apple/version.txt \\
        --ref-type "$GITHUB_REF_TYPE" --ref-name "$GITHUB_REF_NAME"
"""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path

TAG_PREFIX = "lingua-apple-v"

# Three plain integers. Apple accepts more shapes, but release-please produces this one and
# anything else means the file was edited by hand.
VERSION = re.compile(r"^\d+\.\d+\.\d+$")


class VersionError(Exception):
    """Why this build must not go ahead."""


def resolve(version_text: str, ref_type: str, ref_name: str) -> str:
    """The marketing version to stamp, or raise with the reason."""
    version = version_text.strip()
    if not VERSION.match(version):
        raise VersionError(
            f'version.txt holds "{version}", which is not three plain integers. '
            "release-please writes this file; it looks hand-edited."
        )
    if ref_type != "tag":
        return version
    if not ref_name.startswith(TAG_PREFIX):
        raise VersionError(
            f'tag "{ref_name}" is not a {TAG_PREFIX}* tag, so this lane cannot tell what it releases.'
        )
    tagged = ref_name[len(TAG_PREFIX) :]
    if tagged != version:
        raise VersionError(
            f'tag "{ref_name}" says {tagged} but version.txt says {version}. '
            "The build would ship a version the changelog does not describe."
        )
    return version


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--version-file", type=Path, required=True)
    parser.add_argument("--ref-type", default="branch", help="GITHUB_REF_TYPE: tag or branch")
    parser.add_argument("--ref-name", default="", help="GITHUB_REF_NAME")
    args = parser.parse_args(argv)
    try:
        print(resolve(args.version_file.read_text(), args.ref_type, args.ref_name))
    except VersionError as e:
        print(f"::error::{e}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
