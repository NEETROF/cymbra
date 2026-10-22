# Tasks

Implemented after change 32.

## 1. The Italian catalogue

- [ ] 1.1 `src/i18n/it/`: the 23 text modules, `typeof fr`, from the French source with Music's Italian terms and the Spanish draft's choices; French typography undone; `languages` with the articles, the gendered forms, `levelScale: "QCER"` and the `da()` helper (D1, D4).
- [ ] 1.2 `src/i18n/README.md`: an Italian section; the surfaces' list brought to 24.

## 2. Helpers, alias and wiring

- [ ] 2.1 `src/i18n/index.ts`: `LOCALES.it = "it-IT"`; `formatPercent`'s Italian branch (« 96% »); the counted messages' `many` forms (D3).
- [ ] 2.2 `src/i18n/language.ts`: `InterfaceLanguage = NativeLanguage`; `state/store.ts`: change 32's `interfaceOf` and its test removed; the 23 sites' Italian entries; `PRIVACY_POLICY_URLS.it` and `LEAVING_PAGES.it` the English pages; `READ_AS_NOUN.it = "feminine"`; `reading-copy.ts` mapping Italian to the English renderer until change 34 (D2, D4, D5); `INTERFACE_LANGUAGES` unchanged (D6).

## 3. Lints and tests

- [ ] 3.1 `test/lint-language-labels.spec.ts`: Italian's names; `test/lint-settings-hosts.spec.ts`: Italian; `test/i18n.spec.ts`: the glob, `SAME_AS_FRENCH.it`, `SAME_SHAPE_AS_FRENCH.it`, the loops, the helpers' Italian cases (plurals 1, 3, a million; « 20.000 »; « 5000 »; « 96% »; the dates; `regionName`), the parity test expecting every surface but `grammar` for Italian (D7).
- [ ] 3.2 The tables the compiler forces in six tests; an Italian case in `language-labels.spec.ts`; the URL cases; `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test`, `yarn build`, `yarn check:variants`; every snapshot unchanged.

## 4. The Italian reviewer (I2)

- [ ] 4.1 [agent] The dedicated Italian reviewer reads every module and the README section against I6, Music's Italian terms and D4's list; its findings applied in this pull request and its report attached before merge (D8).

## 5. Gates

- [ ] 5.1 `openspec validate add-lingua-italian-interface --strict`; the archive order; the MODIFIED block re-copied from the main spec as it then stands; `docs/lingua/italian-programme.md` row 33's status.
