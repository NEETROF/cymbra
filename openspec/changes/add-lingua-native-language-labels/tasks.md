# Tasks

## 1. The catalogue entries (apps/lingua-extension/src/i18n)

- [ ] 1.1 `fr/languages.ts` (change 13's module, completed): today's `WORDS` and the twelve messages, byte for byte; `levelScale` « CEFR » (D1, D3).
- [ ] 1.2 `en/languages.ts` and `es/languages.ts`, `typeof fr.languages`: the words — `windowsVoice` in the interface's language — and the twelve messages in each language's grammar, `levelScale` "CEFR" / « MCER » (D1, D3); "CEFR" on the same-in-every-language list. Tests: *An English-native reader*, *A Spanish-native reader*, every message called with every studied language in every interface language (no empty, no French in en/es).

## 2. The seam and the lint (apps/lingua-extension)

- [ ] 2.1 `src/analyzer/language-labels.ts` takes the interface language first and delegates (D2); every caller passes the language it reads (`reader/app.ts`, `settings-view.ts`, `studied-languages-view.ts`, `review-page.ts`, `review/view.ts`, `stats/view.ts`, `stats/ladder.ts`, `popup.ts`, `level-row.ts`); `settings-view.spec.ts` and `stats.spec.ts` pass unchanged (*Every reader today*); a new `language-labels.spec.ts` calls every message in the three languages.
- [ ] 2.2 `test/lint-language-labels.spec.ts`: string literals from the syntax tree, every language's name in every interface language, whole words, `.ts` and the HTML pages, outside `src/i18n/`, the named data exceptions, change 13's baseline and its stale check (D4). Test: *Checked by lint*.

## 3. Gates, review and docs

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.2 [manual] The owner reviews the English and Spanish messages in the pull request (M9).
- [ ] 3.3 `openspec validate add-lingua-native-language-labels --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-labels` exits 0; change 19 is marked done in `docs/lingua/language-matrix-programme.md`.
