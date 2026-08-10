## Why

Cymbra Music ships to the iOS App Store and Google Play, but its macOS build stops
at an **unsigned, ad-hoc `.app`** zipped onto the GitHub Release — Gatekeeper blocks
it on any machine but the builder's, and nothing in the tree targets the Mac App
Store. The desktop build is otherwise feature-complete (same Flutter/Rust binary,
same branding pass as `prepare-store-distribution` task 8), so the only thing
between it and a Mac App Store listing is the signing/distribution chain plus a
handful of sandbox-conformance gaps that the unsigned artifact has been hiding.

## What Changes

- **Signed Mac App Store distribution.** The `macos` job in `release-build.yml`
  stops disabling code signing and instead imports an Apple Distribution cert, a
  Mac Installer Distribution cert and a Mac App Store provisioning profile into a
  throwaway keychain, archives with `xcodebuild`, exports a `.pkg` with
  `method: app-store`, and delivers it with `xcrun altool --upload-app --type macos`.
  The unsigned `.zip` is replaced, not kept alongside — a Gatekeeper-blocked
  download is worse than no download.
- **Store-validation metadata.** `macos/Runner/Info.plist` gains
  `LSApplicationCategoryType` (absent today → hard validation failure) and
  `ITSAppUsesNonExemptEncryption` (present on iOS, missing here → export-compliance
  prompt on every delivery).
- **Sandbox conformance.** `Release.entitlements` / `DebugProfile.entitlements`
  gain `keychain-access-groups`, which `flutter_secure_storage` 10.x requires under
  App Sandbox on macOS. Without it a *signed* sandboxed build loses its session
  tokens — a failure the current unsigned artifact cannot surface.
- **Distribution signing identity.** The Runner Release configuration moves off
  `Apple Development` / automatic signing (a *development* identity) onto Apple
  Distribution with the Mac App Store profile, driven from `Release.xcconfig` the
  way the `ios` job already does it.
- **Runtime verification under a signed sandbox.** CoreMIDI input and hot-plug,
  cpal/rustysynth audio output, `.sf2` import through `file_picker`, Google/Apple
  sign-in and the encrypted offline cache are each exercised on a signed sandboxed
  build — none of them has ever run under a real sandbox.
- **macOS listing assets.** `apps/music/store/macos/` screenshots at an
  Apple-accepted size, plus the macOS category recorded next to the existing
  en/fr/it/es copy.
- Out of scope: Developer ID / notarized direct distribution, push notifications on
  macOS (no `aps-environment`), and a macOS `PrivacyInfo.xcprivacy` (Apple does not
  require privacy manifests on macOS).

## Capabilities

### New Capabilities
- `music-macos-store-distribution`: the macOS release artifact of Cymbra Music is a
  signed, sandbox-conformant Mac App Store package — signing chain, store-validation
  metadata, entitlements the app's dependencies actually need under App Sandbox,
  automated delivery, and the listing assets.

### Modified Capabilities
<!-- None. The iOS/Android `store-distribution` capability from the in-flight
     `prepare-store-distribution` change is not yet archived into `openspec/specs/`,
     and none of its requirements change here — macOS is additive. -->

## Impact

**Products.** Cymbra Music only. Consumes the existing Cymbra ID sign-in
(`account-access`), score catalog and soundfont delivery unchanged — no backend, no
API, no proto, no database change. Cymbra Live, the back-office and the Rust
workspace are untouched.

**Code and config**
- `apps/music/macos/Runner/Info.plist` — two new keys
- `apps/music/macos/Runner/Release.entitlements`, `DebugProfile.entitlements` —
  keychain access group
- `apps/music/macos/Runner/Configs/Release.xcconfig` (CI-generated overlay) and
  `Runner.xcodeproj/project.pbxproj` — distribution signing
- `apps/music/macos/ExportOptions-ci.plist` (new, CI-generated)
- `.github/workflows/release-build.yml` — the `macos` job, rewritten
- `apps/music/store/macos/` (new), `apps/music/store/copy/` — category line
- `apps/music/README.md` — a macOS signing section beside the iOS one

**Secrets** (new, GitHub Actions): `MAC_DIST_CERT_BASE64`,
`MAC_DIST_CERT_PASSWORD`, `MAC_INSTALLER_CERT_BASE64`,
`MAC_INSTALLER_CERT_PASSWORD`, `MAC_PROVISIONING_PROFILE_BASE64`,
`MAC_PROVISIONING_PROFILE_NAME`. Reuses `IOS_TEAM_ID`, `ASC_API_KEY_ID`,
`ASC_API_ISSUER_ID`, `ASC_API_KEY_P8`, `GOOGLE_CLIENT_ID`.

**Manual, account-holder only** (cannot be automated from the repo): enabling the
macOS platform on the `com.cymbra.music` App ID, issuing the two distribution
certificates and the Mac App Store profile, and adding the macOS platform to the
existing App Store Connect record (Universal Purchase — same bundle id as iOS).

**Risk.** Signing is environment-sensitive: the first tagged run may need the
profile name adjusted, exactly as the iOS job did. `use_frameworks!` means every
embedded pod framework must sign under the same team; Xcode's embed-and-sign phase
handles this once a real identity is in place, but it is the most likely source of
a first-run failure.
