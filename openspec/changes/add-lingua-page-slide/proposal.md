# add-lingua-page-slide — book pages can slide as they turn

## Why

The reader page (`add-lingua-reader`) turns a page in one jump, on purpose: an e-ink screen
redraws every frame it is shown, so a single jump is a single refresh (design D4). On a laptop,
a tablet or a phone, the same jump reads as a flicker rather than a page being turned; readers
asked for the feeling of a real book. foliate-js, the engine behind the reader, already knows
how to slide a page aside — the reader simply never turned it on.

## What Changes

- A **"Tourne des pages" choice — Directe / Glissée** — in the "Aa" panel of the reader, and
  therefore in Réglages → Affichage, which renders the same panel.
- **Directe stays the default**: the e-ink device keeps its one refresh per turn.
- **Glissée** slides the page aside in 300 ms, and the page follows the finger on a swipe.
- A system asking for **reduced motion** gets the jump, whatever was chosen.
- The choice is part of the reader's display preference (`cymbra-lingua-reader-display`), kept
  on the device like the text size and the page colour.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-reader`: "A page turn paints once" — the flow has no transition unless the reader
  chooses a sliding turn. The capability is created by `add-lingua-reader`, still open; this
  change archives after it (`archiveAfter` in `.openspec.yaml`).

## Impact

- **Products**: Cymbra Lingua only — the browser extension (`apps/lingua-extension`), every
  variant, and the Safari app that hosts it. ID, Music, Live, back office and site are untouched.
- **Code**: `src/state/storage.ts` (`ReaderTurn`, `ReaderDisplay.turn`), the shared panel
  `src/reading/book-display-view.ts`, the foliate adapter `src/reader/foliate.ts`. Tests in
  `test/`.
- **No** manifest permission, no new storage key, no message type, no backend, no `.proto`.
  A stored display without `turn` reads as `instant`.
