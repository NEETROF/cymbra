# Tasks — add-lingua-apple

## 1. Apple: container app + Safari extension

- [ ] 1.1 Scaffold `apps/lingua-apple`: `safari-web-extension-converter` on the `safari` variant (the new variant of the multi-target build), **universal purchase** Xcode project (one iOS + macOS listing, bundle `com.cymbra.lingua`)
- [ ] 1.2 Native handler: `SafariWebExtensionHandler` → `lingua-core` FFI (static lib), packs in the app bundle; nativeMessaging `AnalyzerPort` impl (event page, batches, memoisation); graceful degradation tested (bridge cut ⇒ no highlighting, page intact)
- [ ] 1.3 Container app (minimal SwiftUI, Cymbra design language): decks + FSRS review + calibration + LingQ import + settings — the app lives without the extension
- [ ] 1.4 Guided activation: step-by-step iOS walkthrough + App Group heartbeat; macOS `showPreferencesForExtension` deep link + `SFSafariExtensionManager` state; home screen reflecting the state
- [ ] 1.5 Signing/TestFlight: clone music's `release-build` pattern (App ID, profiles, CI lane); dogfooding through internal TestFlight
- [ ] 1.6 Manual Safari pass on macOS (dev mode, then signed) and iOS (TestFlight): activation → calibration → reading → +Deck → review; App Store submission
- [ ] 1.7 `ci-units`: add `apps/lingua-apple` (and the lane that watches it) to the filter
