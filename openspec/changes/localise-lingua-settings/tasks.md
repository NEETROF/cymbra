# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [ ] 1.1 `{fr,en,es}/{settings,colours,display,translation,account-setting,sync}.ts`: the French byte for byte (#696's strings included), the titles, the tabs, the slot messages and the cost/state functions; the English and Spanish drafts per `README.md` (D1, D3, D4).

## 2. Réglages (apps/lingua-extension)

- [ ] 2.1 `reading/settings-view.ts`: `opts.interfaceLanguage` (absent = French), the copy picked, the titles from the catalogue, the fragments as slot messages (D1–D3); the popup and the side panel pass the language they read, the drawer the one the session handed it. Tests: `settings-view` spec unchanged; one test in English.
- [ ] 2.2 `colour-settings-view.ts`, `book-display-view.ts`, `studied-languages-view.ts`, `translation-setting.ts`, `account-setting.ts`, `sync/status.ts`: copy from the catalogue, formats and plurals per language (D3, D4). Tests: their specs unchanged; *A Spanish-native reader* on the translation setting's cost and on `lastSyncLabel`'s plurals.

## 3. The lints and the gates

- [ ] 3.1 `test/lint-settings-hosts.spec.ts` imports the titles from `src/i18n/fr/settings.ts` and excludes that file alone (D2; *The titles live once*); `test/lint-copy.spec.ts`: the files leave the baseline (*Off the baseline*).
- [ ] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; bundle sizes per host in the pull request.
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries of Réglages (M9).
- [ ] 3.4 `openspec validate localise-lingua-settings --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-settings` exits 0; change 15 is marked done in `docs/lingua/language-matrix-programme.md`.
