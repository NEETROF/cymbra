## 1. Android release blocker (do first — ships broken without it)

- [x] 1.1 Add `<uses-permission android:name="android.permission.INTERNET"/>` to `apps/music/android/app/src/main/AndroidManifest.xml`
- [x] 1.2 Verify the merged release manifest contains INTERNET (confirmed in `build/app/intermediates/merged_manifests/release/.../AndroidManifest.xml`: `uses-permission android.permission.INTERNET` present and `screenOrientation="sensorLandscape"` preserved)
- [x] 1.3 Smoke-test a release/profile build reaching `api.cymbra.app:443` (gRPC call succeeds, no `SecurityException`)
      (Verified 2026-08-15 on the Pixel 3a API 34 emulator with the **release** APK built
      `--dart-define-from-file=config/prod.json`. `aapt2 dump permissions` on the built APK lists
      `android.permission.INTERNET`. On a clean cold start the app uid (10196) holds ESTABLISHED
      sockets to `149.202.227.119:443` = `api.cymbra.app` — package:grpc's `ClientChannel` dials
      lazily on the first RPC, so the connection existing at all proves an RPC was dispatched and
      TLS completed. No `SecurityException` / permission denial in logcat.
      NOTE: the anonymous Score Library is bundled content — verified by `pm clear` + network off,
      which still renders it — so it is NOT a network indicator; the socket evidence is the proof.)

## 2. App icons

- [x] 2.1 Add the source icon asset (≥1024×1024, no alpha) and adaptive foreground/background assets under `apps/music/assets/branding/`
- [x] 2.2 Add `flutter_launcher_icons` dev dependency and its config block to `apps/music/pubspec.yaml` (`remove_alpha_ios: true`, `adaptive_icon_background`, `adaptive_icon_foreground`)
- [x] 2.3 Run the generator (`dart run flutter_launcher_icons`) and review the full diff
- [x] 2.4 Confirm no default Flutter logo remains in `ios/Runner/Assets.xcassets/AppIcon.appiconset/` or `android/app/src/main/res/mipmap-*/`, and `mipmap-anydpi-v26/ic_launcher.xml` exists
      (NOTE: v0.14.4 wrote empty adaptive foreground drawables — regenerated them by hand from the isolated mark; verified via composited render, incl. circle mask.)
- [x] 2.5 Verify the icon on an iOS device/simulator and an Android device/emulator (Android 8.0+ adaptive rendering)
      (Verified 2026-08-15. Android: release APK on the Pixel 3a API 34 emulator — the launcher
      renders the adaptive icon circle-masked, navy background + mark, labelled "Cymbra …".
      iOS: simulator build on iPhone 15 Pro (17.5) — home screen shows the opaque navy squircle
      with the mark, labelled "Cymbra Music". No stock Flutter logo on either platform.)

## 3. Splash screen

- [x] 3.1 Add the splash source asset(s) (light + dark) under `apps/music/assets/branding/`
- [x] 3.2 Add `flutter_native_splash` dev dependency and config block to `apps/music/pubspec.yaml`
- [x] 3.3 Run `dart run flutter_native_splash:create` and review the diff
- [x] 3.4 Confirm iOS `Info.plist` orientation/`UIRequiresFullScreen` and the landscape config are unchanged after generation
      (NOTE: the generators reserialized Info.plist AND AndroidManifest.xml, and the latter stripped `android:screenOrientation="sensorLandscape"` — both restored from HEAD; only the intended `UIStatusBarHidden` key was re-added by hand.)
- [x] 3.5 Verify branded launch screen on cold start (iOS + Android, light + dark)
      (Verified 2026-08-15 on cold starts in all four combinations. Android (release APK, emulator):
      light and dark both render the navy `#0F1633` splash with the full mark, no white flash —
      dark captured via `screenrecord` because a warm process outruns `screencap`. iOS (simulator,
      `simctl ui appearance light|dark`): same navy + mark in both. The config has no
      `color_dark`/`image_dark`, so light and dark are intentionally identical.)

## 4. Apple privacy manifest

- [x] 4.1 Enumerate required-reason APIs and collected data types from dependencies (`flutter_secure_storage` → Keychain; `shared_preferences` → UserDefaults `CA92.1`; Sign in with Apple/Google account data)
- [x] 4.2 Author `apps/music/ios/Runner/PrivacyInfo.xcprivacy` with `NSPrivacyAccessedAPITypes` and `NSPrivacyCollectedDataTypes`
- [x] 4.3 Add the manifest to the Runner target's bundle resources in the Xcode project
      (pbxproj: file reference + Runner group + Resources build phase; `plutil -lint` passes.)
- [x] 4.4 Build the IPA and verify `PrivacyInfo.xcprivacy` is present inside the bundle
      (NSPrivacyCollectedDataTypes confirmed accurate against backend data handling.
      Verified 2026-08-15: `flutter build ios --release` → `build/ios/iphoneos/Runner.app/PrivacyInfo.xcprivacy`
      at the bundle root — the IPA payload — `plutil -lint` OK and byte-identical to
      `ios/Runner/PrivacyInfo.xcprivacy`. Plugin manifests ship alongside it
      (`firebase_messaging_Privacy.bundle`, `google_sign_in_ios_privacy.bundle`).)

## 5. Store-listing assets

- [x] 5.1 Create `apps/music/store/` layout (per platform, screenshots + graphics)
- [x] 5.2 Capture iOS landscape screenshots for 6.7" iPhone and 12.9" iPad at required resolutions (iPhone 15 Pro Max 2796×1290 + iPad Pro 12.9" 2732×2048, in `store/ios/`)
- [x] 5.3 Capture ≥2 Android phone landscape screenshots (3 at 2160×1080 on the Pixel 3a emulator, in `store/android/phone/`)
- [x] 5.4 Produce the Play 512×512 hi-res icon and the 1024×500 feature graphic
- [x] 5.5 Confirm every asset meets its store's size/format rules (iOS 2796×1290 & 2732×2048; Android cropped to exactly 2:1 for Play; Play icon 512² opaque; feature graphic 1024×500)

## 6. Store-listing copy

- [x] 6.1 Write app description and subtitle/short promo text into `apps/music/store/` (en, fr, it, es in `store/copy/` — all four shipping locales)
- [x] 6.2 Write iOS keywords and pick iOS primary/secondary categories (Education / Music)
- [x] 6.3 Pick the Google Play category and short description (Education)
- [x] 6.4 Review copy for both stores' character limits and content policies (all constrained fields verified within limits)

## 7. Validation

- [x] 7.1 `openspec validate prepare-store-distribution --strict` passes
- [x] 7.2 `melos run analyze` and `dart format` clean after pubspec/asset changes (`flutter analyze` → No issues found; `dart format` → 0 changed)
- [ ] 7.3 Full release build of AAB + IPA succeeds locally (or via `release-build.yml` dispatch) with the new assets
      (SUPERSEDED NOTE: the old dev-dependency plugin-registrant failure was a local Flutter 3.38.10
      artifact. Local Flutter is now **3.41.8** and the release build works — that blocker is gone.
      Prerequisite discovered 2026-08-15: this worktree had **stale gRPC stubs** (missing the streak
      RPCs), which fails the kernel snapshot; fix with `melos run gen-grpc` + `build_runner` first.
      STATUS 2026-08-15, all with `--dart-define-from-file=config/prod.json`:
      • Android AAB — `✓ Built build/app/outputs/bundle/release/app-release.aab (102.0MB)`
      • Android APK — `✓ Built build/app/outputs/flutter-apk/app-release.apk (76.8MB)`
      • iOS archive — `✓ Built build/ios/archive/Runner.xcarchive (291.4MB)`, codesigned
        (TeamIdentifier VMFJ6KRW77), version 1.21.0 build 28, `PrivacyInfo.xcprivacy` in the payload.
      • iOS IPA export — **BLOCKED locally**: `exportArchive No Accounts` /
        `No profiles for 'com.cymbra.music' were found`. This Mac's Xcode has no App Store Connect
        account signed in, so automatic signing resolves only an *Apple Development* identity and
        cannot fetch the App Store distribution profile. Environment/credentials limitation — NOT a
        defect in this change; the compile/archive of the new assets all succeed. Close this task by
        dispatching `release-build.yml` (which holds the distribution credentials) or by exporting
        once from Xcode with the account signed in.)

## 8. Desktop window title (branding)

- [x] 8.1 Set the desktop window title to "Cymbra Music" (was the default "music"): linux `my_application.cc` (header bar + window title), windows `main.cpp` window title + `Runner.rc` ProductName/FileDescription, macos window title in `MainFlutterWindow.swift` + `CFBundleName` + `MainMenu.xib` About/Hide/Quit items
- [x] 8.2 Verify macOS shows "Cymbra Music" in the title bar and menu bar (built + launched `-d macos`; confirmed on screen). Windows/Linux are code-only edits, to be confirmed at their next build.
