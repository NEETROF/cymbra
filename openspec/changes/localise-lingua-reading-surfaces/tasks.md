# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [x] 1.1 Change 13's `{fr,en,es}/{popup,hud,drawer,card,selection,sidepanel,reader}.ts` completed where a literal was missed (none re-extracted); `index.ts` gains `fillPage(document, copy)`, with tests; the `data-copy-pending` rule (D1, D2, D4).

## 2. The surfaces (apps/lingua-extension)

- [x] 2.1 `popup/popup.ts` and `popup.html`: the key read first, copy from the catalogue, `data-copy` on the page's text nodes and attributes, `fillPage` before the page shows, `lang`, `formatPercent` (D1, D2, D4). Tests: a new `popup.spec.ts` (*A surface without a spec today*, *A page before its script*).
- [x] 2.2 `content.ts` reads the key before `new ReadingSession`; the session hands the HUD, the drawer, the word card and the selection card their copy and the interface language at construction; `reading/hud.ts`, `reading/drawer.ts`: copy from the catalogue, `lang` on the hosts, `formatPercent` (D1, D3, D4). Tests: a new `drawer.spec.ts`; the `hud` spec unchanged.
- [x] 2.3 `reading/wordpopup.ts`, `reading/selection-card.ts`: copy handed in, the rarity bands through `formatNumber`, `lang` on the host (D1, D3, D4). Tests: *An English-native reader*; `wordpopup`, `selection-card` and `rarity-text` specs unchanged.
- [x] 2.4 `sidepanel.html` filled before it shows (a new `sidepanel.spec.ts`); `reader/copy.ts` keeps exporting `COPY` in today's shape, built from the catalogue's `reader` module for the interface language; `reader/app.ts`, `library.ts`, `reader.html` read the module, `${n} %` through `formatPercent`; `reader/reader.ts` reads the key before `new ReadingSession` and hands it the language and the copy, as `content.ts` does (D1, D2, D4). Tests: `reader-app` spec unchanged.

## 3. The lint and the gates

- [x] 3.1 `test/lint-copy.spec.ts`: the files and pages of this change leave the baseline (*Off the baseline*).
- [x] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (every spec asserting these surfaces' French copy unchanged, the three new specs; *Every reader today*), `yarn build`, `yarn check:variants`; bundle sizes per entry in the pull request.
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries of these surfaces (M9).
- [x] 3.4 `openspec validate localise-lingua-reading-surfaces --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-reading-surfaces` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 14 is marked done in `docs/lingua/language-matrix-programme.md`.
- [ ] 3.5 [manual] The five targets opened by eye: the popup (Safari's popover size included), the side panel and a page — no flash of empty copy.
