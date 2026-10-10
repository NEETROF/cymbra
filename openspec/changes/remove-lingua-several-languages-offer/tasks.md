# Tasks

Implemented on `main` as it stands, in either order with changes 52 and 53 and with #786 (design D5);
released and deployed no later than change 52 (recommended).

## 1. The box (apps/lingua-extension)

- [ ] 1.1 `src/i18n/fr/studied-languages.ts`: `severalLanguagesOffer` and its doc comment removed; the module's header says the onboarding page carries this one text. `src/i18n/en/studied-languages.ts` and `src/i18n/es/studied-languages.ts`: the key removed (design D2).
- [ ] 1.2 `src/reading/studied-languages-view.ts`: the `offer` element and the comment citing `enable-lingua-spanish` D6 removed, `block.append(row, note)`; the header comment says "Its note" (D2).
- [ ] 1.3 Tests (D4):
  - `test/studied-languages-view.spec.ts`: « says that several languages at once are free for now » becomes « says nothing of price, in its one note », run for `fr`, `en` and `es`. It mounts the box with `settingsCopy(language).studiedLanguages` and two languages offered, and expects it shown, its `.set-note` texts exactly `[studiedNote]`.
  - `test/settings-language.spec.ts`: « An English-native reader: its notes are English… » loses its "Several languages at once: free for now." assertion, the rest kept.
- [ ] 1.4 Nothing is left of the line: `severalLanguagesOffer`, « gratuit pour l'instant », "free for now" and « gratis por ahora » are found nowhere under `apps/lingua-extension/src` and `apps/lingua-extension/test`.

## 2. The Lingua pages (apps/site)

- [ ] 2.1 `src/lib/lingua-text.ts`: each table's `languagesCard` keeps its first sentence alone — « Choisissez vos langues dans les Réglages : chaque page est lue dans la sienne. », "Choose your languages in Settings: each page is read in its own.", « Elige tus idiomas en los Ajustes: cada página se lee en el suyo. » (design D3).
- [ ] 2.2 `test/lingua-text.spec.ts`:
  - the French card's expected body loses the line;
  - the English test (« English: today's sentence… ») and the Spanish one (« Spanish: English for Spanish speakers first… ») each assert their card's body, the sentence of 2.1 exactly (D3).
- [ ] 2.3 `test/fixtures/lingua/main.fr.html` and `main.en.html` re-recorded by `apps/site/README.md`'s command after `yarn build` with `PUBLIC_DISCORD_URL` unset. Each changes by that sentence and nothing else; `taken-with.json` unchanged (D3, D4).
- [ ] 2.4 Nothing is left of the line: « gratuit pour l'instant », "free for now" and « gratis por ahora » are found nowhere under `apps/site/src` and `apps/site/test`.

## 3. Gates

- [ ] 3.1 In `apps/lingua-extension`, all pass:
  - `yarn lint`, `yarn format:check`, `yarn typecheck`;
  - `yarn test`: coverage ≥ 80 %, 3,135 tests where `main` has 3,133;
  - `yarn build` and `yarn check:variants`.

  No bundle of `dist-chromium`, `dist-firefox` or `dist-safari` holds the three texts. On `main` they are in `content.js`, `onboarding.js`, `popup.js`, `reader.js` and `sidepanel.js` of each.
- [ ] 3.2 In `apps/site`, all pass:
  - `yarn check`, `yarn typecheck`;
  - `yarn test`: 84 tests, as on `main`;
  - `yarn build`;
  - `yarn check:routes`: 33 passed and 1 skipped, as on `main`.

  `dist/` built from `origin/main` and from the branch, in the same directory: `lingua/index.html` and `en/lingua/index.html` differ by that sentence alone, and every other page and asset is byte for byte the same.
- [ ] 3.3 Nothing else moves: `git diff --stat origin/main...HEAD` names no file under `crates/`, `scripts/`, `apps/lingua-extension/test/baseline/` or `apps/lingua-apple/`, and none of `apps/lingua-extension/STORE-LISTING.md`, `apps/lingua-extension/src/i18n/README.md`, `apps/site/test/fixtures/lingua/taken-with.json` or `apps/site/src/data/lingua-coverage.json`.
- [ ] 3.4 `openspec validate remove-lingua-several-languages-offer --strict` passes. `python3 scripts/openspec_archive_order.py remove-lingua-several-languages-offer` exits 0. Row 53b of `docs/lingua/language-matrix-programme.md` says where the change stands.

## 4. Owner

- [x] 4.1 [manual] Decided by the owner on 2026-10-09 (in session): the line leaves the interface, in French, English and Spanish — the programme's rule « …nor the French interface » departed from for this line.
- [x] 4.2 [manual] Settled by the owner on 2026-10-10 (in session), the design's settled questions:
  - (1) the French store description loses the line too, in change 53's implementation;
  - (2) the site's Lingua pages lose it too, in this change.
- [ ] 4.3 [manual] With the change merged, the owner, no later than change 52's release (recommended):
  - releases the extension (Chrome Web Store, addons.mozilla.org, the Safari host app);
  - deploys the site, so `/lingua/` and `/en/lingua/` lose the line.
