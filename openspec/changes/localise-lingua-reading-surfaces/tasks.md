# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [ ] 1.1 `{fr,en,es}/{popup,hud,drawer,card,selection,sidepanel,reader}.ts`, the French byte for byte from the files and pages, the English and Spanish drafts per `README.md` (change 13 D1, D7); `index.ts` `fillPage(copy)` (D2).

## 2. The surfaces (apps/lingua-extension)

- [ ] 2.1 `popup/popup.ts` and `popup.html`: copy from the catalogue, `data-copy` on the page's text nodes and attributes, `fillPage` at start, `lang` (D1, D2). Test: *A page before its script*; the popup's spec unchanged.
- [ ] 2.2 `reading/hud.ts`, `reading/drawer.ts`: copy from the catalogue through `ReadingSession`, `lang` on the hosts (D1, D3).
- [ ] 2.3 `reading/wordpopup.ts`, `reading/selection-card.ts`: copy from the catalogue, the rarity bands through `formatNumber`, `lang` on the host (D1, D3, D4). Tests: *An English-native reader*; `wordpopup`, `selection-card` and `rarity-text` specs unchanged.
- [ ] 2.4 `sidepanel.html` filled at mount; `reader/copy.ts` becomes the catalogue's module, `reader/app.ts`, `library.ts`, `reader.html` follow, `${n} %` through the catalogue (D1, D2, D4). Tests: `reader-app` spec unchanged.

## 3. The lint and the gates

- [ ] 3.1 `test/lint-copy.spec.ts`: the files and pages of this change leave the baseline (*Off the baseline*).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (every spec asserting these surfaces' French copy unchanged; *Every reader today*), `yarn build`, `yarn check:variants`; bundle sizes per entry in the pull request; the five targets opened by eye on the popup, the side panel and a page (no flash of empty copy).
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries of these surfaces (M9).
- [ ] 3.4 `openspec validate localise-lingua-reading-surfaces --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-reading-surfaces` exits 0; change 14 is marked done in `docs/lingua/language-matrix-programme.md`.
