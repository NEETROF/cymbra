# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [ ] 1.1 `{fr,en,es}/{account,onboarding}.ts`: the French byte for byte (the error tables, the flows' messages, the pages' text), the slot messages, the English and Spanish drafts per `README.md` (D1, D3).

## 2. The surfaces (apps/lingua-extension)

- [ ] 2.1 `account/account.ts`, `flow.ts`, `view.ts`, `copy.ts`, `account.html`: the key read from `chrome.storage.local` first, copy from the catalogue, `errorCopy` taking the language as an optional last parameter (the private `linkCopy` likewise), the page filled at mount with change 14's `fillPage` and `lang`, dates per language (D1, D3). Tests: the five account spec files unchanged; *An English-native reader signs in*.
- [ ] 2.2 `account/locale.ts` `accountLocale(interfaceLanguage, browserLanguage)` as `deps.locale` on the four requests, and `deleteAccountUrl(interfaceLanguage)` (D2). Tests: each scenario of *The account's e-mails follow the interface language* and of the three MODIFIED `lingua-account` requirements; the list of languages Cymbra speaks checked against the `app_<code>.arb` files of `apps/music/lib/l10n/`.
- [ ] 2.3 `onboarding/onboarding.ts`, `level-row.ts`, `onboarding.html`: the key read before the engine, copy from the catalogue, the page filled at mount with `lang` (D1). Tests: `onboarding-level-row` and `level-choice` specs unchanged; a new `onboarding` spec (the page filled, `lang`, one test in English).

## 3. The lint and the gates

- [ ] 3.1 `test/lint-copy.spec.ts`: the files leave the baseline, which then names changes 18's and 19's two files alone (*Off the baseline*).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`.
- [ ] 3.3 [manual] The owner acknowledges that a reader whose browser is not in French gets French e-mails from their next locale-carrying request while French is every reader's interface language (proposal, Impact), or picks D2's second alternative (the interface language once chosen, change 20's marker) — before the pull request merges.
- [ ] 3.4 [manual] The owner reviews the English and Spanish entries of the account and the onboarding (M9).
- [ ] 3.5 `openspec validate localise-lingua-account-onboarding --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-account-onboarding` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 17 is marked done in `docs/lingua/language-matrix-programme.md`.
