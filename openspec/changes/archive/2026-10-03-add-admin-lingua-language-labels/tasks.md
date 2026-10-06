## 1. The names

- [x] 1.1 `lingua.languages` in `fr.json` and `en.json`; `languageLabel` in `src/i18n/` (design D1).
- [x] 1.2 `LinguaView.vue` names the breakdown's languages and the filter's options, values unchanged (design D2).

## 2. Tests

- [x] 2.1 Unit: `languageLabel` in both locales, a code without a name, both locales naming the same codes.
- [x] 2.2 End to end (`e2e/lingua.spec.ts`): the breakdown and the filter read « English ».

## 3. Gates

- [x] 3.1 In `apps/back-office`: typecheck, lint, unit tests, format check and build.
- [x] 3.2 `openspec validate add-admin-lingua-language-labels --strict` passes. In `docs/lingua/spanish-programme.md`, change 3 is marked done.
