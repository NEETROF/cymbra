## Context

`apps/music` already ships to the iOS App Store and Google Play. Its macOS target
builds and runs, carries the branding pass from `prepare-store-distribution`
(task 8: window title, `CFBundleName`, MainMenu items) and a complete
`AppIcon.appiconset` (16→1024). What it does not have is a distribution chain: the
`macos` job in [release-build.yml:156](../../../.github/workflows/release-build.yml:156)
appends `CODE_SIGNING_ALLOWED = NO` to `Release.xcconfig` and publishes a
`ditto`-zipped `.app`. That artifact is ad-hoc signed, so Gatekeeper quarantines it
on every machine except the builder's — it is not a usable download today.

Three consequences follow, and they are the substance of this change:

1. **Nothing targets the store.** `flutter build macos` produces a `.app`; the Mac
   App Store consumes a `.pkg` produced by `xcodebuild -exportArchive`. There is no
   archive step, no export options, no upload.
2. **The committed signing config is a development identity.**
   [project.pbxproj:757](../../../apps/music/macos/Runner.xcodeproj/project.pbxproj:757)
   sets the *Release* configuration to `CODE_SIGN_STYLE = Automatic` with
   `Apple Development`. Correct for `flutter run -d macos`, invalid for submission.
3. **Sandbox gaps are invisible.** `Release.entitlements` enables
   `com.apple.security.app-sandbox`, but an ad-hoc-signed binary never exercises the
   entitlement set the way a store-signed one does. The clearest example:
   `flutter_secure_storage: ^10.3.1` needs a `keychain-access-groups` entitlement to
   reach the keychain under the sandbox, and it is absent. Nobody has noticed because
   no sandboxed signed build has ever been run.

Constraints: the Apple-side prerequisites (App ID platform, certificates, profile,
App Store Connect record) require the account holder and cannot be produced from the
repository. The `ios` job is a proven, in-repo pattern for everything that *can* be
automated, and this design deliberately mirrors it rather than inventing a second
idiom.

## Goals / Non-Goals

**Goals:**

- A `music-v*` tag produces a Mac App Store `.pkg` and delivers it to App Store
  Connect, with no manual Xcode step.
- The committed project keeps working for local development (`flutter run -d macos`
  with automatic development signing); distribution settings are an overlay, not a
  rewrite.
- Every entitlement the shipped app actually needs is present in *both* entitlement
  files, so nothing works in debug and fails in release.
- Sandbox conformance of MIDI, audio, `.sf2` import, sign-in and local storage is
  verified on a real signed build before submission.
- A maintainer can reproduce the whole setup from `apps/music/README.md`.

**Non-Goals:**

- **Developer ID / notarized direct distribution.** A separate cert, a separate
  export method and a notarization round-trip. Worth doing later so macOS users can
  install outside the store; not required to publish.
- **Push notifications on macOS.** No `aps-environment` on this target; the
  `add-push-notifications` change scoped iOS/Android deliberately.
- **A macOS `PrivacyInfo.xcprivacy`.** Apple's privacy-manifest requirement covers
  iOS, iPadOS, tvOS, watchOS and visionOS — not macOS.
- **Hardened Runtime.** Required for notarization, not for the Mac App Store, where
  App Sandbox is the gate. Enabling it would add a failure mode for no submission
  benefit.
- Any backend, proto, database or Rust change. This is packaging only.

## Decisions

### D1 — `flutter build macos` first, then `xcodebuild archive`/`-exportArchive`

The Flutter tool has no macOS equivalent of `flutter build ipa
--export-options-plist`: it stops at the `.app`. So the job runs
`flutter build macos --release --dart-define-from-file=config/prod.json` to
materialise the ephemeral Flutter config (dart-defines, engine, pods, and the
cargokit Rust build), then archives the workspace with
`xcodebuild -workspace macos/Runner.xcworkspace -scheme Runner -configuration Release
-destination 'generic/platform=macOS' -archivePath … archive`, and exports with
`-exportArchive -exportOptionsPlist` at `method: app-store`, which yields the `.pkg`.

*Alternative considered:* archiving directly without the `flutter build` pass. The
archive would still compile, but the dart-defines live in Flutter's generated
xcconfig, which only the Flutter tool writes — the app would ship pointing at the
dev gRPC endpoint. Rejected: silent misconfiguration is the worst possible failure
here.

### D2 — Distribution signing as a CI-only `project.pbxproj` patch

**Superseded the original xcconfig plan.** The workflow runs
`macos/tool/ci_release_signing.py`, which rewrites three settings inside the Runner
target's *Release* build-settings block — the block is located by the unique anchor
`CODE_SIGN_ENTITLEMENTS = Runner/Release.entitlements;`, and the script refuses to
run if that anchor is missing or ambiguous. The committed project keeps automatic
*development* signing, so contributors build without a distribution certificate.

The first three approaches all failed, each in an instructive way, and the script's
docstring records them so nobody re-walks the path:

1. **xcconfig overlay** (the plan, copied from the `ios` job) — cannot win. The
   macOS Runner target sets `CODE_SIGN_STYLE`, `PROVISIONING_PROFILE_SPECIFIER` and
   the *conditional* `CODE_SIGN_IDENTITY[sdk=macosx*] = "Apple Development"` at
   **target** level, which outranks any xcconfig. The iOS job gets away with it only
   because Flutter's iOS target leaves those unset. CI died with "couldn't find any
   Mac App Development provisioning profiles" — a *development* profile, on a runner
   that has no development identity at all.
2. **`xcodebuild SETTING=value`** — does outrank the target (verified with
   `-showBuildSettings`), but applies to *every* target in the workspace. All ~30
   CocoaPods targets failed with "<pod> does not support provisioning profiles".
3. **Unsigned archive, let `-exportArchive` re-sign** — the dangerous one. It
   *succeeds*: signed app, signed `.pkg`, no errors. But `CODE_SIGNING_ALLOWED = NO`
   means `CODE_SIGN_ENTITLEMENTS` is never processed, so the shipped app carries only
   the profile-derived `application-identifier` and `team-identifier`. No App Sandbox
   (an automatic rejection), no network, no keychain. Caught only by diffing the
   delivered entitlements against a known-good package.

*Alternative still rejected:* committing distribution signing into the project. It
would hardcode a profile name and break `flutter build macos --release` for any
contributor without the certificate.

### D3 — Two certificates, two signing roles

Mac App Store submission signs twice: the `.app` with **Apple Distribution**, the
installer package with **Mac Installer Distribution**. The iOS `IOS_DIST_CERT_BASE64`
covers neither role for macOS. Both are imported into the same throwaway keychain
(the `ios` job's `security create-keychain` / `set-key-partition-list` sequence,
reused verbatim), and the installer identity is named in the export options via
`installerSigningCertificate`.

### D4 — The `.pkg` is uploaded, not attached to the GitHub Release

A `method: app-store` package is a submission artifact: double-clicking it fails,
because it is only installable by the store. Attaching it would advertise a download
that cannot be installed. The unsigned `.zip` is dropped at the same time — it is
already Gatekeeper-blocked for everyone but the builder, so nothing usable is lost.
Until the listing goes live, macOS has no direct download; the Developer ID
notarized `.dmg` (non-goal above) is the follow-up that restores one.

*Alternative considered:* keeping the unsigned `.zip` alongside. Rejected: it
implies a working download and generates "the app is damaged" reports.

### D5 — `keychain-access-groups` in both entitlement files

`$(AppIdentifierPrefix)com.cymbra.music` is added to `Release.entitlements` **and**
`DebugProfile.entitlements`. It is an implicit entitlement — provisioning profiles
grant it without an App ID capability toggle — but it must be declared for a
sandboxed app to reach its own keychain items, which is what `flutter_secure_storage`
does for the session token. Putting it in both files is the point of the "debug and
release entitlements stay consistent" requirement: the asymmetry is how this bug got
in.

The same audit says what *not* to add: `com.apple.security.network.server` stays
debug-only. macOS sign-in goes through the native Google SDK and Sign in with Apple
([oidc_token_source.dart:70](../../../apps/music/lib/services/oidc_token_source.dart:70)),
not the desktop loopback flow used on Windows and Linux, so no listening socket is
needed and requesting one would invite review questions.

### D6 — Universal Purchase on the existing `com.cymbra.music` record

Same bundle identifier as iOS, macOS added as a platform on the same App Store
Connect app record. Users who buy or download on one platform get the other. This
requires enabling macOS on the App ID; it does not require a second record or a
second bundle id. Build numbers are tracked per platform, so iOS and macOS can both
deliver from `version: 1.20.0+27` without collision.

### D7 — Metadata values

`LSApplicationCategoryType = public.app-category.education`, matching the Education
primary category already chosen for iOS (`prepare-store-distribution` task 6.2).
`ITSAppUsesNonExemptEncryption = false`, matching the iOS declaration and the
reality: TLS to `api.cymbra.app` plus platform-provided storage encryption, both
exempt.

### D8 — Sandbox verification is a manual checklist, not CI

CI has no MIDI hardware, no audio device and no interactive sign-in, so the five
runtime scenarios in the spec cannot be automated. They become explicit manual tasks
executed once on a signed sandboxed build installed from the exported package —
which is also the only way to exercise the real entitlement set.

## Risks / Trade-offs

- **First tagged run fails on signing.** The iOS job needed profile-name tuning on
  its first real run, and macOS signing is at least as environment-sensitive. →
  `workflow_dispatch` builds and signs without uploading (D4/spec), so the chain can
  be validated repeatedly before a tag is ever pushed.

- **Nested code signing under `use_frameworks!`.** The macOS `Podfile` uses dynamic
  frameworks, so every pod — `GoogleSignIn`, `AppAuth`, `GTMSessionFetcher`,
  `flutter_secure_storage`, the Flutter engine — must be signed under the same team,
  or the package is rejected for invalid nested code. → Xcode's embed-and-sign phase
  does this automatically once a real identity is configured; the mitigation is
  simply that D2 configures one. The Rust engine is *not* an extra risk: the
  cargokit podspec produces a **static** `librust_lib_music.a` force-loaded into the
  pod framework, so there is no separate dylib to sign.

- **Changing the keychain access group orphans locally stored credentials.**
  Existing dev builds' keychain items become unreadable under the new group. →
  Affects developer machines only, and the app already handles an absent token by
  showing the sign-in flow. Sign out and back in.

- **No macOS download during review.** Between dropping the `.zip` (D4) and the
  listing going live, macOS users have nothing to install. → Accepted: the `.zip`
  was not installable either. Bounded by review time, and the notarized `.dmg`
  follow-up closes it permanently.

- **Sandbox may break a feature this design assumes is fine.** CoreMIDI is the one to
  watch: the app creates a notifying MIDI client on the main run loop for hot-plug
  (`AppDelegate.swift`), and it has never run inside a sandbox. → It is a named
  spec scenario with its own verification task; if it fails, the fix is a
  `com.apple.security.device.*` entitlement and a review note, discovered before
  submission rather than after a rejection.

- **Store review rejects on content or account grounds.** Sign-in-required apps and
  user-generated score/soundfont content draw scrutiny. → Out of this change's
  scope, but the existing legal links, moderation pipeline and the guest "try without
  an account" path from `add-welcome-onboarding` are the relevant answers, and the
  reviewer notes should point at them.

## Open Questions

- Which display size for the macOS screenshots — 1440×900 or 2880×1800? Either is
  accepted; 2880×1800 on a Retina display is the better-looking listing and is what
  the task assumes unless capture proves impractical.
- Does the Mac App Store listing reuse the iOS Education category as secondary
  category too, or pick Music? Deferred to the listing task; it does not affect the
  bundle, which declares only the primary.
