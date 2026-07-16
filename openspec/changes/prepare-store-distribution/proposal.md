## Why

The build, signing, and CI pipeline for Cymbra Music is store-ready, but the app still ships the **default Flutter branding** (icon and splash), is **missing a release-critical Android permission**, and lacks the **Apple privacy manifest** and **store-listing assets** required for submission. In its current state a submission to either the App Store or Google Play would be rejected — and the Android release build would silently fail all network calls. This change closes those gaps so the app can be submitted.

## What Changes

- **App icons** — replace the stock Flutter icons with the real Cymbra branding on iOS (`AppIcon.appiconset`) and Android (legacy mipmaps + an **adaptive icon**), generated from a single source asset via `flutter_launcher_icons` wired into `pubspec.yaml`.
- **Android INTERNET permission (release blocker)** — add `android.permission.INTERNET` to `src/main/AndroidManifest.xml`. It currently exists only in the `debug`/`profile` manifests, so **release builds have no network access** and all gRPC calls to `api.cymbra.app:443` fail.
- **Apple privacy manifest** — add `ios/Runner/PrivacyInfo.xcprivacy` declaring required-reason API usage (triggered by `flutter_secure_storage` / `shared_preferences`) plus collected data types. Mandatory for App Store acceptance.
- **Custom splash screen** — replace the default white launch screen on iOS and Android with Cymbra branding via `flutter_native_splash`.
- **Store-listing assets** — produce the required marketing images: iOS screenshots (6.7" iPhone, 12.9" iPad, **landscape** — the app is landscape-only), Android phone screenshots, the Play 512×512 hi-res icon, and the Play 1024×500 feature graphic.
- **Store-listing copy** — write the listing text: app description, subtitle/promo, iOS keywords, and category selection, checked into the repo so it is version-controlled and reusable by CI/fastlane later.

Out of scope: Terms of Service / Privacy Policy legal documents (handled separately); the build/signing/CI pipeline (already in place); microphone usage strings (the app does not record audio).

## Capabilities

### New Capabilities
- `store-distribution`: The requirements for making the app submittable to the Apple App Store and Google Play — branding assets (icon, splash), platform metadata and manifests (Android permissions, Apple privacy manifest), and the store-listing assets and copy.

### Modified Capabilities
<!-- None — no existing capability's spec-level behavior changes. -->

## Impact

- **Dependencies (dev)**: adds `flutter_launcher_icons` and `flutter_native_splash` to `apps/music/pubspec.yaml` dev dependencies, plus their config blocks.
- **iOS**: `ios/Runner/Assets.xcassets/AppIcon.appiconset/*`, `ios/Runner/Base.lproj/LaunchScreen.storyboard` + launch assets, new `ios/Runner/PrivacyInfo.xcprivacy` (must be added to the Runner target/bundle).
- **Android**: `android/app/src/main/res/mipmap-*/`, new `mipmap-anydpi-v26/ic_launcher.xml` + adaptive foreground/background resources, `android/app/src/main/AndroidManifest.xml`.
- **Source branding asset**: a new logo/icon source image checked into the repo (e.g. `assets/branding/`).
- **Store assets**: new directory (e.g. `apps/music/store/`) holding screenshots, Play icon, feature graphic, and listing copy.
- **No app runtime/logic changes** and no changes to the existing signing or `release-build.yml` CI pipeline.
