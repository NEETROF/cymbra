# Tasks

## 1. The data state (apps/lingua-extension)

- [x] 1.1 `src/sync/sync.ts` `checkErasure` records `languageLabels` from `GetDataState` (D1); `src/gen/` stubs regenerated from the merged change 10 protos (`yarn gen:proto`, nothing committed).

## 2. Cards (apps/lingua-extension)

- [x] 2.1 `pushCards` sends `glossLanguage` (an absent label as empty), and holds a card whose label is other than `fr` while `languageLabels` is false; `fetchCards` sends `anyGlossLanguage: true`, passes `gloss_language` to `applyCardOps`, and resets the card cursor once under `cymbra-lingua-sync-labels` (D2) — the status cursor stands; the echo of a card held this sync is dropped before the pulled operations are applied (D2). `test/sync.spec.ts`: each scenario of *A device sends the language of a gloss only to a server that stores it*, *A French card*, *The first pull of this build* and *A server that no longer stores the labels* included; the review half of *A card pulled with its label* (a gloss the reader can read) is pinned by change 11's tests (`add-lingua-card-gloss-language`).

## 3. Daily statistics v4 (apps/lingua-extension)

- [x] 3.1 `src/state/dailystats.ts`: `cymbra-lingua-daily-v4`, `native` beside the counts, v3 read once as `fr` (v2 as English and French when neither exists), `bump` and the recorders take the native language, `clearDailyStats` covers v2, v3 and v4; v4 joins `STORE_KEYS` and v3 stays there, not retired (D3). Tests: *Statistics written before the label* (from v3, and from v2 alone), a day counted under two natives keeps the last, v3 still present after the first v4 write.
- [x] 3.2 The callers (`reading/session.ts`, `review/review-page.ts`, `review/session.ts`) pass the engine's native language; `pushStats` sends `nativeLanguage` and holds a non-French statistic while `languageLabels` is false (D3). `test/sync.spec.ts`: each scenario of *A device's daily statistics carry its native language*.

## 4. Disclosures (apps/site, apps/lingua-apple)

- [x] 4.1 `apps/site/src/pages/confidentialite.md` and `en/privacy.md`, Lingua annex: the deck row names the translation's language, the statistics row the native language (D5; if `add-lingua-card-sentence-translation` landed first, rebase its hunk of the deck row). `apps/lingua-apple/README.md`, App Store privacy answers: the « What it is in Lingua » column names the gloss language under User Content and the native language under Usage Data, categories and linkage unchanged.

## 5. Gates and spec

- [x] 5.1 In `apps/lingua-extension`: `yarn gen:proto`, `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; `site-check` green on the pull request.
- [x] 5.2 `openspec validate add-lingua-native-language-sync-client --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-native-language-sync-client` exits 0; change 12 is marked done in `docs/lingua/language-matrix-programme.md`.

## 6. Release (owner)

- [ ] 6.1 [manual] Change 10 is deployed and checked from outside (its task 5.2) before this change is built for a store.
