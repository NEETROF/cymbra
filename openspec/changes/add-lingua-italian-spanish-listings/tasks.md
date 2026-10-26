# Tasks

Implemented after changes 21 and 26, after change 27's committed tables, change 29's figures and the owner's answer under M15; before change 30 (design D8).

**Where it stood when implemented (2026-10-25).** Changes 27 (#1068) and 28 (#1070) merged; change 29's
figures measured (#1071: 90 of 95 marks right, 5 % withheld, the first tier) and the owner's answer under
M15 given: the marks are shown. Changes 21 (#1057, merged with change 20's #1058 after the owner's
dogfood) and 26 (its branch, waiting on #1057) are **not** on `main`, nor change 46: this change builds on
changes 21's and 26's drafts, so its pull request is based on `main` and keeps to what merges with both
branches — a section of its own in each listing file — and lists what waits on #1057's merge (1.1–1.4,
and 2.1's and 2.2's notes edited in place). #1057, #1058, #1074 and #1075 merged on 2026-10-26: the
branch was rebased on `main` and those tasks done.

**One release** (the owner, 2026-10-26): changes 20, 25 and 30 ship in one release. The listings'
texts with Italian for French, English and Spanish speakers are pasted together with it (4.2), the
single fields and review notes being this change's, which name all three audiences.

## 1. The site (apps/site)

- [x] 1.1 `test/lingua-text.spec.ts`: a test feeding nine pairs with change 2's routes asserts the Spanish home card « Lee la web en inglés, en francés o en italiano… », the Spanish page's « Pensada para hispanohablantes que aprenden inglés, francés o italiano » and its row « Italiano → español » (D1). — Done on `main` after #1057 and #1074 merged: `NINE_PAIRS` (`test/support/lingua.ts`, it-es's committed figures) and it-es's route, five cases (the home card; the Spanish page's audience, 🧭 title, card and level sentences — « En francés, en italiano y en español » — and nine rows, « Italiano → español » third; the French and English pages' notes and rows; the note without it-es's route, « Pendiente: del italiano al español »; the English and French cards unmoved). The tests feed their own pairs, as change 26's did before change 25 listed it-en: the coverage file is change 30's to write.
- [x] 1.2 `test/astro/spanish-pages.spec.ts`: en-es, fr-es and it-es, with change 2's routes, render « Lee la web en inglés, en francés o en italiano… », no audience, and the button to `/es/lingua`. — Done: `/es/ with it-es` renders the card and `/es/lingua` in the nav.
- [x] 1.3 `apps/site/README.md`'s home-card examples name Italian. If the owner names Wikidata (D6), the French, English and Spanish `glossesOrigin` in `src/lib/lingua-text.ts` and their pinned tests follow. — Done: the README's Spanish home-card examples (« Lee la web en inglés… », « …en inglés o en francés… », « …en inglés, en francés o en italiano… ») and the Spanish page's « En francés, en italiano y en español… ». The owner named Wikidata (Open Question 5, a): the `glossesOrigin` sentences are static, so they follow in change 30's pull request, which lists it-es, not here — written earlier, they would name Wikidata before any pack reads it.
- [x] 1.4 `yarn test`, `yarn build`, `yarn check:routes`; the Spanish Lingua page with nine pairs read at 375 px in a browser, the page not scrolling sideways (ten pairs if change 45 has listed en-it). — Done on `main` (2026-10-26): `yarn test` (16 files, 254 tests), `yarn build` (35 pages), `yarn check:routes` (61 passed), `yarn check` (0 errors) and `yarn typecheck` pass; with it-es added to the coverage file on a scratch build, `yarn check:routes` gives 59 passed and 2 skipped (the byte-pinned fixtures, which change 30 refreshes), and the three Lingua pages read in headless Chromium at 375 px measure 375 px with the table 335 px in its 335 px box (nine rows), at 320 px the table scrolling in its 280 px box (300.5–327 px). Ten pairs cannot be built before change 45: the site has no Italian speakers' audience yet.

## 2. The listings

- [x] 2.1 `apps/lingua-extension/STORE-LISTING.md`, each count recorded as the file counts, then `yarn format:check`:
  - the Spanish description (D2);
  - the single purpose and the remote code (D4);
  - the notes that say what Spanish speakers study (D4);
  - the claims table's rows (D6);
  - `## Summary` quoting change 30's Spanish summary;
  - the dashboards' procedure for change 30's submission (D7);
  - the variants for it-es's translation withheld (D5);
  - the single purpose's and the remote code's note: pasted with change 30's release (D4, D8).

  — **Done**: the texts, fields, claims, `## Summary`'s quote of change 30's Spanish summary, the owner's choice (105 / 112) and the dashboards' procedure are in a section of their own, _Italian for Spanish speakers (change 30's release)_ — ES 3,604 / 16,000 with the meanings' sentence naming Wikidata (Open Question 5's option a), 3,563 without; withheld 3,555 / 3,514; single purpose 620; test instructions unchanged (change 26's 988) —; `yarn format:check` passes. After #1057 and #1074 merged, the section follows change 26's, its dashboards' procedure at the file's end, and change 21's lines name Italian for Spanish speakers and point to it: the *Listing languages* paragraph, the *Description* introduction, a paragraph on the one release, the Spanish description's header, the claims table's lead sentence, its « Who learns what » and « English a Spanish reader's » rows, the test instructions' and the single purpose's notes, the permissions' introduction and the remote code's.
- [x] 2.2 `apps/lingua-apple/STORE-LISTING.md`, each count recorded:
  - the es-ES and es-MX subtitle, promotional text, keywords (heading in bytes), description and « What's New » paragraph (D2);
  - the review notes (D4), labelled for change 30's submission, the block without it-es kept for a submission before it;
  - the locales note, the fields table and the es description's lead note;
  - the review notes' lead paragraphs and *What the app itself says* (D4);
  - the *Screenshots* section (D7).

  — **Done**: a section of its own, _Italian for Spanish speakers (change 30's release)_ — subtitle `Inglés, francés o italiano` 26 / 30, promotional text 159 / 170, keywords 90 / 100 characters, 94 bytes (`aprender` giving way), description 3,617 / 4,000 with Wikidata named, 3,576 without (withheld 3,568 / 3,527), « What's New » 485 (430 without the voice, 507 with « escrito por personas », 373 withheld), review notes 3,970 / 4,000 (3,992 with line breaks counted twice; change 26's block measured 3,971, not the 3,994 *Measured* counted on, so D4's cuts leave 30 of room; Open Question 4's alternative 3,966), the es-ES capture, change 30's `lede` and step 3 quoted. After #1057 and #1074 merged, the section follows change 26's, and change 21's lines name Italian for Spanish speakers and point to it: the locales note, the *Italian studied* paragraph, the *Fields* table's es-ES column and its note, the es description's lead note, the review notes' lead and *What the app itself says*. The support rows read `https://cymbra.app/es/soporte/` throughout, as change 26 set them.
- [x] 2.3 Change 46's Italian line on what Spanish speakers study, if change 46 is on `main`: both forms kept, labelled, the one with it-es for change 30's release (D3, D8). The I2 reviewer agent's pass on it, its verdict in the pull request. — Change 46 is not on `main`: by D3 it is implemented second and keeps both forms; the lines with it-es are recorded in the extension file's section (_The Italian texts' line on what Spanish speakers study_) for it to carry, and its reviewer agent reads them with its own texts.
- [x] 2.4 The figures of D5, taken from `main` when this is implemented: it-en's and en-es's served sizes, it-es's committed coverage beside en-es's, change 29's figures and the owner's answer at change 30. Each sentence that depends on one is written to match. — Taken on `main` at `e63e7d7f4` (design D5): it-en 26,242,301 B and en-es 25,373,354 B, so « 51,6 MB con el francés o el italiano » and « 77,9 MB con los dos » (77,850,370 B; not « 77,8 »); it-es 85.2 / 76.1 / 65.1 % against en-es's 93.0 / 85.0 / 71.7, under it at each top, so the dictionary sentence names Italian; change 29's 90 of 95 marks right, 5 % withheld, and the owner's answer under M15 (the marks shown): translation offered, marked, the withheld variants written beside the drafts.

## 3. Gates

- [x] 3.1 `openspec validate add-lingua-italian-spanish-listings --strict` passes.
- [x] 3.2 `python3 scripts/openspec_archive_order.py add-lingua-italian-spanish-listings` names its prerequisites. — It exits 10, naming the ten changes of `.openspec.yaml` still unarchived, change 30 among them.
- [x] 3.3 The MODIFIED blocks are re-copied from changes 45's, 46's and `extend-site-spanish-locale`'s requirements as they then stand. — Checked on 2026-10-25: each block differs from its holder's only by this change's clauses and scenarios.
- [x] 3.4 Row 31 of `docs/lingua/italian-programme.md` names this change's pull requests.

## 4. Owner

- [x] 4.1 [manual] The owner reads the drafts (M9) and answers the design's Open Questions 1 to 7. — **Answered by the owner on 2026-10-25** (design, *Settled by the owner*): the drafts as written (1); `aprender` gives way (2); « What's New » with the voice, without « , escrito por personas » (3); D4's cuts (4); Wikidata named, option (a), the site's `glossesOrigin` sentences following with change 30 (5); the Chrome Web Store's Spanish captures unchanged (6); the Support URL `https://cymbra.app/es/soporte/` (7). Change 30's summary, « La web en inglés, francés o italiano: palabras desconocidas resaltadas, porcentaje honesto. Sin conexión. » (105), quoted in 2.1's section; its activation page keeps the drafted `lede` and step 3.
- [ ] 4.2 [manual] With change 30's release — one release with changes 20 and 25 (the owner, 2026-10-26): the App Store es-ES capture on an Italian page (D7), the listings pasted after the site deploy that publishes it-es's figures, and what each dashboard shows recorded (I12, M18).
