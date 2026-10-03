## 1. The preference

- [x] 1.1 `src/state/storage.ts`: `ReaderTurn` (`instant` | `slide`), `ReaderDisplay.turn`, `instant` in `DEFAULT_READER_DISPLAY`; `readerDisplayOf` reads anything but `slide` as `instant` (design D1)
- [x] 1.2 `test/storage.spec.ts`: a sliding turn kept, an unknown one read as instant, older displays read as instant

## 2. The panel and the adapter

- [x] 2.1 `src/reading/book-display-view.ts`: a "Tourne des pages" row, Directe / Glissée, beside "Thème" — one segmented builder for both rows
- [x] 2.2 `src/reader/foliate.ts`: `applyDisplay` toggles foliate's `animated` on the paginator for `slide`, and never under `prefers-reduced-motion: reduce` (design D2, D3)
- [x] 2.3 `test/book-display-view.spec.ts`: Directe by default, Glissée saved keeping the size and the page
- [x] 2.4 Lint, format, typecheck, tests and coverage of `apps/lingua-extension` green

## 3. On the devices

- [x] 3.1 Chromium (built extension, Chrome for Testing): Glissée slides a page turned by tap; Directe jumps
  - 2026-10-03, by hand (Guillaume): OK.
- [ ] 3.2 A touch device (Tab S6 Lite or iPad): the page follows the finger on a swipe with Glissée; a selection's handle drag on Safari still selects (`touch-guard.ts`)
  - 2026-10-03, Boox Go 10.3 Lumi, Firefox 157, temporary add-on built from this branch: OK by hand (Guillaume). Safari's selection drag is still to see.
- [x] 3.3 The Boox: Directe unchanged, one refresh per turn
  - 2026-10-03, Boox Go 10.3 Lumi, Firefox 157, temporary add-on built from this branch: OK by hand (Guillaume).

## 4. Release

- [ ] 4.1 Shipped with the extension release of `add-lingua-reader` (its task 7.4) — not before Guillaume's go-ahead
