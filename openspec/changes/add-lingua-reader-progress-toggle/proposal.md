# add-lingua-reader-progress-toggle — the bottom of the book says how much is left, the way the reader asks

## Why

The reader's footer shows how far through the book the page is, as a percentage. On a long book a
percentage moves slowly and says little about the next sitting: a reader deciding whether to finish
a chapter before stopping wants the pages left in it, and sometimes the pages left in the book.
Asked by the product owner on 2026-10-12 while dogfooding the e-ink reader, choosing a tap that goes
through the three in turn.

## What Changes

- **A tap on the footer's figure changes what it shows**, in turn: the percentage through the book,
  then the pages left in the chapter (« Chapitre : 4 pages restantes »), then an estimate of the
  pages left in the book (« Livre : ≈ 120 pages restantes »), then the percentage again.
- **The choice is kept on the device** and applies to every book; the percentage until the reader
  changes it.
- **The pages left in the chapter are exact**: the chapter on screen is laid out at the reader's
  size and on the reader's screen. **The pages left in the book are an estimate**, said so with
  « ≈ »: the rest of the book is not laid out, so its length is extrapolated from the chapter on
  screen, and moves when the text size changes.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-reader` (added by `add-lingua-reader`, not yet archived): added — what the footer shows of
  the progress, chosen by a tap.

This change archives after `add-lingua-reader`, which creates the capability (`archiveAfter` in
`.openspec.yaml`).

## Impact

**Products.**
- **Cymbra Lingua — new behaviour.** The extension's book reader (`apps/lingua-extension`, every
  browser), hence the Apple container app (`apps/lingua-apple`) that ships its bundle.
- Cymbra ID, Music, Live, the back office and the public site are untouched. Nothing is consumed
  from `id-*` or `platform-*`: no flag, no RPC, no `.proto`, no backend change. The core, its WASM
  build and the agent plugin are untouched.

**Code.** `src/reader/renderer.ts` (the location gains the pages left), `src/reader/foliate.ts`
(reads them from the laid-out section), `src/reader/app.ts` (the footer's toggle),
`src/state/storage.ts` (the stored choice), the catalogue's `reader` module in French, English and
Spanish.

**Data.** One preference in `chrome.storage.local`. Nothing leaves the device.
