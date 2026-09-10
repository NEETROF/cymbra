# add-lingua-apple — Apple container app + Safari extension (macOS + iOS)

## Why

On iOS, Safari is the only route to an extension (Apple's rule) — a constraint we
accept — and the extension lands there **disabled** by default: the documented #1
drop-off of the category. The answer is not a README but a container app in its own
right (guideline 4.4): it hosts the converted Safari extension, carries the
decks/review screens, runs the analysis **natively** (no WASM on Apple), and owns the
guided activation flow. This change completes the MVP's browser matrix: the `safari`
variant joins the multi-target build introduced by `add-lingua-firefox`.

**Position in the stack** (12 changes): 9th, after `add-lingua-firefox`. Explicit
prerequisites: `add-lingua-extension-review` (the complete extension — reading +
review, injected drawer included) and `add-lingua-firefox` (the manifest-variant
system the `safari` variant joins).

## What Changes

- **A new Apple container app** (`apps/lingua-apple`): one Xcode project, **one
  universal App Store listing (iOS + macOS)** that hosts the converted Safari
  extension, the **native analysis** (`lingua-core` linked natively over
  nativeMessaging — no WASM on Apple, packs in the bundle), the decks/review screens,
  and the extension's **activation flow** (step-by-step walkthrough on iOS +
  detection through an App Group heartbeat; deep link and state API on macOS).
- **A `safari` variant of the extension build**: a third manifest variant from the
  same source (`safari-web-extension-converter`), analysis over nativeMessaging
  behind the `AnalyzerPort`, and the injected drawer carrying in-browser review on
  its own (Safari has no panel API).
- **Signing/TestFlight**: music's `release-build` pattern cloned (the existing Apple
  chain); iOS dogfooding through internal TestFlight.
- "Tier 3" channels (Edge Canary Android by ID, curated Edge/Samsung stores, Chromium
  forks) are explicitly **unsupported**: the standard build may well run there,
  nothing is promised or tested.

## Capabilities

### New Capabilities
- `lingua-apple-app`: the Safari container app (iOS + macOS) — its own functions
  (decks/review), guided activation of the extension (iOS walkthrough + heartbeat,
  macOS deep link + state API), native analysis over nativeMessaging, packs in the
  bundle, one universal App Store listing.

### Modified Capabilities
- `lingua-browser-extension`: adds the "Safari variant" requirement — the converted
  variant hosted by the container app joins the build matrix
  (chromium/firefox/safari), with the "tier 3 is not promised" limit.

## Impact

- **Products**: Lingua; music's Apple signing chain is **consumed** (the pattern is
  cloned), not modified; no proto, no backend crate touched.
- **Tree**: `apps/lingua-apple` (Xcode project: container app + Safari extension +
  native handler); `apps/lingua-extension` gains the `safari` variant in its
  multi-target build.
- **CI**: an Apple signing lane cloned from music's `release-build` pattern (internal
  TestFlight for iOS dogfooding); `apps/lingua-apple` added to the `ci-units` filter.
- **Stores**: App Store (one universal iOS/macOS listing). Accepted cadence: Safari
  fixes go through Apple review — behaviours are shaken out on Chromium/Firefox
  first.
- **New dependencies**: none on the Rust side; the `safari-web-extension-converter`
  tooling (Xcode) on the build side.
