# Tasks

## 1. The catalogue entries (apps/lingua-extension/src/i18n)

- [ ] 1.1 `fr/languages.ts`: today's `WORDS` and the twelve functions, byte for byte; `levelScale` « CEFR » (D1, D3).
- [ ] 1.2 `en/languages.ts` and `es/languages.ts`: the words and the twelve messages in each language's grammar, `levelScale` "CEFR" / « MCER » (D1, D3). Tests: *An English-native reader*, *A Spanish-native reader*, every message called with every studied language in every interface language (no empty, no French in en/es).

## 2. The seam and the lint (apps/lingua-extension)

- [ ] 2.1 `src/analyzer/language-labels.ts` reads the interface language and delegates (D2); its existing tests pass unchanged (*Every reader today*).
- [ ] 2.2 `test/lint-language-labels.spec.ts`: every language's name in every interface language, whole words, `.ts` and `.html`, outside `src/i18n/`, with the baseline and its stale check (D4). Test: *Checked by lint*.

## 3. Gates, review and docs

- [ ] 3.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.2 [manual] The owner reviews the English and Spanish messages in the pull request (M9).
- [ ] 3.3 `openspec validate add-lingua-native-language-labels --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-labels` exits 0; change 19 is marked done in `docs/lingua/language-matrix-programme.md`.
