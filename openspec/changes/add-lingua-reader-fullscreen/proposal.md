# add-lingua-reader-fullscreen — the book reader can take the whole screen

## Why

The reader page (`add-lingua-reader`) opens in a browser tab, under the browser's address bar,
tab strip and toolbars. On a tablet or an e-ink reader these bars take a large share of a small
screen, and they are the one thing a reading app never shows. The need is not a separate web
app (a PWA was weighed and rejected in `add-lingua-reader` D1, for reasons that still hold); it
is only to hide the browser's bars while a book is open. The browser's Fullscreen API does
exactly that, from the extension page as it is.

## What Changes

- A **"Plein écran" button in the reading toolbar** puts the reader page in fullscreen; the
  same button leaves it. The browser's own ways out (Escape, Safari's close control, Android's
  back gesture) are followed: the button always says what pressing it will do.
- The button is **absent where the browser cannot put a page in fullscreen** — Safari on
  iPhone and iPad, which expose no Fullscreen API to the extension's page (measured on iPadOS
  26). Nothing is shown there that cannot work.
- Everything the reader shows stays **on screen in fullscreen**: the toolbar, the word popup,
  the table of contents, the "Aa" panel, the in-page drawer (Firefox, Safari). On Chromium,
  where Réviser / Stats / Réglages open the browser's side panel — which fullscreen hides —
  the page **leaves fullscreen first**, so the panel is seen.
- On a screen with rounded corners or a home indicator, the toolbar and the footer stay
  inside the **safe area** once the browser's bars are gone.
- Fullscreen is **never entered by itself**: browsers only grant it to a gesture, so it is not
  remembered or restored when a book is opened.

## Capabilities

### New Capabilities

_None._

### Modified Capabilities

- `lingua-reader`: a requirement is added — the reader can hide the browser's bars. The
  capability is created by `add-lingua-reader`, still open; this change archives after it
  (`archiveAfter` in `.openspec.yaml`).

## Impact

- **Products**: Cymbra Lingua only — the browser extension (`apps/lingua-extension`), on every
  variant (Chromium, Firefox, Safari), and therefore the Safari app that hosts it. ID, Music,
  Live, back office and site are untouched; nothing is consumed from `id-*` or `platform-*`.
- **Code**: `src/reader/` — a small fullscreen seam (`fullscreen.ts`), the toolbar button in
  `app.ts`, its words in `copy.ts`, safe-area padding in `reader.css`, `viewport-fit=cover` in
  `reader.html`, the wiring in `reader.ts`. Tests in `test/`.
- **No** manifest permission, no storage key, no message type, no backend, no `.proto`.
