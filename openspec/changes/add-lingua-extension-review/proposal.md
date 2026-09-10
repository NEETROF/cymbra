# add-lingua-extension-review — Cymbra Lingua: review inside the extension

## Why

`add-lingua-extension-reading` creates cards ("+ Deck", phrase capture) but offers no way
to review them in the browser: FSRS scheduling exists in `lingua-core`
(`add-lingua-decks-review`) with no UI surface. This change closes the loop reading → deck
→ review → export, right where the user already reads — no back office, no separate app.
The UX decision is settled: the page is **pushed**, not covered (Side Panel API), with an
injected drawer for micro-reviews.

**Position in the stack (12 changes): 7th.** Direct prerequisite:
`add-lingua-extension-reading` (which pulls `add-lingua-decks-review`,
`add-lingua-data-pack`, `add-lingua-wasm`). The later changes (`add-lingua-firefox`,
`add-lingua-apple`) will port these surfaces: the Firefox sidebar is the same page as the
side panel; Safari, having no panel API, will lean on the drawer alone — the requirement
allows for that from now.

## What Changes

- **Side panel** (Side Panel API, the `sidePanel` permission added to the manifest): deck,
  FSRS review session (answer hidden/revealed), due-card counter — the page is pushed and
  the panel survives navigation.
- **A collapsible injected drawer** (closed shadow DOM) sharing the same review logic, for
  micro-reviews without leaving the page.
- **Backup/restore from the side panel**: the lossless backup from `lingua-decks-review`
  (file download + re-import), made reachable in the browser — the safety net of the
  local-only phase.
- **Versioned storage completed**: migrations + full reset.
- **An attributions page** (the pack's NOTICE) + a privacy note stating "nothing leaves
  the device".
- The full manual walkthrough documented (calibration → reading → +Deck → review → export)
  on 5 real sites.

## Capabilities

### New Capabilities
_None._

### Modified Capabilities
- `lingua-browser-extension` (created by `add-lingua-extension-reading`): adds the "Two
  review surfaces" requirement — a native side panel plus an injected drawer over the same
  local state, backup/restore reachable from the side panel, and the Safari posture
  (drawer only) anticipated.

## Impact

- **Products**: Lingua only; Cymbra ID / Music / Live / back office untouched.
- **Tree**: `apps/lingua-extension` (side panel / drawer pages, review-session logic,
  export, attributions) — no new unit; the CI lanes from `add-lingua-extension-reading`
  cover everything (vitest, the "lemma"/hex lints).
- **Dependencies**: none new — FSRS and backup/restore come from `lingua-core` through the
  `add-lingua-wasm` bindings.
- **Out of scope**: a "due cards" badge and alarms/notifications (v1: the counter lives in
  the icon popup and the side panel), the Firefox sidebar and the Safari drawer (the
  `add-lingua-firefox` / `add-lingua-apple` changes — they reuse these surfaces), card sync
  (`add-lingua-backend`).
