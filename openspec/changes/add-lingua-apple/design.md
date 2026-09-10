# Design — add-lingua-apple

## Context

The stack has shipped the complete Chromium extension (`add-lingua-extension-reading`
+ `add-lingua-extension-review`, injected drawer included) and the build-variant
system together with its Firefox variant (`add-lingua-firefox`). This change adds the
third variant — `safari` — and the Apple container app that hosts it. Decisions
inherited by reference: the `AnalyzerPort` and the WASM target (`add-lingua-wasm`),
the review surfaces and the injected drawer (`add-lingua-extension-review`), the
manifest variants (`add-lingua-firefox`), and the versioned storage schema
(`add-lingua-extension-reading`).

## Decisions

### D1 — The Safari variant joins the build matrix (the Safari half of D12 in the source design)

The `safari` variant joins the multi-target build introduced by
`add-lingua-firefox`: the build now produces chromium / firefox / safari from the
same source. What differs stays confined behind the two existing seams: the
**`AnalyzerPort`** — Safari means **no WASM at all**: nativeMessaging to the
container app's native handler, which links `lingua-core` compiled for ARM — and the
**panel surface** — the injected drawer alone on Safari, which has no panel API.
Tier-3 channels get neither a test nor a promise.

### D2 — Apple container app: one listing, two OSes, analysis in native code (D13 in the source design)

One Xcode project (`apps/lingua-apple`), **one universal iOS + macOS App Store
listing** (universal purchase). The app is not a shell (guideline 4.4): it hosts
decks/review and the activation flow — which is indispensable, because the extension
arrives **disabled**: on iOS, a step-by-step walkthrough plus activation detected
through an **App Group heartbeat** (iOS exposes no extension-state API); on macOS, a
`SFSafariApplication.showPreferencesForExtension` deep link plus
`SFSafariExtensionManager` for the real state. Analysis goes through the
`SafariWebExtensionHandler` (event page → `sendNativeMessage` → native
`lingua-core`); the **packs live in the app bundle** (working around iOS extension
storage quotas of ~3 MB). With no sync (a later change), each device keeps its own
local state, seeded by calibration or a LingQ import; the schemas share
`lingua-core`'s types, so a later merge is mechanical. Signing/TestFlight: music's
`release-build` pattern cloned (the existing Apple chain); iOS dogfooding through
internal TestFlight (no public review); the cadence of Safari fixes is App Store
review, so behaviours are shaken out on Chromium first.

## Risks / Trade-offs

- [Safari fragility (killed service workers, storage quotas, Apple review on every
  fix)] → event pages everywhere on Apple, analysis and packs on the native side,
  behaviours shaken out on Chromium before they are frozen on Safari; the September
  tax (a new Apple OS) is budgeted.
- [The iOS activation funnel (extension disabled by default, no help from the system)]
  → an animated walkthrough + the App Group heartbeat; this is the documented #1
  drop-off of the category, treated as a product screen in its own right, not a
  README.
- [nativeMessaging bridge failure] → graceful degradation (no highlighting), the page
  intact — explicitly tested (task 1.2).
- [Four browsers for a solo dev] → a single artefact + seams; the implementation
  order stays Chromium → Firefox → Apple; a manual pass matrix per browser before
  each release.
