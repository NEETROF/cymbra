# add-lingua-extension-reading — Cymbra Lingua: the extension that reads (Chromium)

## Why

The earlier changes in the stack ship the brain (analysis, knowledge model, decks/FSRS,
the EN→FR pack, the WASM target) — but nothing a user can open. This change ships the
first usable surface: reading instrumentation inside the browser, **on Chromium (Chrome
+ Edge, one build)** — the founder's daily dogfooding, under Chrome on macOS. Product
principle #1 takes shape here: never a silo — you read the web **in place**, highlighted,
with an honest per-lemma percentage.

**Position in the stack (12 changes): 6th.** Direct prerequisites: `add-lingua-decks-review`
(cards created by "+ Deck", the due counter), `add-lingua-data-pack` (offline glosses and
frequencies), `add-lingua-wasm` (analysis bindings) — which in turn pull `add-lingua-analysis`
and `add-lingua-knowledge-model`. The Firefox (`add-lingua-firefox`) and Safari/Apple
(`add-lingua-apple`) variants come later in the stack: this change lays the seam
(`AnalyzerPort`) that makes them possible without touching the content script. The review
surfaces (side panel / drawer) are the next change, `add-lingua-extension-review`.

## What Changes

- **A new MV3 browser extension** (`apps/lingua-extension`) targeting Chromium: unknown
  words highlighted via the CSS Custom Highlight API (zero DOM mutation), a known-words
  percentage per page (badge + icon popup with a calibration slider), a word popup on
  click (dictionary form, gloss, rarity, status actions), multi-word selection capture on
  a keyboard shortcut with the source sentence, and "+ Deck" card creation.
- **`AnalyzerPort`**: the content script consumes analysis exclusively over messages; in
  this change the implementation is the WASM module instantiated in the content script.
- **A minimal permission posture** (`activeTab` + optional `<all_urls>`), **no network
  requests at all**, and all state in `chrome.storage.local` under a versioned schema.
- **Cymbra visual identity**: a `tokens.css` mirroring `CymbraColors`, highlight tints
  derived from the palette's amber and coral, and a lint forbidding any hex outside
  `tokens.css`.
- UI vocabulary: the word "lemma" **never** appears on screen ("dictionary form",
  "distinct words") — enforced by a lint over UI strings.

## Capabilities

### New Capabilities
- `lingua-browser-extension`: the **reading** experience in the browser — in-place
  highlighting, per-page percentage, word popup, selection capture, calibration, Cymbra
  visual identity, permission posture, zero network, versioned local state. The review
  requirements (side panel / drawer) and the multi-browser matrix are added to this
  capability by the later changes in the stack.

### Modified Capabilities
_None._

## Impact

- **Products**: Lingua only; Cymbra ID / Music / Live / back office untouched.
- **Tree**: `apps/lingua-extension` (framework-free TS, wasm-pack, Yarn, vitest, an
  esbuild/vite build) — a new `apps/*` unit, added to the `ci-units` filter with its own
  vitest/lint lane.
- **CI**: a vitest + lint lane (the "lemma" lint, the out-of-tokens hex lint) over
  `apps/lingua-extension`; the WASM build and the parity tests are already covered by
  `add-lingua-wasm`.
- **Distribution**: load unpacked for dogfooding; publication (Chrome Web Store +
  Microsoft Add-ons, same build) once stable — inherited recommendation: unlisted as soon
  as it is stable, to break in the store review. Tier-3 channels (Chromium forks, curated
  stores) stay unsupported.
- **Out of scope**: side panel / drawer and Anki export inside the extension
  (`add-lingua-extension-review`), the Firefox/Safari manifest variants
  (`add-lingua-firefox`, `add-lingua-apple`), sync and accounts (`add-lingua-backend`,
  `add-lingua-connected-clients`).
