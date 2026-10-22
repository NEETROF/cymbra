# Tasks

## 1. The site (apps/site)

- [ ] 1.1 `src/lib/lingua-text.ts`: `names.it` in the fr, en and es tables (« italien », "Italian", « italiano »), `STUDIED.it = { namesTenseAndGender: true, levelsEstimated: true }`, its JSDoc naming Italian (design D1); `README.md`'s note on `STUDIED` names Italian.
- [ ] 1.2 `test/lingua-text.spec.ts`: the three tests that took Italian as their unnamed language take German (`de`); a test feeding seven pairs with an it-fr route asserts the French page's audience, card, levels and table row and the French home card « Lisez le web en anglais, en espagnol ou en italien… » (D1, D2, the MODIFIED home-card requirement).
- [ ] 1.3 `yarn test`, `yarn build`, `yarn check:routes`; the byte-pinned fixtures unchanged, `dist/` from `origin/main` and from the branch byte for byte; the French Lingua page with a seven-pair list read at 375 px in a browser, the page not scrolling sideways (D2).

## 2. The back office (apps/back-office)

- [ ] 2.1 `src/i18n/locales/fr.json` and `en.json`: `lingua.languages.it` « Italien », "Italian"; `test/lingua.spec.ts`'s names case gains it; `yarn lint`, `yarn format:check`, `yarn typecheck`, `yarn test` (D7).

## 3. The listings (apps/lingua-extension, apps/lingua-apple)

- [ ] 3.1 `apps/lingua-extension/STORE-LISTING.md`: the French description (D3), the English and Spanish sentences (D4), the single purpose, the remote-code answer and the test instructions (D5), the notes that say what French speakers study, the claims table's rows for the French texts' Italian claims (D4), the « without fr-es » texts labelled as history; `## Summary` quoting change 20's French summary, « The French summary does not move » removed; a section « The dashboards' languages: Italian studied (change 20's submission) » (D8); each count recorded as the file counts; `yarn format:check`.
- [ ] 3.2 `apps/lingua-apple/STORE-LISTING.md`: the fr-FR subtitle, promotional text, keywords (heading « (95 / 100; 96 bytes) ») and description (D3), the en and es sentences (D4), the review notes on #994's block (D5), a fr-FR « What's New » paragraph for change 20's release, the locales note, the *Screenshots* section and its fr-FR row (the Italian capture in the three slots, macOS still to capture), *What the app itself says* quoting change 20's step 3 (D8); each count recorded.
- [ ] 3.3 The figures of D6 taken from `main` when this is implemented: it-en's served size from the catalogue (change 2), it-fr's committed coverage (change 16), change 19's figures and the owner's answer at change 20; each sentence that depends on one written to match.

## 4. Gates

- [ ] 4.1 `openspec validate add-lingua-italian-listings --strict` passes; `python3 scripts/openspec_archive_order.py add-lingua-italian-listings` names its prerequisites; the MODIFIED blocks re-copied from `add-lingua-french-listings` as it then stands.
- [ ] 4.2 `docs/lingua/italian-programme.md` row 21's status names this change's pull requests.

## 5. Owner

- [ ] 5.1 [manual] The owner reads the drafts (M9) and answers the design's Open Questions 1 to 4.
- [ ] 5.2 [manual] With change 20's release: the App Store fr-FR capture on an Italian page (D8), the listings pasted after the site deploy that publishes it-fr's figures, and what each dashboard shows recorded (I12, M18).
