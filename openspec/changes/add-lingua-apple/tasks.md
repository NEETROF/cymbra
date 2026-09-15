# Tasks — add-lingua-apple

## 1. Safari build variant

- [x] 1.1 `build.mjs`: `safari` target derived from the firefox manifest (`background: { scripts, persistent: false }`, no `browser_specific_settings`, no `sidePanel`/`identity` permissions, static content script) → `dist-safari/`; `yarn build:safari`
- [x] 1.2 Replace the `__TARGET__ === "firefox"` checks with named capability predicates true for firefox and safari (event-page engine in `create-port.ts`, in-page drawer routing in `content.ts` and `popup.ts`, static injection in `background.ts`, shortcut targets)
- [x] 1.3 Icon set in the base manifest (every variant), sized for the toolbar, the extension list and the converter's app icon
- [x] 1.4 `lingua-extension-check` builds the safari variant and greps the bundle for the expected per-target code

## 2. Reading on long pages (every variant)

- [x] 2.1 Viewport-windowed painting in `reading/highlight.ts` (`IntersectionObserver` on block containers, `rootMargin: "100% 0px"`, rAF-batched repaint, full paint without the observer) and `ResolvedToken.container` in `reading/scan.ts`; unit tests for window membership, repaint on intersection change and the fallback
- [x] 2.2 Manual check on Safari macOS: long Wikipedia article with no declared level — no main-thread stall on click, highlights follow a fast scroll

## 3. Touch-primary devices (every variant)

- [x] 3.1 `state/platform.ts` `hasShortcutEditor()` (`pointer: coarse`); hide the shortcut-editor link in the popup and in `mountSettings`; tests
- [x] 3.2 Full-width popup under `(pointer: coarse)`

## 4. First run and sign-in on Safari

- [x] 4.1 With no declared level, the in-page pastille offers the level choice (the existing settings module in the drawer)
- [x] 4.2 Provider buttons that need `identity.launchWebAuthFlow` are shown only where it exists (feature detection, aligned with `add-lingua-account-parity`); email/password on Safari
- [x] 4.3 iOS 27 sheet title: try an empty `action.default_title` in the safari manifest; otherwise document the known issue and file Apple feedback

## 5. Apple container app

- [x] 5.1 Scaffold `apps/lingua-apple` with `safari-web-extension-converter` (universal, Swift, bundle `com.cymbra.lingua`), commit the project, and point its extension resources at `apps/lingua-extension/dist-safari`
- [x] 5.2 Activation guide (French copy): iOS steps (Settings → Apps → Safari → Extensions, allow websites, the address-bar menu, reload open tabs); macOS button → `SFSafariApplication.showPreferencesForExtension` and enabled state via `SFSafariExtensionManager`
- [ ] 5.3 Signing + TestFlight lane cloned from the `ios` and `macos` jobs of `music-release` (App IDs `com.cymbra.lingua` and `.Extension`)
- [x] 5.4 `lingua-apple-build` workflow (macOS runner: extension `safari` build, then unsigned `xcodebuild` for the iOS simulator and macOS); add `apps/lingua-apple` to the `ci-units` filter

## 6. Validation and release

- [ ] 6.1 Manual pass — Safari macOS (unsigned, then signed), iOS simulator, iPhone via TestFlight: activation → level → reading → "+ Deck" → review in the drawer → two minutes in another app → an action → reload
- [ ] 6.2 Same pass on the oldest iOS device available (memory headroom)
- [ ] 6.3 App Store submission (one universal iOS + macOS listing)
- [ ] 6.4 Remove the spike App IDs `com.cymbra.lingua.spike` and `.Extension` from the developer portal
