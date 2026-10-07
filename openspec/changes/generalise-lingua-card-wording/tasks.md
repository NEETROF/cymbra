# Tasks

## 1. The description (apps/lingua-extension)

- [ ] 1.1 `src/reading/grammar-description.ts`: `describeForm(grammar, headword, surface, written)` → `FormDescription` (own readings with persons merged, the dictionary form left out, others, pieces, sameAsHeadword, senses) with no word of any language (D1). Tests: the structural half of `test/word-grammar.spec.ts` (merging, dictionary form, others, pieces), and *A tag the renderer must know* (every member of the engine's vocabulary).

## 2. The renderers (apps/lingua-extension/src/i18n)

- [ ] 2.1 `fr/grammar.ts`: today's wording moved from `grammar-labels.ts`, byte for byte — tables, articles, elision, joining, headings, `tenseRank`'s order; `TENSES` by studied language (D2, D3). `test/word-grammar.spec.ts` passes without a change (the diff of the file is empty).
- [ ] 2.2 `en/grammar.ts` and `es/grammar.ts`: full renderers drafted per `src/i18n/README.md` and M10 (English Wiktionary form-of wording; RAE/ASALE terms), `TENSES` for en and es studied, fr studied drafted (D2, D3). `test/word-grammar-en.spec.ts` and `-es.spec.ts` over the French spec's inputs: *An English-native reader of Spanish*, *A Spanish-native reader of English*, *The order of tenses in each language*.

## 3. The card (apps/lingua-extension)

- [ ] 3.1 `wordpopup.ts` picks the renderer by the interface language and sets `lang` on the host, the headword and each studied-language segment; `hud.ts` and `drawer.ts` set `lang` on their hosts (D4). `grammar-labels.ts` keeps only what other modules import, or is removed. Tests: *Every reader today* (`lang` attributes, French lines unchanged).

## 4. Gates, review and docs

- [ ] 4.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (`test/word-grammar.spec.ts` unchanged and green), `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the English and Spanish renderers' wording and the tense tables in the pull request (M9).
- [ ] 4.3 `openspec validate generalise-lingua-card-wording --strict` passes, and `python3 scripts/openspec_archive_order.py generalise-lingua-card-wording` exits 0; change 18 is marked done in `docs/lingua/language-matrix-programme.md`.
