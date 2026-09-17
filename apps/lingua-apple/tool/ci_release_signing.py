#!/usr/bin/env python3
"""App Store signing for the Cymbra Lingua host app, in CI only (lingua-apple-release).

The committed project keeps automatic *development* signing, so anyone can build and run
it with a personal team. For the store, each platform's two targets — the app and its
Safari extension — are switched to manual signing with their own App Store profile:

* ``install``: read each profile, copy it where Xcode looks, patch the *Release*
  configurations of the matching targets and write the export options.
* ``verify``: read the export's ``DistributionSummary.plist`` and fail unless every bundle
  is distribution-signed with a profile and carries the entitlements the app relies on.

Why a project edit rather than ``xcodebuild SETTING=value``: a command-line setting hits
both targets of a scheme, and the app and the extension need different profiles. Why not
an unsigned archive re-signed at export: the export then signs without processing
``CODE_SIGN_ENTITLEMENTS`` — no App Sandbox, no App Group, no Sign in with Apple — and
still reports success (``prepare-macos-app-store``, Music). ``verify`` exists for that trap.
"""

from __future__ import annotations

import argparse
import plistlib
import re
import shutil
import subprocess
import sys
from pathlib import Path

APP_ID = "com.cymbra.lingua"
EXTENSION_ID = "com.cymbra.lingua.Extension"
SDK = {"ios": "iphoneos", "macos": "macosx"}
PROFILE_EXT = {"ios": ".mobileprovision", "macos": ".provisionprofile"}
PROFILE_DIRS = [
    Path.home() / "Library/MobileDevice/Provisioning Profiles",
    Path.home() / "Library/Developer/Xcode/UserData/Provisioning Profiles",
]
APP_GROUP = {"ios": "group.com.cymbra.lingua", "macos": "VMFJ6KRW77.com.cymbra.lingua"}

# One XCBuildConfiguration object: `\t\t<id> /* Release */ = { ... \t\t};`
CONFIG = re.compile(r"\n\t\t[0-9A-F]{24} /\* (?:Debug|Release) \*/ = \{\n.*?\n\t\t\};", re.S)


def patch_project(text: str, platform: str, team: str, profiles: dict[str, str]) -> str:
    """Switch the Release configuration of each target in ``profiles`` (bundle id → profile
    name) on ``platform`` to manual App Store signing. Exactly one configuration per bundle
    id must match."""
    sdk = f"SDKROOT = {SDK[platform]};"
    seen: dict[str, int] = {bundle: 0 for bundle in profiles}

    def switch(match: re.Match[str]) -> str:
        block = match.group(0)
        if "name = Release;" not in block or sdk not in block:
            return block
        bundle = re.search(r"PRODUCT_BUNDLE_IDENTIFIER = ([^;]+);", block)
        if not bundle or bundle.group(1) not in profiles:
            return block
        name = profiles[bundle.group(1)]
        seen[bundle.group(1)] += 1
        if "CODE_SIGN_STYLE = Automatic;" not in block:
            raise SystemExit(f"{bundle.group(1)} ({platform}): no automatic signing to switch")
        return block.replace(
            "CODE_SIGN_STYLE = Automatic;",
            "CODE_SIGN_STYLE = Manual;\n"
            '\t\t\t\tCODE_SIGN_IDENTITY = "Apple Distribution";\n'
            f"\t\t\t\tDEVELOPMENT_TEAM = {team};\n"
            f'\t\t\t\tPROVISIONING_PROFILE_SPECIFIER = "{name}";',
        )

    patched = CONFIG.sub(switch, text)
    wrong = {bundle: count for bundle, count in seen.items() if count != 1}
    if wrong:
        raise SystemExit(f"expected one {platform} Release configuration per target, found {wrong}")
    return patched


def export_options(platform: str, team: str, profiles: dict[str, str]) -> dict:
    options = {
        "method": "app-store-connect",
        "teamID": team,
        "signingStyle": "manual",
        "signingCertificate": "Apple Distribution",
        "provisioningProfiles": profiles,
        "destination": "export",
        "manageAppVersionAndBuildNumber": False,
        "uploadSymbols": True,
    }
    if platform == "macos":
        options["installerSigningCertificate"] = "3rd Party Mac Developer Installer"
    return options


def read_profile(path: Path) -> dict:
    decoded = subprocess.run(["security", "cms", "-D", "-i", str(path)], check=True, capture_output=True).stdout
    return plistlib.loads(decoded)


def install(args: argparse.Namespace) -> None:
    profiles: dict[str, str] = {}
    for spec in args.profile:
        bundle, _, file = spec.partition("=")
        info = read_profile(Path(file))
        app_id = info["Entitlements"].get("application-identifier") or info["Entitlements"].get(
            "com.apple.application-identifier"
        )
        if app_id != f"{args.team}.{bundle}":
            raise SystemExit(f"{file}: profile is for {app_id}, not {args.team}.{bundle}")
        if info.get("ProvisionedDevices"):
            raise SystemExit(f"{file}: a development or ad hoc profile, not an App Store one")
        for directory in PROFILE_DIRS:
            directory.mkdir(parents=True, exist_ok=True)
            shutil.copy(file, directory / f"{info['UUID']}{PROFILE_EXT[args.platform]}")
        profiles[bundle] = info["Name"]
    if set(profiles) != {APP_ID, EXTENSION_ID}:
        raise SystemExit(f"need a profile for {APP_ID} and {EXTENSION_ID}, got {sorted(profiles)}")

    project = Path(args.project)
    project.write_text(patch_project(project.read_text(), args.platform, args.team, profiles))
    with open(args.export_options, "wb") as out:
        plistlib.dump(export_options(args.platform, args.team, profiles), out)
    print(f"{args.platform}: manual App Store signing with {profiles}")


def problems(summary: dict, platform: str) -> list[str]:
    """What is wrong with an export, from its DistributionSummary.plist."""
    found: list[str] = []
    bundles: dict[str, dict] = {}

    def walk(items: list[dict]) -> None:
        for item in items:
            bundles[item.get("name", "?")] = item
            walk(item.get("embeddedBinaries", []))

    for items in summary.values():
        walk(items)
    for name, item in bundles.items():
        ent = item.get("entitlements", {})
        if item.get("certificate", {}).get("type") != "Apple Distribution":
            found.append(f"{name}: not signed with Apple Distribution")
        if not item.get("profile"):
            found.append(f"{name}: no provisioning profile (TestFlight refuses it)")
        if APP_GROUP[platform] not in ent.get("com.apple.security.application-groups", []):
            found.append(f"{name}: missing the App Group {APP_GROUP[platform]}")
        if platform == "macos" and not ent.get("com.apple.security.app-sandbox"):
            found.append(f"{name}: missing the App Sandbox")
        if name.endswith(".app") and "com.apple.developer.applesignin" not in ent:
            found.append(f"{name}: missing Sign in with Apple")
    if not any(name.endswith(".appex") for name in bundles):
        found.append("no Safari extension in the package")
    return found


def verify(args: argparse.Namespace) -> None:
    with open(args.summary, "rb") as f:
        found = problems(plistlib.load(f), args.platform)
    if found:
        print("\n".join(f"::error::{line}" for line in found))
        sys.exit(1)
    print(f"{args.platform}: every bundle is store-signed with its profile and entitlements")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__, formatter_class=argparse.RawDescriptionHelpFormatter)
    sub = parser.add_subparsers(dest="command", required=True)
    inst = sub.add_parser("install")
    inst.add_argument("project")
    inst.add_argument("--platform", choices=SDK, required=True)
    inst.add_argument("--team", required=True)
    inst.add_argument("--profile", action="append", required=True, metavar="BUNDLE_ID=FILE")
    inst.add_argument("--export-options", required=True)
    inst.set_defaults(run=install)
    ver = sub.add_parser("verify")
    ver.add_argument("summary")
    ver.add_argument("--platform", choices=SDK, required=True)
    ver.set_defaults(run=verify)
    args = parser.parse_args()
    args.run(args)


if __name__ == "__main__":
    main()
