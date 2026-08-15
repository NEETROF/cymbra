## 1. Capture seam

- [ ] 1.1 Wire `integrationDriver(onScreenshot:)` in `apps/music/test_driver/integration_test.dart` so screenshot bytes are written to a path derived from the run's platform/class/locale and the shot name (today it is a bare `integrationDriver()`).
- [ ] 1.2 Add a declared manifest (Dart constant + a machine-readable file) holding, per target, the required landscape pixel dimensions and the ordered surface list — the single place a future Apple class change is edited (D5).
- [ ] 1.3 Add a `melos run screenshots` script taking a target and a locale, delegating to `flutter drive --driver=test_driver/integration_test.dart` against the capture scenario.

## 2. Seeded capture state

- [ ] 2.1 Create `integration_test/capture_test.dart` booting the real app inside a `ProviderScope`, reusing the override set proven in `app_test.dart` (fixture score, in-memory preferences, fake connectivity).
- [ ] 2.2 Override `deviceLocaleProvider` from a `--dart-define` locale argument so `AppLocale.build()` resolves to the run's language (D2).
- [ ] 2.3 Ensure the capture run is signed out or overrides `accountServiceProvider`, so `AppLocale`'s restore never pushes `SetLocale` to a real account (D2 consequence).
- [ ] 2.4 Add a fake `midiServiceProvider` reporting a connected, plausibly-named device so no "no MIDI device" indicator can appear (D4).
- [ ] 2.5 Script a note stream against the fixture score so the player shots show a plausible non-zero accuracy rather than 0% (D4).
- [ ] 2.6 Seed fixtures for the non-player surfaces in contention — library, courses, leaderboard, badges/shop, measure selection — so none renders an empty state.
- [ ] 2.7 Assert in-test that the seeded state actually took effect (device connected, accuracy non-zero, lists non-empty) so a silently-failed override fails the run instead of producing a bad image.

## 3. Navigation and surface coverage

- [ ] 3.1 Resolve the open question from design: pick the five surfaces per set, keeping library / falling-notes player / staff and choosing among courses, leaderboard, badges-shop and measure selection.
- [ ] 3.2 Drive the app through the chosen surfaces, settling animations before each shot.
- [ ] 3.3 Ensure playback is not left running between shots — the July pass lost captures to the end-of-session summary covering the view (`store/README.md`).
- [ ] 3.4 Record the covered surfaces in the manifest so a shipped-but-uncaptured feature is auditable without opening every image.

## 4. Per-platform capture

- [ ] 4.1 macOS: size the window from the harness with `isRestorable` off for the run, and confirm no edit to `macos/Runner/MainFlutterWindow.swift` is needed (D8).
- [ ] 4.2 iPhone: capture on an iPhone 17 Pro Max simulator and confirm the output is 2868×1320 landscape (6.9").
- [ ] 4.3 iPad: capture on an iPad Pro 13" simulator and confirm the output is 2752×2064 landscape (13").
- [ ] 4.4 Android: capture on an emulator profile giving 1920×1080 landscape (16:9), with demo mode on for a clean status bar.
- [ ] 4.5 Flatten alpha on write — Apple rejects screenshots carrying transparency.

## 5. Verification gate

- [ ] 5.1 Fail the capture run when any produced image mismatches the manifest dimensions, instead of writing an off-size file.
- [ ] 5.2 Add a standalone check (PNG header read, no simulator) asserting every committed asset matches the manifest in dimensions and carries no alpha.
- [ ] 5.3 Wire that check into the existing Flutter CI job.

## 6. Regenerate the assets

- [ ] 6.1 Restructure `apps/music/store/` to `<platform>/<class>/<locale>/NN_<surface>.png` (D6).
- [ ] 6.2 Regenerate all four targets in `en` and compare against the current English sets to confirm the harness reproduces known-good framing (migration step 2).
- [ ] 6.3 Regenerate `fr`, `it` and `es`.
- [ ] 6.4 Delete the superseded `ios/iphone_6.7/`, `ios/ipad_12.9/`, `android/phone/` and flat `macos/` directories in the same commit as their replacements.
- [ ] 6.5 Check the resulting repository weight; if PNG proves heavy, switch the committed assets to JPEG (accepted by both stores) as the design allows.

## 7. Documentation and spec alignment

- [ ] 7.1 Rewrite `apps/music/store/README.md`: locale level in the layout, the command replacing the manual reproduction instructions, and the covered-surface list.
- [ ] 7.2 Delete the README's "Known caveat on the macOS captures" section — the MIDI/0% problem is structurally fixed by 2.4/2.5, not a standing warning.
- [ ] 7.3 Update the `store-distribution` spec's "Store-listing assets" requirement, which names the retired 6.7"/12.9" classes: edit in place if `prepare-store-distribution` is still in flight, or ship a MODIFIED delta here once it is archived.

## 8. Gates

- [ ] 8.1 `cd apps/music && dart run build_runner build --delete-conflicting-outputs`, then `melos run analyze`, `dart format`, and `dart run custom_lint` clean.
- [ ] 8.2 `flutter test --coverage --exclude-tags golden` still passes and the coverage gate holds (the capture scenario is integration-only and must not regress unit coverage).
- [ ] 8.3 `melos run integration` still passes — the capture scenario must not disturb the existing e2e gate.
- [ ] 8.4 Manual: run one full locale sweep per platform and eyeball the 20 images for framing, language and populated content before they go near a listing.
- [ ] 8.5 `openspec validate add-store-screenshot-harness --strict` passes.
