# Tasks

Implemented on `main` as it stands, in either order with changes 52 and 53 and with #786 (design D5);
released no later than change 52 (recommended).

## 1. The box (apps/lingua-extension)

- [ ] 1.1 `src/i18n/fr/studied-languages.ts`: `severalLanguagesOffer` and its doc comment removed; the module's header says the onboarding page carries this one text. `src/i18n/en/studied-languages.ts` and `src/i18n/es/studied-languages.ts`: the key removed (design D2).
- [ ] 1.2 `src/reading/studied-languages-view.ts`: the `offer` element and the comment citing `enable-lingua-spanish` D6 removed, `block.append(row, note)`; the header comment says "Its note" (D2).
- [ ] 1.3 Tests (D3):
  - `test/studied-languages-view.spec.ts`: « says that several languages at once are free for now » becomes « says nothing of price, in its one note », run for `fr`, `en` and `es`. It mounts the box with `settingsCopy(language).studiedLanguages` and two languages offered, and expects it shown, its `.set-note` texts exactly `[studiedNote]`.
  - `test/settings-language.spec.ts`: « An English-native reader: its notes are English… » loses its "Several languages at once: free for now." assertion, the rest kept.
- [ ] 1.4 Nothing is left of the line: `severalLanguagesOffer`, « gratuit pour l'instant », "free for now" and « gratis por ahora » are found nowhere under `apps/lingua-extension/src` and `apps/lingua-extension/test`.

## 2. Gates

- [ ] 2.1 In `apps/lingua-extension`, all pass:
  - `yarn lint`, `yarn format:check`, `yarn typecheck`;
  - `yarn test`: coverage ≥ 80 %, 3,135 tests where `main` has 3,133;
  - `yarn build` and `yarn check:variants`.

  No bundle of `dist-chromium`, `dist-firefox` or `dist-safari` holds the three texts. On `main` they are in `content.js`, `onboarding.js`, `popup.js`, `reader.js` and `sidepanel.js` of each.
- [ ] 2.2 Nothing else moves: `git diff --stat origin/main...HEAD` names no file under `crates/`, `scripts/`, `apps/lingua-extension/test/baseline/`, `apps/lingua-apple/` or `apps/site/`, and neither `apps/lingua-extension/STORE-LISTING.md` nor `src/i18n/README.md`.
- [ ] 2.3 `openspec validate remove-lingua-several-languages-offer --strict` passes. `python3 scripts/openspec_archive_order.py remove-lingua-several-languages-offer` exits 0. Row 53b of `docs/lingua/language-matrix-programme.md` says where the change stands.

## 3. Owner

- [x] 3.1 [manual] Decided by the owner on 2026-10-09 (in session): the line leaves the interface, in French, English and Spanish — the programme's rule « …nor the French interface » departed from for this line.
- [ ] 3.2 [manual] The owner answers the design's open questions 1 (the French store description) and 2 (the site's Lingua pages). Each answer either keeps the text or names the change that removes it; this change edits neither.
- [ ] 3.3 [manual] The owner releases the extension with the change (Chrome Web Store, addons.mozilla.org, the Safari host app), no later than change 52's release (recommended).
