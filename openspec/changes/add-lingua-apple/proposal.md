# add-lingua-apple — Safari extension (macOS + iOS) in a minimal Apple container app

## Why

On iOS, Safari is the only route to a browser extension (Apple's rule): Firefox and
Chrome for iOS run no extensions, so Lingua simply does not exist on an iPhone today.
Safari on macOS comes with the same port.

The previous version of this change assumed Safari could not run the engine and planned
native analysis over nativeMessaging plus a SwiftUI decks app. A spike on 2026-09-14
measured the port instead, and overturned that premise: the **unchanged WASM engine**
runs in Safari's non-persistent event page — the path the Firefox variant already takes
— on macOS, on the iOS 26.5 simulator and on an iPhone under iOS 27. A long Wikipedia
article analyses in ~150 ms, and after two minutes in another app an action that wakes
the engine completes with no perceptible latency. The Safari variant is therefore the
Firefox build plus a thin host app, not a second engine.

The spike also surfaced what does not carry over as-is: WebKit stalls the page for
seconds when ~15 000 highlight ranges are registered; the `identity` permission is not
supported; the manifest has no icons (Safari shows none); no first-run page opens, so an
uncalibrated reader sees every word highlighted; the popup's "configure shortcuts" link
lands on an error page on touch devices, and the popup keeps a narrow desktop column.

**Position in the stack**: after `add-lingua-firefox` (archived), whose manifest-variant
system and event-page `AnalyzerPort` this change reuses.

## What Changes

- **A `safari` build variant** of `apps/lingua-extension`, derived from the firefox
  variant: non-persistent event page hosting the WASM engine behind the `AnalyzerPort`,
  static content script, the in-page drawer as the only review surface, no `sidePanel`
  or `identity` permission, and an icon set (added for every variant).
- **Viewport-windowed highlighting** (every variant): only the blocks within about one
  viewport of the visible area are painted, following the scroll.
- **Touch-primary adaptations** (every variant): full-width popup, no link to a
  keyboard-shortcut editor that does not exist there.
- **First run on Safari**: with no onboarding page, the level choice is offered from
  the page itself.
- **Sign-in on Safari**: the extension's own flow; providers that need
  `identity.launchWebAuthFlow` are offered only where it exists (email/password on
  Safari).
- **A minimal Apple container app** (`apps/lingua-apple`): the converter's Xcode
  project, one universal iOS + macOS listing (bundle `com.cymbra.lingua`), whose only
  job is to host the extension and guide its activation.
- **Signing / TestFlight / CI** for the app, cloned from music's Apple release jobs.
- **Deferred** to a later change, only if real use asks for it: native analysis, a
  SwiftUI decks/review app, an App Group activation heartbeat, native sign-in in the app.
- "Tier 3" channels (Edge Canary Android, curated Edge/Samsung stores, Chromium forks)
  stay explicitly **unsupported**: nothing is promised or tested there.

## Capabilities

### New Capabilities
- `lingua-apple-app`: the minimal Safari container app (iOS + macOS) — hosts the
  `safari` variant under one universal listing and guides activation; no learning
  feature or state of its own.

### Modified Capabilities
- `lingua-browser-extension`: adds the "Safari variant" and "Touch-primary devices"
  requirements; modifies "In-place highlighting without DOM mutation" (per-variant
  engine placement, viewport-windowed painting).

## Impact

- **Products**: **Lingua** only — new `safari` variant and container app. **Cymbra ID**:
  consumed unchanged (the extension's existing sign-in). **Music**: nothing modified;
  its Apple signing jobs are the pattern cloned. **Live / back office / site**: none.
- **Other in-flight changes**: `add-lingua-connected-clients` D3 (native sign-in in the
  Apple app, session shared with the Safari extension through an App Group) no longer
  has an app to live in — the Safari extension signs in and syncs on its own, like the
  other variants. Amend D3 when that change is next touched. `add-lingua-account-parity`
  already detects `identity.launchWebAuthFlow` by feature, which this change relies on.
- **Tree**: `apps/lingua-extension` (build target, highlight painting, touch
  adaptations, first-run level prompt, icons); `apps/lingua-apple` (new unit).
- **CI**: `lingua-extension-check` also builds the `safari` variant; a new
  `lingua-apple-build` workflow builds the Xcode project; `apps/lingua-apple` joins the
  `ci-units` filter.
- **Stores**: App Store, one universal iOS + macOS listing. Safari fixes go through
  App Review, so behaviours are shaken out on Chromium/Firefox first.
- **New dependencies**: none (Xcode's `safari-web-extension-converter` scaffolds the
  project once).
