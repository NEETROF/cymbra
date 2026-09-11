# Design — add-lingua-extension-review

## Context

Seventh change in the Lingua stack, sitting directly on top of
`add-lingua-extension-reading`: the Chromium extension reads, highlights and creates cards;
this change gives it its review surfaces. The engine is inherited: FSRS state and lossless backup/restore in `lingua-core` (`add-lingua-decks-review`), the bindings (`add-lingua-wasm`), the
identity/tokens, storage and `AnalyzerPort` (`add-lingua-extension-reading`). The preshot
(`~/workspace/lingua-preshot`) had already proven both surfaces (side panel / floating
panel).

## Goals / Non-Goals

**Goals:**
- The full loop in the browser: a browsable deck, an FSRS review session, lossless backup/restore — on
  the same local state as reading.
- Two surfaces: a native side panel by default, an injected drawer for micro-reviews —
  designed to be ported as-is (Firefox sidebar, drawer-only Safari) by the later changes.

**Non-Goals:**
- A "due" badge on the icon, alarms, notifications (v1: the counter lives in the icon popup
  and the side panel).
- Porting the surfaces to Firefox/Safari (`add-lingua-firefox`, `add-lingua-apple`).
- Syncing cards and review history (`add-lingua-backend`).

## Decisions

The algorithm (FSRS, the `again/hard/good/easy` grading, "Je connais" → `known` with
provenance `srs`) and the lossless backup/restore format are decided by `add-lingua-decks-review`; this change only decides the surfaces.

### D1 — Two UI surfaces: Side Panel API by default, injected drawer as the fallback
UI: the side panel (Side Panel API — the page is pushed, the panel survives navigation) by
default; an injected overlay drawer (closed shadow DOM) as the fallback and for
micro-reviews — both share the same session logic and operate on the same
`chrome.storage.local` state. The icon badge stays dedicated to the page percentage; the
due-card counter is visible in the icon popup and the side panel (no "due" badge and no
alarm in v1). This duality is the stack target: Firefox will expose the same page through
`sidebar_action`; Safari, having no panel API, will carry in-browser review on the drawer
alone. Backup/restore: triggered from the side panel, using `lingua-core`'s lossless LinguaState round trip.

## Risks / Trade-offs

- [Two surfaces over one logic = risk of divergence] → the review session is a single
  module (tested with vitest); the side panel and the drawer are only two rendering hosts
  for that module.
- [The drawer lives in hostile pages (aggressive styles, z-index)] → closed shadow DOM +
  embedded tokens (the identity from `add-lingua-extension-reading`), legible on light and
  dark pages alike.
