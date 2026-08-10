#!/usr/bin/env python3
"""Switch the Runner target's *Release* configuration to Mac App Store signing.

Why this exists instead of an xcconfig or an xcodebuild flag — both were tried and
both fail, for different reasons:

* An **xcconfig** cannot win. The Runner target sets `CODE_SIGN_STYLE`,
  `PROVISIONING_PROFILE_SPECIFIER` and, crucially, the conditional
  `CODE_SIGN_IDENTITY[sdk=macosx*] = "Apple Development"` at *target* level, which
  outranks any xcconfig. (The iOS job gets away with an xcconfig precisely because
  Flutter's iOS target leaves those unset.) The build then looks for a *development*
  profile and dies with "couldn't find any Mac App Development provisioning
  profiles" — on a runner that has no development identity at all.
* **`xcodebuild SETTING=value`** does outrank the target, but it applies to *every*
  target in the workspace. All ~30 CocoaPods targets then fail with "<pod> does not
  support provisioning profiles, but provisioning profile ... has been manually
  specified".
* Letting the archive go **unsigned** and having `-exportArchive` re-sign does
  produce a signed .pkg — but silently drops every entitlement, because
  `CODE_SIGN_ENTITLEMENTS` is never processed. The result has no App Sandbox (an
  automatic App Store rejection) and no keychain or network access.

So the signing settings have to land on the Runner target itself, which is what
Xcode's "Signing & Capabilities" editor would write. This script does exactly that
and nothing else.

It is a CI-only mutation: the committed project keeps automatic *development*
signing so contributors can build without a distribution certificate.
"""

from __future__ import annotations

import argparse
import re
import sys

# Unique to the Runner target's Release build settings — the Debug and Profile
# configurations reference DebugProfile.entitlements, and no other target sets it.
ANCHOR = "CODE_SIGN_ENTITLEMENTS = Runner/Release.entitlements;"

REPLACEMENTS = (
    (
        re.compile(r'"CODE_SIGN_IDENTITY\[sdk=macosx\*\]" = "[^"]*";'),
        '"CODE_SIGN_IDENTITY[sdk=macosx*]" = "Apple Distribution";',
    ),
    (re.compile(r"CODE_SIGN_STYLE = \w+;"), "CODE_SIGN_STYLE = Manual;"),
)


def patch(text: str, profile: str, team: str) -> str:
    start = text.find(ANCHOR)
    if start == -1:
        raise SystemExit(f"anchor not found in project.pbxproj: {ANCHOR!r}")
    if text.find(ANCHOR, start + 1) != -1:
        raise SystemExit(f"anchor is not unique: {ANCHOR!r}")

    # Operate only on the enclosing buildSettings block, so no other target is
    # touched. The block ends at the first "};" after the anchor.
    end = text.index("};", start)
    block = text[start:end]

    for pattern, replacement in REPLACEMENTS:
        block, n = pattern.subn(replacement, block)
        if n != 1:
            raise SystemExit(
                f"expected exactly 1 match for {pattern.pattern!r}, got {n}"
            )

    block, n = re.subn(
        r"PROVISIONING_PROFILE_SPECIFIER = [^;]*;",
        f'PROVISIONING_PROFILE_SPECIFIER = "{profile}";',
        block,
    )
    if n != 1:
        raise SystemExit(f"expected exactly 1 PROVISIONING_PROFILE_SPECIFIER, got {n}")

    block, n = re.subn(r"DEVELOPMENT_TEAM = [^;]*;", f"DEVELOPMENT_TEAM = {team};", block)
    if n != 1:
        raise SystemExit(f"expected exactly 1 DEVELOPMENT_TEAM, got {n}")

    return text[:start] + block + text[end:]


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("pbxproj", help="path to macos/Runner.xcodeproj/project.pbxproj")
    ap.add_argument("--profile", required=True, help="Mac App Store profile name")
    ap.add_argument("--team", required=True, help="Apple Developer team id")
    args = ap.parse_args()

    with open(args.pbxproj, encoding="utf-8") as fh:
        original = fh.read()

    patched = patch(original, args.profile, args.team)
    with open(args.pbxproj, "w", encoding="utf-8") as fh:
        fh.write(patched)

    print(f"Release configuration switched to manual Apple Distribution signing "
          f"(profile: {args.profile}, team: {args.team}).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
