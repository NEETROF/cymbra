# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [x] 1.1 `{fr,en,es}/{review,stats}.ts`: the French byte for byte (#696's strings when it is on main, the ladder's escapes, change 13's `stats.noLevels` kept), the plural forms, the English and Spanish drafts per `README.md` (D1–D3, D5). Change 13's entries cover every text the surfaces held: none added, none changed; #696 is not on main (D4).

## 2. The surfaces (apps/lingua-extension)

- [x] 2.1 `review/view.ts`, `review/review-page.ts`: copy from the catalogue through `opts.interfaceLanguage` (absent = French); the side panel passes what it read, the drawer what the session handed it (D1, D2). Tests: `review-page` and `view` specs unchanged; *An English-native reader's review*.
- [x] 2.2 `stats/view.ts`, `stats/ladder.ts`, `stats/stats.ts`, `stats/stats.html`: `mountStats`' fifth parameter, copy from the catalogue, `cardsAdded` as plural forms, the ladder's numbers per language (French unchanged), `stats.ts` reading the key and filling the page with change 14's `fillPage` and `lang` (D1–D3, D5). Tests: `stats` and `stats-view` specs unchanged; *A Spanish-native reader's statistics*.

## 3. The lint and the gates

- [x] 3.1 `test/lint-copy.spec.ts`: the files leave the baseline (*Off the baseline*).
- [x] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries (M9).
- [x] 3.4 `openspec validate localise-lingua-review-stats --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-review-stats` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 16 is marked done in `docs/lingua/language-matrix-programme.md`.
- [ ] 3.5 [manual] By eye on the targets: the statistics tab shows no flash of empty copy (change 14's pending rule), and review and statistics in English and Spanish, in the side panel and the drawer, hold their longer drafts.
