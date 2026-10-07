# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [ ] 1.1 `{fr,en,es}/{review,stats}.ts`: the French byte for byte (#696's strings when it is on main, the ladder's escapes, the scale's note with « CEFR » / "CEFR" / « MCER »), the plural forms, the English and Spanish drafts per `README.md` (D1–D3, D5).

## 2. The surfaces (apps/lingua-extension)

- [ ] 2.1 `review/view.ts`, `review/review-page.ts`: copy from the catalogue through `opts.interfaceLanguage` (absent = French); the side panel passes what it read, the drawer what the session handed it (D1, D2). Tests: `review-page` spec unchanged; *An English-native reader's review*.
- [ ] 2.2 `stats/view.ts`, `stats/ladder.ts`, `stats/stats.ts`, `stats/stats.html`: `mountStats`' fifth parameter, copy from the catalogue, `cardsAdded` as plural forms, the ladder's numbers per language (French unchanged), `stats.ts` reading the key and filling the page with change 14's `fillPage` and `lang` (D1–D3, D5). Tests: `stats` and `stats-view` specs unchanged; *A Spanish-native reader's statistics*.

## 3. The lint and the gates

- [ ] 3.1 `test/lint-copy.spec.ts`: the files leave the baseline (*Off the baseline*).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries (M9).
- [ ] 3.4 `openspec validate localise-lingua-review-stats --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-review-stats` exits 0; change 16 is marked done in `docs/lingua/language-matrix-programme.md`.
