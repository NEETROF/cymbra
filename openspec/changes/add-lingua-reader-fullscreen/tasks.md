## 1. The fullscreen seam

- [x] 1.1 `src/reader/fullscreen.ts`: `FullscreenHost` and `documentFullscreen(doc)` over the standard API with the `webkit` fallbacks (design D2); `available` false when neither `fullscreenEnabled` nor `webkitFullscreenEnabled` is true
- [x] 1.2 `test/reader-fullscreen.spec.ts`: standard and `webkit` documents, a document without the API, a rejected request, change events

## 2. The reader page

- [x] 2.1 `ReaderDeps.fullscreen?` and `ReaderDeps.reviewOutsidePage?`; the toolbar control (⛶, label and title from `COPY`, `aria-pressed`) shown only when `available`, its state set from `onChange` (design D3)
- [x] 2.2 Réviser / Stats / Réglages leave fullscreen first when `reviewOutsidePage` and fullscreen is active, calling the action in the same click (design D4)
- [x] 2.3 `COPY.fullscreen`, `COPY.leaveFullscreen`
- [x] 2.4 `reader.html` `viewport-fit=cover`; `.reading-bar` / `.reading-foot` safe-area padding (design D5)
- [x] 2.5 `reader.ts` wires `documentFullscreen(document)` and `reviewOutsidePage: !__REVIEW_IN_PAGE__`
- [x] 2.6 `test/reader-app.spec.ts`: control absent without the API; enter, leave, leave through the browser; a refused request; review actions leave fullscreen on Chromium only; opening a book does not enter fullscreen
- [x] 2.7 Lint, format, typecheck, tests and coverage of `apps/lingua-extension` green

## 3. On the devices

- [x] 3.1 Chromium (built extension, Playwright): the control enters and leaves fullscreen, the word popup opens in fullscreen
  - 2026-09-30, built `dist-chromium` under Playwright: the control shows in the reading toolbar with its label, and the reader's surfaces hang from `documentElement`. The grant itself cannot be driven there — Chromium under automation answers "not granted" to any page, a plain web page included, once its window is not focused — so it is checked by hand with 3.2.
  - 2026-09-30, by hand (Guillaume): fullscreen entered and left from the control on Firefox desktop, Chrome for Testing and Safari macOS (the macOS app built from this branch).
- [x] 3.2 Headed Chrome: Réviser from fullscreen leaves it and the side panel opens
  - 2026-09-30, Chrome for Testing with the unpacked build, by hand: OK.
- [x] 3.3 Firefox for Android (Tab S6 Lite or the Boox): the control is shown and works in the extension page, or is absent
  - 2026-09-30, Galaxy Tab S6 Lite (SM-P610), Firefox release, temporary add-on built from this branch: the control is shown and works (Guillaume). The Boox is still to see, with the release.
- [x] 3.4 Safari iPad: fullscreen, safe area, word selection and page turns; Safari iPhone: no control
  - 2026-09-30, iPad Air 13" (M2), iPadOS 26, the iOS app built from this branch: **Safari exposes no Fullscreen API to the extension's reader page** — `requestFullscreen`, `webkitRequestFullscreen`, `fullscreenEnabled` and `webkitFullscreenEnabled` are all `undefined` (read by a throwaway diagnostic build). The control is absent, as specified; the reader works as before.
  - Safari's own "Masquer la barre d'outils" (page menu) is not offered on the reader page either, while it is on a website's page: Safari on iPadOS treats the extension's pages as its own, with neither the web's Fullscreen API nor its toolbar hiding (Safari on macOS offers both: the control works, and "Masquer la barre d'outils" is in its menu on the reader page too). On iPad the bars stay; hiding them would need the reader outside an extension page (a site, or the Safari host app's own view) — out of this change.
  - 2026-09-30, iPhone 15 Pro Max, same build: no control, the rest of the reader works (Guillaume).

## 4. Release

- [ ] 4.1 Shipped with the extension release of `add-lingua-reader` (its task 7.4) — not before Guillaume's go-ahead
