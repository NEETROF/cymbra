# Design — add-lingua-apple

## Context

The extension ships from one MV3 source as two build variants (`add-lingua-firefox`):
`chromium` runs the WASM engine in the content script; `firefox` runs it in the event
page and reaches it over the `MessagingLinguaPort`, injects the reader statically and
keeps review in the in-page drawer. Safari needs a third variant and, because Apple only
distributes Safari extensions inside an app, a host app.

A spike (2026-09-14) built the firefox variant, converted it with
`safari-web-extension-converter`, and ran it on Safari macOS, the iOS 26.5 simulator
and an iPhone 15 Pro Max under iOS 27, with timing instrumentation in the content
script and the event page. Its measurements drive every decision below.

| Measurement (Wikipedia article, ~1 800 blocks) | Result |
|---|---|
| `analyse` RPC, event page, 144 KB in / 2.3 MB out | 83–172 ms |
| Word click with ~15 300 ranges registered | handled in 2–4 ms, **dispatched 2.4–2.8 s late** |
| Same page after `CSS.highlights.clear()` | dispatched 1–5 ms late |
| Same page, windowed painting (~1 070 ranges) | dispatched 7–9 ms late; "+ Deck" end to end 270 ms |
| iPhone, 2 min in another app, then "+ Deck" and reload | no perceptible latency, page highlighted again |

## Decisions

### D1 — Safari takes the Firefox path; no native analysis

The Safari variant hosts the same WASM engine in its event page and serves the content
script through the existing `MessagingLinguaPort`. The earlier plan — nativeMessaging
to a `SafariWebExtensionHandler` linking `lingua-core` — is dropped: the measurements
show no need for it, and it would add a second engine build, an FFI surface, a native
process with its own (historically very low) memory cap, and an App Review round for
every engine change.

It stays the documented fallback if a device shows the event page being killed for
memory — the iPhone pass, including a suspension, did not.

### D2 — The `safari` variant is derived from the firefox manifest

`build.mjs` gains a `safari` target built like `firefox`, with these manifest
differences: `background` is `{ scripts, persistent: false }` (iOS refuses persistent
background pages); no `browser_specific_settings.gecko`; no `sidePanel` and no
`identity` permission (the converter reports `identity` unsupported); a static
`content_scripts` entry on `<all_urls>`, like Firefox. The icon set is added to the base
manifest, so every variant gets it.

In the source, the checks that today read `__TARGET__ === "firefox"` actually mean one
of a few capabilities: the engine lives in the event page, review opens in the in-page
drawer, the reader is injected statically. They become named predicates that are true
for both `firefox` and `safari`, rather than `firefox || safari` sprinkled at each
call site.

### D3 — Highlights are painted for a viewport window only (every variant)

WebKit re-evaluates every registered highlight range on each rendering update: with
~15 000 ranges the page's main thread blocks for 0.7–5 s at a time, so clicks arrive
seconds late even though our handler takes milliseconds. `render()` keeps the full
resolved token list (click hit-testing is unchanged) but registers only the ranges of
blocks within one viewport above and below the visible area. An `IntersectionObserver`
on the block containers (`rootMargin: "100% 0px"`) maintains that set, and repaints are
batched per animation frame. Without `IntersectionObserver`, everything is painted
as before.

It applies to every variant — one implementation — because Chromium and Firefox also
pay for thousands of live ranges on long pages; they merely hide it better.

The 15 000 figure is the uncalibrated worst case: `lingua-core` starts at calibration 0,
so with no declared level every word is "unknown". A declared level lowers the count
but does not bound it on long pages, so the window stays.

### D4 — The container app is minimal

Apple requires a host app; this change gives it exactly two jobs.

- **Host the `safari` variant.** The Xcode project is the converter's output, committed
  in `apps/lingua-apple`. Its extension resources point at
  `apps/lingua-extension/dist-safari`, built before `xcodebuild`, instead of a copy that
  would drift.
- **Guide activation.** Safari ships extensions disabled.
  - On iOS, which exposes no extension-state API, the app shows the steps (Settings →
    Apps → Safari → Extensions, allow websites) and where the extension lives in Safari
    (the address-bar menu).
  - On macOS, a button calls `SFSafariApplication.showPreferencesForExtension`, and
    `SFSafariExtensionManager` reports the real enabled state.

No decks, review, state or session live in the app: the extension already has all of
them, on every variant.

### D5 — First run and sign-in on Safari

**First run.** Chromium and Firefox open `onboarding.html` on install to pick a level;
Safari does not (an extension is enabled from Settings, not installed in the browser).
When no level is declared, the in-page pastille therefore offers the level choice,
reusing the existing settings module in the drawer. Until then the page is highlighted
at calibration 0, exactly as today.

**Sign-in.** The Safari extension signs in with its own flow, like the other variants.
Buttons for providers that need `identity.launchWebAuthFlow` are shown only where that
API exists — a feature detection, not a `__TARGET__` check, consistent with
`add-lingua-account-parity`. On Safari that leaves email/password. This supersedes the
App Group session sharing that `add-lingua-connected-clients` D3 planned around a
native app sign-in.

### D6 — Touch-primary devices (every variant)

`(pointer: coarse)` identifies Safari on iPhone/iPad and Firefox for Android. There the
popup uses the full width instead of the 280 px desktop column, and the "configure the
browser's shortcuts" link is hidden: its targets (`about:addons`,
`chrome://extensions/shortcuts`) are error pages, and there is no keyboard to use the
shortcuts anyway.

## Risks / Trade-offs

- **iOS memory / suspension on other devices.** Measured fine on one recent iPhone;
  older or smaller devices are unverified. → Manual pass on the oldest device at hand
  before submission; D1's native fallback stays documented.
- **Unreadable sheet title on iOS 27.** Safari draws the popup sheet's native title in
  black over the dark popup under light mode; iOS 26.5 draws it white. Three page-side
  levers had no effect on iOS 27: a `theme-color` meta, an empty `<title>`, and a
  canvas following the system `color-scheme`. → Try an empty `action.default_title` in
  the safari manifest; otherwise ship it as a known issue and report it to Apple.
- **App Review, guideline 4.2 (minimum functionality).** A host app that only explains
  activation can be judged thin. → A genuine activation guide with state detection on
  macOS; the deferred SwiftUI features are the answer if review insists.
- **Tabs open before an app update** lose their content script until reloaded (seen on
  the simulator). → Accepted; mentioned in the activation guide.
- **Fix cadence.** Every Safari fix goes through App Review. → Behaviours are shaken
  out on Chromium/Firefox first.
- **Duplicated settings UI.** `popup.html` carries its own settings markup next to
  `mountSettings` (drawer / side panel), and their copy has already diverged. This
  change touches both for the shortcut link but does not merge them; that is a
  follow-up.
