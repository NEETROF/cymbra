## 1. Android release blocker (do first — ships broken without it)

- [x] 1.1 Add `<uses-permission android:name="android.permission.INTERNET"/>` to `apps/music/android/app/src/main/AndroidManifest.xml`
- [ ] 1.2 Verify the merged release manifest contains INTERNET (`flutter build apk --release` → inspect `build/app/outputs/.../AndroidManifest.xml` or `manifest-merger` report)
- [ ] 1.3 Smoke-test a release/profile build reaching `api.cymbra.app:443` (gRPC call succeeds, no `SecurityException`)

## 2. App icons

- [x] 2.1 Add the source icon asset (≥1024×1024, no alpha) and adaptive foreground/background assets under `apps/music/assets/branding/`
- [x] 2.2 Add `flutter_launcher_icons` dev dependency and its config block to `apps/music/pubspec.yaml` (`remove_alpha_ios: true`, `adaptive_icon_background`, `adaptive_icon_foreground`)
- [x] 2.3 Run the generator (`dart run flutter_launcher_icons`) and review the full diff
- [x] 2.4 Confirm no default Flutter logo remains in `ios/Runner/Assets.xcassets/AppIcon.appiconset/` or `android/app/src/main/res/mipmap-*/`, and `mipmap-anydpi-v26/ic_launcher.xml` exists
      (NOTE: v0.14.4 wrote empty adaptive foreground drawables — regenerated them by hand from the isolated mark; verified via composited render, incl. circle mask.)
- [ ] 2.5 Verify the icon on an iOS device/simulator and an Android device/emulator (Android 8.0+ adaptive rendering)

## 3. Splash screen

- [x] 3.1 Add the splash source asset(s) (light + dark) under `apps/music/assets/branding/`
- [x] 3.2 Add `flutter_native_splash` dev dependency and config block to `apps/music/pubspec.yaml`
- [x] 3.3 Run `dart run flutter_native_splash:create` and review the diff
- [x] 3.4 Confirm iOS `Info.plist` orientation/`UIRequiresFullScreen` and the landscape config are unchanged after generation
      (NOTE: the generators reserialized Info.plist AND AndroidManifest.xml, and the latter stripped `android:screenOrientation="sensorLandscape"` — both restored from HEAD; only the intended `UIStatusBarHidden` key was re-added by hand.)
- [ ] 3.5 Verify branded launch screen on cold start (iOS + Android, light + dark)

## 4. Apple privacy manifest

- [x] 4.1 Enumerate required-reason APIs and collected data types from dependencies (`flutter_secure_storage` → Keychain; `shared_preferences` → UserDefaults `CA92.1`; Sign in with Apple/Google account data)
- [x] 4.2 Author `apps/music/ios/Runner/PrivacyInfo.xcprivacy` with `NSPrivacyAccessedAPITypes` and `NSPrivacyCollectedDataTypes`
- [x] 4.3 Add the manifest to the Runner target's bundle resources in the Xcode project
      (pbxproj: file reference + Runner group + Resources build phase; `plutil -lint` passes.)
- [ ] 4.4 Build the IPA and verify `PrivacyInfo.xcprivacy` is present inside the bundle
      (REVIEW: confirm NSPrivacyCollectedDataTypes matches actual backend data handling before submission.)

## 5. Store-listing assets

- [x] 5.1 Create `apps/music/store/` layout (per platform, screenshots + graphics)
- [x] 5.2 Capture iOS landscape screenshots for 6.7" iPhone and 12.9" iPad at required resolutions (iPhone 15 Pro Max 2796×1290 + iPad Pro 12.9" 2732×2048, in `store/ios/`)
- [ ] 5.3 Capture ≥2 Android phone landscape screenshots
- [x] 5.4 Produce the Play 512×512 hi-res icon and the 1024×500 feature graphic
- [ ] 5.5 Confirm every asset meets its store's size/format rules (512 icon + feature graphic done; screenshots pending)

## 6. Store-listing copy

- [x] 6.1 Write app description and subtitle/short promo text into `apps/music/store/` (en + fr in `store/copy/`; it + es to translate)
- [x] 6.2 Write iOS keywords and pick iOS primary/secondary categories (Education / Music)
- [x] 6.3 Pick the Google Play category and short description (Education)
- [x] 6.4 Review copy for both stores' character limits and content policies (all constrained fields verified within limits)

## 7. Validation

- [x] 7.1 `openspec validate prepare-store-distribution --strict` passes
- [ ] 7.2 `melos run analyze` and `dart format` clean after pubspec/asset changes (no Dart changed by this work — assets/native/plist/pubspec only; run once alongside the build)
- [ ] 7.3 Full release build of AAB + IPA succeeds locally (or via `release-build.yml` dispatch) with the new assets
