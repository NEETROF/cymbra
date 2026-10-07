# Tasks

## 1. The description (apps/lingua-extension)

- [ ] 1.1 `src/reading/grammar-description.ts`: `describeForm(grammar, headword, surface, written)` → `FormDescription` (own readings with persons merged and duplicates dropped by tag, the dictionary form left out, others, pieces, sameAsHeadword, senses) with no word of any language (D1). Tests: the structural half of `test/word-grammar.spec.ts` (merging, dictionary form, others, pieces), and *What each renderer names* (every member of the engine's vocabulary: the description names it; each renderer names the French renderer's set).

## 2. The renderers (apps/lingua-extension/src/i18n)

- [ ] 2.1 `fr/grammar.ts`: today's tables, byte for byte (change 13's module completed: `TENSES`, `TENSE_ORDER`, the gerund's and infinitive's names by studied language); `src/reading/grammar-labels.ts` stays the French renderer with its exported API (`grammarLines`, `readingName`, `senseHeading`, `lineText`, `joinFrench`), reading those tables and the description (D2, D3). `test/word-grammar.spec.ts` passes without a change (the diff of the file is empty).
- [ ] 2.2 `en/grammar.ts` and `es/grammar.ts`: renderers behind the shared `GrammarRenderer` interface, drafted per `src/i18n/README.md`, M10 and D4 (English Wiktionary form-of wording; RAE/ASALE terms), their `TENSES` and `TENSE_ORDER` for en and es studied (D2, D3); change 13's parity test treats them as renderers. `test/word-grammar-en.spec.ts` and `-es.spec.ts` over the French spec's inputs: *An English-native reader of Spanish*, *A Spanish-native reader of English*, *The order of tenses in each language*, and the scenarios of *A Spanish card names its forms as French schools do* in each language.

## 3. The card (apps/lingua-extension)

- [ ] 3.1 `wordpopup.ts` picks the renderer by the interface language `ReadingSession` hands it (change 14) and sets `lang` of the studied language on the headword and each `{word}` segment (D5). Tests: *Every reader today* (`lang` attributes, French lines unchanged).

## 4. Gates, review and docs

- [ ] 4.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (`test/word-grammar.spec.ts` unchanged and green), `yarn build`, `yarn check:variants`.
- [ ] 4.2 [manual] The owner reviews the English and Spanish renderers' wording and the tense tables in the pull request (M9).
- [ ] 4.3 `openspec validate generalise-lingua-card-wording --strict` passes, and `python3 scripts/openspec_archive_order.py generalise-lingua-card-wording` exits 0; change 18 is marked done in `docs/lingua/language-matrix-programme.md`.
