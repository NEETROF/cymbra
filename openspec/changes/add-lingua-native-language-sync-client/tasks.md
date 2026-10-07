# Tasks

## 1. The data state (apps/lingua-extension)

- [ ] 1.1 `src/sync/sync.ts` `checkErasure` records `nativeLanguage` from `GetDataState` (D1); `src/gen/` stubs regenerated from the merged change 10 protos (`yarn gen:proto`, nothing committed).

## 2. Cards (apps/lingua-extension)

- [ ] 2.1 `pushCards` sends `glossLanguage`, and holds a card whose gloss language is not `fr` while `nativeLanguage` is false; `fetchCards` passes `gloss_language` to `applyCardOps` (D2). `test/sync.spec.ts`: each scenario of *A device sends the language of a gloss only to a server that stores it*.

## 3. Daily statistics v4 (apps/lingua-extension)

- [ ] 3.1 `src/state/dailystats.ts`: `cymbra-lingua-daily-v4`, `native` beside the counts, v3 read once as `fr`, `bump` and the recorders take the native language, `clearDailyStats` covers v4; `STORE_KEYS` and `RETIRED_KEYS` follow (D3). Tests: *Statistics written before the label*, a day counted under two natives keeps the last.
- [ ] 3.2 The callers (`reading/session.ts`, `review/review-page.ts`, `review/session.ts`) pass the engine's native language; `pushStats` sends `nativeLanguage` and holds a non-French statistic while `nativeLanguage` is false (D3). `test/sync.spec.ts`: each scenario of *A device's daily statistics carry its native language*.

## 4. Disclosures (apps/site, apps/lingua-apple)

- [ ] 4.1 `apps/site/src/pages/confidentialite.md` and `en/privacy.md`, Lingua annex: the deck row names the translation's language, the statistics row the native language (D5). `apps/lingua-apple/README.md`, App Store privacy answers: the gloss language under User Content, the native language under Usage Data, categories and linkage unchanged.

## 5. Gates and spec

- [ ] 5.1 In `apps/lingua-extension`: `yarn gen:proto`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; `site-check` green on the pull request.
- [ ] 5.2 `openspec validate add-lingua-native-language-sync-client --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-sync-client` exits 0; change 12 is marked done in `docs/lingua/language-matrix-programme.md`.

## 6. Release (owner)

- [ ] 6.1 [manual] Change 10 is deployed and checked from outside (its task 5.2) before this change is built for a store.
