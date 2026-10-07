# Tasks

## 1. The key (apps/lingua-extension)

- [ ] 1.1 `src/i18n/language.ts`: `InterfaceLanguage`, `INTERFACE_LANGUAGE_KEY`, `interfaceLanguage(area)` (absent or unknown → `fr`), `setDocumentLanguage(document, language)` (D3). `src/state/storage.ts` lists the key; `src/background.ts` writes it from the profile's native language where `storedNativeLanguage` is read and where the profile is set. Tests: *Every installed reader*, *The key follows the profile*, *A surface before any engine* (the reader through a fake area, no engine).

## 2. The catalogue (apps/lingua-extension/src/i18n)

- [ ] 2.1 `fr/<surface>.ts` for every surface of the inventory (popup, hud, card, selection, review, stats, settings, colours, display, translation, account, onboarding, reader, sync, languages, grammar), every literal extracted byte for byte, keys named by meaning; the fragments as slot messages, the counts as plural forms (D1, D2). The one-off comparison script's output (every inventory literal found in the catalogue) in the pull request.
- [ ] 2.2 `index.ts`: `copyFor(surface, language)`, `plural`, `formatNumber`, `formatDate` (D2, D4); tests: *A count in each language*, the French formats kept (*A French format `Intl` would change*), English and Spanish through `Intl`.
- [ ] 2.3 `en/<surface>.ts` and `es/<surface>.ts`, typed `typeof fr.<surface>`, full drafts per `README.md` (D7): US English; tú, neutral, no vosotros; « forma en -ing »; RAE tense names; CEFR / MCER; RAE numbers. `README.md`: register, terms, typography, naming.

## 3. Lints and parity (apps/lingua-extension/test)

- [ ] 3.1 `lint-copy.spec.ts`: a French literal outside `src/i18n/` fails unless the file is on the baseline; a baseline file holding no literal fails (D5). The baseline is the inventory's 24 files and 6 pages. `lint-language-labels.spec.ts` admits `src/i18n/`. Tests: *A French literal outside the catalogue*, *A baseline that went stale* (on a scratch tree).
- [ ] 3.2 `i18n.spec.ts`: *The drafts are whole* (presence, non-empty, slots taken, no untranslated text outside the same-everywhere list); *A key missing in Spanish* is the compiler's (`yarn typecheck`), shown by a type test.

## 4. Gates and docs

- [ ] 4.1 In `apps/lingua-extension`: `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (the 42 spec files asserting French copy unchanged), `yarn build`, `yarn check:variants`; bundle sizes per entry before and after, in the pull request (no surface imports the catalogue: the difference is the key's reader only).
- [ ] 4.2 `openspec validate add-lingua-interface-language --strict` passes, and `python3 scripts/openspec_archive_order.py add-lingua-interface-language` exits 0; change 13 is marked done in `docs/lingua/language-matrix-programme.md`.

## 5. Review (owner)

- [ ] 5.1 [manual] The owner reviews the English and the Spanish drafts in the pull request (M9); corrections land in it.
