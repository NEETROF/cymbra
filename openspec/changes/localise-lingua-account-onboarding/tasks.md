# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [ ] 1.1 `{fr,en,es}/{account,onboarding}.ts`: the French byte for byte (the error tables, the flows' messages, the pages' text), the slot messages, the English and Spanish drafts per `README.md` (D1, D3).

## 2. The surfaces (apps/lingua-extension)

- [ ] 2.1 `account/account.ts`, `flow.ts`, `view.ts`, `copy.ts`, `account.html`: the key read first, copy from the catalogue, the page filled at mount with `lang`, dates per language (D1, D3). Tests: the four account spec files unchanged; *An English-native reader signs in*.
- [ ] 2.2 `account/locale.ts` `accountLocale(interfaceLanguage, browserLanguage)` and its use on sign-up and in `deleteAccountUrl` (D2). Tests: each scenario of *The account's e-mails follow the interface language*; the list of languages Cymbra speaks checked against `apps/music/lib/l10n/`.
- [ ] 2.3 `onboarding/onboarding.ts`, `level-row.ts`, `onboarding.html`: the key read before the engine, copy from the catalogue, the page filled at mount with `lang` (D1). Tests: the onboarding specs unchanged; one test in English.

## 3. The lint and the gates

- [ ] 3.1 `test/lint-copy.spec.ts`: the files leave the baseline, which is then empty (*The baseline is empty*).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries of the account and the onboarding (M9).
- [ ] 3.4 `openspec validate localise-lingua-account-onboarding --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-account-onboarding` exits 0; change 17 is marked done in `docs/lingua/language-matrix-programme.md`.
