## 1. Bundle metadata (no Apple account needed)

- [x] 1.1 Add `LSApplicationCategoryType = public.app-category.education` to `apps/music/macos/Runner/Info.plist` (D7)
- [x] 1.2 Add `ITSAppUsesNonExemptEncryption = false` to `apps/music/macos/Runner/Info.plist`, mirroring `ios/Runner/Info.plist` (D7)
- [x] 1.3 `plutil -lint apps/music/macos/Runner/Info.plist` passes

## 2. Entitlements audit (no Apple account needed)

- [x] 2.1 Add `keychain-access-groups = [$(AppIdentifierPrefix)com.cymbra.music]` to `apps/music/macos/Runner/Release.entitlements` (D5)
- [x] 2.2 Add the same key to `apps/music/macos/Runner/DebugProfile.entitlements` so debug and release stay consistent
- [x] 2.3 Diff the two entitlement files and confirm the only remaining differences are debug-only capabilities (`allow-jit`, `network.server`) that the shipped app does not need — in particular confirm `network.server` stays out of Release (D5)
- [x] 2.4 `plutil -lint` both entitlement files
      (Validated end-to-end on a locally signed `flutter build macos --debug`:
      `codesign -d --entitlements -` on the built bundle shows
      `keychain-access-groups = VMFJ6KRW77.com.cymbra.music` — `$(AppIdentifierPrefix)`
      expands and the profile grants it — alongside `app-sandbox`. The app launches
      and creates `~/Library/Containers/com.cymbra.music`. Both new `Info.plist` keys
      are present in the built `Contents/Info.plist`.)

## 3. Local signed archive (needs the Apple account — section 6 first)

- [x] 3.1 Write `apps/music/macos/ExportOptions.plist` for local archives: `method: app-store`, `teamID VMFJ6KRW77`, automatic signing, `installerSigningCertificate` set to the Mac Installer Distribution identity (mirrors `ios/ExportOptions.plist`)
- [ ] 3.2 Produce a `.pkg` locally: `flutter build macos --release --dart-define-from-file=config/prod.json`, then `xcodebuild -workspace macos/Runner.xcworkspace -scheme Runner -configuration Release -destination 'generic/platform=macOS' -archivePath build/macos.xcarchive archive`, then `-exportArchive -exportOptionsPlist macos/ExportOptions.plist` (D1)
- [ ] 3.3 Verify the exported app's signature: `codesign -dv --verbose=4` shows an Apple Distribution identity under team `VMFJ6KRW77`, and `codesign -d --entitlements -` shows `app-sandbox` plus the keychain access group
- [ ] 3.4 Verify no unsigned nested code: `codesign --verify --deep --strict --verbose=2` on the exported `.app` passes over every embedded pod framework (D-risk: `use_frameworks!`)
- [ ] 3.5 Verify the binary is universal (`lipo -archs` shows `x86_64 arm64`), confirming cargokit built both slices of the static Rust library

## 4. CI release job (needs the Apple account for a real run; editable before)

- [x] 4.1 Replace the "Disable code signing" step in the `macos` job of `.github/workflows/release-build.yml` with a secrets presence check that fails with an explicit error naming the missing material (mirrors the `ios` job's "Check iOS signing secrets")
- [x] 4.2 Add a step importing the Apple Distribution cert, the Mac Installer Distribution cert and the Mac App Store provisioning profile into a throwaway keychain, reusing the `ios` job's `security create-keychain` / `set-key-partition-list` sequence (D3)
- [x] 4.3 Append `CODE_SIGN_STYLE = Manual`, `DEVELOPMENT_TEAM`, `PROVISIONING_PROFILE_SPECIFIER`, `CODE_SIGN_IDENTITY = Apple Distribution` to `macos/Runner/Configs/Release.xcconfig` at build time (D2)
- [x] 4.4 Generate `macos/ExportOptions-ci.plist` with `signingStyle: manual` and the bundle-id → profile-name mapping (D3)
- [x] 4.5 Replace the `flutter build macos` + `ditto` steps with the build → archive → export sequence from 3.2, producing `cymbra-${TAG}-macos.pkg`
- [x] 4.6 Drop the `.zip` release asset and do **not** attach the `.pkg` (D4)
- [x] 4.7 Add the delivery step: `xcrun altool --upload-app --type macos` gated on `github.event_name == 'push'`, skipping with a warning when the `ASC_API_*` secrets are absent
- [x] 4.8 Update the workflow header comment (lines 42-43) which currently states the macOS artifact is unsigned
- [x] 4.9 `actionlint` (or a YAML parse) clean on the edited workflow
      (`actionlint` is not installed on this machine; validated with a `yaml.safe_load`
      parse + step-order dump, and by confirming the heredocs de-indent to column 0
      inside the `run: |` block scalars, same as the `ios` job's.)
- [x] 4.10 **Consequence to accept before the next tag**: with 4.1 in place the `macos`
      job now **fails** the release build until the section 6 secrets exist. That is the
      specced behaviour (fail loudly, never silently ship unsigned) and mirrors the `ios`
      job — but unlike iOS, these secrets are not configured yet.

## 5. Documentation

- [x] 5.1 Add a "macOS signing (Mac App Store)" section to `apps/music/README.md` beside the iOS one: Apple-side prerequisites, the six new secrets, and the local command from 3.2
- [x] 5.2 Document in the same section that the release publishes no macOS download until the listing is live, and why (D4)

## 6. Apple-side prerequisites (manual, account holder only)

- [ ] 6.1 Enable the macOS platform on the `com.cymbra.music` App ID, keeping Sign in with Apple
- [x] 6.2 Issue an **Apple Distribution** certificate and export it as `.p12`
      (Already issued and installed: `Apple Distribution: NEETROF (VMFJ6KRW77)`, valid to
      2027-07-10, private key present. It is the multi-platform cert — it signs macOS too.)
- [ ] 6.3 Issue a **Mac Installer Distribution** certificate and export it as `.p12`
- [ ] 6.4 Create a **Mac App Store** provisioning profile for `com.cymbra.music`
- [ ] 6.5 Add the macOS platform to the existing App Store Connect record (Universal Purchase, same bundle id) (D6)
- [ ] 6.6 Set the GitHub secrets: `MAC_INSTALLER_CERT_BASE64`, `MAC_INSTALLER_CERT_PASSWORD`, `MAC_PROVISIONING_PROFILE_BASE64`, `MAC_PROVISIONING_PROFILE_NAME`
      (The app-signing cert is NOT a secret to add: `Apple Distribution: NEETROF (VMFJ6KRW77)`
      is multi-platform and already in `IOS_DIST_CERT_BASE64`, which the macOS job falls
      back to. Verified present in the login keychain with its private key, valid to
      2027-07-10. So 6.2 is already satisfied and only the installer cert is new.)

## 7. Sandbox runtime verification (manual, on the signed build from 3.2)

- [ ] 7.1 Install the exported build and confirm it launches sandboxed (`codesign -d --entitlements -` on the installed bundle, and a container appears under `~/Library/Containers/com.cymbra.music`)
- [ ] 7.2 MIDI: a connected keyboard is detected, its notes drive playback and scoring, and a device plugged in **while the app runs** is picked up (hot-plug via the main-run-loop MIDI client)
- [ ] 7.3 Audio: playing a score produces audible synthesized output
- [ ] 7.4 SoundFont import: pick a `.sf2` through the file picker, confirm it imports and can be selected as the playback instrument
- [ ] 7.5 Sign-in: Google and Apple sign-in both complete; relaunch the app and confirm the session persists (this is the 2.1 keychain fix under real conditions)
- [ ] 7.6 Local storage: preferences, the local database and the encrypted offline score cache all persist across a relaunch, inside the sandbox container

## 8. Listing assets

- [ ] 8.1 Capture at least one macOS screenshot of the real UI at an accepted size (2880×1800 preferred, 1440×900 acceptable) into `apps/music/store/macos/`
- [ ] 8.2 Record the chosen macOS category alongside the existing `apps/music/store/copy/` material, reusing the en/fr/it/es description and promo text rather than duplicating it

## 9. Validation

- [x] 9.1 `openspec validate prepare-macos-app-store --strict` passes
- [x] 9.2 `melos run analyze` and `dart format` clean (no Dart change expected — confirm the tree is untouched)
- [ ] 9.3 Dispatch `release-build.yml` manually and confirm the `macos` job builds, signs and exports without uploading (D4)
- [ ] 9.4 Deliver a build to App Store Connect and confirm it passes processing with no metadata or signing error
