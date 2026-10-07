# Tasks

## 1. The key (apps/lingua-extension)

- [ ] 1.1 `src/i18n/language.ts`: `InterfaceLanguage`, `INTERFACE_LANGUAGE_KEY`, `interfaceLanguage(area)` (absent or unknown → `fr`), `setDocumentLanguage(document, language)` (D3). `src/state/store.ts`: the store's owner writes the key from the backup's profile on every backup write (the one path a profile change, a restore from a file, a full reset and the first hydration go through). Tests: *Every installed reader*, *The key follows the profile*, *A restore from a file*, *A surface before any engine* (the reader through a fake area, no engine).

## 2. The catalogue (apps/lingua-extension/src/i18n)

- [ ] 2.1 `fr/<surface>.ts` for every surface of the inventory (popup, hud, card, selection, review, stats, settings, colours, display, translation, account, onboarding, reader, sync, languages, grammar), every literal extracted byte for byte — #696's copy included, once it is merged — keys named by meaning, no `as const`; the fragments as slot messages, the counts as plural forms with the raw count in French (D1, D2). The one-off comparison script's output (every inventory literal found in the catalogue) in the pull request.
- [ ] 2.2 `index.ts`: the helpers `plural` (`{one, many?, other}`, `many` falling back to `other`), `formatNumber`, `formatDate`, and the types; no map of surfaces (D1, D2, D4); tests: *A count in each language*, the French formats kept (*A French format `Intl` would change*), English and Spanish through `Intl`, `many` for Spanish and French at a million.
- [ ] 2.3 `en/<surface>.ts` and `es/<surface>.ts`, typed `typeof fr.<surface>`, full drafts per `README.md` (D7): US English; tú, neutral, no vosotros; « forma en -ing »; RAE tense names; CEFR / MCER; RAE numbers. `README.md`: register, terms, typography, naming.

## 3. Lints and parity (apps/lingua-extension/test)

- [ ] 3.1 `lint-copy.spec.ts`: string and template literals from the TypeScript syntax tree (comments and regular expressions excluded) and the HTML pages' text; a French literal outside `src/i18n/` fails unless the file is on the baseline; a baseline file holding no literal fails (D5). The baseline is the inventory's 24 files and 6 pages, and no other file trips it on main (verified in the pull request). `lint-language-labels.spec.ts` admits `src/i18n/`. Tests: *A French literal outside the catalogue*, *A baseline that went stale* (on a scratch tree).
- [ ] 3.2 `i18n.spec.ts`: *The drafts are whole* (presence, non-empty, slots taken, no untranslated text outside the same-everywhere list), *A message in three languages*; *A key missing in Spanish* is the compiler's (`yarn typecheck`), shown by a type test.

## 4. Gates and docs

- [ ] 4.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (the 42 spec files asserting French copy unchanged), `yarn build`, `yarn check:variants`; bundle sizes per entry before and after, in the pull request (no surface imports the catalogue: the difference is the key's reader only).
- [ ] 4.2 `openspec validate add-lingua-interface-language --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-interface-language` exits 0; change 13 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Review (owner)

- [ ] 5.1 [manual] The owner reviews the English and the Spanish drafts in the pull request (M9); corrections land in it.
