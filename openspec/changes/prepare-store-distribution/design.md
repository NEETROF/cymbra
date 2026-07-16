## Context

Cymbra Music (`apps/music`) has a mature native build/signing/CI setup: correct display name (`Cymbra Music`), bundle IDs (`com.cymbra.music`), versioning (`1.10.0+14`), Android + iOS signing, and a `release-build.yml` pipeline that builds AAB/APK/IPA and pushes to TestFlight. What remains before a store submission is branding, one platform-manifest fix, the Apple privacy manifest, and the listing assets/copy.

Current state that this change addresses:
- iOS `AppIcon.appiconset` and Android `mipmap-*/ic_launcher.png` are the **stock Flutter logo**; no Android adaptive icon exists.
- `INTERNET` permission is present only in `android/app/src/{debug,profile}/AndroidManifest.xml`, **not** in `src/main/` — so release builds have no network access (verified).
- No `ios/Runner/PrivacyInfo.xcprivacy`.
- Default white launch screen on both platforms.
- No store screenshots, Play hi-res icon, feature graphic, or listing copy.

Constraints: the app is **landscape-only** and requires full screen; screenshots must be landscape. No app runtime logic changes are in scope, and the signing/CI pipeline must not be disturbed.

## Goals / Non-Goals

**Goals:**
- Ship real Cymbra branding (icon + splash) on iOS and Android from a single source asset.
- Make Android release builds network-capable (the release blocker).
- Satisfy Apple's privacy-manifest requirement for the app's dependencies.
- Assemble the store-listing assets and copy, version-controlled in the repo.

**Non-Goals:**
- Terms of Service / Privacy Policy legal documents (handled elsewhere).
- Changes to signing, provisioning, or `release-build.yml`.
- Microphone/other usage-description strings — the app records no audio.
- App feature/UI changes beyond branding assets.
- Automated screenshot generation / fastlane deliver wiring (may follow later; copy is stored so it's reusable).

## Decisions

**1. Generate icons with `flutter_launcher_icons` from one source asset (over hand-editing each set).**
Hand-maintaining ~15 iOS PNG sizes plus 5 Android density buckets plus adaptive layers is error-prone and drifts. A single `flutter_launcher_icons` config in `pubspec.yaml`, driven from a high-res source (e.g. `assets/branding/icon.png` ≥ 1024×1024, plus a separate foreground for adaptive), regenerates everything deterministically. Config sets `remove_alpha_ios: true` (Apple rejects alpha on the marketing icon) and `adaptive_icon_background` + `adaptive_icon_foreground` for Android 26+. The generator is a dev dependency; generated assets are committed.

**2. Splash via `flutter_native_splash` (over manual storyboard/drawable editing).**
Same rationale: one config block produces the iOS storyboard assets and Android `launch_background`/API-31 splash resources, with light + dark variants. Keeps the two platforms consistent and regenerable.

**3. Add `INTERNET` to `src/main/AndroidManifest.xml` (not only rely on debug/profile).**
Flutter's template intentionally scopes `INTERNET` to debug/profile for hot-reload, and does **not** add it to the release manifest automatically. Since the app needs the network in production, the permission belongs in the main manifest so it merges into every build type. This is a one-line, low-risk fix and the single most important item — without it the shipped app is broken.

**4. Author `PrivacyInfo.xcprivacy` by hand and add it to the Runner target.**
The manifest must declare `NSPrivacyAccessedAPITypes` (required-reason APIs) and `NSPrivacyCollectedDataTypes`. `flutter_secure_storage` uses Keychain and `shared_preferences` uses `UserDefaults` (`CA92.1` reason). Data-collection entries reflect account/auth data (Sign in with Apple/Google). Written as a plist and referenced from the Xcode project so it lands in the bundle; verified by inspecting the built IPA.

**5. Store listing assets and copy live in `apps/music/store/` (over storing them only in App Store Connect / Play Console).**
Keeping screenshots, the 512×512 Play icon, the 1024×500 feature graphic, and the listing text (`description`, `subtitle`, `keywords`, `category`) in-repo makes the listing reproducible, reviewable, and later automatable (fastlane `deliver`/`supply` read this layout). Screenshots are captured from the real landscape UI on the required device sizes.

## Risks / Trade-offs

- **Inaccurate privacy manifest → App Store rejection or policy violation.** → Enumerate every plugin that touches a required-reason API or collects data; base declarations on plugin docs, and verify the IPA contains the manifest before submitting.
- **`flutter_launcher_icons`/`flutter_native_splash` overwrite committed native files** (LaunchScreen, mipmaps) unexpectedly. → Run generators once, review the full diff, and commit the generated output as the source of truth; document the regenerate command so it's intentional.
- **Regenerating splash/icons could disturb the existing landscape-only / full-screen iOS config.** → Diff `Info.plist` and the storyboard after generation; keep orientation and `UIRequiresFullScreen` settings intact.
- **Screenshots go stale as the UI evolves.** → Capture close to submission; storing them in-repo makes refreshes a visible diff.
- **Icon source art may not be final.** → Treat the source asset as the single input; a rebrand is a re-run of the generator, not manual rework. (See Open Questions.)

## Migration Plan

No runtime migration — assets and manifest edits only. Rollout is a normal PR merged before the next `music-v*` release tag. Rollback is reverting the PR (icons/splash/manifest revert to prior state; no data or API impact). The `INTERNET` fix should land regardless, as current release builds are effectively broken without it.

## Resolved Decisions

- **Source artwork**: provided — `apps/music/assets/branding/icon_source.png` (1024×1024, navy tile with the "C" + waveform mark). Post-processing: fill the rounded corners to a full-bleed square for iOS (no alpha), remove the small generation-artifact sparkle. Adaptive Android background color read from the navy (~`#1A2340`); adaptive foreground uses the full mark as a fallback (no separate transparent cutout for now). Splash reuses the mark on the navy background.
- **Store categories**: iOS primary **Education**, secondary **Music**; Google Play category **Education**.
- **Screenshots**: captured **manually** from simulator/device for this first submission; a repeatable capture flow is deferred.

## Open Questions

- None blocking. Optional future refinement: a transparent "C"-only cutout for a cleaner Android adaptive foreground.

## Deferred (out of scope, revisit when adding analytics)

- **Firebase Analytics (or any usage analytics)** is intentionally NOT part of this change. When it is added later, the privacy posture must be revisited:
  - `PrivacyInfo.xcprivacy`: the Firebase pods (10.x+) ship their own manifests that Apple aggregates, so the app manifest needs little change — EXCEPT `NSPrivacyTracking` must flip to `true` (and ATT be implemented) if IDFA/ad signals are enabled.
  - App Store Connect **App Privacy** label and Google Play **Data Safety** form must be updated (Usage/Product Interaction, Diagnostics, Device ID).
  - Privacy policy must mention Firebase/Google.
  - **RGPD/ePrivacy**: EU users require opt-in consent before Analytics initialises (default collection disabled, enabled after consent / Consent Mode).
