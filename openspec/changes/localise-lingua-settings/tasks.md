# Tasks

## 1. The catalogue modules (apps/lingua-extension/src/i18n)

- [x] 1.1 `{fr,en,es}/{settings,colours,display,translation,account-setting,sync}.ts`: change 13's modules completed where a literal was missed (#696's strings when it is on main), `settings.voiceLabel(name, place)` added in the three languages, the studied languages' two notes moved to `studied-languages.ts` (the onboarding page mounts that block); `index.ts` gains `regionName(language, code)`, with tests (D1, D3, D4).

## 2. Réglages (apps/lingua-extension)

- [x] 2.1 `reading/settings-view.ts`: `opts.interfaceLanguage` (absent = French), the copy picked, the titles from the catalogue, the fragments as slot messages (D1–D3); the popup and the side panel pass the language they read, the drawer the one the session handed it. Tests: `settings-view` spec unchanged; one test in English.
- [x] 2.2 `colour-settings-view.ts`, `book-display-view.ts`, `studied-languages-view.ts`, `translation-setting.ts`, `account-setting.ts`, `sync/status.ts`: copy from the catalogue, formats per language, `reading/speech.ts`'s `voiceLabel` through `regionName` and `settings.voiceLabel` (D3, D4). Tests: their specs unchanged; *A Spanish-native reader* on the translation setting's cost, on `lastSyncLabel`'s date and on a voice's label.

## 3. The lints and the gates

- [x] 3.1 `test/lint-settings-hosts.spec.ts` imports the eleven titles from `src/i18n/fr/settings.ts` and keeps change 13's exclusion of `src/i18n/` (D2; *The titles live once*); `test/lint-copy.spec.ts`: the files leave the baseline (*Off the baseline*).
- [x] 3.2 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test`, `yarn build`, `yarn check:variants`; bundle sizes per host in the pull request.
- [ ] 3.3 [manual] The owner reviews the English and Spanish entries of Réglages (M9).
- [x] 3.4 `openspec validate localise-lingua-settings --strict` passes, and `python3 scripts/openspec_archive_order.py localise-lingua-settings` exits 10 naming only the changes of `.openspec.yaml`'s `archiveAfter` (0 once they are archived); change 15 is marked done in `docs/lingua/language-matrix-programme.md`.
- [ ] 3.5 [manual] Réglages in English and Spanish opened by eye in the popup (Safari's popover included), the side panel, the drawer and the reader's "Aa" panel — tab row, segmented rows (`.set-segmented` is `overflow: hidden`), shortcut lines.
