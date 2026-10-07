# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [ ] 1.1 Change 13's `{fr,en,es}/{popup,hud,card,selection,reader}.ts` completed where a literal was missed (none re-extracted); `{fr,en,es}/{drawer,sidepanel}.ts` split out of their nearest surface's module; the slot messages for « Réviser (n) », « Niveau : » and `knownOnPage`; `index.ts` gains `fillPage(document, copy)`, `formatPercent(language, n, form)` and the RAE grouping of `formatNumber` for Spanish, with tests; the `data-copy-pending` rule (D1, D2, D4).

## 2. The surfaces (apps/lingua-extension)

- [ ] 2.1 `popup/popup.ts` and `popup.html`: the key read first, copy from the catalogue, `data-copy` on the page's text nodes and attributes, `fillPage` before the page shows, `lang`, `formatPercent` (D1, D2, D4). Tests: a new `popup.spec.ts` (*A surface without a spec today*, *A page before its script*).
- [ ] 2.2 `content.ts` reads the key before `new ReadingSession`; the session hands the HUD, the drawer, the word card and the selection card their copy and the interface language at construction; `reading/hud.ts`, `reading/drawer.ts`: copy from the catalogue, `lang` on the hosts, `formatPercent` (D1, D3, D4). Tests: a new `drawer.spec.ts`; the `hud` spec unchanged.
- [ ] 2.3 `reading/wordpopup.ts`, `reading/selection-card.ts`: copy handed in, the rarity bands through `formatNumber`, `lang` on the host (D1, D3, D4). Tests: *An English-native reader*; `wordpopup`, `selection-card` and `rarity-text` specs unchanged.
- [ ] 2.4 `sidepanel.html` filled before it shows (a new `sidepanel.spec.ts`); `reader/copy.ts` becomes the catalogue's module, `reader/app.ts`, `library.ts`, `reader.html` follow, `${n} %` through `formatPercent` (D1, D2, D4). Tests: `reader-app` spec unchanged.

## 3. The lint and the gates

- [ ] 3.1 `test/lint-copy.spec.ts`: the files and pages of this change leave the baseline (*Off the baseline*).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (every spec asserting these surfaces' French copy unchanged, the three new specs; *Every reader today*), `yarn build`, `yarn check:variants`; bundle sizes per entry in the pull request; the five targets opened by eye on the popup (Safari's popover size included), the side panel and a page (no flash of empty copy).
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries of these surfaces (M9).
- [ ] 3.4 `openspec validate localise-lingua-reading-surfaces --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-reading-surfaces` exits 0; change 14 is marked done in `docs/lingua/language-matrix-programme.md`.
