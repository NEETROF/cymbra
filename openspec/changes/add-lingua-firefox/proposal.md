# add-lingua-firefox — Firefox port (desktop + Android)

## Why

The Chromium extension shipped by the upstream changes covers the founder's daily
dogfooding (Chrome on macOS), but Firefox is the only Android browser with an open
store (AMO): the same MV3 zip ships the extension on desktop **and** on mobile,
almost for free. This change is also the one that introduces the **manifest-variant
system** — the extension becomes a multi-browser artefact built from a single
source, the foundation the `safari` variant plugs into at the next change
(`add-lingua-apple`).

**Position in the stack** (12 changes): 8th, after `add-lingua-extension-review`.
Explicit prerequisites: `add-lingua-extension-review` (the complete Chromium
extension — reading + review) and, transitively, `add-lingua-wasm` (the
`AnalyzerPort` and the WASM target the Firefox variant rewires).

## What Changes

- The `apps/lingua-extension` build now produces **two manifest variants from the
  same source**: `chromium` (existing) and `firefox` (new). What differs stays
  confined behind the existing seams (`AnalyzerPort`, panel surface).
- **Firefox variant**: an event page (`background.scripts` declared alongside
  `service_worker`), **WASM loaded in the event page** behind the `AnalyzerPort`
  (Firefox's CSP blocks WASM in a content script — day-one spike), optional host
  permissions requested at install (prompt), the panel via `sidebar_action` (the same
  page as the side panel).
- **AMO publication**: desktop + Android, the same zip.

## Capabilities

### New Capabilities
_None._

### Modified Capabilities
- `lingua-browser-extension`: adds the "Firefox variant" requirement — the
  multi-browser promise (single source, build variants) enters the spec with its
  first variant; existing Chromium behaviour is unchanged.

## Impact

- **Products**: Lingua only; no existing app, no backend crate, no proto touched.
- **Tree**: `apps/lingua-extension` (multi-target `chromium`/`firefox` builds); no
  new unit — the `ci-units` filter is unchanged.
- **CI**: the extension lane now builds both manifest variants; the manual Firefox
  desktop + Android pass is documented (`web-ext run` / adb).
- **Stores**: AMO (desktop + Android, the same zip). "Tier 3" channels stay
  unsupported (formalised in `add-lingua-apple`, with the full matrix).
- **New dependencies**: the `web-ext` tooling (dev + AMO publication).
