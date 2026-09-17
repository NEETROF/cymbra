"""Tests for ci_release_signing.py, against the committed project (run: python3 -m unittest)."""

import copy
import re
import unittest
from pathlib import Path

import ci_release_signing as signing

PROJECT = (Path(__file__).parent.parent / "Cymbra Lingua.xcodeproj/project.pbxproj").read_text()
PROFILES = {signing.APP_ID: "Lingua App Store", signing.EXTENSION_ID: "Lingua Extension App Store"}


def configs(text: str) -> list[str]:
    return [m.group(0) for m in signing.CONFIG.finditer(text)]


class PatchProjectTest(unittest.TestCase):
    def test_switches_only_the_platforms_release_configurations(self):
        for platform, sdk in signing.SDK.items():
            patched = signing.patch_project(PROJECT, platform, "TEAM", PROFILES)
            changed = [c for c in configs(patched) if "CODE_SIGN_STYLE = Manual;" in c]
            self.assertEqual(len(changed), 2, platform)
            for block in changed:
                self.assertIn("name = Release;", block)
                self.assertIn(f"SDKROOT = {sdk};", block)
                self.assertIn('CODE_SIGN_IDENTITY = "Apple Distribution";', block)
                self.assertIn("DEVELOPMENT_TEAM = TEAM;", block)
            names = sorted(re.search(r'PROVISIONING_PROFILE_SPECIFIER = "([^"]+)"', b).group(1) for b in changed)
            self.assertEqual(names, sorted(PROFILES.values()))
            # Everything else, Debug included, is untouched.
            self.assertEqual(len(configs(patched)), len(configs(PROJECT)))
            self.assertEqual(patched.count("CODE_SIGN_STYLE = Automatic;"), PROJECT.count("CODE_SIGN_STYLE = Automatic;") - 2)

    def test_refuses_a_target_it_cannot_find(self):
        with self.assertRaises(SystemExit):
            signing.patch_project(PROJECT, "ios", "TEAM", {**PROFILES, "com.cymbra.other": "x"})

    def test_export_options_sign_the_package_on_macos_only(self):
        self.assertNotIn("installerSigningCertificate", signing.export_options("ios", "TEAM", PROFILES))
        mac = signing.export_options("macos", "TEAM", PROFILES)
        self.assertEqual(mac["installerSigningCertificate"], "3rd Party Mac Developer Installer")
        self.assertEqual(mac["provisioningProfiles"], PROFILES)
        self.assertEqual(mac["method"], "app-store-connect")


def bundle(name, entitlements, profile="p", cert="Apple Distribution", embedded=()):
    return {
        "name": name,
        "certificate": {"type": cert},
        "profile": {"name": profile} if profile else None,
        "entitlements": entitlements,
        "embeddedBinaries": list(embedded),
    }


def good_summary(platform):
    group = {"com.apple.security.application-groups": [signing.APP_GROUP[platform]]}
    sandbox = {"com.apple.security.app-sandbox": True} if platform == "macos" else {}
    ext = bundle("Cymbra Lingua Extension.appex", {**group, **sandbox})
    app = bundle(
        "Cymbra Lingua.app", {**group, **sandbox, "com.apple.developer.applesignin": ["Default"]}, embedded=[ext]
    )
    return {"Cymbra Lingua.ipa": [app]}


class ProblemsTest(unittest.TestCase):
    def test_accepts_a_store_signed_package(self):
        for platform in signing.SDK:
            self.assertEqual(signing.problems(good_summary(platform), platform), [])

    def test_reports_the_entitlement_and_profile_traps(self):
        summary = good_summary("macos")
        app = summary["Cymbra Lingua.ipa"][0]
        ext = app["embeddedBinaries"][0]
        ext["profile"] = None
        del app["entitlements"]["com.apple.security.app-sandbox"]
        del app["entitlements"]["com.apple.developer.applesignin"]
        found = "\n".join(signing.problems(summary, "macos"))
        self.assertIn("Extension.appex: no provisioning profile", found)
        self.assertIn("Lingua.app: missing the App Sandbox", found)
        self.assertIn("Lingua.app: missing Sign in with Apple", found)

    def test_reports_a_development_signature_and_a_missing_extension(self):
        summary = good_summary("ios")
        app = copy.deepcopy(summary["Cymbra Lingua.ipa"][0])
        app["certificate"]["type"] = "Apple Development"
        app["embeddedBinaries"] = []
        found = "\n".join(signing.problems({"x": [app]}, "ios"))
        self.assertIn("not signed with Apple Distribution", found)
        self.assertIn("no Safari extension", found)


if __name__ == "__main__":
    unittest.main()
