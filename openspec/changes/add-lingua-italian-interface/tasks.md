# Tasks

Implemented after change 32.

## 1. The Italian catalogue

- [x] 1.1 `src/i18n/it/`: the 23 text modules, `typeof fr`, from the French source with Music's Italian terms and the Spanish draft's choices; French typography undone (« » without inner spaces, no space before « : ; ? ! »); D4's list; the voice help's menu paths from a cited source or marked unverified; `languages` with the articles, the gendered forms, `levelScale: "QCER"` and the `da()` helper (D1, D4).
- [x] 1.2 `src/i18n/README.md`: an Italian section, its title no longer « three languages », the surfaces' list brought to 24; the stale comments of `index.ts` and `language-labels.ts`.

## 2. Helpers, alias and wiring

- [x] 2.1 `src/i18n/index.ts`: `LOCALES.it = "it-IT"`; four-digit numbers as « 5000 » (Open Question 1, settled; `useGrouping: "min2"`, with `"ES2023.Intl"` added to `tsconfig.json`'s `lib`); `formatPercent` an exhaustive switch with an Italian branch (« 96% »); the counted messages' `many` forms (D3).
- [x] 2.2 `src/i18n/language.ts`: `InterfaceLanguage = NativeLanguage`; `state/store.ts`: change 32's `interfaceOf` and its test removed; the 23 sites' Italian entries; `PRIVACY_POLICY_URLS.it` and `LEAVING_PAGES.it` the English pages; `READ_AS_NOUN.it = "feminine"`; `reading-copy.ts` mapping Italian to the English renderer until change 34 (D2, D4, D5); `INTERFACE_LANGUAGES` unchanged; a test that every native of the shipped pairs is an interface language, and one that a stored `it` reads French (D6).

## 3. Lints and tests

- [x] 3.1 `test/lint-language-labels.spec.ts`: Italian's names; `test/lint-settings-hosts.spec.ts`: Italian; `test/i18n.spec.ts`: the glob, `SAME_AS_FRENCH.it`, `SAME_SHAPE_AS_FRENCH.it`, the loops, the helpers' Italian cases (plurals 1, 3, a million; « 20.000 »; four digits per Open Question 1; « 96% »; the dates; `regionName`), the parity test expecting every surface but `grammar` for Italian (D7).
- [x] 3.2 The tables the compiler forces in six tests; an Italian case in `language-labels.spec.ts`; the URL cases; `yarn typecheck`, `yarn lint`, `yarn format:check`, `yarn test`, `yarn build`, `yarn check:variants`; every snapshot unchanged.

## 4. The Italian reviewer (I2)

- [x] 4.1 `.claude/agents/lingua-italian-reviewer.md`: read-only tools, the criteria and the report format of D8.
- [x] 4.2 [agent] The Italian reviewer reads every module and the README section; its findings applied in this pull request and its report pasted in its description before merge (D8). The pass found 30 findings (5 must-fix, 25 suggestions), all applied but « Stats »; its report goes into the pull request's description. Provisional choices of the owner (2026-10-26), checked at change 45: « nome utente » kept (Music says « Handle »); « Stats » kept in the HUD and the drawer; the voice help's menu paths and Firefox's consent name stay marked unverified, checked at change 45 or 44.

## 5. Gates

- [x] 5.1 `openspec validate add-lingua-italian-interface --strict`; the archive order; the MODIFIED block re-copied from the main spec as it then stands; `docs/lingua/italian-programme.md` row 33's status.
